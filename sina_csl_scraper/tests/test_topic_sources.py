from datetime import datetime
from dataclasses import FrozenInstanceError

import pytest
import requests

NOW = datetime.fromisoformat('2026-10-13T12:00:00+08:00')
BODY = '成都蓉城一线队将在本周进行中超联赛训练，主教练介绍了球队的备战情况，球员按照教练安排完成恢复与战术演练。' * 3


class Response:
    def __init__(self, text, url):
        self.text, self.url = text, url
        self.headers = {'Content-Type': 'text/html; charset=utf-8'}
        self.encoding = 'utf-8'
    def raise_for_status(self):
        pass


class Session:
    def __init__(self, pages):
        self.pages, self.calls = pages, []
    def get(self, url, **kwargs):
        self.calls.append(url)
        page = self.pages[url]
        if isinstance(page, Exception):
            raise page
        return Response(page, url)


def test_listing_discovers_matching_full_originals_only():
    from sina_csl_scraper.topics.sources import collect_sources
    listing = '<a href="/news/1"><span>成都蓉城训练</span></a>' \
        '<a href="/news/2">其它球队</a><a href="https://ads.example/news/3">蓉城广告</a>' \
        '<a href="/video/4">蓉城视频</a><a href="/news/1#share">蓉城训练</a>'
    original = f'<title>成都蓉城训练</title><meta property="article:published_time" content="2026-10-13T09:00:00+08:00"><article>{BODY}</article>'
    session = Session({'https://news.example/list': listing, 'https://news.example/news/1': original})
    result = collect_sources({'listings': [{'url': 'https://news.example/list', 'publisher': '媒体',
        'allowed_hosts': ['news.example'], 'path_pattern': r'^/news/\d+$', 'keywords': ['蓉城']}]}, NOW, session=session)
    assert len(result) == 1
    assert result[0].body == BODY
    assert session.calls == ['https://news.example/list', 'https://news.example/news/1']


def test_daily_collection_distinguishes_outage_from_no_matching_news():
    from sina_csl_scraper.topics.sources import collect_sources
    config = {'listings': [{'url': 'https://news.example/list', 'publisher': '媒体',
        'allowed_hosts': ['news.example'], 'path_pattern': r'^/news/\d+$', 'keywords': ['蓉城']}]}
    with pytest.raises(ValueError, match='unavailable'):
        collect_sources(config, NOW, session=Session({'https://news.example/list': requests.RequestException('private')}), require_available=True)
    assert collect_sources(config, NOW, session=Session({'https://news.example/list': '<a href="/news/1">其它球队</a>'}), require_available=True) == []


def test_daily_original_body_outage_is_failure_even_when_listing_works():
    from sina_csl_scraper.topics.sources import collect_sources
    config = {'listings': [{'url': 'https://news.example/list', 'publisher': '媒体',
        'allowed_hosts': ['news.example'], 'path_pattern': r'^/news/\d+$', 'keywords': ['蓉城']}]}
    session = Session({'https://news.example/list': '<a href="/news/1">成都蓉城训练</a>',
                       'https://news.example/news/1': requests.ConnectionError('private failure')})
    with pytest.raises(ValueError, match='original.*unavailable'):
        collect_sources(config, NOW, session=session, require_available=True)


@pytest.mark.parametrize('body,publication', [('成都蓉城', '2026-10-13T09:00:00+08:00'),
                                            (BODY, '2026-09-01T09:00:00+08:00')])
def test_strict_collection_rejects_heading_only_body_but_accepts_old_valid_original(body, publication):
    from sina_csl_scraper.topics.sources import collect_sources
    url = 'https://news.example/news/1'
    html = f'<title>成都蓉城训练</title><meta property="article:published_time" content="{publication}"><article>{body}</article>'
    session = Session({url: html})
    config = {'articles': [{'url': url, 'publisher': '媒体'}]}
    if body == '成都蓉城':
        with pytest.raises(ValueError, match='original.*unavailable'):
            collect_sources(config, NOW, session=session, require_available=True)
    else:
        assert collect_sources(config, NOW, session=session, require_available=True) == []


def imported():
    return dict(title='成都蓉城备战', url='https://club.example/news/1', publisher='俱乐部', published_at='2026-10-13T09:00:00+08:00', body=BODY)


