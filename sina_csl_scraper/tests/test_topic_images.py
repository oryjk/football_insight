from datetime import datetime
from dataclasses import replace
import io

import pytest
from PIL import Image

from sina_csl_scraper.topics.models import SourceArticle, source_fingerprint
from sina_csl_scraper.topics.sources import _extract_original

NOW = datetime.fromisoformat('2026-10-13T12:00:00+08:00')
URL = 'https://www.ttplus.cn/publish/app/data/2026/10/11/660765/os_news.html'
PHOTO = 'https://resource.ttplus.cn/photo.jpg'


def source():
    return SourceArticle('成都蓉城赛况', URL, '体坛周报', NOW, '成都蓉城赛后观察。' * 10)


def test_extract_body_photos_not_sidebar_or_avatar_and_keep_identity():
    html = '<title>蓉城</title><span class="articleBox-time">10-11 17:30</span>' \
        '<main><img src="/logo.jpg"><div class="articleBox-bd"><p>成都蓉城正文</p>' \
        f'<img src="{PHOTO}"><img data-src="/second.jpg"><aside><img src="/ad.jpg"></aside>' \
        '</div><img src="/avatar.jpg"></main>'
    article = _extract_original(html, {'url': URL, 'publisher': '体坛周报'}, NOW)
    assert article.images == (PHOTO, 'https://www.ttplus.cn/second.jpg')
    assert source_fingerprint([article]) == source_fingerprint([replace(article, images=())])
    assert article.as_material()['images'] == list(article.images)
    assert 'images' not in article.as_source()


def photo_bytes(size=(640, 444)):
    stream = io.BytesIO()
    Image.new('RGB', size, 'green').save(stream, 'JPEG')
    return stream.getvalue()


def test_publish_only_cited_photos_cache_upload_and_preserve_fields(tmp_path):
    from sina_csl_scraper.topics.images import TopicImagePublisher
    article = replace(source(), images=(PHOTO, PHOTO))
    downloads, uploads = [], []
    def fetch(url, source_url):
        downloads.append((url, source_url))
        return photo_bytes()
    class Target:
        def upload_bytes(self, name, content, content_type):
            uploads.append(name)
            assert content_type == 'image/jpeg'
            return 'https://oryjk.cn:82/minio/football-insight/' + name
    publisher = TopicImagePublisher(target=Target(), cache_dir=tmp_path, fetch=fetch)
    item = {'id': 'topic', 'team_id': 77680, 'paragraphs': ['段落一', '段落二'],
            'sources': [article.as_source()], 'audio': {'status': 'ready'}, 'heat': 123}
    uncited = replace(article, url='https://www.ttplus.cn/other', images=('https://resource.ttplus.cn/other.jpg',))
    publisher.enrich([item], [article, uncited])
    publisher.enrich([item], [article, uncited])
    assert len(item['images']) == 1
    assert item['images'][0]['width'] == 640
    assert item['images'][0]['source_url'] == URL
    assert item['images'][0]['after_paragraph'] == 1
    assert item['audio'] == {'status': 'ready'} and item['heat'] == 123
    assert len(downloads) == len(uploads) == 1
    assert len(list((tmp_path / 'images').glob('*.jpg'))) == 1


@pytest.mark.parametrize('content', [b'<html>not an image</html>', photo_bytes((20, 20)), b'x' * (8 * 1024 * 1024 + 1)], ids=['html', 'tiny', 'oversized'])
def test_bad_photo_preserves_existing_images_and_readable_topic(tmp_path, content):
    from sina_csl_scraper.topics.images import TopicImagePublisher
    article = replace(source(), images=(PHOTO,))
    class Target:
        def upload_bytes(self, *args):
            pytest.fail('invalid photo must never upload')
    item = {'id': 'topic', 'team_id': 77680, 'paragraphs': ['正文'],
            'sources': [article.as_source()], 'images': [{'url': 'old-photo'}]}
    TopicImagePublisher(target=Target(), cache_dir=tmp_path, fetch=lambda *args: content).enrich([item], [article])
    assert item['images'] == [{'url': 'old-photo'}]
    assert item['paragraphs'] == ['正文']


@pytest.mark.parametrize('url', ['http://127.0.0.1/x', 'https://169.254.169.254/x', 'http://localhost/a', 'https://user:secret@public.example/x'])
def test_image_public_url_rejects_private_addresses(url):
    from sina_csl_scraper.topics.images import validate_public_url
    with pytest.raises(ValueError):
        validate_public_url(url)


def test_truncated_jpeg_with_valid_dimensions_is_not_publishable():
    from sina_csl_scraper.topics.images import inspect_photo
    with pytest.raises(OSError):
        inspect_photo(photo_bytes()[:-100])


def test_pipeline_image_cache_failure_cannot_block_voice_or_text(tmp_path, monkeypatch):
    from sina_csl_scraper.topics import pipeline
    from test_topic_pipeline import Generator
    class BrokenPublisher:
        def __init__(self, **kwargs):
            raise OSError('cache is read-only')
    monkeypatch.setattr(pipeline, 'TopicImagePublisher', BrokenPublisher)
    edition = pipeline.generate_edition([replace(source(), images=(PHOTO,))], generator=Generator(),
        tts=None, target=object(), cache_dir=tmp_path, now=NOW)
    assert edition['payload']['items'][0]['paragraphs'] == ['成都蓉城今天进行了训练。']
    assert edition['payload']['items'][0]['audio']['status'] == 'pending'


def test_download_pins_checked_ip_with_original_tls_hostname(monkeypatch):
    from sina_csl_scraper.topics import images
    resolutions, connections = [], []
    def resolve(*args, **kwargs):
        resolutions.append(args[0])
        # A second lookup would return a private address.
        return [(2, 1, 6, '', ('93.184.216.34' if len(resolutions) == 1 else '127.0.0.1', 443))]
    class Response:
        status = 200
        headers = {}
        def stream(self, *args, **kwargs):
            yield photo_bytes()
        def close(self): pass
    class Pool:
        def __init__(self, host, **kwargs):
            connections.append(host)
            assert kwargs['server_hostname'] == kwargs['assert_hostname'] == 'photos.example.com'
        def urlopen(self, method, path, **kwargs):
            assert kwargs['headers']['Host'] == 'photos.example.com'
            return Response()
        def close(self): pass
    monkeypatch.setattr(images.socket, 'getaddrinfo', resolve)
    monkeypatch.setattr(images.urllib3, 'HTTPSConnectionPool', Pool)
    assert images.fetch_photo('https://photos.example.com/a.jpg', URL) == photo_bytes()
    assert resolutions == ['photos.example.com']
    assert connections == ['93.184.216.34']
