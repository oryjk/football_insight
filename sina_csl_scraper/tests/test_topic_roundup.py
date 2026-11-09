import copy
import hashlib
import json
from datetime import datetime, timedelta

import pytest

from sina_csl_scraper.topics.models import SourceArticle
from sina_csl_scraper.topics.roundup import (
    MAX_NEW_ENTRIES, RoundupGenerator, build_roundup_item, merge_entries, roundup_edition_id,
    roundup_item_id, roundup_narration, run_roundup_update,
)
from sina_csl_scraper.topics.scope import TeamTag
from sina_csl_scraper.topics.tts import GeneratedAudio

NOW = datetime.fromisoformat('2026-11-09T12:00:00+08:00')
RONGCHENG, TAISHAN, SHENHUA = TeamTag(77680, '成都蓉城'), TeamTag(136, '山东泰山'), TeamTag(135, '上海申花')
CATALOG = [RONGCHENG, TAISHAN, SHENHUA]


def source(number, body, published_at=None, title=None):
    return SourceArticle(title or f'新闻{number}', f'https://news.example/{number}', '媒体',
                         published_at or NOW - timedelta(hours=1), body)


TAISHAN_NEWS = '山东泰山外援泽卡合同到期，正在与其他中超球队商谈。' * 3
SHENHUA_NEWS = '上海申花主教练续约一年，球队备战新赛季。' * 3
RONGCHENG_NEWS = '成都蓉城主帅赛后表示球队将总结经验。' * 3
CAMPUS_NEWS = '浙江省高校校园超级联赛今日开幕，比赛持续一个月。' * 3


class Generator(RoundupGenerator):
    """真实的条目校验，只替换大模型请求。"""

    def __init__(self, responses):
        super().__init__(base_url='https://model.example/v1', model='test', team_catalog=CATALOG,
                         selected_team_ids=[team.team_id for team in CATALOG])
        self.responses, self.requests = list(responses), []

    def _complete(self, system, user):
        self.requests.append(user)
        response = self.responses.pop(0)
        return response(user) if callable(response) else response


def entries_for(*specs):
    """specs: (title, text, [source numbers]) -> 按输入原文的 source_id 组装大模型回复。"""
    def respond(user):
        ids = {item['url'].rsplit('/', 1)[1]: item['source_id'] for item in user['original_sources']}
        return {'entries': [{'title': title, 'text': text, 'source_ids': [ids[str(n)] for n in numbers]}
                            for title, text, numbers in specs]}
    return respond


class Repository:
    def __init__(self):
        self.rows, self.urls = {}, set()

    def roundup_sources(self, team_id):
        return set(self.urls)

    def roundup_for_date(self, team_id, edition_date):
        row = self.rows.get(roundup_edition_id(team_id, edition_date))
        return copy.deepcopy(row)

    def save_roundup(self, edition):
        self.rows[edition['id']] = copy.deepcopy(edition)
        self.urls.update(s['url'] for item in edition['payload']['items'] for s in item['sources'])


class Audio:
    def __init__(self, path):
        self.path, self.texts, self.fail = path, [], False

    def synthesize(self, text):
        if self.fail:
            raise RuntimeError('edge unavailable')
        self.texts.append(text)
        return GeneratedAudio(self.path, 30, hashlib.sha256(text.encode()).hexdigest())


class Target:
    def upload_bytes(self, name, data, content_type):
        return 'https://assets.example/' + name


@pytest.fixture
def env(tmp_path):
    path = tmp_path / 'voice.m4a'
    path.write_bytes(b'audio')
    return Repository(), Audio(path), Target()


def run(sources, generator, env, tmp_path, now=NOW):
    repository, audio, target = env
    return run_roundup_update(sources, repository=repository, generator_factory=lambda: generator,
                              tts_factory=lambda: audio, target_factory=lambda: target,
                              cache_dir=tmp_path, now=now, selected_teams=CATALOG)


def todays_item(repository):
    return repository.rows[roundup_edition_id(77680, '2026-11-09')]['payload']['items'][0]


