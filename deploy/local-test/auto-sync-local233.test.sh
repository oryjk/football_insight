#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
SYNC_SCRIPT="$SCRIPT_DIR/auto-sync-local233.sh"

TEST_TMPDIR=${TMPDIR:-/tmp}
TEST_ROOT=$(mktemp -d "${TEST_TMPDIR%/}/football-insight-auto-sync-test.XXXXXX")
TEST_ROOT=$(cd -- "$TEST_ROOT" && pwd -P)
trap 'rm -rf -- "$TEST_ROOT"' EXIT

ORIGIN_REPO="$TEST_ROOT/origin.git"
LOCAL_REPO="$TEST_ROOT/local"
WORK_REPO="$TEST_ROOT/work"
FAKE_BIN="$TEST_ROOT/bin"
COMMAND_LOG="$TEST_ROOT/commands.log"

mkdir -p "$FAKE_BIN"
git init --bare -b main "$ORIGIN_REPO" >/dev/null

git init -b main "$WORK_REPO" >/dev/null
git -C "$WORK_REPO" config user.name "Test Agent"
git -C "$WORK_REPO" config user.email "test@example.com"
mkdir -p \
    "$WORK_REPO/deploy/local-test" \
    "$WORK_REPO/football_insight_service_backend_rs/src" \
    "$WORK_REPO/football_insight_service_backend_rs/migrations" \
    "$WORK_REPO/football_insight_mini/src/config" \
    "$WORK_REPO/football_insight_mini/scripts"

cp "$SYNC_SCRIPT" "$WORK_REPO/deploy/local-test/auto-sync-local233.sh"
chmod +x "$WORK_REPO/deploy/local-test/auto-sync-local233.sh"

cat >"$WORK_REPO/deploy/local-test/up.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'up.sh executed\n' >>"$COMMAND_LOG"
EOF
chmod +x "$WORK_REPO/deploy/local-test/up.sh"

cat >"$WORK_REPO/football_insight_mini/src/manifest.json" <<'EOF'
{
  "versionName": "1.1.0",
  "versionCode": "10100"
}
EOF
cat >"$WORK_REPO/football_insight_mini/src/config/generatedMiniProgramVersion.ts" <<'EOF'
export const MINI_PROGRAM_VERSION = "1.1.0";
export const MINI_PROGRAM_VERSION_CODE = "10100";
EOF
cat >"$WORK_REPO/football_insight_mini/scripts/sync-manifest-version.mjs" <<'EOF'
// stub
EOF

git -C "$WORK_REPO" add -A
git -C "$WORK_REPO" commit -m "chore: initial commit [gemini]" >/dev/null
git -C "$WORK_REPO" remote add origin "$ORIGIN_REPO"
git -C "$WORK_REPO" push -u origin main >/dev/null

git clone "$ORIGIN_REPO" "$LOCAL_REPO" >/dev/null 2>&1
git -C "$LOCAL_REPO" config user.name "Local233 Sync"
git -C "$LOCAL_REPO" config user.email "local233@example.com"

cat >"$FAKE_BIN/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >>"$COMMAND_LOG"
EOF

cat >"$FAKE_BIN/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'curl %s\n' "$*" >>"$COMMAND_LOG"
printf '{"status":"ok"}'
EOF

cat >"$FAKE_BIN/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gh %s\n' "$*" >>"$COMMAND_LOG"
EOF

cat >"$FAKE_BIN/bun" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'bun %s\n' "$*" >>"$COMMAND_LOG"
if [[ "$*" == *"scripts/sync-manifest-version.mjs"* && "${FAKE_ALLOCATE_NEW_VERSION:-0}" == "1" ]]; then
    cat >"$PWD/src/manifest.json" <<'JSON'
{
  "versionName": "1.1.1",
  "versionCode": "10101"
}
JSON
    cat >"$PWD/src/config/generatedMiniProgramVersion.ts" <<'TS'
export const MINI_PROGRAM_VERSION = "1.1.1";
export const MINI_PROGRAM_VERSION_CODE = "10101";
TS
fi
EOF

chmod +x "$FAKE_BIN/cargo" "$FAKE_BIN/curl" "$FAKE_BIN/gh" "$FAKE_BIN/bun"
export PATH="$FAKE_BIN:$PATH"
export COMMAND_LOG

# Case 1: Backend + migration + mini changes in one push, with new review version allocation
echo "-- migration" >"$WORK_REPO/football_insight_service_backend_rs/migrations/20261118000000_test.sql"
echo "// backend" >"$WORK_REPO/football_insight_service_backend_rs/src/lib.rs"
echo "// mini" >"$WORK_REPO/football_insight_mini/src/app.ts"
git -C "$WORK_REPO" add -A
git -C "$WORK_REPO" commit -m "feat: full-stack change [gemini]" >/dev/null
git -C "$WORK_REPO" push origin main >/dev/null
PUSHED_SHA=$(git -C "$WORK_REPO" rev-parse HEAD)

FAKE_ALLOCATE_NEW_VERSION=1 "$LOCAL_REPO/deploy/local-test/auto-sync-local233.sh" --once

grep -q "cargo run --bin run_migrations" "$COMMAND_LOG"
grep -q "up.sh executed" "$COMMAND_LOG"
grep -q "curl -fsS http://127.0.0.1:18092/api/health" "$COMMAND_LOG"
grep -q "bun run mp:release -- --robot 2 --desc feat: full-stack change \[gemini\]" "$COMMAND_LOG"
grep -q "gh api repos/oryjk/football_insight/statuses/${PUSHED_SHA} -f state=success -f context=local233/backend-18092" "$COMMAND_LOG"
grep -q "gh api repos/oryjk/football_insight/statuses/${PUSHED_SHA} -f state=success -f context=local233/mp-release-robot2" "$COMMAND_LOG"

# Verify the allocated version commit was pushed to origin/main and a subsequent tick does not re-trigger mp:release
NEW_HEAD=$(git -C "$LOCAL_REPO" rev-parse HEAD)
ORIGIN_HEAD=$(git -C "$ORIGIN_REPO" rev-parse main)
[[ "$NEW_HEAD" == "$ORIGIN_HEAD" ]]
[[ "$NEW_HEAD" != "$PUSHED_SHA" ]]

: >"$COMMAND_LOG"
"$LOCAL_REPO/deploy/local-test/auto-sync-local233.sh" --once
if grep -q "mp:release" "$COMMAND_LOG"; then
    echo "expected no duplicate mp:release when already in sync" >&2
    exit 1
fi

echo "auto-sync-local233.sh tests passed"
