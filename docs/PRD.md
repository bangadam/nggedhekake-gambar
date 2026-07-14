# Nggedhekaké Gambar v1 PRD

## Problem Statement

Desktop users who want to enlarge a single image locally must often choose between heavyweight Electron applications, opaque cloud services, or low-level command-line tools. The desired workflow is much smaller: choose one image, choose 2× or 4×, run the upscale locally, inspect the result, and save it.

The product must make that workflow dependable across macOS, Windows, and Linux without uploading the image, exposing engine complexity, inventing progress percentages, or silently damaging color, transparency, metadata, or existing files.

## Solution

Build **Nggedhekaké Gambar**, an open-source, no-webview Rust desktop application with a Slint interface and a pinned external Real-ESRGAN NCNN/Vulkan engine.

The v1 experience opens directly into a two-column workspace. The user chooses or drops one PNG, JPEG, or WebP image, keeps the default 2× scale or selects 4×, and starts the upscale. The general-purpose model is selected automatically; an illustration model remains available as a secondary option. The app shows truthful processing phases and elapsed time, then presents a swipe comparison. The result remains temporary until the user chooses **Save**, which opens the native save dialog with a predictable filename beside the source by default.

All image processing remains local. The app normalizes input color to sRGB, preserves transparency, applies orientation, preserves DPI, strips sensitive metadata, and enforces a 200-megapixel output ceiling. It ships as signed/notarized DMG and signed MSI artifacts plus an AppImage, with checksums for every artifact.

## User Stories

