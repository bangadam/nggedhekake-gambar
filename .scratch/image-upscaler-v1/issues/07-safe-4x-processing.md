# Add safe 4× processing and resource preflight

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Extend the proven single-image path with a compact 2×/4× scale control while keeping 2× as the default. Before either scale starts, compute exact output dimensions, enforce the 200-megapixel ceiling, and estimate sufficient temporary-disk capacity for normalization, engine output, preview, and final encoding overhead.

## Acceptance criteria

- [ ] The primary workspace exposes only 2× and 4× and defaults to 2× on first run.
- [ ] Both scales use the same typed request, engine adapter, temporary-result, preview, and save contracts.
- [ ] Exact output dimensions are shown before processing and checked against the 200-megapixel ceiling without overflow.
- [ ] Jobs exceeding the pixel ceiling are blocked with the maximum and computed output dimensions.
- [ ] Insufficient temporary capacity is detected before engine launch with an actionable storage message.
- [ ] Boundary tests cover just-below, exactly-at, and just-above the pixel ceiling plus arithmetic overflow and low-disk cases.
- [ ] Real-engine smoke coverage proves 4× reaches an unsaved result without changing the 2× default.

## Blocked by

- [Upscale one image at 2× to an unsaved result](04-upscale-2x-unsaved-result.md)
