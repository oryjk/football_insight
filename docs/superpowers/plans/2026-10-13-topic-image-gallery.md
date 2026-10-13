# Topic image gallery implementation plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan inline.

**Goal:** Put topic images in a carousel before the article body and hide image provenance from the reading UI.

**Architecture:** Keep the existing published image collection and API. A page-local gallery receives images and emits preview intent; the page renders paragraphs independently. Image provenance remains internal metadata, while the article's existing news-source section remains available.

**Tech Stack:** Vue 3, uni-app, TypeScript, Bun.

**Spec:** User's explicit layout correction in this conversation: concentrate related images before the body in a carousel; do not display image source credits.

## Constraints and review focus

- Preserve title, audio controls, heat, paragraph text, news sources and article sharing.
- No paragraph-to-image association in the reading view; no backend rebuild or content regeneration needed.
- Manual swiping and click-to-preview; no autoplay while reading.
- Preserve complete images with aspectFit; hide failed images and reset the selected slide after content changes.
- Verify multiple, single, missing and broken images across H5 and compiled mp-weixin; use design tokens.
- Preserve unrelated dirty documentation; upload robot 2 from clean main after committing and pushing.

## Steps

- [x] Replace TopicArticleImage with page-local TopicImageGallery and restore independent paragraph rendering; remove obsolete reading-block helpers.
- [x] Run Bun tests, typecheck and H5/MP builds; inspect 320/390/430 widths and swipe/preview/error behavior with real images.
- [x] Readonly review, update project memory, commit/push task changes and upload the mini experience version without QR generation.
