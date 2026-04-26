#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_ENV_FILE="${ROOT_DIR}/football_insight_service_backend_rs/.env"
LOCAL_ENV_FILE="${SCRIPT_DIR}/.env.sync"

if [[ "${1:-}" == "--no-lock" ]]; then
  shift
elif [[ "${FI_AVATAR_SYNC_USE_FLOCK:-1}" != "0" ]] && command -v flock >/dev/null 2>&1; then
  LOCK_FILE="${FI_AVATAR_SYNC_LOCK_FILE:-${SCRIPT_DIR}/.avatar_sync.lock}"
  exec flock -n -E 0 "${LOCK_FILE}" "$0" --no-lock "$@"
fi

if [[ -f "${BACKEND_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${BACKEND_ENV_FILE}"
  set +a
fi

if [[ -f "${LOCAL_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${LOCAL_ENV_FILE}"
  set +a
fi

export FI_DATABASE_URL="${FI_DATABASE_URL:-${DATABASE_URL:-}}"
export FI_MINIO_ENDPOINT="${FI_MINIO_ENDPOINT:-https://oryjk.cn:82/minio}"
export FI_MINIO_BUCKET="${FI_MINIO_BUCKET:-football-insight}"
export FI_MINIO_REGION="${FI_MINIO_REGION:-us-east-1}"
export FI_MINIO_PREFIX="${FI_MINIO_PREFIX:-summary}"
export UV_DEFAULT_INDEX="${UV_DEFAULT_INDEX:-https://mirrors.ustc.edu.cn/pypi/web/simple}"

if [[ -z "${FI_DATABASE_URL}" ]]; then
  echo "FI_DATABASE_URL or DATABASE_URL is required." >&2
  exit 1
fi

if [[ -z "${FI_MINIO_ACCESS_KEY:-}" || -z "${FI_MINIO_SECRET_KEY:-}" ]]; then
  echo "FI_MINIO_ACCESS_KEY and FI_MINIO_SECRET_KEY are required. Put them in ${LOCAL_ENV_FILE}." >&2
  exit 1
fi

ARGS=()
SEASON="${FI_AVATAR_SYNC_SEASON:-${FI_AUTO_SYNC_SEASON:-}}"
if [[ -n "${SEASON}" ]]; then
  ARGS+=(--season "${SEASON}")
fi

cd "${SCRIPT_DIR}"

exec uv run sina-csl-scraper scrape \
  "${ARGS[@]}" \
  --upload-avatars \
  --write-db \
  "$@"
