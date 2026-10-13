#!/usr/bin/env bash
set -euo pipefail
umask 077

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${PROJECT_DIR}"
mkdir -p data/topics

if [[ "${1:-}" == "--no-lock" ]]; then
    shift
else
    exec flock -n -E 0 "${PROJECT_DIR}/data/topics/.daily-run.lock" \
        "${PROJECT_DIR}/update_daily_topics.sh" --no-lock "$@"
fi

export TZ=Asia/Shanghai
exec >> "${PROJECT_DIR}/data/topics/daily.log" 2>&1
TOPIC_TIMEOUT_SECONDS=21600
if [[ "${1:-}" == "--scheduled" ]]; then
    shift
    TOPIC_HOUR=$((10#$(date +%H)))
    if (( TOPIC_HOUR < 7 || TOPIC_HOUR >= 23 )); then
        printf '[%s] daily topics outside scheduled window; skipped\n' "$(date -Iseconds)"
        exit 0
    fi
    TOPIC_WINDOW_END="$(date -d 'today 23:00:00' +%s)"
    TOPIC_REMAINING=$((TOPIC_WINDOW_END - $(date +%s)))
    if (( TOPIC_REMAINING <= 0 )); then
        exit 0
    fi
    if (( TOPIC_REMAINING < TOPIC_TIMEOUT_SECONDS )); then
        TOPIC_TIMEOUT_SECONDS="${TOPIC_REMAINING}"
    fi
fi
printf '\n[%s] daily topics start\n' "$(date -Iseconds)"
exec timeout --kill-after=30s "${TOPIC_TIMEOUT_SECONDS}s" uv run --no-sync python scripts/run_topics_local.py \
    daily-update --config config/rongcheng-news-auto.json --write-db "$@"
