#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/../.." && pwd)
DEPLOY_SCRIPT="$REPO_ROOT/deploy_local233.sh"

TEST_TMPDIR=${TMPDIR:-/tmp}
TEST_ROOT=$(mktemp -d "${TEST_TMPDIR%/}/football-insight-deploy-test.XXXXXX")
TEST_ROOT=$(cd -- "$TEST_ROOT" && pwd -P)
trap 'rm -rf -- "$TEST_ROOT"' EXIT

FIXTURE_REPO="$TEST_ROOT/repo"
REMOTE_REPO="$TEST_ROOT/remote-repo"
FAKE_BIN="$TEST_ROOT/bin"
COMMAND_LOG="$TEST_ROOT/commands.log"
DEPLOY_MARKER="$TEST_ROOT/deployed"
PULL_MARKER="$TEST_ROOT/pulled"

mkdir -p "$FIXTURE_REPO/.git" "$FIXTURE_REPO/deploy/local-test" "$REMOTE_REPO/.git" "$REMOTE_REPO/deploy/local-test" "$FAKE_BIN"
cp "$DEPLOY_SCRIPT" "$FIXTURE_REPO/deploy_local233.sh"
chmod +x "$FIXTURE_REPO/deploy_local233.sh"

cat >"$FAKE_BIN/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "-C" ]]; then
    git_dir=$2
    shift 2
else
    git_dir=$PWD
fi

printf 'git[%s] %s\n' "$git_dir" "$*" >>"$COMMAND_LOG"

case "${1:-} ${2:-}" in
    "status --porcelain")
        if [[ "$git_dir" == "$FIXTURE_REPO" && "${FAKE_LOCAL_DIRTY:-0}" == "1" ]]; then
            printf ' M local-change.txt\n'
        elif [[ "$git_dir" == "$REMOTE_REPO" && "${FAKE_REMOTE_DIRTY:-0}" == "1" ]]; then
            printf ' M remote-change.txt\n'
        fi
        ;;
    "branch --show-current")
        printf 'feature/test\n'
        ;;
    "check-ref-format --branch")
        ;;
    "rev-parse HEAD")
        if [[ "$git_dir" == "$FIXTURE_REPO" && "${FAKE_LOCAL_BEHIND:-0}" == "1" && ! -e "$PULL_MARKER" ]]; then
            printf '0000000000000000000000000000000000000000\n'
        else
            printf '1111111111111111111111111111111111111111\n'
        fi
        ;;
    "rev-parse origin/feature/test")
        printf '1111111111111111111111111111111111111111\n'
        ;;
    "show-ref --verify")
        exit 1
        ;;
    "pull --ff-only")
        touch "$PULL_MARKER"
        ;;
    "fetch origin"|"checkout -b")
        ;;
    *)
        printf 'unexpected git command: %s\n' "$*" >&2
        exit 90
        ;;
esac
EOF

cat >"$FAKE_BIN/ssh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

while (($# > 0)); do
    case "$1" in
        -o)
            shift 2
            ;;
        *)
            break
            ;;
    esac
done

host=$1
shift
printf 'ssh[%s] %s\n' "$host" "$*" >>"$COMMAND_LOG"
exec /bin/bash -c "$*"
EOF

cat >"$FAKE_BIN/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'curl %s\n' "$*" >>"$COMMAND_LOG"
printf '{"status":"ok"}'
EOF

cat >"$FAKE_BIN/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'docker %s\n' "$*" >>"$COMMAND_LOG"
if [[ "${1:-}" == "ps" ]]; then
    printf 'football-insight-local-test-backend Up healthy\n'
fi
EOF

cat >"$REMOTE_REPO/deploy/local-test/up.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
touch "$DEPLOY_MARKER"
EOF

chmod +x "$FAKE_BIN/git" "$FAKE_BIN/ssh" "$FAKE_BIN/curl" "$FAKE_BIN/docker"
chmod +x "$REMOTE_REPO/deploy/local-test/up.sh"
cp "$REMOTE_REPO/deploy/local-test/up.sh" "$FIXTURE_REPO/deploy/local-test/up.sh"

export PATH="$FAKE_BIN:/usr/bin:/bin"
export COMMAND_LOG FIXTURE_REPO REMOTE_REPO DEPLOY_MARKER PULL_MARKER

help_output=$(cd "$TEST_ROOT" && "$FIXTURE_REPO/deploy_local233.sh" --help)
[[ "$help_output" == *"DEPLOY_HOST"* ]]
[[ "$help_output" == *"DEPLOY_REPO_DIR"* ]]

if FAKE_LOCAL_DIRTY=1 DEPLOY_REPO_DIR="$REMOTE_REPO" "$FIXTURE_REPO/deploy_local233.sh" >"$TEST_ROOT/dirty.out" 2>&1; then
    echo "expected dirty local worktree to block deployment" >&2
    exit 1
fi
grep -q "working tree is not clean" "$TEST_ROOT/dirty.out"
[[ ! -e "$DEPLOY_MARKER" ]]

if FAKE_REMOTE_DIRTY=1 DEPLOY_REPO_DIR="$REMOTE_REPO" "$FIXTURE_REPO/deploy_local233.sh" >"$TEST_ROOT/remote-dirty.out" 2>&1; then
    echo "expected dirty remote worktree to block deployment" >&2
    exit 1
fi
grep -q "remote working tree is not clean" "$TEST_ROOT/remote-dirty.out"
[[ ! -e "$DEPLOY_MARKER" ]]

DEPLOY_REPO_DIR="$REMOTE_REPO" "$FIXTURE_REPO/deploy_local233.sh" >"$TEST_ROOT/success.out"

[[ -e "$DEPLOY_MARKER" ]]
grep -q "git\[$REMOTE_REPO\] checkout -b feature/test origin/feature/test" "$COMMAND_LOG"
grep -q "git\[$REMOTE_REPO\] pull --ff-only origin feature/test" "$COMMAND_LOG"
grep -q "curl -fsS http://127.0.0.1:18092/api/health" "$COMMAND_LOG"
grep -q "curl -fsS http://127.0.0.1:18092/api/v1/ticket-watch/regions" "$COMMAND_LOG"
grep -q "curl -fsS http://127.0.0.1:18092/api/v1/seat-swap/current" "$COMMAND_LOG"

rm -f "$DEPLOY_MARKER" "$PULL_MARKER"
ssh_count_before=$(grep -c '^ssh\[' "$COMMAND_LOG" || true)
FAKE_LOCAL_BEHIND=1 DEPLOY_REPO_DIR="$FIXTURE_REPO" "$FIXTURE_REPO/deploy_local233.sh" >"$TEST_ROOT/local-target.out"
ssh_count_after=$(grep -c '^ssh\[' "$COMMAND_LOG" || true)

[[ -e "$DEPLOY_MARKER" ]]
[[ -e "$PULL_MARKER" ]]
[[ "$ssh_count_after" == "$ssh_count_before" ]]

echo "deploy_local233.sh tests passed"
