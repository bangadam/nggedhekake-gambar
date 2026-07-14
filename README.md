# Nggedhekaké Gambar

Rust-native desktop image upscaler built around the same engine class used by Upscayl: NCNN/Vulkan-backed Real-ESRGAN-compatible binaries.

## Goal

Clone the product category, not the Electron stack.

Build a lighter local desktop app for image upscaling with:
- Rust-native shell
- external NCNN/Vulkan engine process
- bundled model packs
- focused single-image and batch workflows

## Core product call

- **Engine:** `upscayl-ncnn` / `Real-ESRGAN-ncnn-vulkan` style binary
- **Inference runtime:** NCNN
- **Acceleration:** Vulkan
- **App shell:** Rust-native desktop app
- **Positioning:** native, local, fast, clear

## Docs

- PRD: `docs/PRD.md`
- Design system: `docs/design-system.md`

## v1 priorities

1. Single image upscale
2. Batch folder upscale
3. Bundled + custom model support
4. Native progress + cancellation
5. Output preview and compare
6. Diagnostics for engine/model/Vulkan readiness

## Non-goals for v1

- cloud inference
- video upscaling
- training/fine-tuning
- rewriting the inference engine in Rust

## Notes

The performance bet is simple: keep the proven engine, replace the heavy shell.
