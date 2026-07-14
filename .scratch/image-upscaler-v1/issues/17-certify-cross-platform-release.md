# Certify the cross-platform v1 release

Status: ready-for-human
Type: HITL

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Execute and record the final human release decision against the three packaged artifacts. Run the fixed photo, illustration, transparency, color-profile, Unicode-path, cancellation, resource-boundary, and diagnostics corpus on representative macOS arm64, Windows x86_64, and Ubuntu x86_64 Vulkan/MoltenVK hardware. Review perceptual output and the complete install-to-save experience; do not substitute static checks or pixel-identical assertions for real evidence.

## Acceptance criteria

- [ ] The exact candidate DMG, MSI, and AppImage install or launch cleanly on every release-blocking OS baseline.
- [ ] Signatures, notarization, checksums, bundled license notices, engine/model checksums, and update links match the approved release records.
- [ ] The golden corpus produces exact expected dimensions/formats and approved perceptual similarity on all three target GPU paths.
- [ ] Transparency, sRGB conversion, orientation, DPI, metadata stripping, JPEG/WebP quality policy, Unicode paths, and the 200 MP boundary pass on packaged builds.
- [ ] Cancellation, unsaved-result protection, save recovery, zero pre-consent networking, diagnostics, redacted export, keyboard flow, and update notification pass packaged smoke scenarios.
- [ ] Known differences and non-blocking best-effort Linux findings are recorded without silently weakening release-blocking criteria.
- [ ] A human records approve/reject for the candidate; rejection names the blocking issue and does not publish the release as v1.

## Blocked by

- [Ship the signed and notarized macOS DMG](14-ship-macos-dmg.md)
- [Ship the signed Windows MSI](15-ship-windows-msi.md)
- [Ship the Linux AppImage](16-ship-linux-appimage.md)
