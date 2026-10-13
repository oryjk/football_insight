"""Fetch configured public originals, never search snippets or feed summaries."""
from __future__ import annotations

from datetime import datetime, timedelta, timezone
from html.parser import HTMLParser
import hashlib
import json
import logging
import re
from urllib.parse import urljoin
import xml.etree.ElementTree as ET

import requests

from .models import SourceArticle, canonical_url, parse_timestamp
from .discovery import listing_entries
from .publishers import is_article_body, normalize_publication_time, public_json_document, visible_publication_times

logger = logging.getLogger(__name__)
MAX_SOURCES = 30
MAX_DOWNLOADS = 90
MAX_DOCUMENT_CHARS = 2_000_000
MIN_BODY_CHARS = 40


def _valid_original_body(article: SourceArticle) -> bool:
    return (MIN_BODY_CHARS <= len(article.body.strip()) <= 100_000
            and len(article.title.strip()) + len(article.body.strip()) >= 80)


class _ArticleHTML(HTMLParser):
    _void = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}
    _ignored = {"script", "style", "nav", "aside", "footer", "header", "noscript", "button", "form"}

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.stack: list[tuple[str, bool, bool]] = []
        self.body_parts: list[str] = []
        self.images: list[str] = []
        self.title_parts: list[str] = []
        self.heading_parts: list[str] = []
        self.meta_title = ""
        self.dates: list[str] = []
        self.article_dates: list[str] = []
        self.jsonld_parts: list[str] = []
        self.jsonld_documents: list[str] = []
        self.in_jsonld = False
        self.explicit_body_found = False
        self.semantic_body_found = False

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if tag == "meta":
            key = (attributes.get("property") or attributes.get("name") or attributes.get("itemprop") or "").lower()
            value = attributes.get("content") or ""
            if key in {"article:published_time", "bytedance:published_time", "datepublished", "pubdate", "publishdate", "date", "dc.date.issued"}:
                self.dates.append(value)
            elif key in {"og:title", "twitter:title"}:
                self.meta_title = value
        if tag == "script" and attributes.get("type", "").lower() == "application/ld+json":
            self.in_jsonld = True
            self.jsonld_parts = []
        parent_capture = self.stack[-1][1] if self.stack else False
        parent_ignore = self.stack[-1][2] if self.stack else False
        if tag == "time" and attributes.get("datetime") and parent_capture and not parent_ignore:
            self.article_dates.append(attributes["datetime"])
        explicit = is_article_body(attributes)
        if explicit and not self.explicit_body_found:
            self.explicit_body_found = True
            self.body_parts = []
            self.images = []
            # A precise publisher root replaces a broader main/article shell.
            self.stack = [(name, False, ignored) for name, _, ignored in self.stack]
            capture = True
        else:
            semantic = tag in {"article", "main"} and not self.semantic_body_found and not self.explicit_body_found
            capture = parent_capture or semantic
            if semantic:
                self.semantic_body_found = True
        ignored = parent_ignore or tag in self._ignored
        if capture and not ignored and tag == 'img':
            image = attributes.get('data-src') or attributes.get('data-original') or attributes.get('src')
            if image and len(self.images) < 8:
                self.images.append(image)
        if capture and not ignored and tag in {"p", "div", "br", "h1", "h2", "li"}:
            self.body_parts.append("\n")
        if tag not in self._void:
            self.stack.append((tag, capture, ignored))

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in self._void:
            self.handle_endtag(tag)

    def handle_endtag(self, tag):
        if tag == "script" and self.in_jsonld:
            self.jsonld_documents.append("".join(self.jsonld_parts))
            self.in_jsonld = False
        for index in range(len(self.stack)-1, -1, -1):
            if self.stack[index][0] == tag:
                del self.stack[index:]
                break
        if tag in {"p", "div", "h1", "h2", "li"}:
            self.body_parts.append("\n")

    def handle_data(self, data):
        if self.in_jsonld:
            self.jsonld_parts.append(data)
        if self.stack and self.stack[-1][0] == "title":
            self.title_parts.append(data)
        if any(frame[0] == "h1" and frame[1] and not frame[2] for frame in self.stack):
            self.heading_parts.append(data)
        if self.stack and self.stack[-1][1] and not self.stack[-1][2]:
            self.body_parts.append(data)

    @property
    def body(self):
        lines = (line.strip() for line in "".join(self.body_parts).splitlines())
        return "\n".join(line for line in lines if line and not re.fullmatch(r'【编辑[:：].*】', line))

    @property
    def title(self):
        return self.meta_title.strip() or "".join(self.title_parts).strip() or "".join(self.heading_parts).strip()


def _jsonld_dates(value):
    if isinstance(value, dict):
        if isinstance(value.get("datePublished"), str):
            yield value["datePublished"]
        for child in value.values():
            yield from _jsonld_dates(child)
    elif isinstance(value, list):
        for child in value:
            yield from _jsonld_dates(child)


def _extract_original(text: str, item: dict, now: datetime) -> SourceArticle:
    parser = _ArticleHTML()
    parser.feed(text)
    dates = list(parser.dates) or visible_publication_times(text, item['url'])
    for document in parser.jsonld_documents:
        try:
            dates.extend(_jsonld_dates(json.loads(document)))
        except (ValueError, TypeError, RecursionError):
            continue
    match = re.search(r"\b(?:var\s+)?ct\s*=\s*['\"]?(\d{10})['\"]?", text)
    if match:
        dates.append(datetime.fromtimestamp(int(match[1]), timezone.utc).isoformat())
    # Original metadata and article-scoped clocks precede feed/config fallbacks.
    if not dates:
        dates.extend(parser.article_dates)
    if not dates and item.get("published_at"):
        dates.append(item["published_at"])
    if not dates:
        raise ValueError("missing_publication_time")
    publication = parse_timestamp(normalize_publication_time(dates[0], item['url']), now=now)
    images = []
    for value in parser.images:
        try:
            images.append(canonical_url(urljoin(item['url'], value)))
        except ValueError:
            continue
    return SourceArticle(parser.title or item.get("title", ""), item["url"], item["publisher"], publication, parser.body, tuple(images))


