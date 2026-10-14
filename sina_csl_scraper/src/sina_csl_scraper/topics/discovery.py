"""Discover original article links from explicitly configured publisher lists."""
from __future__ import annotations

from html.parser import HTMLParser
import re
from urllib.parse import urljoin, urlsplit

from .models import canonical_url


class _NewsLinks(HTMLParser):
    def __init__(self, listing: dict):
        super().__init__(convert_charrefs=True)
        self.listing = listing
        self.pattern = re.compile(listing['path_pattern'])
        self.hosts = set(listing['allowed_hosts'])
        self.href = None
        self.parts = []
        self.entries = []
        self.seen = set()
        self.eligible = 0

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if tag == 'a':
            self.href = attributes.get('href')
            self.parts = [attributes.get('title', '')]
        elif self.href and tag == 'img':
            self.parts.append(attributes.get('alt', ''))

    def handle_data(self, data):
        if self.href:
            self.parts.append(data)

    def handle_endtag(self, tag):
        if tag != 'a' or not self.href:
            return
        href, self.href = self.href, None
        try:
            url = canonical_url(urljoin(self.listing['url'], href))
        except ValueError:
            return
        parts = urlsplit(url)
        if parts.hostname not in self.hosts or not self.pattern.fullmatch(parts.path):
            return
        self.eligible += 1
        title = ' '.join(''.join(self.parts).split())
        keywords = self.listing.get('keywords', ['蓉城'])
        if keywords and not any(word in title for word in keywords):
            return
        if url not in self.seen and len(self.entries) < 90:
            self.seen.add(url)
            self.entries.append({'url': url, 'title': title[:1000], 'publisher': self.listing['publisher']})


def listing_entries(text: str, listing: dict) -> list[dict]:
    parser = _NewsLinks(listing)
    parser.feed(text)
    if not parser.eligible:
        raise ValueError('listing has no recognized original article links')
    return parser.entries
