#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/../.." && pwd)

usage() {
    cat <<'EOF'
Wait for local233 to pull a pushed commit, restart the 18092 backend, and/or
upload the WeChat Mini Program experience version (reported via GitHub Commit Status).

Usage:
  ./deploy/local-test/wait-local233-sync.sh [commit_sha] [--timeout <seconds>]

Environment variables:
  GH_REPO   GitHub owner/repo (default: oryjk/football_insight)
EOF
}

TARGET_SHA=""
TIMEOUT=300

while (($# > 0)); do
    case "$1" in
        --timeout)
            TIMEOUT=$2
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            TARGET_SHA=$1
            shift
            ;;
    esac
done

GH_REPO=${GH_REPO:-oryjk/football_insight}
if [[ -z "$TARGET_SHA" ]]; then
    TARGET_SHA=$(git -C "$REPO_ROOT" rev-parse HEAD)
fi

echo "Waiting for local233 status on commit ${TARGET_SHA:0:7} (timeout=${TIMEOUT}s)..."
deadline=$((SECONDS + TIMEOUT))
last_summary=""

while ((SECONDS < deadline)); do
    status_json=$(gh api "repos/${GH_REPO}/commits/${TARGET_SHA}/status" 2>/dev/null || echo "{}")
    parsed=$(python3 - "$status_json" <<'PY'
import json
import sys

data = json.loads(sys.argv[1] or "{}")
statuses = [
    s for s in data.get("statuses", [])
    if str(s.get("context", "")).startswith("local233/")
]
if not statuses:
    print("WAITING|No local233 status reported yet...")
    raise SystemExit(0)

parts = [f"{s['context']}={s['state']} ({s.get('description', '')})" for s in statuses]
summary = "; ".join(parts)
states = {s["state"] for s in statuses}
if "failure" in states or "error" in states:
    print(f"FAILURE|{summary}")
elif "pending" in states:
    print(f"PENDING|{summary}")
else:
    print(f"SUCCESS|{summary}")
PY
)
    overall=${parsed%%|*}
    summary=${parsed#*|}

    if [[ "$summary" != "$last_summary" ]]; then
        printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$summary"
        last_summary="$summary"
    fi

    case "$overall" in
        SUCCESS)
            echo "local233 completed all tasks for ${TARGET_SHA:0:7}."
            exit 0
            ;;
        FAILURE)
            echo "local233 reported a failure for ${TARGET_SHA:0:7}." >&2
            exit 1
            ;;
    esac

    sleep 4
done

echo "Timed out waiting for local233 status on ${TARGET_SHA:0:7}." >&2
exit 124
