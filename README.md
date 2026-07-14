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

- **Engine:** pinned official `Real-ESRGAN-ncnn-vulkan` v0.2.0 binary
- **Inference runtime:** NCNN
- **Acceleration:** Vulkan
- **App shell:** Rust-native desktop app
- **Positioning:** native, local, fast, clear

## Docs

- PRD: `docs/PRD.md`
- Design system: `docs/design-system.md`
- Implementation issues: https://github.com/bangadam/nggedhekake-gambar/issues

## v1 priorities

1. Single-image 2× and 4× upscale
2. Curated Photo & general and Illustration models
3. Native phase progress and cancellation
4. Swipe comparison and explicit Save flow
5. Diagnostics for engine, model, and Vulkan readiness
6. Signed macOS/Windows and packaged Linux releases

## Non-goals for v1

- batch folder processing
- custom models and engines
- cloud inference
- video upscaling
- training/fine-tuning
- rewriting the inference engine in Rust

## Notes

The performance bet is simple: keep the proven engine, replace the heavy shell.
