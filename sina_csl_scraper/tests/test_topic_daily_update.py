import copy
import hashlib
import json
from datetime import datetime, timedelta

import pytest
from typer.testing import CliRunner

from sina_csl_scraper.topics.models import SourceArticle
from sina_csl_scraper.topics.tts import GeneratedAudio

NOW = datetime.fromisoformat('2026-10-13T12:00:00+08:00')


def source(number=1, published_at=None):
    return SourceArticle('成都蓉城训练', f'https://news.example/{number}', '媒体',
                         published_at or NOW - timedelta(hours=1), '成都蓉城训练报道。' * 12)


class Repository:
    def __init__(self, urls=(), watermark=None):
        self.urls, self.watermark, self.writes = set(urls), watermark, []
        self.fail = False

    def check_schema(self): pass

    def published_sources(self, team_id):
        assert team_id == 77680
        return self.urls, self.watermark

    def save(self, edition):
        if self.fail:
            raise ConnectionError('private database failure')
        self.writes.append(copy.deepcopy(edition))
        self.urls.update(s['url'] for item in edition['payload']['items'] for s in item['sources'])
        self.watermark = max(datetime.fromisoformat(s['published_at']) for item in edition['payload']['items'] for s in item['sources'])


class Generator:
    model, base_url = 'test', 'https://model.example/v1'

    def __init__(self): self.calls = 0

    def generate(self, sources, team_id, now, limit):
        self.calls += 1
        return [{'id': 'a' * 64, 'team_id': team_id, 'title': '蓉城备战', 'summary': '球队训练',
                 'paragraphs': ['成都蓉城今天进行了训练。'], 'published_at': now.isoformat(),
                 'sources': [sources[0].as_source()],
                 'audio': {'status': 'pending', 'url': None, 'dialect': 'sichuan', 'duration_seconds': None}}]


class Audio:
    def __init__(self, path): self.path, self.calls, self.fail = path, 0, False

    def synthesize(self, text):
        self.calls += 1
        if self.fail: raise RuntimeError('private native failure')
        return GeneratedAudio(self.path, 3, hashlib.sha256(self.path.read_bytes()).hexdigest())


class Target:
    def __init__(self): self.fail = False

    def upload_bytes(self, name, data, content_type):
        assert content_type == 'audio/mp4'
        if self.fail: raise RuntimeError('private upload failure')
        return 'https://assets.example/' + name


def run(sources, repo, tmp_path, generator, audio, target, now=NOW):
    from sina_csl_scraper.topics.daily import run_daily_update
    return run_daily_update(sources, repository=repo, generator_factory=lambda: generator,
        tts_factory=lambda: audio, target_factory=lambda: target, cache_dir=tmp_path, now=now)


def setup(tmp_path):
    path = tmp_path / 'voice.m4a'; path.write_bytes(b'audio')
    return Generator(), Audio(path), Target()


def test_no_new_across_dates_keeps_original_without_llm_or_tts(tmp_path):
    from sina_csl_scraper.topics.daily import run_daily_update
    old = source()
    repo = Repository([old.url], old.published_at)
    def forbidden(): raise AssertionError('no-new must not initialize providers')
    result = run_daily_update([old, source(2, old.published_at - timedelta(hours=1))], repository=repo,
        generator_factory=forbidden, tts_factory=forbidden, target_factory=forbidden,
        cache_dir=tmp_path, now=NOW + timedelta(days=1))
    assert result['status'] == 'no_new'
    assert repo.writes == []


@pytest.mark.parametrize('failure', ['tts', 'upload', 'database'])
def test_failure_retains_database_and_retries_pending_without_llm_across_days(tmp_path, failure):
    generator, audio, target = setup(tmp_path); repo = Repository()
    audio.fail = failure == 'tts'; target.fail = failure == 'upload'; repo.fail = failure == 'database'
    with pytest.raises(Exception): run([source()], repo, tmp_path, generator, audio, target)
    assert repo.writes == []
    assert json.loads((tmp_path / 'daily-state.json').read_text())['pending'] is not None
    audio.fail = target.fail = repo.fail = False
    result = run([], repo, tmp_path, generator, audio, target, now=NOW + timedelta(days=1))
    assert result['status'] == 'published'
    assert len(repo.writes) == 1 and generator.calls == 1
    assert repo.writes[0]['payload']['edition_date'] == '2026-10-13'
    assert all(item['audio']['status'] == 'ready' for item in repo.writes[0]['payload']['items'])
    assert json.loads((tmp_path / 'daily-state.json').read_text())['pending'] is None
    assert run([source()], repo, tmp_path, generator, audio, target, now=NOW + timedelta(days=1))['status'] == 'no_new'
    assert len(repo.writes) == 1


