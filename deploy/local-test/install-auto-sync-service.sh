#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/../.." && pwd)
SYNC_SCRIPT="$SCRIPT_DIR/auto-sync-local233.sh"
SERVICE=football-insight-auto-sync.service

for command_name in git systemctl systemd-analyze python3; do
    command -v "$command_name" >/dev/null || { echo "required command not found: $command_name" >&2; exit 1; }
done

chmod +x "$SYNC_SCRIPT"

USER_PATH="$HOME/.bun/bin:$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"

python3 - "$REPO_ROOT" "$SYNC_SCRIPT" "$SERVICE" "$USER_PATH" <<'PY'
import json
import pathlib
import sys

repo_root, sync_script, service, user_path = sys.argv[1:]
def quote(value):
    return json.dumps(value, ensure_ascii=False).replace('%', '%%')

unit_dir = pathlib.Path.home() / '.config/systemd/user'
unit_dir.mkdir(parents=True, exist_ok=True)
(unit_dir / service).write_text(f'''[Unit]
Description=Football Insight local233 auto-sync (18092 backend + mp:release robot 2)
After=network-online.target football-insight-db-tunnel.service

[Service]
Type=simple
WorkingDirectory={repo_root.replace('%', '%%')}
ExecStart={quote(sync_script)}
Environment="AUTO_SYNC_INTERVAL_SECONDS=5"
Environment={quote('PATH=' + user_path)}
Restart=always
RestartSec=5
KillMode=control-group
TimeoutStopSec=30

[Install]
WantedBy=default.target
''')
PY

systemd-analyze --user verify "$HOME/.config/systemd/user/$SERVICE"
systemctl --user daemon-reload
systemctl --user enable "$SERVICE"
systemctl --user restart "$SERVICE"
systemctl --user is-active "$SERVICE"

echo "Installed and started $SERVICE"
echo "View live logs: journalctl --user -u $SERVICE -f"

if ! command -v gh >/dev/null 2>&1; then
    echo "NOTE: 'gh' (GitHub CLI) is not installed on this machine yet."
    echo "Install gh and run 'gh auth login' so local233 can report sync/upload status back to cloud agents."
elif ! gh auth status >/dev/null 2>&1; then
    echo "NOTE: 'gh' is installed but not logged in. Run 'gh auth login' to enable GitHub status reporting."
else
    echo "GitHub CLI (gh) is authenticated; commit status reporting is active."
fi
