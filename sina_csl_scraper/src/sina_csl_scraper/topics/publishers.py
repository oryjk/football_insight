"""Known public-news formats; source-specific rules stay out of the collector."""
from __future__ import annotations

from datetime import datetime, timezone
from html import escape
import json
import re
from urllib.parse import parse_qs, urlsplit

from .models import SHANGHAI


def is_article_body(attributes: dict) -> bool:
    if attributes.get('id') in {'js_content', 'artibody'}:
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
    return []


def normalize_publication_time(value: str, url: str) -> str:
    # Weibo's absolute article metadata uses its publisher's China-local clock.
    # Unrecognized publishers retain strict aware-time validation in models.py.
    if urlsplit(url).hostname in {'weibo.com', 'www.weibo.com'} and re.fullmatch(r'\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}', value):
        return datetime.strptime(value, '%Y-%m-%d %H:%M:%S').replace(tzinfo=SHANGHAI).isoformat()
    return value


def public_json_document(url: str, session) -> tuple[str, str] | None:
    """Read Shangguan's public article JSON, not a search snippet or JS shell."""
    parts = urlsplit(url)
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
