"""Upload through an existing scraper host without exporting its MinIO credentials."""
from __future__ import annotations

import re
import shlex
import subprocess
from pathlib import PurePosixPath
from typing import Callable

_REMOTE_UPLOAD = '''import os,sys
from sina_csl_scraper.assets import MinioAssetTarget
access,secret=os.getenv("FI_MINIO_ACCESS_KEY"),os.getenv("FI_MINIO_SECRET_KEY")
if not access or not secret:
    raise SystemExit("MinIO credentials missing")
target=MinioAssetTarget.from_credentials(
    endpoint=os.getenv("FI_MINIO_ENDPOINT","https://oryjk.cn:82/minio"),
    access_key=access,secret_key=secret,
    bucket=os.getenv("FI_MINIO_BUCKET","football-insight"),
    region=os.getenv("FI_MINIO_REGION","us-east-1"),
    public_base_url=os.getenv("FI_MINIO_PUBLIC_BASE_URL"))
print(target.upload_bytes(sys.argv[1],sys.stdin.buffer.read(),sys.argv[2]))
'''


class SshMinioTarget:
    def __init__(self, *, host: str, project: str, run: Callable = subprocess.run):
        if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.@-]*', host):
            raise ValueError('invalid SSH host')
        if not project.startswith('/') or '\n' in project:
            raise ValueError('remote scraper project must be an absolute path')
        self.host, self.project, self.run = host, project, run

    def upload_bytes(self, object_name: str, content: bytes, content_type: str) -> str:
        if not object_name.startswith(('summary/audio/topics/', 'summary/images/topics/')) or '..' in PurePosixPath(object_name).parts:
            raise ValueError('topic assets must use public summary/audio/topics or summary/images/topics prefixes')
        command = (f'cd {shlex.quote(self.project)} && set -a && . ./.env.sync && set +a && '
                   f'uv run --no-sync python -c {shlex.quote(_REMOTE_UPLOAD)} '
                   f'{shlex.quote(object_name)} {shlex.quote(content_type)}')
        try:
            result = self.run(['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=10', self.host, command],
                              input=content, capture_output=True, check=True, timeout=120)
        except (subprocess.SubprocessError, OSError):
            raise RuntimeError('remote MinIO upload failed; credentials remain on the scraper host') from None
        url = result.stdout.decode('utf-8').strip()
        if not re.fullmatch(r'https://[^\s]+', url):
            raise RuntimeError('remote MinIO upload returned no valid public URL')
        return url
