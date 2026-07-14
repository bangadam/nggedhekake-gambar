# Offer the secondary Illustration model

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Add the bundled `realesrgan-x4plus-anime` model as a secondary **Illustration** option without making model choice a required step. The normal flow continues to use **Photo & general** automatically; selecting Illustration travels through the same manifest, typed request, checksum validation, engine execution, result naming, preview, and save path.

## Acceptance criteria

- [ ] Photo & general remains selected automatically and the primary flow does not require opening model options.
- [ ] Illustration is available from a clearly secondary control with plain-language explanatory copy.
- [ ] Both model entries come from the approved manifest with file pairs, scale support, provenance, license, slug, and checksums.
- [ ] A missing, incomplete, or checksum-invalid Illustration model is unavailable with an actionable diagnostic and cannot reach process launch.
- [ ] Illustration results use the `illustration` filename slug and otherwise share the existing result/save contract.
- [ ] Manifest, controller, command, and real-engine smoke tests verify the selected model reaches the expected unsaved result.

## Blocked by

- [Upscale one image at 2× to an unsaved result](04-upscale-2x-unsaved-result.md)
