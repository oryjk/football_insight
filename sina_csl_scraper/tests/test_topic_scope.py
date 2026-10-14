from datetime import datetime
from types import SimpleNamespace

import pytest

NOW = datetime.fromisoformat('2026-10-14T10:00:00+08:00')


def test_dynamic_scope_follows_rank_order_and_keeps_opponents_in_catalog():
    from sina_csl_scraper.topics.scope import resolve_news_scope
    names = ['成都蓉城', '大连英博', '北京国安', '天津津门虎', '武汉三镇', '青岛海牛']
    names += [f'其它中超队{i}' for i in range(10)]
    rows = {213: [SimpleNamespace(order=i + 1, team_id=100 + i, team_name=n) for i, n in enumerate(names)],
            214: [SimpleNamespace(order=i + 1, team_id=200 + i, team_name=n) for i, n in enumerate(['广东广州豹', '南通支云', *[f'其它中甲队{i}' for i in range(14)]])]}
    class Client:
        def __init__(self, league_id): self.lid = league_id
        def fetch_league_info(self): return SimpleNamespace(name='中超' if self.lid == 213 else '中甲', current_season=2026, max_round=30)
        def fetch_standings(self, season): return list(reversed(rows[self.lid]))
    config = {'league_selections': [{'league_id': 213, 'top': 2, 'bottom': 2}, {'league_id': 214, 'top': 2, 'bottom': 0}]}
    scope = resolve_news_scope(config, NOW, client_factory=Client)
    assert [t.team_id for t in scope.selected] == [100, 101, 114, 115, 200, 201]
    assert 103 in {t.team_id for t in scope.catalog}
    rows[213][0].order, rows[213][2].order = 3, 1
    assert [t.team_id for t in resolve_news_scope(config, NOW, client_factory=Client).selected][:2] == [102, 101]
    rows[213][0].order = 1
    with pytest.raises(ValueError, match='ranking'):
        resolve_news_scope(config, NOW, client_factory=Client)
    rows[213][0].order = 3
    rows[213] = [r for r in rows[213] if r.order <= 13]
    with pytest.raises(ValueError, match='ranking'):
        resolve_news_scope(config, NOW, client_factory=Client)


def test_team_tags_match_football_context_and_do_not_alias_youth_or_city():
    from sina_csl_scraper.topics.scope import TeamTag, tags_for_text
    teams = [TeamTag(77680, '成都蓉城'), TeamTag(148, '天津津门虎'), TeamTag(469, '广东广州豹')]
    assert [t.team_id for t in tags_for_text('蓉城0比1津门虎，主帅赛后接受采访。', teams)] == [77680, 148]
    assert [t.team_id for t in tags_for_text('广州豹中甲比赛拿下三分。', teams)] == [469]
    assert tags_for_text('蓉城文化展，成都天气晴朗。', teams) == []
    assert tags_for_text('成都蓉城希拉谷在中乙备战。', teams) == []
    assert [t.team_id for t in tags_for_text('成都蓉城官方宣布与费利佩续约三年。', teams)] == [77680]
    assert [t.team_id for t in tags_for_text('蓉城下半程12轮仅3胜6平3负，足协杯被淘汰。', teams)] == [77680]


def test_full_body_recall_without_team_in_listing_title():
    from sina_csl_scraper.topics.sources import collect_sources
    from test_topic_sources import Session
    config = {'team_keywords': ['蓉城'], 'listings': [{'url': 'https://news.example/list', 'publisher': '媒体',
        'allowed_hosts': ['news.example'], 'path_pattern': r'^/news/\d+$', 'keywords': []}]}
    body = '成都蓉城在中超比赛后恢复训练，主帅介绍了备战和球员恢复的情况。' * 4
    session = Session({'https://news.example/list': '<a href="/news/1">主帅谈备战</a><a href="/news/2">城市资讯</a>',
        'https://news.example/news/1': f'<title>主帅谈备战</title><article><time datetime="2026-10-14T08:00:00+08:00"></time>{body}</article>',
        'https://news.example/news/2': '<title>城市资讯</title><article><time datetime="2026-10-14T08:00:00+08:00"></time>成都城市文化报道。蓉城天气与展览活动介绍。</article>'})
    assert [s.url for s in collect_sources(config, NOW, session=session)] == ['https://news.example/news/1']


