# Save the normalized result through the native dialog

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Turn an unsaved full-resolution result into a user-owned file only after the user presses Save. Open the native save dialog beside the source with the deterministic scale/model filename, preserve the source format by default, and encode PNG, JPEG, or WebP according to the fixed v1 quality and metadata policy. Saving must preserve transparency where supported, refuse transparent JPEG, and never overwrite silently.

## Acceptance criteria

- [ ] Save opens the native dialog at the source directory with `<stem>-2x-photo.<source-extension>` proposed.
- [ ] Cancelling the dialog leaves the app in Result unsaved with the temporary result intact.
- [ ] PNG is lossless, JPEG uses quality 92, lossy WebP uses quality 90, and the source format is the default.
- [ ] Output pixels are tagged sRGB, retain DPI, bake orientation, and omit EXIF, GPS, comments, and unrelated ancillary metadata.
- [ ] PNG/WebP preserve alpha without premultiplication artifacts; JPEG is refused when alpha is present.
- [ ] Existing-file replacement confirmation is delegated to the native dialog and no path is silently overwritten.
- [ ] A successful save exposes Open file and Open containing folder actions and preserves the exact full-resolution result.
- [ ] Fixture-image and filesystem-port tests verify formats, quality policy, metadata, transparency, cancellation, and collision behavior.

## Blocked by

- [Upscale one image at 2× to an unsaved result](04-upscale-2x-unsaved-result.md)
