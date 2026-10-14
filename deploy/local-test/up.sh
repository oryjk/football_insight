#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/../.." && pwd)
COMPOSE_FILE="$SCRIPT_DIR/compose.yaml"
BACKEND_DIR="$REPO_ROOT/football_insight_service_backend_rs"
SERVICE=football-insight-local-backend.service

for command_name in cargo docker curl systemctl systemd-analyze python3; do
    command -v "$command_name" >/dev/null || { echo "required command not found: $command_name" >&2; exit 1; }
done
if [[ ! -f "$BACKEND_DIR/.env" ]]; then
    echo "Configure $BACKEND_DIR/.env first; see deploy/local-test/README.md" >&2
    exit 1
fi
chmod 600 "$BACKEND_DIR/.env"

# Compile before interrupting the running backend. Starting never runs migrations.
(cd "$BACKEND_DIR" && cargo build --bin football_insight_service_backend_rs)
systemctl --user start football-insight-db-tunnel.service
docker compose -f "$COMPOSE_FILE" up -d redis

python3 - "$BACKEND_DIR" "$(command -v cargo)" "$SERVICE" <<'PY'
import json
import pathlib
import sys

backend, cargo, service = sys.argv[1:]
def quote(value):
    return json.dumps(value, ensure_ascii=False).replace('%', '%%')
unit_dir = pathlib.Path.home() / '.config/systemd/user'
unit_dir.mkdir(parents=True, exist_ok=True)
(unit_dir / service).write_text(f'''[Unit]
Description=Football Insight local233 Cargo backend (18092)
Wants=football-insight-db-tunnel.service
After=football-insight-db-tunnel.service

[Service]
Type=simple
WorkingDirectory={backend.replace('%', '%%')}
ExecStart={quote(cargo)} run --bin football_insight_service_backend_rs
Environment="PORT=18092"
Environment={quote('PATH=' + str(pathlib.Path(cargo).parent) + ':/usr/local/bin:/usr/bin:/bin')}
Restart=on-failure
RestartSec=3
KillMode=control-group
TimeoutStopSec=20

[Install]
WantedBy=default.target
''')
PY

systemd-analyze --user verify "$HOME/.config/systemd/user/$SERVICE"
systemctl --user daemon-reload
systemctl --user enable "$SERVICE"

# One-time transition from the former local Docker backend; keep it for rollback.
if [[ "$(docker inspect -f '{{.State.Running}}' football-insight-local-test-backend 2>/dev/null || true)" == true ]]; then
    docker update --restart=no football-insight-local-test-backend >/dev/null
    docker stop football-insight-local-test-backend >/dev/null
fi
if ! systemctl --user is-active --quiet "$SERVICE"; then
    python3 - <<'PY'
import socket
with socket.socket() as probe:
    try:
        probe.bind(('0.0.0.0', 18092))
    except OSError:
        raise SystemExit('Port 18092 is occupied by another process; stop the foreground backend first.')
PY
fi
systemctl --user restart "$SERVICE"

for _ in $(seq 1 60); do
    if curl -fsS http://127.0.0.1:18092/api/health >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
systemctl --user is-active "$SERVICE"
curl -fsS http://127.0.0.1:18092/api/health
echo
echo "Local Cargo API: http://172.16.60.233:18092/"
echo "Database: production football_data through a private SSH tunnel"
echo "Logs: journalctl --user -u $SERVICE -f"
