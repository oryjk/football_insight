#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
systemctl --user stop football-insight-local-backend.service
docker compose -f "$SCRIPT_DIR/compose.yaml" stop redis
echo "Local Cargo backend and Redis stopped; shared database tunnel remains managed by systemd."
