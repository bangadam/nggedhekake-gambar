# Ship the Linux AppImage

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Produce the official Ubuntu 22.04+ x86_64 AppImage. It must run the complete Slint single-image workflow with the pinned Linux engine, approved models, attribution/notices, and native desktop integration expected from the chosen toolkit. The packaged artifact must prove real Vulkan inference on supported Ubuntu hardware; other contemporary distributions remain best-effort.

## Acceptance criteria

- [ ] The x86_64 AppImage launches on a clean Ubuntu 22.04+ environment without installing the application system-wide.
- [ ] The pinned engine/models, executable permissions, runtime paths, checksums, About attribution, and distributed notices work from the mounted artifact.
- [ ] Wayland-default and X11-compatible supported sessions can open the workspace and native file/save dialogs.
- [ ] A clean supported Ubuntu environment can select, 2× upscale, compare, save, cancel, and run Diagnostics successfully.
- [ ] Unsupported Vulkan environments receive the defined actionable no-fallback diagnostic.
- [ ] The release publishes the AppImage and its SHA-256 checksum through the approved channel.
- [ ] Packaging automation fails closed on checksum drift, missing resources, permission errors, or packaged smoke-test failure.

## Blocked by

- [Review and close accessibility and interaction gaps](13-accessibility-interaction-review.md)