1. As a creative user, I want to choose one local image, so that I can upscale it without learning a command-line tool.
2. As a creative user, I want to drag and drop an image into the workspace, so that starting a job is fast.
3. As a creative user, I want a native file picker alternative to drag and drop, so that the workflow is keyboard- and pointer-friendly.
4. As a creative user, I want the app to accept PNG, JPEG, and WebP files, so that common creative assets work predictably.
5. As a creative user, I want unsupported or animated files rejected before processing, so that failures are immediate and understandable.
6. As a creative user, I want to preview the selected source, so that I can confirm I chose the correct image.
7. As a creative user, I want EXIF orientation applied before preview and processing, so that the image is not rotated incorrectly.
8. As a creative user, I want embedded color profiles normalized to sRGB, so that the result looks consistent across operating systems and viewers.
9. As a creative user, I want transparent PNG and WebP images to remain transparent, so that design assets are not flattened.
10. As a creative user, I want a 2× scale option, so that I can enlarge an image without producing an unnecessarily large result.
11. As a creative user, I want a 4× scale option, so that I can produce a larger result when needed.
12. As a first-time user, I want 2× preselected, so that the default is conservative about runtime, memory, and disk use.
13. As a general creative user, I want the photo-and-general model selected automatically, so that model selection is not a required step.
14. As an illustration user, I want an illustration model available under secondary options, so that line art and flat-color images can use a better fit.
15. As a user, I want model names described in plain language, so that I do not need to understand Real-ESRGAN internals.
16. As a user, I want one obvious Upscale action, so that the primary flow stays focused.
17. As a user, I want the app to validate the source, model, engine, output dimensions, and temporary storage before starting, so that avoidable failures happen early.
18. As a user, I want images whose estimated result exceeds 200 megapixels rejected with an explanation, so that the app does not start an unsafe job.
19. As a user, I want processing states such as Preparing, Upscaling, Building preview, Done, Failed, and Cancelled, so that I know what the app is doing.
20. As a user, I want elapsed time shown during processing, so that the app feels alive without presenting a fabricated percentage.
21. As a user, I want the app to avoid numeric progress when the engine does not expose a stable progress protocol, so that status remains truthful.
22. As a user, I want to stop an active upscale, so that I can recover from a wrong selection or unexpectedly long run.
23. As a user, I want Stop to terminate the engine process tree and clean this run’s temporary files, so that cancellation is immediate and leaves no partial result.
24. As a user, I want prior completed files left untouched when I cancel, so that cancellation cannot destroy earlier work.
25. As a user, I want the selected source and settings retained after a processing failure, so that I can diagnose or retry without starting over.
26. As a user, I want failures written as actionable messages, so that I know whether the problem is the file, engine, model, GPU, storage, or save destination.
27. As a user, I want the completed result shown before saving, so that I can decide whether it is worth keeping.
28. As a user, I want a swipe comparison between source and result, so that I can inspect detail changes directly.
29. As a user, I want previews downsampled independently of the full-resolution result, so that very large outputs do not make the interface unusable.
30. As a user, I want the full-resolution temporary result preserved while viewing a downsampled preview, so that saving does not reduce quality.
31. As a user, I want Save to open the operating system’s native save dialog, so that destination and filename behavior is familiar.
32. As a user, I want the save dialog to default beside the source image, so that the result is easy to find.
33. As a user, I want the proposed filename to follow `stem-2x-photo.ext` or the equivalent selected scale/model slug, so that output names are predictable.
34. As a user, I want the source format preserved by default, so that the app does not change formats unexpectedly.
35. As a user, I want to save as PNG, JPEG, or WebP, so that I can choose a common output format.
36. As a user, I want PNG encoded losslessly, so that no additional image loss is introduced.
37. As a user, I want JPEG encoded at quality 92, so that v1 has a strong fixed quality default without another setting.
38. As a user, I want lossy WebP encoded at quality 90, so that v1 has a strong fixed quality default without another setting.
39. As a user, I want JPEG refused when the result contains transparency, so that alpha is never destroyed silently.
40. As a user, I want replacement confirmation handled by the native save dialog, so that existing files are never overwritten silently.
41. As a user, I want a failed save to retain the temporary result, so that I can choose another destination without rerunning the upscale.
42. As a user, I want confirmation before an unsaved result is discarded by a new source, changed processing settings, rerun, or app close, so that expensive work is not lost accidentally.
43. As a user, I want only one temporary result at a time, so that v1 does not hide a history or queue behind the simple workflow.
44. As a privacy-conscious user, I want all source and result pixels processed locally, so that my images never leave my computer.
45. As a privacy-conscious user, I want update checks disabled until I explicitly opt in, so that local-only operation is the default.
46. As a user who opts into update checks, I want a notification and download link rather than silent installation, so that I control upgrades.
47. As a privacy-conscious user, I want no analytics, crash uploads, identifiers, or background telemetry, so that app use is not tracked.
48. As a returning user, I want scale, model, format policy, update consent, and window state remembered, so that repeated use is faster.
49. As a privacy-conscious user, I want source paths, output paths, and recent jobs omitted from persisted settings, so that the app does not retain file history.
50. As a user needing support, I want seven days of size-capped rotating local logs, so that a recent failure can be diagnosed after restart.
51. As a user sharing diagnostics, I want exported paths and filenames redacted by default, so that support data does not expose my filesystem.
52. As a user sharing diagnostics, I want images excluded from the diagnostics bundle, so that support export cannot leak image content.
53. As a user, I want diagnostics to verify engine presence, checksums, model files, app version, and platform details, so that installation problems are visible.
54. As a user, I want an on-demand smoke inference in Diagnostics, so that Vulkan or MoltenVK readiness is proven by real execution.
55. As a user on unsupported hardware, I want a clear Vulkan-readiness failure and remediation guidance, so that the app fails honestly.
56. As a user on unsupported hardware, I do not want an unexpectedly slow CPU fallback, so that runtime behavior remains predictable.
57. As a macOS user on Apple Silicon, I want a signed and notarized DMG, so that installation does not require bypassing Gatekeeper.
58. As a Windows x64 user, I want a signed MSI, so that installation and removal use familiar operating-system behavior.
59. As an Ubuntu x64 user, I want an AppImage, so that I can run the app without a distribution-specific installation path.
60. As a security-conscious user, I want SHA-256 checksums for every app and engine artifact, so that downloads can be verified.
61. As a keyboard user, I want every primary control, swipe mode control, secondary option, diagnostics action, and save action reachable without a pointer, so that the workflow is accessible.
62. As an assistive-technology user, I want controls, errors, status changes, and image states exposed semantically, so that the app is understandable without relying on color alone.
63. As an international user, I want concise English interface copy in v1, so that the initial release has one fully reviewed language.
64. As a user, I want the app to open directly into the workspace without a setup wizard, so that I can choose an image immediately.
65. As a user, I want a two-column, canvas-dominant layout, so that controls stay compact and the source/result remains central.
66. As a user, I want the app to open the saved file or containing folder, so that I can continue my workflow quickly.
67. As a maintainer, I want the UI to submit typed upscale requests instead of raw command arguments, so that presentation changes cannot corrupt engine invocation.
68. As a maintainer, I want the exact engine release and checksums pinned per target, so that builds are reproducible.
69. As a maintainer, I want bundled model metadata and checksums in a manifest, so that labels, licensing, scale support, and files are validated independently of UI code.
70. As a maintainer, I want engine stdout and stderr mapped into typed events, so that state changes and diagnostics do not depend on UI string parsing.
71. As a maintainer, I want one active job and one result state, so that cancellation and unsaved-result behavior have a small deterministic state machine.
72. As a maintainer, I want source orientation, ICC conversion, alpha handling, model inference, preview generation, and final encoding separated into explicit pipeline stages, so that each observable boundary can be verified.
73. As a maintainer, I want the external engine isolated behind an adapter, so that a future engine upgrade does not leak across the app.
74. As a maintainer, I want app, Slint, engine, model, and third-party notices packaged explicitly, so that redistribution obligations are reviewable.
75. As a release maintainer, I want the same golden image corpus exercised on all three operating systems, so that cross-platform output regressions are caught before release.

