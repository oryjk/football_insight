"""Explicit collect/generate/TTS commands, separate from league synchronization."""
from __future__ import annotations

import fcntl
import json
import os
import sys
from contextlib import contextmanager
from datetime import datetime
from pathlib import Path

import typer

from ..assets import MinioAssetTarget
from .generation import TopicGenerator
from .models import SHANGHAI, parse_timestamp
from .pipeline import NoTopicsError, generate_edition, refresh_edition_audio
from .repository import TopicRepository
from .sources import collect_sources
from .storage import SshMinioTarget
from .tts import LocalQwenTts, discover_model

app = typer.Typer(no_args_is_help=True, help='采集成都蓉城新闻、生成今日话题和本地四川话播报。')


def _now(at: str | None) -> datetime:
    # An explicit clock supports replay/testing; originals are checked against it.
    return parse_timestamp(at, now=datetime(9999, 1, 1, tzinfo=SHANGHAI)) if at else datetime.now(SHANGHAI)


def _config(path: Path) -> dict:
    value = json.loads(path.read_text(encoding='utf-8'))
    if not isinstance(value, dict):
        raise ValueError('source config must be a JSON object')
    return value


def _write(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding='utf-8')


def _tts(cache_dir: Path) -> LocalQwenTts:
    default_runner = Path(__file__).resolve().parents[3] / 'data/native-qwen/bin/football-qwen-tts'
    runner = Path(os.getenv('FI_QWEN_RUNNER', str(default_runner))).expanduser().resolve()
    if not runner.is_file():
        raise ValueError('run uv run python scripts/setup_qwen_native.py first, or set FI_QWEN_RUNNER')
    timeout = int(os.getenv('FI_QWEN_TIMEOUT_SECONDS', '1200'))
    if timeout <= 0:
        raise ValueError('FI_QWEN_TIMEOUT_SECONDS must be positive')
    return LocalQwenTts(model_path=discover_model(), runner=runner, cache_dir=cache_dir / 'audio',
                        converter=os.getenv('FI_AUDIO_CONVERTER'),
                        timeout=timeout,
                        device=os.getenv('FI_QWEN_DEVICE', 'metal' if sys.platform == 'darwin' else 'cpu'),
                        dtype=os.getenv('FI_QWEN_DTYPE', 'bf16'))


def _target() -> MinioAssetTarget | SshMinioTarget:
    host = os.getenv('FI_TOPIC_MINIO_SSH_HOST')
    if host:
        project = os.getenv('FI_TOPIC_MINIO_REMOTE_PROJECT')
        if not project:
            raise ValueError('FI_TOPIC_MINIO_REMOTE_PROJECT is required for remote audio upload')
        return SshMinioTarget(host=host, project=project)
    access, secret = os.getenv('FI_MINIO_ACCESS_KEY'), os.getenv('FI_MINIO_SECRET_KEY')
    if not access or not secret:
        raise ValueError('FI_MINIO_ACCESS_KEY and FI_MINIO_SECRET_KEY are required for upload')
    return MinioAssetTarget.from_credentials(endpoint=os.getenv('FI_MINIO_ENDPOINT', 'https://oryjk.cn:82/minio'),
        access_key=access, secret_key=secret, bucket=os.getenv('FI_MINIO_BUCKET', 'football-insight'),
        region=os.getenv('FI_MINIO_REGION', 'us-east-1'), public_base_url=os.getenv('FI_MINIO_PUBLIC_BASE_URL'))


@contextmanager
def _lock(cache_dir: Path):
    cache_dir.mkdir(parents=True, exist_ok=True)
    with (cache_dir / '.generate.lock').open('w') as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise ValueError('another topic generation is using this cache') from None
        yield


@app.command('collect')
def collect(config: Path = typer.Option(..., exists=True, dir_okay=False),
            output: Path = Path('data/topics/materials.json'), max_age_hours: int = typer.Option(72, min=1, max=168),
            at: str | None = None) -> None:
    """Fetch recent original article bodies; an empty result is reported honestly."""
    try:
        sources = collect_sources(_config(config), _now(at), max_age_hours)
        _write(output, {'imports': [dict(source.as_source(), body=source.body) for source in sources]})
        typer.echo(f'collected {len(sources)} verified original articles -> {output}')
    except Exception as error:
        typer.echo(f'collection failed ({type(error).__name__})', err=True)
        raise typer.Exit(1) from None