def _fetch_text(session, url: str) -> tuple[str, str]:
    url = canonical_url(url)
    document = public_json_document(url, session)
    if document:
        return document
    response = session.get(url, timeout=30, headers={"User-Agent": "FootballInsight/1.0 public-news-reader"})
    response.raise_for_status()
    if getattr(response, "encoding", None) in {None, "ISO-8859-1"}:
        response.encoding = "utf-8"
    text = response.text
    if len(text) > MAX_DOCUMENT_CHARS:
        raise ValueError("document_too_large")
    return text, canonical_url(getattr(response, "url", None) or url)


def _feed_entries(text: str, feed: dict):
    root = ET.fromstring(text)
    for entry in root.iter():
        tag = entry.tag.rsplit("}", 1)[-1]
        if tag not in {"item", "entry"}:
            continue
        fields = {}
        for child in entry:
            key = child.tag.rsplit("}", 1)[-1]
            if key == "link":
                if tag == "item":
                    fields["url"] = (child.text or "").strip()
                elif child.attrib.get("rel", "alternate") == "alternate":
                    fields["url"] = child.attrib.get("href", "")
            elif key == "title":
                fields["title"] = "".join(child.itertext()).strip()
            elif key in {"pubDate", "published"}:
                fields["published_at"] = (child.text or "").strip()
        if fields.get("url"):
            yield {**fields, "url": urljoin(feed["url"], fields["url"]), "publisher": feed["publisher"]}


def collect_sources(config: dict, now: datetime, max_age_hours: int = 72, *, session=None,
                    require_available: bool = False) -> list[SourceArticle]:
    """Return at most 30 recent full originals; warn safely per failed source."""
    if now.tzinfo is None or now.utcoffset() is None or max_age_hours <= 0:
        raise ValueError("Collection requires aware now and positive maximum age")
    if not isinstance(config, dict):
        raise ValueError("Source configuration must be an object")
    session = session or requests.Session()
    accepted: list[SourceArticle] = []
    seen_urls: set[str] = set()
    seen_bodies: set[str] = set()
    attempted_urls: set[str] = set()
    cutoff = now - timedelta(hours=max_age_hours)

    def accept(article):
        if not cutoff <= article.published_at <= now:
            return
        if not _valid_original_body(article) or "蓉城" not in article.title + article.body:
            return
        body_hash = hashlib.sha256(" ".join(article.body.split()).encode()).hexdigest()
        if article.url not in seen_urls and body_hash not in seen_bodies:
            seen_urls.add(article.url)
            seen_bodies.add(body_hash)
            accepted.append(article)

    for item in config.get("imports", [])[:MAX_DOWNLOADS]:
        try:
            accept(SourceArticle(item["title"], item["url"], item["publisher"], parse_timestamp(item["published_at"], now), item["body"], tuple(item.get('images') or [])))
        except (ValueError, TypeError, KeyError):
            logger.warning("Skipped imported news: invalid original or publication metadata")
        if len(accepted) >= MAX_SOURCES:
            return accepted

    candidates = list(config.get("articles", [])[:MAX_DOWNLOADS])
    available = 0
    for feed in config.get("feeds", [])[:20]:
        try:
            text, _ = _fetch_text(session, feed["url"])
            entries = list(_feed_entries(text, feed))[:MAX_DOWNLOADS]
            keywords = feed.get('keywords')
            candidates.extend(item for item in entries if not keywords or any(word in item.get('title', '') for word in keywords))
            available += 1
        except (requests.RequestException, ValueError, TypeError, KeyError, ET.ParseError):
            logger.warning("Skipped news feed: unavailable or invalid feed")

    for listing in config.get('listings', [])[:20]:
        try:
            text, resolved = _fetch_text(session, listing['url'])
            candidates.extend(listing_entries(text, {**listing, 'url': resolved}))
            available += 1
        except (requests.RequestException, ValueError, TypeError, KeyError, re.error):
            logger.warning('Skipped news listing: unavailable or unrecognized publisher list')
    if require_available and (config.get('listings') or config.get('feeds')) and not available:
        raise ValueError('all configured news discovery sources unavailable')

    original_attempts = original_successes = 0
    for item in candidates:
        if len(accepted) >= MAX_SOURCES or len(attempted_urls) >= MAX_DOWNLOADS:
            break
        try:
            url = canonical_url(item["url"])
            if url in attempted_urls or url in seen_urls:
                continue
            attempted_urls.add(url)
            if item.get("published_at") and parse_timestamp(item["published_at"], now) < cutoff:
                continue
            original_attempts += 1
            text, resolved_url = _fetch_text(session, url)
            article = _extract_original(text, {**item, "url": resolved_url}, now)
            original_successes += int(_valid_original_body(article))
            accept(article)
        except (requests.RequestException, ValueError, TypeError, KeyError):
            logger.warning("Skipped original news: unavailable, invalid body, or publication metadata")
    if require_available and original_attempts and not original_successes:
        raise ValueError('all discovered original bodies unavailable or unverifiable')
    return accepted
