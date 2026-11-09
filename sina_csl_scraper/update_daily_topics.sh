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
# 每日话题：cron 每天 07:00–23:00 每个整点调用一次，每次只追加新快讯。
# 单次最多 55 分钟，保证不会拖到下一个整点。
TOPIC_TIMEOUT_SECONDS=3300
if [[ "${1:-}" == "--scheduled" ]]; then
    shift
    TOPIC_HOUR=$((10#$(date +%H)))
    if (( TOPIC_HOUR < 7 || TOPIC_HOUR > 23 )); then
        printf '[%s] daily topics outside scheduled window; skipped\n' "$(date -Iseconds)"
        exit 0
    fi
fi
printf '\n[%s] daily topics start\n' "$(date -Iseconds)"
exec timeout --kill-after=30s "${TOPIC_TIMEOUT_SECONDS}s" uv run --no-sync python scripts/run_topics_local.py \
    roundup-update --config config/rongcheng-news-auto.json --write-db "$@"
