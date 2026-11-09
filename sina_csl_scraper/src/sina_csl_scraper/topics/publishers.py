"""Known public-news formats; source-specific rules stay out of the collector."""
from __future__ import annotations

from datetime import datetime, timedelta, timezone
from html import escape
import json
import re
from urllib.parse import parse_qs, urlsplit

from .models import SHANGHAI

_ZHIBO8_LABELS = ('中超', '中甲', '中乙', '中国足球', '国足', '中国男足', '留洋', '足协杯', '亚冠')


def is_article_body(attributes: dict) -> bool:
    if attributes.get('id') in {'js_content', 'artibody', 'signals'}:
        return True
    classes = attributes.get('class', '').split()
    return bool({'left_zw', 'articleBox-bd'}.intersection(classes)) or any(
        name.startswith('styles-module__') and name.endswith('__htmlText') for name in classes)


def visible_publication_times(text: str, url: str) -> list[str]:
    parts = urlsplit(url)
    if parts.hostname in {'chinanews.com.cn', 'www.chinanews.com.cn'}:
        match = re.search(r'<span\b[^>]*\bid=["\']pubtime_baidu["\'][^>]*>\s*([^<]+)', text)
        if match:
            return [datetime.strptime(match[1].strip(), '%Y-%m-%d %H:%M:%S').replace(tzinfo=SHANGHAI).isoformat()]
    if parts.hostname in {'ttplus.cn', 'www.ttplus.cn'}:
        route = re.search(r'/publish/app/data/(\d{4})/(\d{2})/(\d{2})/', parts.path)
        clock = re.search(r'class=["\']articleBox-time["\'][^>]*>\s*(\d{2}-\d{2})\s+(\d{2}:\d{2})', text)
        if route and clock and clock[1] == f'{route[2]}-{route[3]}':
            value = f'{route[1]}-{clock[1]} {clock[2]}'
            return [datetime.strptime(value, '%Y-%m-%d %H:%M').replace(tzinfo=SHANGHAI).isoformat()]
    if parts.hostname in {'news.zhibo8.com', 'www.zhibo8.com'}:
        match = re.search(r'ms-controller=["\']title_controller["\'][^>]*>\s*(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2})', text)
        if match:
            return [datetime.strptime(match[1].strip(), '%Y-%m-%d %H:%M:%S').replace(tzinfo=SHANGHAI).isoformat()]
    return []


def normalize_publication_time(value: str, url: str) -> str:
    # Weibo and Sina's secondary China-local timestamps use Shanghai time.
    # Unrecognized publishers retain strict aware-time validation in models.py.
    if urlsplit(url).hostname in {'weibo.com', 'www.weibo.com', 'sports.sina.com.cn', 'm.dongqiudi.com', 'www.dongqiudi.com', 'api.dongqiudi.com'} and re.fullmatch(r'\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}', value):
        return datetime.strptime(value, '%Y-%m-%d %H:%M:%S').replace(tzinfo=SHANGHAI).isoformat()
    return value


def _zhibo8_rolling_html(url: str, session, now: datetime | None = None) -> tuple[str, str]:
    reference = (now or datetime.now(SHANGHAI)).astimezone(SHANGHAI)
    dates = [(reference - timedelta(days=offset)).strftime('%Y-%m-%d') for offset in (0, 1)]
    links: list[str] = []
    for day in dates:
        response = session.get(f'https://news.zhibo8.com/zuqiu/json/{day}.htm',
                               timeout=30, headers={'User-Agent': 'Mozilla/5.0 FootballInsight/1.0'})
        if getattr(response, 'status_code', 200) == 404:
            continue
        response.raise_for_status()
        if len(response.text) > 2_000_000:
            continue
        payload = json.loads(response.text)
        for item in payload.get('video_arr', []):
            if not isinstance(item, dict):
                continue
            labels = str(item.get('lable') or '')
            if not any(tag in labels for tag in _ZHIBO8_LABELS):
                continue
            href = str(item.get('url') or '').strip()
            title = str(item.get('title') or item.get('indextitle') or '').strip()
            if href.startswith('/zuqiu/') and href.endswith('.htm') and title:
                links.append(f'<a href="https://news.zhibo8.com{escape(href)}">{escape(title)}</a>')
            if len(links) >= 90:
                break
        if len(links) >= 90:
            break
    return '<html><body>' + '\n'.join(links) + '</body></html>', url