## Implementation Decisions

- **Release boundary:** v1 supports one image at a time. Batch folders, custom models, job history, queues, custom width, 3× scale, TTA, manual tile sizing, GPU selection, compression sliders, and metadata-copy controls are not part of v1.
- **Supported platforms:** release-blocking targets are macOS 13+ on arm64, Windows 10 22H2/Windows 11 on x86_64, and Ubuntu 22.04+ on x86_64. Other contemporary Linux distributions are best-effort.
- **UI architecture:** the application uses Slint with no HTML/webview layer. The interface opens directly to a two-column workspace: compact controls on the left, a canvas-dominant source/result area on the right, and a thin status area.
- **Licensing:** application source is dual-licensed `MIT OR Apache-2.0`. Distribution uses Slint’s royalty-free desktop terms with required attribution unless the project deliberately changes to a compatible alternative. Engine, model, and dependency notices remain separate.
- **Application identity:** the display name is `Nggedhekaké Gambar`; executable and artifact names use the ASCII slug `nggedhekake-gambar`. The default package identifier is `io.github.bangadam.nggedhekake-gambar` unless the publishing organization establishes a different owned namespace before signing.
- **Engine:** bundle checksum-pinned official `xinntao/Real-ESRGAN-ncnn-vulkan` v0.2.0 artifacts. Do not use the AGPL Upscayl fork in v1. The app must record the exact source URL, release tag, artifact digest, bundled license, and build target for each platform.
- **Engine release risk:** v0.2.0 is an April 2022 release. Shipping is gated on the real-engine golden corpus and Vulkan/MoltenVK smoke inference passing on every release-blocking target. A failing target is not masked by a CPU fallback.
- **Models:** bundle two NCNN model pairs: `realesrgan-x4plus` labeled **Photo & general** and `realesrgan-x4plus-anime` labeled **Illustration**. Photo & general is the automatic default; Illustration is a secondary option. Distribution is gated on recorded provenance, redistribution permission, license notices, and SHA-256 checksums for every `.param` and `.bin` file.
- **Primary request:** the UI creates a typed request containing source identity, scale (`2` or `4`), model identity, and validated source properties. The UI never constructs process arguments.
- **Job state model:** the controller owns one state machine: `Empty → Ready → Preparing → Upscaling → BuildingPreview → ResultUnsaved → Saving → Saved`, with `Failed` and `Cancelled` terminal branches. A terminal state can return to `Ready`; replacing `ResultUnsaved` requires explicit discard confirmation.
- **Single active process:** only one engine process may run. Starting another job while processing is unavailable; Stop remains available.
- **Engine adapter:** the adapter locates the pinned executable and model files, verifies checksums, constructs arguments, spawns the child in a dedicated process group/job object, captures stdout/stderr, terminates the entire process tree, and emits typed lifecycle/log events.
- **Progress:** v1 guarantees phase labels and elapsed time, not a percentage. Upstream verbose text may be retained in logs but does not become a user-facing numeric progress contract.
- **Cancellation:** Stop terminates the process tree, removes files owned by the active run, preserves prior completed files, and transitions to `Cancelled`. Stale run-owned temporary files are removed on the next launch.
- **Input contract:** officially support static PNG, JPEG, and WebP. Reject unsupported formats, animated WebP, corrupt files, zero-sized images, and jobs whose computed result exceeds 200 megapixels.
- **Preprocessing:** decode the source in Rust, apply orientation, convert embedded ICC color to sRGB, preserve DPI, separate alpha when present, and write an engine-compatible temporary input. Processing must not depend on source filename encoding.
- **Transparency:** model inference applies to color channels. Alpha is resized to the exact output dimensions with a high-quality deterministic filter and recombined without premultiplication artifacts.
- **Engine execution:** run the selected bundled model at exactly 2× or 4×. Tile size and GPU selection remain engine-auto in v1.
- **Postprocessing and preview:** preserve a full-resolution temporary result and independently generate a bounded preview image for Slint. The preview must never decode or retain both full-resolution source and result solely to render the swipe view when bounded representations suffice.
- **Preview interaction:** before processing, show the source. After processing, show a swipe comparison with accessible keyboard alternatives for changing the divider and switching to source-only/result-only views.
- **Save flow:** inference does not write the user’s final file. The completed result remains temporary until Save opens the native save dialog. The dialog defaults to the source directory and proposes `<source-stem>-<scale>x-<model-slug>.<source-extension>`.
- **Collision behavior:** replacement confirmation is delegated to the native save dialog. The app never silently overwrites an existing file.
- **Format policy:** preserve the source format by default; allow PNG, JPEG, and WebP through the save filename/type selection. PNG is lossless, JPEG uses fixed quality 92, and lossy WebP uses fixed quality 90. A transparent result cannot be saved as JPEG.
- **Metadata policy:** output is tagged sRGB, preserves DPI, and excludes source EXIF, GPS, comments, and other ancillary metadata. Orientation is baked into pixels rather than copied as an orientation tag.
- **Save failures:** failed encoding or writing leaves the temporary result intact and returns to `ResultUnsaved`, allowing another Save attempt without rerunning inference.
- **Temporary storage:** each run owns a unique app-managed temporary directory. Preflight checks estimate enough free space for normalized input, engine output, preview, and final encoding overhead before process launch.
- **Diagnostics:** startup performs lightweight binary/model/checksum checks only. An explicit Diagnostics action runs a tiny bundled smoke image through the real engine and reports engine, model, GPU/Vulkan or MoltenVK, operating system, and app versions.
- **Unsupported hardware:** no CPU fallback and no custom engine path. The app reports that Vulkan/MoltenVK inference is unavailable and provides platform-appropriate remediation.
- **Settings:** persist only model, scale, output-format policy, update-check consent, window geometry, and schema version. Do not persist source paths, output paths, recent jobs, or image thumbnails.
- **Settings schema:** the store uses a versioned Rust-managed format with explicit migration. Invalid or newer unsupported settings fall back safely with a diagnostic entry rather than preventing startup.
- **Logs:** retain structured local logs for seven days under a total size cap. Logs may retain operational paths locally. Manual diagnostics export redacts home paths and source/output names by default and never includes image bytes.
- **Network and privacy:** all image work is offline. The only v1 network operation is an update-manifest request after explicit opt-in or an explicit manual check. There is no analytics, crash upload, account, cloud inference, or background telemetry.
- **Updates:** notify the user that a signed release is available and open the project download page. Do not download or install updates automatically in v1.
- **Packaging:** publish a signed/notarized DMG for macOS arm64, an Authenticode-signed MSI for Windows x86_64, and an x86_64 AppImage for Ubuntu/Linux. Publish SHA-256 checksums for every package and bundled engine artifact.
- **Release channel prerequisite:** the project must establish its public source/release repository and signing identities before update checks and final packages can ship. Until then, update checks remain disabled in development builds.
- **Accessibility:** every action is keyboard reachable; focus is always visible; status and errors are announced semantically; color is never the only state signal; swipe comparison has non-pointer controls.
- **Language:** v1 UI and support copy are English only. Copy is short, operational, and free of AI marketing language.
- **Observable app seams:** the highest test seam is the application controller driven through fake ports for file dialog, filesystem/temp storage, clock, engine process, settings, and update source. Image transformation and real process execution remain separately testable boundaries because they cross native/engine contracts.