def test_first_run_creates_one_item_per_day_with_entries_and_legacy_fields(env, tmp_path):
    sources = [source(1, TAISHAN_NEWS, NOW - timedelta(hours=3)), source(2, SHENHUA_NEWS, NOW - timedelta(hours=1))]
    generator = Generator([entries_for(('泽卡与中超球队商谈', '据报道，山东泰山外援泽卡合同到期，正在与其他中超球队商谈。', [1]),
                                       ('申花主帅续约一年', '上海申花主教练续约一年。', [2]))])
    result = run(sources, generator, env, tmp_path)
    assert result['status'] == 'published' and result['appended'] == 2
    item = todays_item(env[0])
    assert item['id'] == roundup_item_id(77680, '2026-11-09')
    assert [entry['title'] for entry in item['entries']] == ['申花主帅续约一年', '泽卡与中超球队商谈']
    assert item['entries'][1]['published_at'] == (NOW - timedelta(hours=3)).isoformat()
    assert item['paragraphs'][0] == '【申花主帅续约一年】上海申花主教练续约一年。'
    assert item['title'] == '申花主帅续约一年；泽卡与中超球队商谈'
    assert item['audio']['status'] == 'ready'
    assert {tag['name'] for tag in item['team_tags']} == {'上海申花', '山东泰山'}
    assert env[1].texts[0].startswith('11月9日每日电报。\n申花主帅续约一年。')


def test_later_run_appends_without_rewriting_published_entries(env, tmp_path):
    first = Generator([entries_for(('泽卡与中超球队商谈', '据报道，泽卡正在与其他中超球队商谈。', [1]))])
    run([source(1, TAISHAN_NEWS)], first, env, tmp_path)
    before = copy.deepcopy(todays_item(env[0])['entries'])

    second = Generator([entries_for(('申花主帅续约一年', '上海申花主教练续约一年。', [2]))])
    later = NOW + timedelta(hours=1)
    result = run([source(1, TAISHAN_NEWS), source(2, SHENHUA_NEWS, later - timedelta(minutes=10))],
                 second, env, tmp_path, now=later)
    assert result['appended'] == 1 and result['entries'] == 2
    # 已发布原文不再送给大模型，已发布标题作为去重依据传入。
    assert [s['url'] for s in second.requests[0]['original_sources']] == ['https://news.example/2']
    assert second.requests[0]['published_titles'] == ['泽卡与中超球队商谈']
    item = todays_item(env[0])
    assert item['entries'][1] == before[0]
    assert item['id'] == roundup_item_id(77680, '2026-11-09')
    assert len(env[0].rows) == 1


def test_ungrounded_entries_are_dropped_not_the_whole_batch(env, tmp_path):
    sources = [source(1, TAISHAN_NEWS), source(3, RONGCHENG_NEWS)]
    generator = Generator([entries_for(
        ('泽卡与中超球队商谈', '据报道，泽卡正在与其他中超球队商谈。', [1]),
        ('申花续约主帅', '上海申花主教练续约。', [1]),                    # 引用原文里没有申花
        ('蓉城希拉谷备战中乙', '成都蓉城希拉谷备战中乙。', [3]),          # 原文没有中乙
        ('泽卡加盟申花', '泽卡加盟上海申花。', [1]),                      # 点名了原文没有的球队
    )])
    result = run(sources, generator, env, tmp_path)
    assert result['appended'] == 1
    assert [entry['title'] for entry in todays_item(env[0])['entries']] == ['泽卡与中超球队商谈']


def test_sources_already_sent_to_the_model_are_not_resent_even_without_entries(env, tmp_path):
    generator = Generator([{'entries': []}])
    assert run([source(2, CAMPUS_NEWS + TAISHAN_NEWS)], generator, env, tmp_path)['status'] == 'no_entries'
    again = Generator([])
    assert run([source(2, CAMPUS_NEWS + TAISHAN_NEWS)], again, env, tmp_path,
               now=NOW + timedelta(hours=1))['status'] == 'no_new'
    assert again.requests == []


def test_out_of_scope_sources_never_reach_the_model(env, tmp_path):
    generator = Generator([])
    result = run([source(1, CAMPUS_NEWS)], generator, env, tmp_path)
    assert result['status'] == 'no_new'
    assert result['sources'][0]['decision'] == 'out_of_scope'


