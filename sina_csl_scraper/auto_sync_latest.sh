#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_ENV_FILE="${ROOT_DIR}/football_insight_service_backend_rs/.env"
LOCAL_ENV_FILE="${SCRIPT_DIR}/.env.sync"

if [[ "${1:-}" == "--no-lock" ]]; then
  shift
elif [[ "${FI_AUTO_SYNC_USE_FLOCK:-1}" != "0" ]] && command -v flock >/dev/null 2>&1; then
  LOCK_FILE="${FI_AUTO_SYNC_LOCK_FILE:-${SCRIPT_DIR}/.auto_sync.lock}"
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
export FI_AUTO_SYNC_SEASON="${FI_AUTO_SYNC_SEASON:-}"
export FI_AUTO_SYNC_STATE_FILE="${FI_AUTO_SYNC_STATE_FILE:-${SCRIPT_DIR}/.auto_sync_state.json}"
export FI_AUTO_SYNC_OUTPUT_DIR="${FI_AUTO_SYNC_OUTPUT_DIR:-${SCRIPT_DIR}/data}"
export FI_AUTO_SYNC_MATCH_DURATION_MINUTES="${FI_AUTO_SYNC_MATCH_DURATION_MINUTES:-120}"
export FI_AUTO_SYNC_POST_FINISH_DELAY_MINUTES="${FI_AUTO_SYNC_POST_FINISH_DELAY_MINUTES:-10}"
export FI_AUTO_SYNC_WRITE_DB="${FI_AUTO_SYNC_WRITE_DB:-1}"
export FI_AUTO_SYNC_UPLOAD_AVATARS="${FI_AUTO_SYNC_UPLOAD_AVATARS:-0}"
export FI_AUTO_SYNC_ENRICH_CORNERS="${FI_AUTO_SYNC_ENRICH_CORNERS:-1}"
export FI_AUTO_SYNC_LEISU_MATCH_MAP="${FI_AUTO_SYNC_LEISU_MATCH_MAP:-}"
export UV_PYTHON="${UV_PYTHON:-}"
export UV_DEFAULT_INDEX="${UV_DEFAULT_INDEX:-https://mirrors.ustc.edu.cn/pypi/web/simple}"

if [[ -z "${FI_DATABASE_URL}" ]]; then
  echo "FI_DATABASE_URL or DATABASE_URL is required." >&2
  exit 1
fi

if [[ "${FI_AUTO_SYNC_UPLOAD_AVATARS}" != "0" ]] && [[ -z "${FI_MINIO_ACCESS_KEY:-}" || -z "${FI_MINIO_SECRET_KEY:-}" ]]; then
  echo "FI_MINIO_ACCESS_KEY and FI_MINIO_SECRET_KEY are required. Put them in ${LOCAL_ENV_FILE}." >&2
  exit 1
fi

ARGS=(
  auto-sync-due
  --state-file "${FI_AUTO_SYNC_STATE_FILE}"
  --output-dir "${FI_AUTO_SYNC_OUTPUT_DIR}"
  --match-duration-minutes "${FI_AUTO_SYNC_MATCH_DURATION_MINUTES}"
  --post-finish-delay-minutes "${FI_AUTO_SYNC_POST_FINISH_DELAY_MINUTES}"
)

if [[ -n "${FI_AUTO_SYNC_SEASON}" ]]; then
  ARGS+=(--season "${FI_AUTO_SYNC_SEASON}")
fi

if [[ "${FI_AUTO_SYNC_WRITE_DB}" != "0" ]]; then
  ARGS+=(--write-db)
fi

if [[ "${FI_AUTO_SYNC_UPLOAD_AVATARS}" != "0" ]]; then
  ARGS+=(--upload-avatars)
fi

if [[ "${FI_AUTO_SYNC_ENRICH_CORNERS}" != "0" ]]; then
  ARGS+=(--enrich-corners)
fi

if [[ -n "${FI_AUTO_SYNC_LEISU_MATCH_MAP}" ]]; then
  ARGS+=(--leisu-match-map "${FI_AUTO_SYNC_LEISU_MATCH_MAP}")
fi

cd "${SCRIPT_DIR}"

exec uv run sina-csl-scraper "${ARGS[@]}" "$@"
