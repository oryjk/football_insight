#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/../.." && pwd)

usage() {
    cat <<'EOF'
Watch origin/main on local233 and automatically:
  1. Run migrations (if migrations/*.sql changed) and restart local233:18092 backend
     when football_insight_service_backend_rs changes.
  2. Upload WeChat Mini Program experience version (robot 2) via bun run mp:release
     when football_insight_mini changes.
  3. Report execution status (pending / success / failure) back to GitHub commit
     statuses via `gh api` so remote cloud agents can monitor completion.

Usage:
  ./deploy/local-test/auto-sync-local233.sh [--once] [--interval <seconds>]

Environment variables:
  DEPLOY_BRANCH               Branch to watch (default: main)
  AUTO_SYNC_INTERVAL_SECONDS  Polling interval in seconds (default: 5)
  AUTO_SYNC_BACKEND           1 to auto-restart 18092 backend on changes (default: 1)
  AUTO_SYNC_MP_RELEASE        1 to auto-run mp:release --robot 2 on changes (default: 1)
  AUTO_SYNC_MP_ROBOT          WeChat CI robot number (default: 2)
  GH_REPO                     GitHub owner/repo for status reporting (default: oryjk/football_insight)
EOF
}

ONCE=false
INTERVAL=${AUTO_SYNC_INTERVAL_SECONDS:-5}

while (($# > 0)); do
    case "$1" in
        --once)
            ONCE=true
            shift
            ;;
        --interval)
            if (($# < 2)); then
                echo "missing value for --interval" >&2
                exit 2
            fi
            INTERVAL=$2
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ ! "$INTERVAL" =~ ^[0-9]+$ ]] || ((INTERVAL < 1 || INTERVAL > 86400)); then
    echo "interval must be an integer between 1 and 86400 seconds" >&2
    exit 2
fi

DEPLOY_BRANCH=${DEPLOY_BRANCH:-main}
AUTO_SYNC_BACKEND=${AUTO_SYNC_BACKEND:-1}
AUTO_SYNC_MP_RELEASE=${AUTO_SYNC_MP_RELEASE:-0}
AUTO_SYNC_MP_ROBOT=${AUTO_SYNC_MP_ROBOT:-2}
AUTO_SYNC_FETCH_TIMEOUT_SECONDS=${AUTO_SYNC_FETCH_TIMEOUT_SECONDS:-30}
GH_REPO=${GH_REPO:-oryjk/football_insight}

log() {
    printf '[local233-sync %s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*"
}

LAST_NOTICE=""
notice() {
    if [[ "$*" != "$LAST_NOTICE" ]]; then
        log "$*"
        LAST_NOTICE="$*"
    fi
}

report_status() {
    local sha=$1
    local context=$2
    local state=$3
    local description=$4

    if ! command -v gh >/dev/null 2>&1; then
        return 0
    fi

    # Truncate description to 135 chars (GitHub API limit is 140 chars)
    local short_desc=${description:0:135}
    gh api "repos/${GH_REPO}/statuses/${sha}" \
        -f state="$state" \
        -f context="$context" \
        -f description="$short_desc" >/dev/null 2>&1 || true
}

run_backend_sync() {
    local before_sha=$1
    local target_sha=$2
    local run_migrations=$3

    log "Backend changes detected (${before_sha:0:7} -> ${target_sha:0:7}); updating local233:18092..."
    report_status "$target_sha" "local233/backend-18092" "pending" "Compiling & restarting local233:18092 backend..."

    local output_file
    output_file=$(mktemp "${TMPDIR:-/tmp}/local233-backend-sync.XXXXXX")
    if (
        set -euo pipefail
        cd "$REPO_ROOT"
        if [[ "$run_migrations" == "true" ]]; then
            log "New SQL migrations detected; running run_migrations..."
            (cd "$REPO_ROOT/football_insight_service_backend_rs" && cargo run --bin run_migrations)
        fi
        ./deploy/local-test/up.sh
        for endpoint in \
            /api/health \
            /api/v1/ticket-watch/regions \
            /api/v1/seat-swap/current
        do
            curl -fsS "http://127.0.0.1:18092${endpoint}" >/dev/null
        done
    ) >"$output_file" 2>&1; then
        cat "$output_file"
        rm -f "$output_file"
        log "Backend 18092 restarted and verified at ${target_sha:0:7}."
        report_status "$target_sha" "local233/backend-18092" "success" "local233:18092 restarted & healthy (${target_sha:0:7})"
        return 0
    else
        cat "$output_file" >&2
        local tail_err
        tail_err=$(tail -n 1 "$output_file" | tr '\r\n' ' ')
        rm -f "$output_file"
        log "ERROR: Backend 18092 update failed at ${target_sha:0:7}."
        report_status "$target_sha" "local233/backend-18092" "failure" "Failed: ${tail_err:-backend build or healthcheck error}"
        return 1
    fi
}

run_mini_release_sync() {
    local before_sha=$1
    local target_sha=$2
    local deps_changed=$3

    local commit_subject
    commit_subject=$(git -C "$REPO_ROOT" log -1 --format=%s "$target_sha" 2>/dev/null || echo "sync ${target_sha:0:7}")

    # Skip if this commit is itself only an automated review version bump commit
    if [[ "$commit_subject" == "chore(mini): sync allocated review version"* ]]; then
        log "Skipping duplicate mp:release for version-sync commit ${target_sha:0:7}."
        return 0
    fi

    log "Mini Program changes detected (${before_sha:0:7} -> ${target_sha:0:7}); preparing mp:release (robot ${AUTO_SYNC_MP_ROBOT})..."
    report_status "$target_sha" "local233/mp-release-robot2" "pending" "Uploading mp-weixin robot=${AUTO_SYNC_MP_ROBOT}..."

    local output_file
    output_file=$(mktemp "${TMPDIR:-/tmp}/local233-mini-sync.XXXXXX")
    local final_sha=$target_sha

    if (
        set -euo pipefail
        cd "$REPO_ROOT/football_insight_mini"
        if [[ "$deps_changed" == "true" ]]; then
            log "Mini Program dependencies changed; running bun install --frozen-lockfile..."
            bun install --frozen-lockfile
        fi

        # Pre-sync allocated review version so that if the backend allocates a new version,
        # we automatically commit & push the version files before running mp:release.
        MINI_REVIEW_SKIP=0 bun --no-env-file --bun scripts/sync-manifest-version.mjs

        local version_dirty
        version_dirty=$(git -C "$REPO_ROOT" status --porcelain -- \
            football_insight_mini/src/manifest.json \
            football_insight_mini/src/config/generatedMiniProgramVersion.ts)
        if [[ -n "$version_dirty" ]]; then
            log "New review version allocated; committing and pushing version files..."
            git -C "$REPO_ROOT" add \
                football_insight_mini/src/manifest.json \
                football_insight_mini/src/config/generatedMiniProgramVersion.ts
            git -C "$REPO_ROOT" commit -m "chore(mini): sync allocated review version [local233]"
            git -C "$REPO_ROOT" push origin "$DEPLOY_BRANCH"
        fi

        bun run mp:release -- --robot "$AUTO_SYNC_MP_ROBOT" --desc "$commit_subject"
    ) >"$output_file" 2>&1; then
        cat "$output_file"
        rm -f "$output_file"
        final_sha=$(git -C "$REPO_ROOT" rev-parse HEAD)
        log "Mini Program uploaded (robot=${AUTO_SYNC_MP_ROBOT}) at ${final_sha:0:7}."
        report_status "$target_sha" "local233/mp-release-robot2" "success" "Uploaded robot=${AUTO_SYNC_MP_ROBOT} (${final_sha:0:7})"
        if [[ "$final_sha" != "$target_sha" ]]; then
            report_status "$final_sha" "local233/mp-release-robot2" "success" "Uploaded robot=${AUTO_SYNC_MP_ROBOT} (${final_sha:0:7})"
        fi
        return 0
    else
        cat "$output_file" >&2
        local tail_err
        tail_err=$(tail -n 1 "$output_file" | tr '\r\n' ' ')
        rm -f "$output_file"
        log "ERROR: Mini Program mp:release failed at ${target_sha:0:7}."
        report_status "$target_sha" "local233/mp-release-robot2" "failure" "Failed: ${tail_err:-mp:release error}"
        return 1
    fi
}

run_scraper_topics_sync() {
    local before_sha=$1
    local target_sha=$2

    log "Scraper topic changes detected (${before_sha:0:7} -> ${target_sha:0:7}); pruning roundup editions & refreshing audio on local233..."
    report_status "$target_sha" "local233/topics-prune" "pending" "Pruning roundup editions & refreshing audio..."

    local output_file
    output_file=$(mktemp "${TMPDIR:-/tmp}/local233-topics-sync.XXXXXX")
    if (
        set -euo pipefail
        cd "$REPO_ROOT/sina_csl_scraper"
        timeout --kill-after=10s 180s uv run --no-sync python scripts/run_topics_local.py prune-roundup --write-db
    ) >"$output_file" 2>&1; then
        cat "$output_file"
        rm -f "$output_file"
        log "Roundup editions pruned and audio refreshed at ${target_sha:0:7}."
        report_status "$target_sha" "local233/topics-prune" "success" "Roundup editions pruned & audio refreshed (${target_sha:0:7})"
        return 0
    else
        cat "$output_file" >&2
        local tail_err
        tail_err=$(tail -n 1 "$output_file" | tr '\r\n' ' ')
        rm -f "$output_file"
        log "ERROR: Roundup prune failed at ${target_sha:0:7}."
        report_status "$target_sha" "local233/topics-prune" "failure" "Failed: ${tail_err:-topics prune error}"
        return 1
    fi
}

tick() {
    local current_branch
    current_branch=$(git -C "$REPO_ROOT" branch --show-current)
    if [[ "$current_branch" != "$DEPLOY_BRANCH" ]]; then
        notice "Current branch is '${current_branch:-detached}', expected '$DEPLOY_BRANCH'; waiting..."
        return 0
    fi

    local fetch_exit=0
    if command -v timeout >/dev/null 2>&1; then
        timeout --kill-after=5s "${AUTO_SYNC_FETCH_TIMEOUT_SECONDS}s" \
            git -C "$REPO_ROOT" fetch --quiet --no-write-fetch-head origin "$DEPLOY_BRANCH:refs/remotes/origin/$DEPLOY_BRANCH" \
            || fetch_exit=$?
    else
        git -C "$REPO_ROOT" fetch --quiet --no-write-fetch-head origin "$DEPLOY_BRANCH:refs/remotes/origin/$DEPLOY_BRANCH" \
            || fetch_exit=$?
    fi
    if ((fetch_exit != 0)); then
        if ((fetch_exit == 124 || fetch_exit == 137)); then
            notice "git fetch timed out after ${AUTO_SYNC_FETCH_TIMEOUT_SECONDS}s (proxy/SSH stall); will retry next tick."
        else
            notice "git fetch failed (exit ${fetch_exit}); will retry next tick."
        fi
        return 1
    fi

    local before target
    before=$(git -C "$REPO_ROOT" rev-parse HEAD)
    target=$(git -C "$REPO_ROOT" rev-parse "origin/$DEPLOY_BRANCH")

    if [[ "$before" == "$target" ]]; then
        notice "In sync with origin/$DEPLOY_BRANCH (${before:0:7}); waiting for new commits."
        return 0
    fi

    local dirty
    dirty=$(git -C "$REPO_ROOT" status --porcelain --untracked-files=no)
    if [[ -n "$dirty" ]]; then
        notice "Working tree has uncommitted tracked changes; pausing auto-sync until clean."
        report_status "$target" "local233/sync" "pending" "Paused: local233 has uncommitted tracked changes"
        return 0
    fi

    local counts left_count
    counts=$(git -C "$REPO_ROOT" rev-list --left-right --count "${before}...${target}")
    left_count=${counts%%[[:space:]]*}
    if [[ "$left_count" != "0" ]]; then
        local non_auto_commits
        non_auto_commits=$(git -C "$REPO_ROOT" log --format=%s "${target}..${before}" | grep -v '^chore(mini): sync allocated review version' || true)
        if [[ -z "$non_auto_commits" ]]; then
            log "Discarding diverged local version-sync commit(s) to realign with origin/$DEPLOY_BRANCH..."
            git -C "$REPO_ROOT" reset --hard "$target"
            before=$target
        else
            notice "Local branch has unpushed or diverged commits; pausing auto-sync."
            report_status "$target" "local233/sync" "pending" "Paused: local233 has unpushed local commits"
            return 0
        fi
    fi

    local changed_files
    changed_files=$(git -C "$REPO_ROOT" diff --name-only "$before" "$target")

    local backend_changed=false
    local migrations_changed=false
    local mini_changed=false
    local mini_deps_changed=false
    local scraper_topics_changed=false

    while IFS= read -r file; do
        [[ -z "$file" ]] && continue
        case "$file" in
            football_insight_service_backend_rs/migrations/*.sql)
                migrations_changed=true
                backend_changed=true
                ;;
            football_insight_service_backend_rs/*|deploy/local-test/*|deploy_local233.sh)
                backend_changed=true
                ;;
            football_insight_mini/package.json|football_insight_mini/bun.lock|football_insight_mini/bun.lockb)
                mini_changed=true
                mini_deps_changed=true
                ;;
            football_insight_mini/*)
                mini_changed=true
                ;;
            sina_csl_scraper/src/sina_csl_scraper/topics/*|sina_csl_scraper/scripts/run_topics_local.py)
                scraper_topics_changed=true
                ;;
        esac
    done <<<"$changed_files"

    log "Fast-forwarding $DEPLOY_BRANCH: ${before:0:7} -> ${target:0:7}"
    git -C "$REPO_ROOT" -c merge.autoStash=false merge --ff-only --no-edit "$target"
    LAST_NOTICE=""

    local sync_failed=0

    if [[ "$backend_changed" == "true" && "$AUTO_SYNC_BACKEND" == "1" ]]; then
        run_backend_sync "$before" "$target" "$migrations_changed" || sync_failed=1
    fi

    if [[ "$scraper_topics_changed" == "true" ]]; then
        run_scraper_topics_sync "$before" "$target" || sync_failed=1
    fi

    if [[ "$mini_changed" == "true" && "$AUTO_SYNC_MP_RELEASE" == "1" ]]; then
        run_mini_release_sync "$before" "$target" "$mini_deps_changed" || sync_failed=1
    fi

    if [[ "$backend_changed" == "false" && "$mini_changed" == "false" && "$scraper_topics_changed" == "false" ]]; then
        log "Fast-forwarded to ${target:0:7} (no backend, mini, or topics changes)."
        report_status "$target" "local233/sync" "success" "Synced ${target:0:7} on local233"
    fi

    return "$sync_failed"
}

if [[ "$ONCE" == "true" ]]; then
    tick
    exit $?
fi

log "Started local233 auto-sync watcher on branch '$DEPLOY_BRANCH' (interval=${INTERVAL}s, backend=${AUTO_SYNC_BACKEND}, mp_release=${AUTO_SYNC_MP_RELEASE})."
while true; do
    tick || true
    sleep "$INTERVAL"
done
