# Image Upscaling

Nggedhekaké Gambar exists to provide a dependable, private workflow for improving one image at a time on a Mac.

## Language

**Upscale Run**:
One local operation that transforms exactly one source image into one upscaled image.
_Avoid_: Batch job, enhancement pass

**Source Image**:
The original image selected for an upscale run. It remains unchanged.
_Avoid_: Input asset

**Upscaled Image**:
The new image produced by a completed upscale run and saved in the selected destination. It always receives a unique name and never replaces an existing file.
_Avoid_: Result asset, enhanced file

**Cancelled Run**:
An upscale run intentionally stopped before completion. It produces no upscaled image, preserves the source image and settings, and is not a failed run.
_Avoid_: Error, stopped job

**Failed Run**:
An upscale run that could not complete because of a processing or environment problem. It produces no upscaled image and preserves the source image and settings so the user can correct the problem or try again.
_Avoid_: Cancelled run, crash

**Primary Engine**:
The engine used for normal upscale runs.
_Avoid_: Default pipeline

**Alternative Engine**:
The other bundled engine, available only through an eligible cross-engine retry.
_Avoid_: Backup engine, secondary mode

**Engine Preference**:
The user's device-level choice of which bundled engine acts as the primary engine for future upscale runs.
_Avoid_: Engine selection, per-run engine

**Cross-Engine Retry**:
A user-initiated new upscale run that follows a failed run, reuses its source image, destination, model, scale, and format, and uses the alternative engine. Changing any reused value ends the cross-engine retry path.
_Avoid_: Automatic fallback, continued run
