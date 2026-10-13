import json
import subprocess
from types import SimpleNamespace

import pytest

from sina_csl_scraper.topics.storage import SshMinioTarget


def test_remote_upload_streams_bytes_and_preserves_key_without_fetching_credentials():
    calls = []
    def run(args, **kwargs):
        calls.append((args, kwargs))
        return SimpleNamespace(stdout=b'https://oryjk.cn:82/minio/football-insight/summary/audio/topics/a.m4a\n')
    target = SshMinioTarget(host='local233', project='/home/betalpha/projects/football_insight/sina_csl_scraper', run=run)
    url = target.upload_bytes('summary/audio/topics/a.m4a', b'example audio', 'audio/mp4')
    assert url == 'https://oryjk.cn:82/minio/football-insight/summary/audio/topics/a.m4a'
    args, options = calls[0]
    assert args[0] == 'ssh'
    assert options['input'] == b'example audio'
    assert 'summary/audio/topics/a.m4a' in args[-1]
    assert 'FI_MINIO_SECRET_KEY' in args[-1]


def test_failed_remote_upload_does_not_report_an_unuploaded_url():
    def run(*args, **kwargs):
        raise subprocess.CalledProcessError(1, 'ssh', stderr=b'private error')
    target = SshMinioTarget(host='local233', project='/srv/scraper', run=run)
    with pytest.raises(RuntimeError, match='remote MinIO upload failed') as error:
        target.upload_bytes('summary/audio/topics/a.m4a', b'audio', 'audio/mp4')
    assert 'private' not in str(error.value)


def test_remote_destination_and_object_path_are_constrained():
    with pytest.raises(ValueError):
        SshMinioTarget(host='-oProxyCommand=unexpected', project='/srv/scraper')
    target = SshMinioTarget(host='local233', project='/srv/scraper')
    with pytest.raises(ValueError):
        target.upload_bytes('../private/file', b'audio', 'audio/mp4')