@app.command('generate')
def generate(config: Path = typer.Option(..., exists=True, dir_okay=False),
             output_dir: Path = Path('data/topics'), team_id: int = typer.Option(77680, min=1),
             limit: int = typer.Option(5, min=1, max=5), max_age_hours: int = typer.Option(72, min=1, max=168),
             generate_audio: bool = True, upload_audio: bool = False, write_db: bool = False,
             at: str | None = None) -> None:
    """Collect -> LLM -> local Qwen -> optional MinIO -> optional atomic database upsert."""
    try:
        base_url, model = os.getenv('FI_TOPIC_LLM_BASE_URL'), os.getenv('FI_TOPIC_LLM_MODEL')
        if not base_url or not model:
            raise ValueError('set FI_TOPIC_LLM_BASE_URL and FI_TOPIC_LLM_MODEL (TTS model cannot generate articles)')
        database_url = os.getenv('FI_DATABASE_URL') or os.getenv('DATABASE_URL')
        if write_db and not database_url:
            raise ValueError('FI_DATABASE_URL is required for --write-db')
        now = _now(at)
        with _lock(output_dir):
            sources = collect_sources(_config(config), now, max_age_hours)
            if not sources:
                raise NoTopicsError('no recent verified originals; previous edition retained')
            _write(output_dir / 'materials.json', {'imports': [dict(source.as_source(), body=source.body) for source in sources]})
            target, tts, tts_unavailable = None, None, False
            if upload_audio:
                try:
                    target = _target()
                except Exception as error:
                    typer.echo(f'audio upload unavailable ({type(error).__name__}); local audio stays pending', err=True)
            if generate_audio:
                try:
                    tts = _tts(output_dir)
                except Exception as error:
                    tts_unavailable = True
                    typer.echo(f'local TTS unavailable ({type(error).__name__}); text will be retained', err=True)
            generator = TopicGenerator(base_url=base_url, model=model, api_key=os.getenv('FI_TOPIC_LLM_API_KEY'))
            if write_db:
                TopicRepository(database_url).check_schema()
            edition = generate_edition(sources, generator=generator, tts=tts, target=target,
                                       cache_dir=output_dir, now=now, team_id=team_id, limit=limit,
                                       tts_unavailable=tts_unavailable)
            if write_db:
                TopicRepository(database_url).save(edition)
            _write(output_dir / 'latest.json', edition['payload'])
            ready = sum(item['audio']['status'] == 'ready' for item in edition['payload']['items'])
            typer.echo(f'generated {len(edition["payload"]["items"])} topics, {ready} published audio files -> {output_dir / "latest.json"}')
            if write_db:
                typer.echo('database edition upserted; Rust read API is a separate implementation')
    except (ValueError, FileNotFoundError) as error:
        typer.echo(str(error), err=True)
        raise typer.Exit(1) from None
    except Exception as error:
        typer.echo(f'generation failed ({type(error).__name__}); previous database edition retained', err=True)
        raise typer.Exit(1) from None


@app.command('tts')
def tts_command(text_file: Path = typer.Option(..., exists=True, dir_okay=False),
                output_dir: Path = Path('data/topics')) -> None:
    """Generate a local Sichuan AAC file without requiring an article LLM."""
    try:
        with _lock(output_dir):
            audio = _tts(output_dir).synthesize(text_file.read_text(encoding='utf-8'))
        typer.echo(json.dumps({'path': str(audio.path.resolve()), 'duration_seconds': audio.duration_seconds,
                              'sha256': audio.sha256, 'voice': 'eric', 'dialect': 'sichuan'}, ensure_ascii=False))
    except Exception as error:
        typer.echo(f'local TTS failed ({type(error).__name__})', err=True)
        raise typer.Exit(1) from None


@app.command('refresh-audio')
def refresh_audio_command(edition_file: Path = typer.Option(..., exists=True, dir_okay=False),
                          output_dir: Path = Path('data/topics'), write_db: bool = False) -> None:
    """Reuse a real edition, refresh each voice, and publish only a complete batch."""
    database_updated = False
    try:
        original = _config(edition_file)
        database_url = os.getenv('FI_DATABASE_URL') or os.getenv('DATABASE_URL')
        if write_db and not database_url:
            raise ValueError('FI_DATABASE_URL is required for --write-db')
        with _lock(output_dir):
            if write_db:
                TopicRepository(database_url).check_schema()
            refreshed = refresh_edition_audio(original, tts=_tts(output_dir), target=_target())
            _write(output_dir / 'editions' / f'{refreshed["id"]}.json', refreshed)
            if write_db:
                TopicRepository(database_url).save(refreshed)
                database_updated = True
            _write(output_dir / 'latest.json', refreshed['payload'])
        typer.echo(f'refreshed {len(refreshed["payload"]["items"])} independent Sichuan audio files; existing text and edition identity retained')
    except Exception as error:
        state = 'database edition updated; local finalization failed' if database_updated else 'previous database edition retained'
        typer.echo(f'audio refresh failed ({type(error).__name__}); {state}', err=True)
        raise typer.Exit(1) from None