def test_empty_generation_keeps_previous_and_does_not_call_tts(tmp_path):
    generator, audio, target = setup(tmp_path); repo = Repository()
    generator.generate = lambda *args, **kwargs: []
    result = run([source()], repo, tmp_path, generator, audio, target)
    assert result['status'] == 'no_topics'
    assert repo.writes == [] and audio.calls == 0


def test_expired_pending_material_is_not_published_later(tmp_path):
    generator, audio, target = setup(tmp_path); repo = Repository(); audio.fail = True
    with pytest.raises(Exception): run([source()], repo, tmp_path, generator, audio, target)
    audio.fail = False
    assert run([], repo, tmp_path, generator, audio, target, now=NOW + timedelta(days=4))['status'] == 'no_new'
    assert repo.writes == []


def test_daily_cli_no_new_is_success_without_llm_configuration(tmp_path, monkeypatch):
    from sina_csl_scraper.cli import app
    from sina_csl_scraper.topics import cli
    config = tmp_path / 'sources.json'; config.write_text('{}')
    monkeypatch.setenv('FI_DATABASE_URL', 'postgresql://unused')
    monkeypatch.delenv('FI_TOPIC_LLM_BASE_URL', raising=False)
    monkeypatch.setattr(cli, 'TopicRepository', lambda _: Repository())
    monkeypatch.setattr(cli, 'collect_sources', lambda *args, **kwargs: [])
    result = CliRunner().invoke(app, ['topics', 'daily-update', '--config', str(config), '--output-dir', str(tmp_path), '--write-db'])
    assert result.exit_code == 0, result.output
    assert 'no_new' in result.output


def test_busy_cli_does_not_overwrite_active_job_status(tmp_path, monkeypatch):
    from sina_csl_scraper.cli import app
    from sina_csl_scraper.topics import cli
    config = tmp_path / 'sources.json'; config.write_text('{}')
    status = tmp_path / 'daily-status.json'
    status.write_text('{"status":"running","started_at":"original"}')
    before = status.read_bytes()
    monkeypatch.setenv('FI_DATABASE_URL', 'postgresql://unused')
    with cli._lock(tmp_path):
        result = CliRunner().invoke(app, ['topics', 'daily-update', '--config', str(config),
                                         '--output-dir', str(tmp_path), '--write-db'])
    assert result.exit_code == 0, result.output
    assert 'already_running' in result.output
    assert status.read_bytes() == before


def test_cache_failure_after_commit_reports_new_publication_truthfully(tmp_path, monkeypatch):
    from sina_csl_scraper.topics import daily
    generator, audio, target = setup(tmp_path); repo = Repository()
    atomic = daily._atomic_json
    def fail_latest(path, value):
        if path.name == 'latest.json': raise OSError('cache unavailable')
        atomic(path, value)
    monkeypatch.setattr(daily, '_atomic_json', fail_latest)
    with pytest.raises(daily.DailyFinalizationError): run([source()], repo, tmp_path, generator, audio, target)
    assert len(repo.writes) == 1


def test_partial_audio_never_replaces_existing_published_batch(tmp_path):
    generator, audio, target = setup(tmp_path); repo = Repository()
    generate = generator.generate
    def two(sources, team_id, now, limit):
        item = generate(sources, team_id, now, limit)[0]
        return [item, {**copy.deepcopy(item), 'id': 'b' * 64, 'title': '蓉城新赛程'}]
    generator.generate = two
    synthesize = audio.synthesize
    def fail_second(text):
        if audio.calls == 1: raise RuntimeError('second audio unavailable')
        return synthesize(text)
    audio.synthesize = fail_second
    with pytest.raises(ValueError, match='incomplete'): run([source()], repo, tmp_path, generator, audio, target)
    assert repo.writes == []
    edition = json.loads(next((tmp_path / 'editions').glob('*.json')).read_text())
    assert [i['audio']['status'] for i in edition['payload']['items']] == ['ready', 'failed']
