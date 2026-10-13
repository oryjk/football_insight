# Topic article images implementation plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Download original news body photos and show them in the cited topic's detail page.

**Architecture:** Python extracts body-only image URLs, validates/downloads originals and uploads content-addressed public assets. Rust exposes an additive optional images array from existing JSONB; the mini detail page renders images between paragraphs and supports preview.

**Tech Stack:** Python/uv/Pillow, Rust/Axum/SQLx, Vue/uni-app/Bun, existing MinIO.

**Spec:** User request in this conversation: download original article images and also display them in topic articles.

## Global constraints

- Preserve topic identity, text, audio, heat and publication timestamps; no database migration required.
- Only attach photos from the topic's cited originals; no generated captions or invented image URLs.
- Keep existing 07:00–22:30 half-hour schedule and no-new-material behavior.
- No credentials or downloaded media in Git. Preserve unrelated dirty documentation.
- Old payloads without images remain readable. At most four photos per topic.

## Review focus

- Navigation, author avatars and sidebar images must not enter body extraction.
- Private addresses, redirects, oversized files and invalid image bytes must be rejected.
- Image failure must preserve text/audio and previously published photos.
- Backfill must patch only images under row locks, preserving concurrent fields.
- Invalid optional image records must not make an old topic unreadable.

### Task 1: Python collection, publication and backfill

**Files:** topics/models.py, sources.py, images.py, pipeline.py, daily.py, cli.py, repository.py, storage.py; tests/test_topic_images.py.

**Interfaces:** SourceArticle.images is a default-empty tuple of original URLs; as_material() persists them without changing as_source()/source_id. TopicImagePublisher.enrich(items, sources) patches additive images in place. Repository loads published items and patch_images(team_id, updates) changes only their image arrays.

- [x] Test body-only extraction, preserved identity, decode/size limits, provenance, upload cache reuse and failure fallback.
- [x] Implement downloads capped at 8 MiB, JPEG/PNG/WebP decoding, minimum 160×100, maximum 40M pixels, public URL validation and bounded redirects.
- [x] Integrate before TTS and preserve URLs in interruption checkpoints; add refresh-images --write-db for existing topics without LLM/TTS.
- [x] Run uv run pytest; verify actual TTPlus photograph upload and existing topic backfill.

### Task 2: Additive Rust image contract

**Files:** daily_topics/domain/edition.rs, adapters/persistence/stored_payload.rs, adapters/web/dto.rs, application/get_daily_topics.rs; tests/daily_topics_http.rs.

**Interfaces:** DailyTopicImage contains url, width, height, source_url, publisher, after_paragraph; DTO serializes images as an array.

- [x] Test missing/malformed image metadata and valid provenance/position normalization.
- [x] Parse image records independently; filter unsafe URLs and unmatched sources in application.
- [x] Run cargo test/check and boundary scan; rebuild and restart local Docker before HTTP checks.

### Task 3: Mini detail and release

**Files:** types/dailyTopic.ts, utils/dailyTopics.ts, utils/topicImages.ts, detail/index.vue, detail/components/TopicArticleImage.vue; matching utility tests.

**Interfaces:** Mapper defaults missing images to []; topicReadingBlocks(item) yields paragraphs and following photos. Page handles uni.previewImage; presentation component emits preview intent.

- [x] Verify old/new payload mapping, malformed-image fallback and reading order.
- [x] Display full-width photos with publisher attribution; image failures leave readable text.
- [x] Run Bun tests/typecheck/H5/MP builds and verify rendered real image dimensions.
- [ ] Review, update project memory, commit/push task-only changes, deploy production backend and upload robot 2 without preview QR.