## Testing Decisions

- Good tests assert observable contracts: accepted input, produced dimensions and formats, state transitions, process arguments, cancellation effects, saved bytes, settings migration, diagnostics results, and privacy behavior. Tests do not assert Slint widget trees, private helper calls, source text, or incidental log wording.
- **Primary application seam:** drive the controller from `Ready` through processing, preview, save, failure, cancellation, discard confirmation, and retry using fake external ports. Verify visible state, enabled actions, emitted requests, and owned-file cleanup.
- **Image pipeline seam:** use fixture images covering JPEG EXIF orientation, non-sRGB ICC input, transparent PNG, transparent WebP, opaque WebP, malformed input, and output-size boundaries. Verify exact dimensions, sRGB tagging, DPI preservation, alpha behavior, metadata stripping, and format encoding defaults.
- **Engine adapter seam:** execute against a deterministic fixture process that records arguments, emits stdout/stderr, exits successfully or unsuccessfully, spawns a child, and delays for cancellation. Verify quoting for Unicode/spaces, typed events, exit mapping, and process-tree termination on all target operating systems.
- **Model registry seam:** verify manifest loading, plain-language labels, default selection, allowed scales, file-pair completeness, provenance fields, license fields, and checksum failures.
- **Save/export seam:** verify default filename generation, extension-to-encoder selection, fixed JPEG/WebP quality policy, transparent-JPEG refusal, native-dialog cancellation, replacement delegation, save failure recovery, and no mutation of the temporary result.
- **Settings seam:** verify first-run defaults, round-trip persistence of allowed preferences, omission of file history, migration from every shipped schema version, and recovery from malformed/newer settings.
- **Diagnostics/logging seam:** verify lightweight readiness checks, real smoke-test result mapping, seven-day/size-cap rotation, diagnostics redaction, and the invariant that no image bytes enter logs or support bundles.
- **Update/privacy seam:** verify zero network requests before opt-in, opt-in persistence, manual check behavior, release notification, and no automatic download/install path.
- **UI smoke seam:** exercise the packaged primary flow with keyboard and pointer: choose/drop image, select 2×/4×, reveal Illustration, start, cancel, inspect swipe comparison, save, retry failed save, open saved file/folder, and confirm discard of an unsaved result.
- **Golden corpus release gate:** process the same fixed photo, illustration, transparency, color-profile, Unicode-path, and boundary-size corpus with the real pinned engine on macOS arm64, Windows x86_64, and Ubuntu x86_64. Require correct dimensions and formats plus perceptual similarity within an approved tolerance; do not require byte- or pixel-identical GPU output.
- **Hardware release gate:** run the on-demand smoke inference on representative Apple Silicon/MoltenVK, Windows Vulkan, and Ubuntu Vulkan hardware. Static binary launch alone is insufficient evidence.
- **Packaging gate:** install, launch, process, save, uninstall where applicable, verify signatures/notarization/checksums, and confirm bundled license notices on every official artifact.
- This repository is currently documentation-only and has no existing code or test prior art. New tests should follow standard Rust unit/integration conventions and keep OS/hardware-dependent real-engine checks separate from deterministic default test runs.