def test_source_identity_canonical_frozen_and_body_sensitive():
    from sina_csl_scraper.topics.models import SourceArticle, source_fingerprint
    article = SourceArticle(**{**imported(), 'published_at': NOW})
    equivalent = SourceArticle(**{**imported(), 'url': 'HTTPS://CLUB.EXAMPLE/news/1#share', 'published_at': NOW})
    changed = SourceArticle(**{**imported(), 'body': BODY + '新事实', 'published_at': NOW})
    assert article.source_id == equivalent.source_id
    assert article.source_id != changed.source_id
    assert source_fingerprint([article, changed]) == source_fingerprint([changed, article])
    assert article.as_source()['published_at'] == '2026-10-13T12:00:00+08:00'
    with pytest.raises(FrozenInstanceError):
        article.title = 'changed'


@pytest.mark.parametrize('value', ['2026-10-13T09:00:00+08:00', 'Tue, 13 Oct 2026 01:00:00 GMT'])
def test_timestamp_accepts_offsets(value):
    from sina_csl_scraper.topics.models import parse_timestamp
    assert parse_timestamp(value, now=NOW).isoformat() == '2026-10-13T09:00:00+08:00'


@pytest.mark.parametrize('value', ['2026-10-13T09:00:00', '2026-10-14T09:00:00+08:00', '', 'today'])
def test_timestamp_rejects_unknown_naive_future(value):
    from sina_csl_scraper.topics.models import parse_timestamp
    with pytest.raises(ValueError):
        parse_timestamp(value, now=NOW)


def test_imports_filter_stale_future_short_irrelevant_and_duplicates():
    from sina_csl_scraper.topics.sources import collect_sources
    good = imported()
    entries = [good, {**good, 'url': good['url'] + '#share'}, {**good, 'url': 'https://other.example/copy'}, {**good, 'published_at': '2026-10-10T08:00:00+08:00'}, {**good, 'published_at': '2026-10-14T09:00:00+08:00'}, {**good, 'body': '无关新闻' * 40, 'title': '其它球队'}, {**good, 'body': '成都蓉城摘要'}]
    result = collect_sources({'imports': entries}, NOW)
    assert len(result) == 1
    assert result[0].body == BODY


def test_rss_fetches_original_and_failure_keeps_usable_without_secrets(caplog):
    from sina_csl_scraper.topics.sources import collect_sources
    rss = '<rss><channel><item><title>成都蓉城备战</title><link>https://news.example/good</link><pubDate>Tue, 13 Oct 2026 01:00:00 GMT</pubDate><description>摘要不算原文</description></item><item><title>蓉城</title><link>https://news.example/bad?key=private</link><pubDate>Tue, 13 Oct 2026 01:00:00 GMT</pubDate></item></channel></rss>'
    html = f'<title>成都蓉城备战</title><nav>导航</nav><article><h1>成都蓉城备战</h1><p>{BODY}</p><script>邪恶指令</script><style>css</style></article>'
    session = Session({'https://news.example/feed': rss, 'https://news.example/good': html, 'https://news.example/bad?key=private': requests.RequestException('secret-key private')})
    result = collect_sources({'feeds': [{'url': 'https://news.example/feed', 'publisher': '新闻社'}]}, NOW, session=session)
    assert len(result) == 1
    assert BODY in result[0].body
    assert '导航' not in result[0].body and '邪恶指令' not in result[0].body
    assert result[0].published_at.isoformat() == '2026-10-13T09:00:00+08:00'
    assert 'private' not in caplog.text and 'secret-key' not in caplog.text


def test_atom_jsonld_and_wechat_dates():
    from sina_csl_scraper.topics.sources import collect_sources
    atom = '<feed xmlns="http://www.w3.org/2005/Atom"><entry><title>成都蓉城新闻</title><link rel="alternate" href="https://news.example/atom"/><published>2026-10-13T07:00:00+08:00</published><summary>摘要</summary></entry></feed>'
    pages = {'https://news.example/feed': atom, 'https://news.example/atom': f'<title>成都蓉城训练</title><script type="application/ld+json">{{"@type":"NewsArticle","datePublished":"2026-10-13T08:00:00+08:00"}}</script><main><p>{BODY}</p></main>', 'https://mp.weixin.qq.com/s/abc': f'<meta property="og:title" content="蓉城青训"><script>var ct = "1791853200";</script><div id="js_content"><p>{BODY}公众号内容</p><nav>分享</nav></div>'}
    result = collect_sources({'feeds': [{'url': 'https://news.example/feed', 'publisher': '媒体'}], 'articles': [{'url': 'https://mp.weixin.qq.com/s/abc', 'publisher': '公众号'}]}, NOW, session=Session(pages))
    assert len(result) == 2
    dates = {item.url: item.published_at.isoformat() for item in result}
    assert dates['https://news.example/atom'] == '2026-10-13T08:00:00+08:00'
    assert dates['https://mp.weixin.qq.com/s/abc'] == '2026-10-13T09:00:00+08:00'


