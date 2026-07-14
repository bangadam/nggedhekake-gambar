# Protect unsaved results and recover failed saves

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Make the single temporary-result lifecycle safe. When a completed result has not been saved, selecting a different source, changing scale/model, rerunning, or closing the app must require an explicit Discard or Cancel choice. Encoding, permission, storage, or destination failures must return to Result unsaved so the user can save elsewhere without repeating inference.

## Acceptance criteria

- [ ] Source replacement, processing-setting changes, rerun, and app close all request confirmation while a result is unsaved.
- [ ] Cancel preserves the exact temporary result, comparison state, source, and settings.
- [ ] Discard removes only that result’s temporary artifacts and applies the requested next action.
- [ ] Save-dialog cancellation is not treated as an error and leaves the result unsaved.
- [ ] Encoding and filesystem failures show an actionable error and allow another Save attempt without rerunning inference.
- [ ] A successful save clears the unsaved guard without creating hidden history or additional result variants.
- [ ] Controller and filesystem-port tests cover every guarded transition, failure recovery, and cleanup invariant.

## Blocked by

- [Save the normalized result through the native dialog](05-save-normalized-result.md)
