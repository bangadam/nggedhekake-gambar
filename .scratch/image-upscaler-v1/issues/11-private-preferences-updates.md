# Persist private-safe preferences and opt-in updates

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Remember only the approved convenience preferences and add a privacy-preserving update notification path. Scale, model, output-format policy, window state, and explicit update consent survive restart through a versioned settings schema; source/output paths and recent jobs never do. No update request occurs before consent or a manual check, and an available release only produces a notification and download-page action.

## Acceptance criteria

- [ ] First-run defaults are 2×, Photo & general, preserve source format, and update checks off.
- [ ] Only scale, model, format policy, window geometry, update consent, and schema version persist.
- [ ] Source paths, output paths, recent jobs, filenames, thumbnails, and temporary-result state never enter persisted settings.
- [ ] Every shipped schema version migrates explicitly; malformed or newer unsupported data falls back safely with a diagnostic entry.
- [ ] Automated network activity is impossible before explicit opt-in, and manual Check for updates performs exactly one requested check.
- [ ] An available release shows version information and opens the approved download page without downloading or installing anything.
- [ ] Tests verify persistence, omissions, migrations, consent transitions, zero pre-consent requests, and notification behavior.

## Blocked by

- [Establish release identity and signing channel](03-establish-release-identity.md)
- [Add safe 4× processing and resource preflight](07-safe-4x-processing.md)
- [Offer the secondary Illustration model](08-illustration-model-option.md)