def test_tag_backfill_preserves_identity_audio_dates_and_ignores_roundup_source_only_mentions():
    from sina_csl_scraper.topics.scope import TeamTag, tag_existing_items
    item = {'id': 'stable-id', 'title': '蓉城与津门虎的中超比赛', 'summary': '赛后采访',
            'paragraphs': ['成都蓉城在比赛中对阵天津津门虎。'], 'published_at': '2026-10-13T08:00:00+08:00',
            'sources': [{'title': '中超综述北京国安赢球'}], 'audio': {'status': 'ready', 'url': 'https://assets.example/a.m4a'}}
    updated = tag_existing_items([item], [TeamTag(77680, '成都蓉城'), TeamTag(148, '天津津门虎'), TeamTag(136, '北京国安')])
    assert updated[0]['team_tags'] == [{'team_id': 77680, 'name': '成都蓉城'}, {'team_id': 148, 'name': '天津津门虎'}]
    assert {k: v for k, v in updated[0].items() if k != 'team_tags'} == item
    assert 'team_tags' not in item
    comparison = {**item, 'title':'成都蓉城夺冠展望', 'summary':'成都蓉城仍有夺冠机会。',
                  'paragraphs':['成都蓉城等待下一场比赛。同期北京国安拿分更多，天津津门虎表现较好。']}
    assert tag_existing_items([comparison],[TeamTag(77680,'成都蓉城'),TeamTag(148,'天津津门虎'),TeamTag(136,'北京国安')])[0]['team_tags'] == [{'team_id':77680,'name':'成都蓉城'}]
    coach = {**item,'title':'主教练为何遭到质疑','summary':'津门虎的比赛带来讨论。',
             'paragraphs':['成都蓉城的主帅赛后道歉。成都蓉城的球员恢复训练。','成都蓉城主教练谈备战。']}
    assert [t['team_id'] for t in tag_existing_items([coach],[TeamTag(77680,'成都蓉城'),TeamTag(148,'天津津门虎')])[0]['team_tags']] == [77680,148]
    match = {**item,'title':'北京国安拿下三分','summary':'国安在中超比赛中取胜。',
             'paragraphs':['北京国安在中超比赛中以3比0战胜天津津门虎。国安主帅在赛后肯定了球队表现。']}
    assert [t['team_id'] for t in tag_existing_items([match],[TeamTag(136,'北京国安'),TeamTag(148,'天津津门虎')])[0]['team_tags']] == [136,148]


def test_broad_discovery_gives_football_listing_a_download_budget_even_with_large_sports_feed():
    from sina_csl_scraper.topics.sources import collect_sources
    from test_topic_sources import Session
    feed = '<rss><channel>' + ''.join(f'<item><title>篮球新闻{i}</title><link>https://news.example/basket/{i}</link></item>' for i in range(90)) + '</channel></rss>'
    pages = {'https://news.example/feed': feed, 'https://news.example/list': '<a href="/news/1">主帅谈比赛</a>',
        'https://news.example/news/1': '<title>主帅谈比赛</title><article><time datetime="2026-10-14T08:00:00+08:00"></time>' + '北京国安中超比赛恢复训练。' * 10 + '</article>'}
    for i in range(90):
        pages[f'https://news.example/basket/{i}'] = '<title>篮球</title><article><time datetime="2026-10-14T08:00:00+08:00"></time>' + '篮球训练报道。' * 20 + '</article>'
    config = {'team_keywords': ['国安'], 'team_catalog': [{'team_id':136,'name':'北京国安'}],
        'feeds':[{'url':'https://news.example/feed','publisher':'媒体','keywords':[]}],
        'listings':[{'url':'https://news.example/list','publisher':'媒体','allowed_hosts':['news.example'],'path_pattern':r'^/news/\d+$','keywords':[]}]}
    session=Session(pages)
    assert [s.url for s in collect_sources(config,NOW,session=session)] == ['https://news.example/news/1']
    assert len(session.calls) <= 92


def test_manual_collect_uses_same_dynamic_scope_as_scheduled_update(tmp_path, monkeypatch):
    import json
    from typer.testing import CliRunner
    from sina_csl_scraper.cli import app
    from sina_csl_scraper.topics import cli
    from sina_csl_scraper.topics.scope import NewsScope, TeamTag
    team=TeamTag(136,'北京国安')
    monkeypatch.setattr(cli,'resolve_news_scope',lambda *args: NewsScope((team,),(team,)))
    config=tmp_path/'news.json'
    config.write_text(json.dumps({'league_selections':[{'league_id':213,'top':3,'bottom':3}],
        'imports':[{'title':'主帅谈备战','url':'https://news.example/training','publisher':'媒体',
        'published_at':'2026-10-14T08:00:00+08:00','body':'北京国安在中超比赛后进行了恢复训练。'*8}]}))
    output=tmp_path/'collected.json'
    result=CliRunner().invoke(app,['topics','collect','--config',str(config),'--output',str(output),'--at',NOW.isoformat()])
    assert result.exit_code==0,result.output
    assert len(json.loads(output.read_text())['imports'])==1
