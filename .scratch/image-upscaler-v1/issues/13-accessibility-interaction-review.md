# Review and close accessibility and interaction gaps

Status: ready-for-human
Type: HITL

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Review the complete primary workflow as a human using keyboard, pointer, reduced-motion expectations, and available platform accessibility tooling, then close every release-blocking gap found in the same ticket. Confirm that the two-column canvas-dominant design remains the simple “choose image → choose scale → upscale → inspect → save” experience rather than accumulating navigation or advanced controls.

## Acceptance criteria

- [ ] A human completes selection, 2×/4× choice, secondary model choice, start, cancel, comparison, save, retry, diagnostics, update consent, and discard flows using only the keyboard.
- [ ] Every control, phase, validation error, failure, unsaved warning, and diagnostic result has correct accessible semantics and visible focus.
- [ ] The swipe divider has keyboard operation plus source-only/result-only alternatives and never relies on color alone.
- [ ] Focus order, focus restoration after dialogs, screen-reader announcements, contrast, text size, and reduced-motion behavior pass review on all target OSes where tooling differs.
- [ ] English copy is short, operational, consistent with the PRD vocabulary, and contains no AI marketing language.
- [ ] The final workspace remains two-column and canvas-dominant with model selection secondary and no setup wizard.
- [ ] All release-blocking findings discovered by the review are implemented and regression-covered before this ticket is complete.

## Blocked by

- [Save the normalized result through the native dialog](05-save-normalized-result.md)
- [Inspect results with a bounded swipe comparison](06-swipe-result-comparison.md)
- [Add safe 4× processing and resource preflight](07-safe-4x-processing.md)
- [Offer the secondary Illustration model](08-illustration-model-option.md)
- [Cancel the engine process tree and clean the run](09-cancel-and-clean-run.md)
- [Protect unsaved results and recover failed saves](10-protect-unsaved-result.md)
- [Persist private-safe preferences and opt-in updates](11-private-preferences-updates.md)
- [Diagnose engine readiness and export redacted support data](12-diagnostics-support-export.md)
