import json
import os
from pathlib import Path
import subprocess
import sys

import pytest


# 每日话题每小时追加一次：07–23 点每个整点都跑，单次不超过 55 分钟。
@pytest.mark.parametrize('hour,remaining,expected', [('06', 600, None), ('07', 57600, 3300),
                                                    ('22', 1800, 3300), ('23', 86400, 3300)])
def test_scheduled_window_and_hourly_timeout(tmp_path, hour, remaining, expected):
    root = Path(__file__).resolve().parents[1]
    script = tmp_path / 'update_daily_topics.sh'
    script.write_bytes((root / 'update_daily_topics.sh').read_bytes()); script.chmod(0o755)
    tools = tmp_path / 'bin'; tools.mkdir()
    fake_date = tools / 'date'
    fake_date.write_text(f'''#!{sys.executable}
import os,sys
args=sys.argv[1:]
if args==['+%H']:print(os.environ['TOPIC_TEST_HOUR'])
elif '-d' in args:print(100000)
elif args==['+%s']:print(100000-int(os.environ['TOPIC_TEST_REMAINING']))
else:print('2026-10-13T22:30:00+08:00')
'''); fake_date.chmod(0o755)
    timeout = tools / 'timeout'
    timeout.write_text(f'''#!{sys.executable}
import json,os,sys
from pathlib import Path
Path(os.environ['TOPIC_TEST_RESULT']).write_text(json.dumps(sys.argv[1:]))
'''); timeout.chmod(0o755)
    result_path = tmp_path / 'result.json'
    env = dict(os.environ, PATH=f'{tools}:/usr/bin:/bin', TOPIC_TEST_HOUR=hour,
               TOPIC_TEST_REMAINING=str(remaining), TOPIC_TEST_RESULT=str(result_path))
    result = subprocess.run([str(script), '--scheduled'], env=env, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    if expected is None:
        assert not result_path.exists()
    else:
        args = json.loads(result_path.read_text())
        assert args[1] == f'{expected}s'
        assert '--scheduled' not in args
        assert 'roundup-update' in args and '--write-db' in args
