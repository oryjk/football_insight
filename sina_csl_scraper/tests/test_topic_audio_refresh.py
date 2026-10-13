import copy
import json
from pathlib import Path

import pytest
from typer.testing import CliRunner

from sina_csl_scraper.cli import app
from sina_csl_scraper.topics import pipeline
from sina_csl_scraper.topics.tts import GeneratedAudio


def existing_edition():
    return {'id': 'b' * 64, 'source_fingerprint': 'c' * 64, 'generator_version': 'daily-topics-v2:k3-256k',
            'payload': {'team_id': 77680, 'edition_date': '2026-10-13', 'updated_at': '2026-10-13T08:30:00+08:00',
                        'items': [{'id': char * 64, 'team_id': 77680, 'title': '蓉城消息', 'summary': '原有摘要',
                                   'paragraphs': ['成都蓉城备战比赛。'], 'published_at': '2026-10-13T08:30:00+08:00',
                                   'sources': [{'title': '原文', 'url': 'https://example.com/news', 'publisher': '媒体',
                                                'published_at': '2026-10-12T20:00:00+08:00'}],
                                   'audio': {'status': 'ready', 'url': 'https://example.com/old.m4a',
                                             'dialect': 'sichuan', 'duration_seconds': 5}} for char in ['a', 'd']]}}


class Audio:
    def __init__(self, path: Path, fail_second=False):
        self.path, self.calls, self.fail_second = path, 0, fail_second

    def synthesize(self, text):
        self.calls += 1
        if self.fail_second and self.calls == 2:
            raise RuntimeError('native audio failed')
        return GeneratedAudio(self.path, 3.5, 'e' * 64)


class Target:
    def upload_bytes(self, name, data, content_type):
        assert content_type == 'audio/mp4'
        return 'https://example.com/' + name


def test_refreshes_each_audio_without_changing_existing_text_identity_or_metadata(tmp_path):
    old = existing_edition()
    original = copy.deepcopy(old)
    audio = tmp_path / 'voice.m4a'
    audio.write_bytes(b'audio')
    updated = pipeline.refresh_edition_audio(old, tts=Audio(audio), target=Target())
    assert old == original
    for index in range(2):
        item = updated['payload']['items'][index]
        assert item['audio']['status'] == 'ready'
        assert item['audio']['url'] == f'https://example.com/summary/audio/topics/77680/2026-10-13/{original["payload"]["items"][index]["id"]}/{"e" * 64}.m4a'
        item['audio'] = original['payload']['items'][index]['audio']
    assert updated == original


def test_partial_native_failure_does_not_mutate_the_original_published_edition(tmp_path):
    old = existing_edition()
    original = copy.deepcopy(old)
    audio = tmp_path / 'voice.m4a'
    audio.write_bytes(b'audio')
    with pytest.raises(RuntimeError):
        pipeline.refresh_edition_audio(old, tts=Audio(audio, fail_second=True), target=Target())
    assert old == original


def test_refresh_cli_needs_no_llm_and_never_writes_partial_audio_to_database(tmp_path, monkeypatch):
    from sina_csl_scraper.topics import cli
    path = tmp_path / 'edition.json'
    path.write_text(json.dumps(existing_edition()))
    audio = tmp_path / 'voice.m4a'
    audio.write_bytes(b'audio')
    monkeypatch.delenv('FI_TOPIC_LLM_API_KEY', raising=False)
    monkeypatch.delenv('FI_TOPIC_LLM_BASE_URL', raising=False)
    monkeypatch.setenv('FI_DATABASE_URL', 'postgresql://not-used')
    monkeypatch.setattr(cli, '_tts', lambda _: Audio(audio, fail_second=True))
    monkeypatch.setattr(cli, '_target', lambda: Target())
    writes = []

    class Repository:
        def __init__(self, _): pass
        def check_schema(self): pass
        def save(self, edition): writes.append(edition)

    monkeypatch.setattr(cli, 'TopicRepository', Repository)
    result = CliRunner().invoke(app, ['topics', 'refresh-audio', '--edition-file', str(path),
                                     '--output-dir', str(tmp_path / 'cache'), '--write-db'])
    assert result.exit_code != 0
    assert writes == []
    assert 'audio refresh failed' in result.output


def test_post_commit_cache_failure_reports_that_database_was_updated(tmp_path, monkeypatch):
    from sina_csl_scraper.topics import cli
    path = tmp_path / 'edition.json'
    path.write_text(json.dumps(existing_edition()))
    audio = tmp_path / 'voice.m4a'
    audio.write_bytes(b'audio')
    monkeypatch.setenv('FI_DATABASE_URL', 'postgresql://not-used')
    monkeypatch.setattr(cli, '_tts', lambda _: Audio(audio))
    monkeypatch.setattr(cli, '_target', lambda: Target())
    writes = []

    class Repository:
        def __init__(self, _): pass
        def check_schema(self): pass
        def save(self, edition): writes.append(edition)

    original_write = cli._write
    def fail_latest(path, data):
        if path.name == 'latest.json':
            raise OSError('cache unavailable')
        original_write(path, data)
    monkeypatch.setattr(cli, 'TopicRepository', Repository)
    monkeypatch.setattr(cli, '_write', fail_latest)
    result = CliRunner().invoke(app, ['topics', 'refresh-audio', '--edition-file', str(path),
                                     '--output-dir', str(tmp_path / 'cache'), '--write-db'])
    assert result.exit_code != 0
    assert len(writes) == 1
    assert 'database edition updated' in result.output
    assert 'previous database edition retained' not in result.output
