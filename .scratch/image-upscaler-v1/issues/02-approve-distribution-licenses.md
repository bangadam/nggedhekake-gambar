# Approve redistributable engine, models, and UI licenses

Status: ready-for-human
Type: HITL

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Produce the approved distribution manifest for the exact Slint, official Real-ESRGAN NCNN/Vulkan v0.2.0, and two NCNN model artifacts that may ship with the app. The result must give implementation agents authoritative sources, immutable checksums, license obligations, attribution copy, and a clear human approval decision; third-party repackaging is not acceptable provenance.

## Acceptance criteria

- [ ] The exact engine artifact for each release target has an authoritative URL, release tag, architecture, SHA-256 checksum, and license record.
- [ ] `realesrgan-x4plus` and `realesrgan-x4plus-anime` `.param`/`.bin` pairs have authoritative provenance, SHA-256 checksums, and documented redistribution permission.
- [ ] The Slint royalty-free desktop attribution path is documented for an `MIT OR Apache-2.0` application.
- [ ] Required About/Licenses copy and distributed notice files are specified for the app, engine, models, and dependencies.
- [ ] A human records an explicit approve/reject decision for every artifact; rejected or uncertain artifacts block bundling.
- [ ] The manifest is machine-readable enough for build-time and runtime checksum verification.

## Blocked by

None - can start immediately
