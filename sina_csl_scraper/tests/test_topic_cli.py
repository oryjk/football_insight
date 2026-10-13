import json
import pytest

from typer.testing import CliRunner

from sina_csl_scraper.cli import app


def test_generate_reports_missing_llm_without_touching_tts_or_database(tmp_path, monkeypatch):
    config = tmp_path / 'sources.json'
    config.write_text(json.dumps({'imports': []}))
    monkeypatch.delenv('FI_TOPIC_LLM_BASE_URL', raising=False)
    monkeypatch.delenv('FI_TOPIC_LLM_MODEL', raising=False)
    result = CliRunner().invoke(app, ['topics', 'generate', '--config', str(config)])
    assert result.exit_code != 0
    assert 'FI_TOPIC_LLM_BASE_URL' in result.output


def test_collect_outputs_verified_originals_and_never_relabels_stale_articles(tmp_path):
    config = tmp_path / 'sources.json'
    config.write_text(json.dumps({'imports': [{'title': '成都蓉城训练消息', 'url': 'https://club.example/1',
        'publisher': '俱乐部', 'published_at': '2026-10-12T20:00:00+08:00', 'body': '成都蓉城今天进行训练，球员认真准备比赛。' * 15}]}))
    output = tmp_path / 'collected.json'
    result = CliRunner().invoke(app, ['topics', 'collect', '--config', str(config), '--output', str(output),
                                     '--at', '2026-10-13T08:00:00+08:00'])
    assert result.exit_code == 0, result.output
    data = json.loads(output.read_text())
    assert len(data['imports']) == 1
    assert data['imports'][0]['published_at'] == '2026-10-12T20:00:00+08:00'


def test_generate_must_not_write_empty_results_to_db(tmp_path, monkeypatch):
    config = tmp_path / 'sources.json'
    config.write_text(json.dumps({'imports': []}))
    monkeypatch.setenv('FI_TOPIC_LLM_BASE_URL', 'http://localhost:8000/v1')
    monkeypatch.setenv('FI_TOPIC_LLM_MODEL', 'test')
    monkeypatch.setenv('FI_DATABASE_URL', 'postgresql://never-connect')
    result = CliRunner().invoke(app, ['topics', 'generate', '--config', str(config), '--output-dir', str(tmp_path / 'cache'), '--no-generate-audio', '--write-db'])
    assert result.exit_code != 0
    assert 'no recent' in result.output


@pytest.mark.parametrize('audio_flags, expected_status', [([], 'failed'), (['--no-generate-audio', '--upload-audio'], 'pending')])
def test_unavailable_audio_setup_keeps_generated_text(tmp_path, monkeypatch, audio_flags, expected_status):
    from sina_csl_scraper.topics import cli
    config = tmp_path / 'sources.json'
    config.write_text(json.dumps({'imports': [{'title': '成都蓉城训练消息', 'url': 'https://club.example/1',
        'publisher': '俱乐部', 'published_at': '2026-10-12T20:00:00+08:00', 'body': '成都蓉城今天进行训练，球员认真准备比赛。' * 15}]}))
    monkeypatch.setenv('FI_TOPIC_LLM_BASE_URL', 'http://localhost:8000/v1')
    monkeypatch.setenv('FI_TOPIC_LLM_MODEL', 'test')
    monkeypatch.setenv('FI_QWEN_RUNNER', str(tmp_path / 'missing-runner'))
    monkeypatch.delenv('FI_MINIO_ACCESS_KEY', raising=False)
    monkeypatch.delenv('FI_MINIO_SECRET_KEY', raising=False)
    class Generator:
        def __init__(self, **kwargs):
            self.model = kwargs['model']
        def generate(self, sources, team_id, now, limit):
            return [{'id': '1', 'team_id': team_id, 'title': '蓉城训练', 'summary': '球队备战',
                'paragraphs': ['成都蓉城备战比赛。'], 'published_at': now.isoformat(), 'sources': [sources[0].as_source()],
                'audio': {'status': 'pending', 'url': None, 'dialect': 'sichuan', 'duration_seconds': None}}]
    monkeypatch.setattr(cli, 'TopicGenerator', Generator)
    result = CliRunner().invoke(app, ['topics', 'generate', '--config', str(config), '--output-dir', str(tmp_path / 'cache'),
                                    '--at', '2026-10-13T08:00:00+08:00', *audio_flags])
    assert result.exit_code == 0, result.output
    data = json.loads((tmp_path / 'cache/latest.json').read_text())
    assert data['items'][0]['title'] == '蓉城训练'
    assert data['items'][0]['audio']['status'] == expected_status