def _dongqiudi_listing_html(url: str, session) -> tuple[str, str]:
    response = session.get(url, timeout=30, headers={'User-Agent': 'Mozilla/5.0 FootballInsight/1.0'})
    response.raise_for_status()
    if len(response.text) > 2_000_000:
        raise ValueError('listing document too large')
    payload = json.loads(response.text)
    links: list[str] = []
    for item in payload.get('articles', []):
        if not isinstance(item, dict):
            continue
        aid = str(item.get('id') or '').strip()
        title = str(item.get('title') or '').strip()
        share = str(item.get('share') or '')
        if re.fullmatch(r'\d{1,12}', aid) and title and ('/article/' in share or 'articleId=' in str(item.get('url') or '')):
            links.append(f'<a href="https://m.dongqiudi.com/article/{aid}.html">{escape(title)}</a>')
    return '<html><body>' + '\n'.join(links) + '</body></html>', url


def _dongqiudi_article_html(article_id: str, url: str, session) -> tuple[str, str]:
    response = session.get(f'https://api.dongqiudi.com/v2/article/detail/{article_id}',
                           timeout=30, headers={'User-Agent': 'Mozilla/5.0 FootballInsight/1.0'})
    response.raise_for_status()
    if len(response.text) > 2_000_000:
        raise ValueError('article document too large')
    data = json.loads(response.text).get('data') or {}
    title, detail, published = data.get('title'), data.get('body'), data.get('show_time')
    if not isinstance(title, str) or not isinstance(detail, str) or not isinstance(published, (int, float)):
        raise ValueError('invalid Dongqiudi article metadata')
    timestamp = datetime.fromtimestamp(float(published), timezone.utc).astimezone(SHANGHAI).isoformat()
    canonical = f'https://m.dongqiudi.com/article/{article_id}.html'
    html = f'<title>{escape(title)}</title><meta property="article:published_time" content="{timestamp}"><article>{detail}</article>'
    return html, canonical


def public_json_document(url: str, session, now: datetime | None = None) -> tuple[str, str] | None:
    """Read known publisher JSON feeds/details into canonical HTML."""
    parts = urlsplit(url)
    if parts.hostname == 'news.zhibo8.com' and parts.path == '/zuqiu/json/rolling.json':
        return _zhibo8_rolling_html(url, session, now=now)
    if parts.hostname == 'api.dongqiudi.com' and re.fullmatch(r'/app/tabs/iphone/\d+\.json', parts.path):
        return _dongqiudi_listing_html(url, session)
    if parts.hostname in {'m.dongqiudi.com', 'www.dongqiudi.com'}:
        match = re.fullmatch(r'/articles?/(\d{1,12})(?:\.html)?', parts.path)
        if match:
            return _dongqiudi_article_html(match[1], url, session)
    if parts.hostname != 'www.shobserver.com' or parts.path != '/staticsg/res/html/web/newsDetail.html':
        return None
    article_id = parse_qs(parts.query).get('id', [''])[0]
    if not re.fullmatch(r'\d{1,12}', article_id):
        raise ValueError('invalid Shangguan article identifier')
    response = session.get(f'https://www.shobserver.com/staticsg/data/web/detail/{article_id}.json',
                           timeout=30, headers={'User-Agent': 'FootballInsight/1.0 public-news-reader'})
    response.raise_for_status()
    if len(response.text) > 2_000_000:
        raise ValueError('article document too large')
    data = json.loads(response.text)['data']
    title, detail, published = data['title'], data['detail'], data['publishtime']
    if not isinstance(title, str) or not isinstance(detail, str) or not isinstance(published, (int, float)):
        raise ValueError('invalid Shangguan article metadata')
    timestamp = datetime.fromtimestamp(published / 1000, timezone.utc).isoformat()
    html = f'<title>{escape(title)}</title><meta property="article:published_time" content="{timestamp}"><article>{detail}</article>'
    return html, url
