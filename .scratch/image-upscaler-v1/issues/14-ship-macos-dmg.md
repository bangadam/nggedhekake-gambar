# Ship the signed and notarized macOS DMG

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Produce the official macOS 13+ Apple Silicon distribution from the approved release identity. The DMG must install a signed and notarized Slint application containing the pinned universal engine artifact, approved model files, attribution/notices, and the complete single-image workflow. The packaged app must prove real MoltenVK inference rather than only launch successfully.

## Acceptance criteria

- [ ] The DMG installs the correctly named app with the approved bundle identifier and no webview runtime.
- [ ] The app, nested engine executable, and relevant bundle contents are signed correctly and Apple notarization/stapling succeeds.
- [ ] The pinned engine/models, runtime paths, checksums, About attribution, and distributed notices work from the installed bundle.
- [ ] A clean macOS 13+ arm64 environment can select, 2× upscale, compare, save, cancel, and run Diagnostics successfully.
- [ ] Gatekeeper accepts the downloaded artifact without bypass instructions.
- [ ] The release publishes the DMG and its SHA-256 checksum through the approved channel.
- [ ] Packaging automation fails closed on missing credentials, checksum drift, signing failure, or notarization failure.

## Blocked by

- [Establish release identity and signing channel](03-establish-release-identity.md)
- [Review and close accessibility and interaction gaps](13-accessibility-interaction-review.md)
