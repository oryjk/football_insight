"""Body photographs: bounded downloads, verified originals and public publication."""
from __future__ import annotations

import hashlib
import io
import ipaddress
import json
import logging
from pathlib import Path
import socket
from urllib.parse import urljoin, urlsplit

from PIL import Image
import requests
import urllib3

from .models import SourceArticle, canonical_url

logger = logging.getLogger(__name__)
MAX_BYTES = 8 * 1024 * 1024
MAX_IMAGES = 4
Image.MAX_IMAGE_PIXELS = 40_000_000


def _public_address(value: str) -> tuple[str, str]:
    url = canonical_url(value)
    host = urlsplit(url).hostname
    if host == 'localhost' or host.endswith('.localhost'):
        raise ValueError('private image address')
    addresses = socket.getaddrinfo(host, urlsplit(url).port or (443 if url.startswith('https:') else 80), type=socket.SOCK_STREAM)
    if not addresses or any(not ipaddress.ip_address(entry[4][0]).is_global for entry in addresses):
        raise ValueError('private image address')
    return url, addresses[0][4][0]


def validate_public_url(value: str) -> str:
    return _public_address(value)[0]


def fetch_photo(url: str, source_url: str) -> bytes:
    # Redirect targets are checked before requesting them, including metadata hosts.
    for _ in range(4):
        url, address = _public_address(url)
        parts = urlsplit(url)
        # Connect to the checked IP, preserving the original Host, TLS SNI and
        # certificate hostname; a second DNS lookup cannot target a private IP.
        if parts.scheme == 'https':
            pool = urllib3.HTTPSConnectionPool(address, port=parts.port or 443,
                server_hostname=parts.hostname, assert_hostname=parts.hostname,
                cert_reqs='CERT_REQUIRED', ca_certs=requests.certs.where())
        else:
            pool = urllib3.HTTPConnectionPool(address, port=parts.port or 80)
        response = None
        try:
            path = parts.path + ('?' + parts.query if parts.query else '')
            response = pool.urlopen('GET', path, preload_content=False, redirect=False, retries=False,
                timeout=urllib3.Timeout(connect=10, read=20), headers={
                    'Host': parts.netloc, 'User-Agent': 'FootballInsight/1.0 public-news-reader', 'Referer': source_url})
            if response.status in {301, 302, 303, 307, 308}:
                url = urljoin(url, response.headers['Location'])
                continue
            if response.status != 200:
                raise ValueError('image HTTP request failed')
            if int(response.headers.get('Content-Length', '0')) > MAX_BYTES:
                raise ValueError('image too large')
            content = bytearray()
            for chunk in response.stream(64 * 1024, decode_content=True):
                content.extend(chunk)
                if len(content) > MAX_BYTES:
                    raise ValueError('image too large')
            return bytes(content)
        finally:
            if response is not None:
                response.close()
            pool.close()
    raise ValueError('too many image redirects')


def inspect_photo(content: bytes) -> tuple[int, int, str, str]:
    if not content or len(content) > MAX_BYTES:
        raise ValueError('image too large or empty')
    with Image.open(io.BytesIO(content)) as image:
        width, height = image.size
        if width < 160 or height < 100 or width * height > 40_000_000 or image.format not in {'JPEG', 'PNG', 'WEBP'}:
            raise ValueError('not a supported body photograph')
        extension, mime = {'JPEG': ('jpg', 'image/jpeg'), 'PNG': ('png', 'image/png'), 'WEBP': ('webp', 'image/webp')}[image.format]
        image.verify()
    # JPEG.verify() alone does not decode its pixel stream (truncation can pass).
    with Image.open(io.BytesIO(content)) as image:
        image.load()
    return width, height, extension, mime


class TopicImagePublisher:
    def __init__(self, *, target, cache_dir: Path, fetch=fetch_photo):
        self.target, self.cache_dir, self.fetch = target, cache_dir / 'images', fetch
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    def publish(self, url: str, source: SourceArticle, team_id: int) -> dict:
        identity = hashlib.sha256(f'{team_id}\n{source.url}\n{url}'.encode()).hexdigest()
        manifest = self.cache_dir / f'{identity}.json'
        if manifest.exists():
            return json.loads(manifest.read_text())
        content = self.fetch(url, source.url)
        width, height, extension, mime = inspect_photo(content)
        digest = hashlib.sha256(content).hexdigest()
        local = self.cache_dir / f'{digest}.{extension}'
        if not local.exists():
            temporary = local.with_suffix('.tmp')
            temporary.write_bytes(content)
            temporary.replace(local)
        public_url = self.target.upload_bytes(f'summary/images/topics/{team_id}/{digest}.{extension}', content, mime)
        parts = urlsplit(canonical_url(public_url))
        if parts.scheme != 'https':
            raise ValueError('published image requires HTTPS')
        result = {'url': public_url, 'width': width, 'height': height,
                  'source_url': source.url, 'publisher': source.publisher}
        temporary = manifest.with_suffix('.tmp')
        temporary.write_text(json.dumps(result, ensure_ascii=False))
        temporary.replace(manifest)
        return result

    def enrich(self, items: list[dict], sources: list[SourceArticle]) -> None:
        originals = {source.url: source for source in sources}
        for item in items:
            photos, seen = [], set()
            for reference in item['sources']:
                original = originals.get(reference['url'])
                if original is None or original.publisher != reference['publisher']:
                    continue
                for url in original.images:
                    if len(photos) >= MAX_IMAGES:
                        break
                    try:
                        photo = self.publish(url, original, item['team_id'])
                        if photo['url'] in seen:
                            continue
                        seen.add(photo['url'])
                        photos.append({**photo, 'after_paragraph': min(len(photos) + 1, len(item['paragraphs']))})
                    except Exception as error:
                        logger.warning('Original photo unavailable (%s); article retained', type(error).__name__)
            # A failed retry cannot clear previously published photographs.
            if photos:
                existing = item.get('images') or []
                item['images'] = (photos + [photo for photo in existing if photo.get('url') not in seen])[:MAX_IMAGES]
