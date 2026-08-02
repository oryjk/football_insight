#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT="$SCRIPT_DIR"

usage() {
    cat <<'EOF'
Deploy the pushed Football Insight Git commit to the local233 test backend.

Usage:
  ./deploy_local233.sh
  ./deploy_local233.sh --help

Environment overrides:
  DEPLOY_HOST          SSH host (default: local233)
  DEPLOY_REPO_DIR      Remote monorepo directory
                       (default: /home/betalpha/projects/football_insight)
  DEPLOY_BRANCH        Branch to deploy (default: current local branch)
  DEPLOY_PORT          Remote published backend port (default: 18092)
  DEPLOY_CONTAINER     Backend container name
                       (default: football-insight-local-test-backend)
  SSH_CONNECT_TIMEOUT  SSH connection timeout in seconds (default: 10)

The local and remote working trees must be clean. The local HEAD must already
be pushed to origin before deployment starts.
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage
    exit 0
fi

if (($# > 0)); then
    echo "unknown argument: $1" >&2
    usage >&2
    exit 2
fi

for command_name in git ssh; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "required command not found: $command_name" >&2
        exit 1
    fi
done

DEPLOY_HOST=${DEPLOY_HOST:-local233}
DEPLOY_REPO_DIR=${DEPLOY_REPO_DIR:-/home/betalpha/projects/football_insight}
DEPLOY_PORT=${DEPLOY_PORT:-18092}
DEPLOY_CONTAINER=${DEPLOY_CONTAINER:-football-insight-local-test-backend}
SSH_CONNECT_TIMEOUT=${SSH_CONNECT_TIMEOUT:-10}

local_status=$(git -C "$REPO_ROOT" status --porcelain)
if [[ -n "$local_status" ]]; then
    echo "local working tree is not clean; commit all intended changes before deploying:" >&2
    printf '%s\n' "$local_status" >&2
    exit 1
fi

current_branch=$(git -C "$REPO_ROOT" branch --show-current)
if [[ -z "$current_branch" ]]; then
    echo "cannot deploy from a detached HEAD" >&2
    exit 1
fi

DEPLOY_BRANCH=${DEPLOY_BRANCH:-$current_branch}
if ! git -C "$REPO_ROOT" check-ref-format --branch "$DEPLOY_BRANCH" >/dev/null 2>&1; then
    echo "invalid DEPLOY_BRANCH: $DEPLOY_BRANCH" >&2
    exit 1
fi
if [[ "$DEPLOY_BRANCH" != "$current_branch" ]]; then
    echo "DEPLOY_BRANCH must match the current local branch ($current_branch)" >&2
    exit 1
fi
if [[ ! "$DEPLOY_PORT" =~ ^[0-9]+$ ]] || ((DEPLOY_PORT < 1 || DEPLOY_PORT > 65535)); then
    echo "DEPLOY_PORT must be an integer between 1 and 65535" >&2
    exit 1
fi
if [[ ! "$SSH_CONNECT_TIMEOUT" =~ ^[0-9]+$ ]] || ((SSH_CONNECT_TIMEOUT < 1)); then
    echo "SSH_CONNECT_TIMEOUT must be a positive integer" >&2
    exit 1
fi

echo "Fetching origin/$DEPLOY_BRANCH..."
git -C "$REPO_ROOT" fetch origin "$DEPLOY_BRANCH:refs/remotes/origin/$DEPLOY_BRANCH"

local_head=$(git -C "$REPO_ROOT" rev-parse HEAD)
origin_head=$(git -C "$REPO_ROOT" rev-parse "origin/$DEPLOY_BRANCH")
if [[ "$local_head" != "$origin_head" ]]; then
    echo "local HEAD has not been pushed to origin/$DEPLOY_BRANCH" >&2
    echo "local:  $local_head" >&2
    echo "origin: $origin_head" >&2
    exit 1
fi

printf -v remote_command 'bash -s -- %q %q %q %q %q' \
    "$DEPLOY_REPO_DIR" \
    "$DEPLOY_BRANCH" \
    "$local_head" \
    "$DEPLOY_PORT" \
    "$DEPLOY_CONTAINER"

echo "Deploying $DEPLOY_BRANCH@$local_head to $DEPLOY_HOST:$DEPLOY_REPO_DIR..."
ssh \
    -o BatchMode=yes \
    -o "ConnectTimeout=$SSH_CONNECT_TIMEOUT" \
    "$DEPLOY_HOST" \
    "$remote_command" <<'REMOTE_SCRIPT'
set -euo pipefail

deploy_repo_dir=$1
deploy_branch=$2
expected_head=$3
deploy_port=$4
deploy_container=$5

for command_name in git docker curl; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "required remote command not found: $command_name" >&2
        exit 1
    fi
done

if [[ ! -d "$deploy_repo_dir/.git" ]]; then
    echo "remote Git repository not found: $deploy_repo_dir" >&2
    exit 1
fi

remote_status=$(git -C "$deploy_repo_dir" status --porcelain)
if [[ -n "$remote_status" ]]; then
    echo "remote working tree is not clean; refusing to overwrite changes:" >&2
    printf '%s\n' "$remote_status" >&2
    exit 1
fi

git -C "$deploy_repo_dir" fetch origin "$deploy_branch:refs/remotes/origin/$deploy_branch"
if git -C "$deploy_repo_dir" show-ref --verify --quiet "refs/heads/$deploy_branch"; then
    git -C "$deploy_repo_dir" checkout "$deploy_branch"
else
    git -C "$deploy_repo_dir" checkout -b "$deploy_branch" "origin/$deploy_branch"
fi
git -C "$deploy_repo_dir" pull --ff-only origin "$deploy_branch"

deployed_head=$(git -C "$deploy_repo_dir" rev-parse HEAD)
if [[ "$deployed_head" != "$expected_head" ]]; then
    echo "remote HEAD does not match the requested commit" >&2
    echo "expected: $expected_head" >&2
    echo "actual:   $deployed_head" >&2
    exit 1
fi

cd "$deploy_repo_dir"
./deploy/local-test/up.sh

docker ps \
    --filter "name=^/${deploy_container}$" \
    --filter status=running \
    --format 'name={{.Names}} status={{.Status}} ports={{.Ports}}'

api_base_url="http://127.0.0.1:${deploy_port}"
for endpoint in \
    /api/health \
    /api/v1/ticket-watch/regions \
    /api/v1/seat-swap/current
do
    curl -fsS "${api_base_url}${endpoint}" >/dev/null
    echo "verified ${endpoint}"
done

echo "deployed commit: $deployed_head"
REMOTE_SCRIPT

echo "local233 deployment completed"