def test_audio_failure_publishes_text_and_retries_audio_next_run(env, tmp_path):
    repository, audio, _ = env
    run([source(1, TAISHAN_NEWS)], Generator([entries_for(('泽卡与中超球队商谈', '据报道，泽卡正在商谈。', [1]))]),
        env, tmp_path)
    first_audio = todays_item(repository)['audio']

    audio.fail = True
    result = run([source(1, TAISHAN_NEWS), source(2, SHENHUA_NEWS, NOW + timedelta(minutes=30))],
                 Generator([entries_for(('申花主帅续约一年', '上海申花主教练续约一年。', [2]))]),
                 env, tmp_path, now=NOW + timedelta(hours=1))
    assert result['status'] == 'published' and result['audio_error'] == 'RuntimeError'
    item = todays_item(repository)
    assert len(item['entries']) == 2 and item['audio'] == first_audio

    audio.fail = False
    result = run([], Generator([]), env, tmp_path, now=NOW + timedelta(hours=2))
    assert result['status'] == 'audio_refreshed'
    item = todays_item(repository)
    assert item['audio']['url'] != first_audio['url']
    assert '申花主帅续约一年' in audio.texts[-1]


def test_failed_generation_is_cached_per_input_and_sources_stay_unconsumed(env, tmp_path):
    def boom(user):
        raise ValueError('LLM request failed or response was not valid topic JSON')
    with pytest.raises(ValueError):
        run([source(1, TAISHAN_NEWS)], Generator([boom]), env, tmp_path)
    retry = Generator([entries_for(('泽卡与中超球队商谈', '据报道，泽卡正在商谈。', [1]))])
    assert run([source(1, TAISHAN_NEWS)], retry, env, tmp_path)['status'] == 'published'


def test_new_day_starts_a_new_item(env, tmp_path):
    run([source(1, TAISHAN_NEWS)], Generator([entries_for(('泽卡与中超球队商谈', '据报道，泽卡正在商谈。', [1]))]),
        env, tmp_path)
    tomorrow = NOW + timedelta(days=1)
    run([source(2, SHENHUA_NEWS, tomorrow - timedelta(hours=9))],
        Generator([entries_for(('申花主帅续约一年', '上海申花主教练续约一年。', [2]))]), env, tmp_path, now=tomorrow)
    assert len(env[0].rows) == 2
    next_day = env[0].rows[roundup_edition_id(77680, '2026-11-10')]['payload']['items'][0]
    assert [entry['title'] for entry in next_day['entries']] == ['申花主帅续约一年']


def test_generator_caps_entries_and_skips_titles_already_published():
    sources = [source(1, TAISHAN_NEWS)]
    many = [('泽卡与中超球队商谈', '据报道，泽卡正在商谈。', [1])] + [
        (f'泽卡动态第{n}条', '据报道，泽卡正在商谈。', [1]) for n in range(MAX_NEW_ENTRIES + 3)]
    entries = Generator([entries_for(*many)]).generate_entries(sources, ['泽卡与中超球队商谈'], NOW)
    assert len(entries) == MAX_NEW_ENTRIES - 1
    assert '泽卡与中超球队商谈' not in [entry['title'] for entry in entries]


def test_merge_keeps_existing_entries_and_orders_newest_first():
    old = {'id': 'a', 'title': '旧', 'text': 't', 'published_at': '2026-11-09T09:00:00+08:00', 'sources': [], 'team_tags': []}
    new = {'id': 'b', 'title': '新', 'text': 't', 'published_at': '2026-11-08T23:30:00+08:00', 'sources': [], 'team_tags': []}
    changed = {**old, 'title': '改写'}
    assert [e['title'] for e in merge_entries([old], [new, changed])] == ['旧', '新']


def test_narration_stays_within_tts_budget():
    entries = [{'title': f'标题{n}', 'text': '内容' * 60} for n in range(200)]
    text = roundup_narration('2026-11-09', entries)
    assert len(text) <= 5500 and text.startswith('11月9日每日电报。')


def test_legacy_fields_are_bounded_for_old_clients():
    entries = [{'id': str(n), 'title': '很长的标题' * 6, 'text': '很长的正文' * 50, 'published_at': NOW.isoformat(),
                'sources': [{'title': 't', 'url': f'https://news.example/{n}', 'publisher': 'p',
                             'published_at': NOW.isoformat()}], 'team_tags': []} for n in range(3)]
    item = build_roundup_item(team_id=77680, edition_date='2026-11-09', entries=entries, now=NOW, audio=None)
    assert len(item['title']) <= 120 and len(item['summary']) <= 500
    assert json.dumps(item, ensure_ascii=False)
