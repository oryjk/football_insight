# Topic share card first photo implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline.

**Goal:** Include the first topic photo together with the title and body excerpt in the detail-page share card.

**Architecture:** Keep the 1000×800 generated card and current article link. Load only the first photo through uni.downloadFile/getImageInfo with a bounded timeout, render the complete photo alongside body text, and include its identity in cache invalidation. Keep the existing serial draw/revision protection and text fallback.

**Tech Stack:** Vue, TypeScript, uni-app canvas, Bun.

**Spec:** User request in this conversation: the share card should show body content and the first picture.

## Constraints and review focus

- Preserve title, publication time, article body excerpt, article identity/link and hidden image credits.
- Load the first image only; no image or load failure yields the existing text layout.
- Download to a local canvas-safe path, preserve aspect ratio and bound loading time; malformed dimensions cannot break export.
- A photo change must invalidate cached covers; obsolete async loads must not paint a new article's canvas.
- No backend changes or production restart; preserve unrelated dirty docs and upload robot 2 from clean main.

## Steps

- [x] Add first-photo cache identity and bounded asset loading; separate canvas drawing/layout from the existing serial workflow.
- [x] Verify cache/dimensions/loading regressions, Bun tests/typecheck and H5/MP builds; inspect actual generated cards with image, without image and failed image.
- [ ] Readonly review, update memory, commit/push and upload the mini experience version without preview QR.
