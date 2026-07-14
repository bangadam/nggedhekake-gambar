# Inspect results with a bounded swipe comparison

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Make the completed result inspectable without loading full-resolution source and result images into the UI. Present a source/result swipe comparison in the canvas, keep preview memory bounded, expose keyboard-accessible source-only and result-only alternatives, and show saved-file actions only after a successful save.

## Acceptance criteria

- [ ] Result unsaved displays a swipe comparison aligned to the same visible crop and aspect ratio.
- [ ] The divider works with pointer drag and keyboard controls and has an accessible name/value.
- [ ] Source-only and result-only views are available without pointer precision.
- [ ] Preview generation is bounded independently of full-resolution dimensions and does not reduce saved output quality.
- [ ] Resizing the window or changing the divider does not decode the full-resolution artifacts repeatedly.
- [ ] Open file and Open containing folder appear only for a successfully saved destination and surface launch failures.
- [ ] UI/controller smoke tests verify comparison modes, keyboard operation, memory-bounded preview selection, and saved-action visibility.

## Blocked by

- [Upscale one image at 2× to an unsaved result](04-upscale-2x-unsaved-result.md)
