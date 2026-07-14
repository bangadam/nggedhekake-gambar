# Upscale one image at 2× to an unsaved result

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Complete the first real inference tracer bullet. From a ready source, the user starts a 2× Photo & general job; the app validates pinned resources and temporary capacity, applies orientation and sRGB normalization, preserves alpha semantics, invokes the official bundled engine through a typed request, and ends with a full-resolution temporary result plus bounded preview. The result is explicitly unsaved. Status shows truthful phases and elapsed time, never an invented percentage.

## Acceptance criteria

- [ ] The UI submits a typed 2× Photo & general request and never constructs raw process arguments.
- [ ] The app verifies engine/model files and checksums before launching and rejects estimated results over 200 megapixels.
- [ ] Input orientation is baked into pixels, embedded color is normalized to sRGB, DPI is retained for later output, and alpha remains available for recombination.
- [ ] The adapter safely handles Unicode/space-containing paths, captures stdout/stderr, and maps exit behavior into typed events.
- [ ] Visible states progress through Preparing, Upscaling, Building preview, and Result unsaved with elapsed time and no numeric percentage.
- [ ] Success retains one full-resolution temporary result and a separately bounded preview without writing a user-owned final file.
- [ ] Engine failure retains the source/settings and produces an actionable error while cleaning failed run-owned artifacts.
- [ ] Controller, image-pipeline, manifest, and fixture-process tests verify the observable path end to end.

## Blocked by

- [Select, validate, and preview one image](01-select-validate-preview.md)
- [Approve redistributable engine, models, and UI licenses](02-approve-distribution-licenses.md)
