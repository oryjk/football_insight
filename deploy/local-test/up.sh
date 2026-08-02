#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/../.." && pwd)
COMPOSE_FILE="$SCRIPT_DIR/compose.yaml"
TEST_DATABASE_URL='postgresql://football_test:local_test_only@127.0.0.1:55432/football_insight_test'

docker compose -f "$COMPOSE_FILE" up -d postgres redis

for _ in $(seq 1 60); do
    if docker exec football-insight-local-test-postgres \
        pg_isready -U football_test -d football_insight_test >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
docker exec football-insight-local-test-postgres \
    pg_isready -U football_test -d football_insight_test >/dev/null

(
    cd "$REPO_ROOT/football_insight_service_backend_rs"
    DATABASE_URL="$TEST_DATABASE_URL" cargo run --bin run_migrations
)

docker compose -f "$COMPOSE_FILE" up -d --build backend

for _ in $(seq 1 90); do
    if curl -fsS http://127.0.0.1:18092/api/health >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
curl -fsS http://127.0.0.1:18092/api/health
echo
"$SCRIPT_DIR/seed.sh"
echo "Local test API: http://172.16.60.233:18092/"
echo "Admin username: owner"
echo "Admin password: FootballTest2026!"
