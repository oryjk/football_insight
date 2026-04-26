#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SOURCE_DIR="${1:-${PROJECT_ROOT}/dist/dev/mp-weixin/}"
TARGET_DIR="${2:-/mnt/e/projects/football_insight_mini/}"
SYNC_INTERVAL_SECONDS="${SYNC_INTERVAL_SECONDS:-1}"

if [[ ! -d "${SOURCE_DIR}" ]]; then
  echo "[sync-mp-weixin-dist] Source directory not found: ${SOURCE_DIR}" >&2
  exit 1
fi

mkdir -p "${TARGET_DIR}"

echo "[sync-mp-weixin-dist] Mirroring ${SOURCE_DIR} -> ${TARGET_DIR}"
echo "[sync-mp-weixin-dist] Interval: ${SYNC_INTERVAL_SECONDS}s"

while true; do
  rsync -a --delete "${SOURCE_DIR}" "${TARGET_DIR}"
  sleep "${SYNC_INTERVAL_SECONDS}"
done
