# Diagnose engine readiness and export redacted support data

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Provide a complete local diagnostics path. Lightweight checks report app, engine, model, checksum, and platform readiness without running inference; an explicit action runs a tiny bundled image through the real engine to prove Vulkan or MoltenVK execution. Structured logs rotate for seven days under a size cap, and manual support export redacts filesystem identity by default and never includes images.

## Acceptance criteria

- [ ] Lightweight diagnostics distinguish missing, unreadable, and checksum-invalid engine/model resources without invoking inference.
- [ ] The on-demand smoke action uses the real pinned engine and approved bundled model and reports GPU/Vulkan or MoltenVK success/failure.
- [ ] Unsupported hardware produces actionable platform guidance and offers neither CPU fallback nor custom-engine selection.
- [ ] Structured logs include typed lifecycle, process exit, and diagnostic events and rotate after seven days under a documented total-size cap.
- [ ] Manual export includes versions, checksums, smoke output, and relevant logs while excluding all source/result image bytes.
- [ ] Home paths and source/output names are redacted by default; including full paths requires an explicit per-export choice.
- [ ] Tests cover readiness mapping, smoke success/failure, rotation, cap enforcement, redaction, and the no-image invariant.

## Blocked by

- [Approve redistributable engine, models, and UI licenses](02-approve-distribution-licenses.md)
- [Upscale one image at 2× to an unsaved result](04-upscale-2x-unsaved-result.md)
