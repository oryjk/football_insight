#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_ENV_FILE="${ROOT_DIR}/football_insight_service_backend_rs/.env"
LOCAL_ENV_FILE="${SCRIPT_DIR}/.env.sync"

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
export FI_SYNC_ENRICH_CORNERS="${FI_SYNC_ENRICH_CORNERS:-${FI_AUTO_SYNC_ENRICH_CORNERS:-1}}"

if [[ -z "${FI_DATABASE_URL}" ]]; then
  echo "FI_DATABASE_URL or DATABASE_URL is required." >&2
  exit 1
fi

if [[ -z "${FI_MINIO_ACCESS_KEY:-}" || -z "${FI_MINIO_SECRET_KEY:-}" ]]; then
  echo "FI_MINIO_ACCESS_KEY and FI_MINIO_SECRET_KEY are required. Put them in ${LOCAL_ENV_FILE}." >&2
  exit 1
fi

ARGS=()
if [[ $# -ge 1 && -n "${1:-}" ]]; then
  ARGS+=(--season "$1")
  shift
fi

if [[ "${FI_SYNC_ENRICH_CORNERS}" != "0" ]]; then
  ARGS+=(--enrich-corners)
fi

exec uv run sina-csl-scraper scrape \
  "${ARGS[@]}" \
  --upload-avatars \
  --write-db \
  "$@"
