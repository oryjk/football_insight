#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
CONTROL_SOCKET="/tmp/football-insight-admin-db-tunnel-${UID}.sock"

close_tunnel() {
    if ssh -S "$CONTROL_SOCKET" -O check peiqian >/dev/null 2>&1; then
        ssh -S "$CONTROL_SOCKET" -O exit peiqian >/dev/null 2>&1 || true
    fi
    rm -f -- "$CONTROL_SOCKET"
}

trap close_tunnel EXIT
LOCAL_ADMIN_DATABASE_URL=postgresql://unused \
    docker compose -f "$SCRIPT_DIR/compose.yaml" down