def test_unknown_time_metadata_only_and_explicit_time():
    from sina_csl_scraper.topics.sources import collect_sources
    pages = {'https://news.example/unknown': f'<title>成都蓉城</title><article>{BODY}</article>', 'https://news.example/meta': f'<meta name="description" content="{BODY}"><meta property="article:published_time" content="2026-10-13T09:00:00+08:00"><body>验证访问</body>'}
    config = {'articles': [{'url': url, 'publisher': '媒体'} for url in pages]}
    assert collect_sources(config, NOW, session=Session(pages)) == []
    config['articles'][0]['published_at'] = '2026-10-13T09:00:00+08:00'
    assert len(collect_sources(config, NOW, session=Session(pages))) == 1


def test_collection_bounds_downloads():
    from sina_csl_scraper.topics.sources import collect_sources
    pages = {f'https://news.example/{i}': f'<title>蓉城新闻{i}</title><article>{BODY}{i}</article>' for i in range(40)}
    session = Session(pages)
    result = collect_sources({'articles': [{'url': url, 'publisher': '媒体', 'published_at': NOW.isoformat()} for url in pages]}, NOW, session=session)
    assert len(result) == 30
    assert len(session.calls) == 30


def test_article_heading_works_without_page_title_and_imported_markup_is_rejected():
    from sina_csl_scraper.topics.sources import collect_sources
    url = 'https://news.example/heading'
    pages = {url: f'<article><h1>成都蓉城训练</h1><p>{BODY}</p></article>'}
    config = {'imports': [{**imported(), 'title': '<b>成都蓉城</b>'}], 'articles': [{'url': url, 'publisher': '媒体', 'published_at': NOW.isoformat()}]}
    result = collect_sources(config, NOW, session=Session(pages))
    assert len(result) == 1
    assert result[0].title == '成都蓉城训练'


def test_future_original_metadata_is_rejected_even_with_current_feed_time():
    from sina_csl_scraper.topics.sources import collect_sources
    url = 'https://news.example/future'
    pages = {url: f'<title>成都蓉城</title><meta property="article:published_time" content="2026-10-14T09:00:00+08:00"><article>{BODY}</article>'}
    assert collect_sources({'articles': [{'url': url, 'publisher': '媒体', 'published_at': NOW.isoformat()}]}, NOW, session=Session(pages)) == []


def test_original_article_clock_precedes_fresh_feed_or_config_timestamp():
    from sina_csl_scraper.topics.sources import collect_sources
    url = 'https://news.example/stale-original'
    pages = {url: f'<title>成都蓉城</title><article><time datetime="2026-09-01T09:00:00+08:00">原文发布时间</time><p>{BODY}</p></article>'}
    config = {'articles': [{'url': url, 'publisher': '媒体', 'published_at': NOW.isoformat()}]}
    assert collect_sources(config, NOW, session=Session(pages)) == []

def test_page_header_clock_cannot_make_stale_original_fresh():
    from sina_csl_scraper.topics.sources import _extract_original
    from datetime import datetime
    now = datetime.fromisoformat('2026-10-13T12:00:00+08:00')
    html = '<header><time datetime="2026-10-13T09:00:00+08:00">Today</time></header>' \
        '<meta property="article:published_time" content="2026-10-01T08:00:00+08:00">' \
        '<title>成都蓉城训练</title><article>' + '成都蓉城训练报道。' * 20 + '</article>'
    article = _extract_original(html, {'url': 'https://club.example/old', 'publisher': '俱乐部'}, now)
    assert article.published_at.isoformat() == '2026-10-01T08:00:00+08:00'


def test_unscoped_time_without_article_publication_is_not_a_source_date():
    from sina_csl_scraper.topics.sources import _extract_original
    from datetime import datetime
    import pytest
    html = '<header><time datetime="2026-10-13T09:00:00+08:00">Today</time></header>' \
        '<title>成都蓉城训练</title><article>' + '成都蓉城训练报道。' * 20 + '</article>'
    with pytest.raises(ValueError, match='missing_publication_time'):
        _extract_original(html, {'url': 'https://club.example/unknown', 'publisher': '俱乐部'},
                          datetime.fromisoformat('2026-10-13T12:00:00+08:00'))
