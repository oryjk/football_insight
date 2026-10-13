# Daily Topics Automation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Automatically discover recent Chengdu Rongcheng news and update articles plus audio every half hour from07:00 through22:30 Asia/Shanghai, stopping automatic work at23:00; retain the published edition when there is no new material or a failure.

**Architecture:** Extend the existing source collector with bounded, publisher-constrained news-list discovery. A separate daily-update workflow compares discovered URLs with published history and local durable state, resumes pending generation, and publishes only complete audio batches. A local233 cron wrapper loads existing private configuration and serializes the workflow.

**Tech Stack:** Python/uv, requests, existing Kimi Code/Qwen CPU/MinIO/PostgreSQL, Linux cron/flock.

**Spec:** User instructions in this conversation: automatic collection; 07:00–22:30 every30minutes, stop23:00; update both articles and audio; preserve old content without new articles.

## Constraints

- Stay on main and preserve existing unrelated document edits.
- Keep credentials, model files, state and raw originals outside Git.
- Reuse existing news/TTS/uploader/table; no migrations or backend deployment.
- No generator/TTS calls for already consumed material, including across dates.
- Publish only a nonempty batch with all audio ready; failures never replace the current database edition.
- Preserve current league/avatar cron entries.

## Tasks

- [x] Add tested listing discovery and a real rolling-news configuration. Verify publication dates and full original bodies; reject unrelated/off-domain/list-only candidates.
- [x] Add tested daily-update orchestration, published-source history lookup and durable pending/state tracking. Test no-new, audio failure, database failure, retry, partial-source usage, cross-day duplication and source outages.
- [x] Add the local233 daily wrapper and cron template, run live collection and workflow verification, install the07:00–22:30 half-hour job, record status/log locations, update project memory and push main.

## Verification

- `uv run --no-sync pytest` must pass, including the new preservation and retry tests.
- Shell syntax must pass; real live candidates must be fetched as full originals.
- Run the installed wrapper manually and compare the live API before/after; no-new must retain IDs, timestamps and audio URLs.
- Inspect installed cron and process/lock behavior; report actual results and any provider failures honestly.

## Outcome

Live collection retrieved6 complete originals. The installed wrapper returned no_new and preserved the two published articles and audio. Cron is installed every half hour07:00–22:30 Asia/Shanghai, with a23:00 execution cutoff. Independent review findings on original-body outages and status-lock contention were fixed with regressions. Final suite and secret/index checks are recorded in the release report.
