# Select, validate, and preview one image

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Deliver the first runnable Nggedhekaké Gambar workflow: the English Slint workspace opens directly, accepts one static PNG, JPEG, or WebP through drag-and-drop or a native picker, validates it, and renders a bounded preview in the canvas-dominant two-column layout. Unsupported, animated, corrupt, and zero-sized inputs remain visible as actionable errors rather than entering a ready state.

## Acceptance criteria

- [ ] The app opens directly into the two-column workspace without a setup wizard or webview.
- [ ] Drag-and-drop and the native picker accept one static PNG, JPEG, or WebP and show the same ready state.
- [ ] The selected image is orientation-correct in a memory-bounded preview, including Unicode and space-containing paths.
- [ ] Unsupported formats, animated WebP, corrupt files, and zero-sized images produce actionable errors and cannot start processing.
- [ ] Replacing the source updates the preview and readiness state without retaining a recent-file history.
- [ ] Primary controls, source state, and validation errors are keyboard reachable and semantically exposed.
- [ ] Controller-level tests cover selection, validation, replacement, and error states without asserting Slint internals.

## Blocked by

None - can start immediately
