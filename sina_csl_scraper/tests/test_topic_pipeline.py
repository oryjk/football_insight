from datetime import datetime
from pathlib import Path

import pytest

from sina_csl_scraper.topics.models import SourceArticle
from sina_csl_scraper.topics.pipeline import NoTopicsError, generate_edition
from sina_csl_scraper.topics.tts import GeneratedAudio

NOW = datetime.fromisoformat('2026-10-13T08:00:00+08:00')


def material():
    return [SourceArticle('成都蓉城训练', 'https://club.example/news/1', '俱乐部', NOW, '成都蓉城今天进行赛前训练。' * 10)]


class Generator:
    model = 'test-model'
    base_url = 'http://localhost:8000/v1'
    calls = 0

    def generate(self, sources, team_id, now, limit):
        self.calls += 1
        return [{'id': 'article-1', 'team_id': team_id, 'title': '蓉城备战', 'summary': '训练消息',
                 'paragraphs': ['成都蓉城今天进行了训练。'], 'published_at': now.isoformat(),
                 'sources': [sources[0].as_source()],
                 'audio': {'status': 'pending', 'url': None, 'dialect': 'sichuan', 'duration_seconds': None}}]


def test_same_material_reuses_article_and_published_audio(tmp_path):
    generator = Generator()
    audio = tmp_path / 'voice.m4a'
    audio.write_bytes(b'audio')
    class Tts:
        calls = 0
        def synthesize(self, text):
            self.calls += 1
            assert text == '蓉城备战。\n成都蓉城今天进行了训练。'
            return GeneratedAudio(audio, 2.5, 'audiohash')
    class Target:
        calls = 0
        def upload_bytes(self, object_name, content, content_type):
            self.calls += 1
            assert content_type == 'audio/mp4'
            assert object_name.startswith('summary/audio/topics/77680/2026-10-13/')
            return 'https://oryjk.cn:82/minio/football-insight/' + object_name
    tts, target = Tts(), Target()
    first = generate_edition(material(), generator=generator, tts=tts, target=target, cache_dir=tmp_path, now=NOW)
    second = generate_edition(material(), generator=generator, tts=tts, target=target, cache_dir=tmp_path, now=NOW)
    assert first == second
    assert first['payload']['items'][0]['audio']['status'] == 'ready'
    assert generator.calls == tts.calls == target.calls == 1


def test_failed_tts_preserves_article_and_retries_without_llm(tmp_path):
    generator = Generator()
    class BrokenTts:
        calls = 0
        def synthesize(self, text):
            self.calls += 1
            raise RuntimeError('native inference failed')
    tts = BrokenTts()
    for _ in range(2):
        edition = generate_edition(material(), generator=generator, tts=tts, target=None, cache_dir=tmp_path, now=NOW)
        assert edition['payload']['items'][0]['title'] == '蓉城备战'
        assert edition['payload']['items'][0]['audio']['status'] == 'failed'
    assert generator.calls == 1
    assert tts.calls == 2


def test_empty_generation_does_not_replace_previous_published_cache(tmp_path):
    generator = Generator()
    previous = generate_edition(material(), generator=generator, tts=None, target=None, cache_dir=tmp_path, now=NOW)
    class Empty(Generator):
        model = 'other-model'
        def generate(self, *args, **kwargs):
            return []
    with pytest.raises(NoTopicsError):
        generate_edition(material(), generator=Empty(), tts=None, target=None, cache_dir=tmp_path, now=NOW)
    assert previous['payload']['items'][0]['id'] == 'article-1'
    assert len(list(tmp_path.glob('editions/*.json'))) == 1


def test_http_audio_url_is_not_published_as_ready(tmp_path):
    audio = tmp_path / 'voice.m4a'
    audio.write_bytes(b'audio')
    class Tts:
        def synthesize(self, text):
            return GeneratedAudio(audio, 1, 'hash')
    class Target:
        def upload_bytes(self, *args):
            return 'http://insecure.example/voice.m4a'
    edition = generate_edition(material(), generator=Generator(), tts=Tts(), target=Target(), cache_dir=tmp_path, now=NOW)
    assert edition['payload']['items'][0]['audio']['status'] == 'failed'
    assert edition['payload']['items'][0]['audio']['url'] is None


def test_interrupted_audio_resumes_without_regenerating_articles(tmp_path):
    import json
    class TwoTopics(Generator):
        def generate(self, sources, team_id, now, limit):
            first = super().generate(sources, team_id, now, limit)[0]
            return [first, dict(first, id='article-2', title='蓉城赛程')]
    generator = TwoTopics()
    path = tmp_path / 'voice.m4a'
    path.write_bytes(b'audio')
    class Tts:
        calls = 0
        def synthesize(self, text):
            self.calls += 1
            if self.calls == 2:
                raise KeyboardInterrupt()
            return GeneratedAudio(path, 2.5, 'audiohash')
    class Target:
        calls = 0
        def upload_bytes(self, name, content, content_type):
            self.calls += 1
            return 'https://oryjk.cn:82/minio/football-insight/' + name
    tts, target = Tts(), Target()
    with pytest.raises(KeyboardInterrupt):
        generate_edition(material(), generator=generator, tts=tts, target=target, cache_dir=tmp_path, now=NOW)
    saved = json.loads(next((tmp_path / 'editions').glob('*.json')).read_text())
    assert saved['payload']['items'][0]['audio']['status'] == 'ready'
    assert saved['payload']['items'][1]['audio']['status'] == 'pending'
    edition = generate_edition(material(), generator=generator, tts=tts, target=target, cache_dir=tmp_path, now=NOW)
    assert all(item['audio']['status'] == 'ready' for item in edition['payload']['items'])
    assert generator.calls == 1
    assert target.calls == 2
