#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET_DIR="${1:-/mnt/e/projects/football_insight_mini/}"

cleanup() {
  if [[ -n "${SYNC_PID:-}" ]]; then
    kill "${SYNC_PID}" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT INT TERM

bash "${SCRIPT_DIR}/sync-mp-weixin-dist.sh" "${PROJECT_ROOT}/dist/dev/mp-weixin/" "${TARGET_DIR}" &
SYNC_PID=$!

cd "${PROJECT_ROOT}"
npm run dev:mp-weixin
