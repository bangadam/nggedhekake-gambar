# Ship the signed Windows MSI

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Produce the official Windows 10 22H2/Windows 11 x86_64 distribution from the approved release identity. The MSI must install and uninstall a signed Slint application containing the pinned engine artifact, approved models, attribution/notices, and the complete single-image workflow. The installed app must prove real Vulkan inference on supported hardware.

## Acceptance criteria

- [ ] The MSI installs and uninstalls the correctly named x86_64 app with the approved product identifier and shortcuts.
- [ ] The MSI, app executable, and bundled engine are Authenticode-signed through the approved identity.
- [ ] The pinned engine/models, runtime paths, checksums, About attribution, and distributed notices work from the installed location.
- [ ] A clean supported Windows environment can select, 2× upscale, compare, save, cancel, and run Diagnostics successfully.
- [ ] Installation and first launch do not require an unknown-publisher bypass attributable to missing project signing.
- [ ] The release publishes the MSI and its SHA-256 checksum through the approved channel.
- [ ] Packaging automation fails closed on missing credentials, checksum drift, signing failure, or installer validation failure.

## Blocked by

- [Establish release identity and signing channel](03-establish-release-identity.md)
- [Review and close accessibility and interaction gaps](13-accessibility-interaction-review.md)