## Out of Scope

- Batch folder processing.
- Multiple active jobs, background queues, and recent-job history.
- Custom NCNN models or custom engine binaries.
- Model downloads inside the app.
- Cloud inference, accounts, sync, collaboration, or image uploads.
- CPU inference fallback.
- Video, GIF, or animated WebP upscaling.
- Face restoration or a separate face pipeline.
- Training, fine-tuning, or model conversion tools.
- 3× scaling, custom width/height, arbitrary resize, or cropping.
- TTA, manual tile size, GPU selection, thread tuning, and other engine flags.
- Compression sliders or per-save quality controls.
- Full EXIF/GPS/comment preservation.
- Saving transparent results as JPEG or choosing a flatten background.
- Multiple unsaved variants or persistent result caches.
- Automatic update download or installation.
- Analytics, crash uploads, or telemetry.
- Localization beyond English.
- Mobile, web, or webview-based versions.
- DEB, RPM, package-manager, or store distribution as official v1 artifacts.
- Intel macOS, Windows ARM64, and Linux ARM64 as release-blocking targets.
- Reimplementing Real-ESRGAN or NCNN in Rust.

## Further Notes

- Product posture: **choose image → choose scale → upscale → inspect → save → done**.
- The performance strategy is to keep the proven NCNN/Vulkan engine outside the UI process and replace the heavyweight shell, not to invent a new upscaling algorithm.
- The interface should feel like a calm native image utility: no gradients, neon, glassmorphism, hype copy, navigation sprawl, or decorative animation.
- The preview is the visual center, but the full-resolution result is a file artifact rather than UI state. Keep preview memory bounded.
- Official Real-ESRGAN v0.2.0 was selected over the more recent AGPL Upscayl fork. Its age is a material compatibility risk and is addressed by mandatory real-engine release gates, not by abstraction or fallback.
- The upstream v0.2.0 release provides cross-platform executable artifacts; the inspected macOS artifact is universal, while v1 support remains intentionally arm64-only.
- Official NCNN model provenance and redistribution terms must be recorded before packaging. A model found in a third-party bundle is not acceptable provenance.
- Slint attribution and all engine/model notices must be visible from an About/Licenses surface and included with distributed artifacts.
- Implementation is tracked in [GitHub issues #1–#17](https://github.com/bangadam/nggedhekake-gambar/issues). The public repository now exists; signing identities remain release prerequisites, not reasons to expand product scope.
