#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/../.." && pwd)
COMPOSE_FILE="$SCRIPT_DIR/compose.yaml"
PRODUCTION_HOST=peiqian
PRODUCTION_BACKEND_CONTAINER=football-insight-service-backend-rs
CONTROL_SOCKET="/tmp/football-insight-admin-db-tunnel-${UID}.sock"
tunnel_started=false

cleanup_on_error() {
    local status=$?
    if ((status != 0)) && [[ "$tunnel_started" == true ]]; then
        ssh -S "$CONTROL_SOCKET" -O exit "$PRODUCTION_HOST" >/dev/null 2>&1 || true
        rm -f -- "$CONTROL_SOCKET"
    fi
    exit "$status"
}

trap cleanup_on_error EXIT

production_database_url=$(
    ssh -o BatchMode=yes "$PRODUCTION_HOST" \
        "docker inspect --format '{{range .Config.Env}}{{println .}}{{end}}' '$PRODUCTION_BACKEND_CONTAINER'" \
        | sed -n 's/^DATABASE_URL=//p' \
        | head -n 1
)

if [[ -z "$production_database_url" ]]; then
    echo "Production DATABASE_URL was not found in $PRODUCTION_BACKEND_CONTAINER." >&2
    exit 1
fi
if [[ "$production_database_url" != *"@127.0.0.1:5432/"* ]]; then
    echo "Production DATABASE_URL does not use the expected private PostgreSQL endpoint." >&2
    exit 1
fi

if ! ssh -S "$CONTROL_SOCKET" -O check "$PRODUCTION_HOST" >/dev/null 2>&1; then
    rm -f -- "$CONTROL_SOCKET"
    ssh -M -S "$CONTROL_SOCKET" -fnNT \
        -o BatchMode=yes \
        -o ExitOnForwardFailure=yes \
        -L 127.0.0.1:55433:127.0.0.1:5432 \
        -L 172.17.0.1:55434:127.0.0.1:5432 \
        "$PRODUCTION_HOST"
    tunnel_started=true
fi

migration_database_url=${production_database_url/@127.0.0.1:5432/@127.0.0.1:55433}
export LOCAL_ADMIN_DATABASE_URL=${production_database_url/@127.0.0.1:5432/@host.docker.internal:55434}
unset production_database_url

(
    cd "$REPO_ROOT/football_insight_service_backend_rs"
    DATABASE_URL="$migration_database_url" cargo run --bin run_migrations
)
unset migration_database_url

docker compose -f "$COMPOSE_FILE" up -d redis
docker compose -f "$COMPOSE_FILE" up -d --build backend

for _ in $(seq 1 90); do
    if curl -fsS http://127.0.0.1:18092/api/health >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
curl -fsS http://127.0.0.1:18092/api/health
echo
echo "Local admin API: http://172.16.60.233:18092/"
echo "Database: production football_data through a private SSH tunnel"
echo "Admin username: admin"
echo "Admin password: admin123"
trap - EXIT