def test_sina_original_body_excludes_sidebar_and_uses_publication_metadata():
    from sina_csl_scraper.topics.sources import _extract_original
    from datetime import datetime
    text = '<meta property="bytedance:published_time" content="2026-10-12T00:02:49+08:00">' \
        '<meta property="bytedance:updated_time" content="2026-10-13T10:00:00+08:00">' \
        '<title>蓉城赛后报道</title><div class="article-content">' \
        '<div id="artibody"><p>成都蓉城比赛报道。</p><p>记者采访主教练。</p></div>' \
        '<div class="sidebar">其它球队资讯与广告</div></div>'
    article = _extract_original(text, {'url': 'https://finance.sina.com.cn/example', 'publisher': '中国新闻网'},
                                datetime.fromisoformat('2026-10-13T08:00:00+08:00'))
    assert article.body == '成都蓉城比赛报道。\n记者采访主教练。'
    assert article.published_at.isoformat() == '2026-10-12T00:02:49+08:00'


def test_verified_complete_short_official_news_is_not_artificially_padded():
    from sina_csl_scraper.topics.sources import collect_sources
    from datetime import datetime
    body = '成都蓉城希拉谷将在十月十三日晚迎战青岛红狮，比赛地点为双流体育中心。本条消息为俱乐部发布的中乙赛程预告，比赛时间为十九点三十五分。'
    assert 40 <= len(body) < 80
    config = {'imports': [{'title': '成都蓉城希拉谷中乙主场比赛预告', 'url': 'https://club.example/preview',
        'publisher': '俱乐部', 'published_at': '2026-10-12T20:00:00+08:00', 'body': body}]}
    result = collect_sources(config, datetime.fromisoformat('2026-10-13T08:00:00+08:00'))
    assert len(result) == 1
    assert result[0].body == body
def test_ttplus_publication_and_complete_article_body_are_read_from_original():
    from sina_csl_scraper.topics.sources import _extract_original
    from datetime import datetime
    text = '<title>成都蓉城赛后观察</title><span class="articleBox-time">10-12 09:00</span>' \
        '<div class="articleBox-bd"><p>成都蓉城比赛情况。</p></div><div>推荐文章</div>'
    article = _extract_original(text, {'url': 'https://www.ttplus.cn/publish/app/data/2026/10/12/661066/os_news.html',
        'publisher': '体坛周报'}, datetime.fromisoformat('2026-10-13T08:00:00+08:00'))
    assert article.body == '成都蓉城比赛情况。'
    assert article.published_at.isoformat() == '2026-10-12T09:00:00+08:00'


def test_chinanews_publication_clock_has_known_publisher_timezone():
    from sina_csl_scraper.topics.sources import _extract_original
    from datetime import datetime
    text = '<title>成都蓉城赛况</title><span id="pubtime_baidu">2026-10-11 23:54:54</span>' \
        '<div class="left_zw"><p>成都蓉城中超赛况。</p></div>'
    article = _extract_original(text, {'url': 'https://www.chinanews.com.cn/ty/2026/10-11/10711154.shtml',
        'publisher': '中新社'}, datetime.fromisoformat('2026-10-13T08:00:00+08:00'))
    assert article.published_at.isoformat() == '2026-10-11T23:54:54+08:00'
    assert article.body == '成都蓉城中超赛况。'


def test_weibo_captures_only_complete_post_with_absolute_time():
    from sina_csl_scraper.topics.sources import _extract_original
    from datetime import datetime
    text = '<title>成都蓉城希拉谷预告</title><meta property="article:published_time" content="2026-10-10 15:38:38">' \
        '<div>17分钟前</div><div class="styles-module__hash__htmlText">成都蓉城希拉谷对阵青岛红狮。</div>' \
        '<div class="styles-module__hash__htmlText">另一个账号的推荐内容。</div>'
    article = _extract_original(text, {'url': 'https://weibo.com/2/detail/5352495851505911', 'publisher': '青岛红狮'},
                                datetime.fromisoformat('2026-10-13T08:00:00+08:00'))
    assert article.body == '成都蓉城希拉谷对阵青岛红狮。'
    assert article.published_at.isoformat() == '2026-10-10T15:38:38+08:00'
