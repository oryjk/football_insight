"""Manual local233 entry point; credentials stay in private files/process memory."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
from urllib.parse import urlsplit, urlunsplit


def host_database_url(value: str) -> str:
    """The Docker gateway tunnel is bound to 172.17.0.1, not host loopback."""
    parsed = urlsplit(value)
    if parsed.scheme not in {'postgres', 'postgresql'} or parsed.path != '/football_data':
        raise ValueError('expected the backend football_data PostgreSQL database')
    if parsed.hostname == 'host.docker.internal':
        if parsed.port != 55434:
            raise ValueError('unexpected Docker gateway database tunnel')
        credentials, separator, _ = parsed.netloc.rpartition('@')
        netloc = (credentials + separator if separator else '') + '172.17.0.1:55434'
        return urlunsplit(parsed._replace(netloc=netloc))
    return value


def backend_database_url() -> str:
    result = subprocess.run(
        ['docker', 'inspect', '--format', '{{json .Config.Env}}', 'football-insight-local-test-backend'],
        capture_output=True, text=True, check=False,
    )
    if result.returncode:
        raise ValueError('cannot read the local backend database configuration')
    environment = dict(value.split('=', 1) for value in json.loads(result.stdout) if '=' in value)
    if not environment.get('DATABASE_URL'):
        raise ValueError('local backend DATABASE_URL is missing')
    return host_database_url(environment['DATABASE_URL'])


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    arguments = sys.argv[1:]
    loaded = arguments[:1] == ['--env-loaded']
    if loaded:
        arguments = arguments[1:]
    if not arguments or arguments[0] not in {'collect', 'generate', 'tts', 'refresh-audio'}:
        print('Usage: uv run --no-sync python scripts/run_topics_local.py {collect|generate|tts|refresh-audio} [topic options]')
        raise SystemExit(0 if arguments[:1] in [['--help'], ['-h']] else 2)
    if not loaded:
        files = [root / '.env.topics', root / '.env.sync']
        if not all(path.is_file() for path in files):
            raise SystemExit('Local .env.topics and .env.sync are required; do not commit these files.')
        command = ['uv', 'run', '--no-sync']
        for path in files:
            command.extend(['--env-file', str(path)])
        command.extend(['python', str(Path(__file__).resolve()), '--env-loaded', *arguments])
        os.chdir(root)
        os.execvp('uv', command)
    # This is local233 itself: upload through MinioAssetTarget, never SSH to self.
    os.environ.pop('FI_TOPIC_MINIO_SSH_HOST', None)
    os.environ.pop('FI_TOPIC_MINIO_REMOTE_PROJECT', None)
    if '--write-db' in arguments:
        try:
            os.environ['FI_DATABASE_URL'] = backend_database_url()
        except (ValueError, TypeError, KeyError):
            raise SystemExit('Cannot safely determine the local backend football_data connection.') from None
    os.chdir(root)
    sys.argv = ['sina-csl-scraper', 'topics', *arguments]
    from sina_csl_scraper.cli import main as cli_main
    cli_main()


if __name__ == '__main__':
    main()
