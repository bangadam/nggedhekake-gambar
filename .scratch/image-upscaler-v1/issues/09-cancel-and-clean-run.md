# Cancel the engine process tree and clean the run

Status: ready-for-agent
Type: AFK

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Complete the Stop path for an active upscale. Stop must terminate the engine and every descendant using native process-group/job-object behavior, clean only the active run’s owned temporary artifacts, preserve prior completed files, and land in a clear Cancelled state from which the source can be rerun.

## Acceptance criteria

- [ ] Stop is available only while a job is cancellable and responds without waiting for a nonexistent graceful engine protocol.
- [ ] Cancellation terminates the entire spawned process tree on macOS, Windows, and Linux.
- [ ] Only active run-owned temporary files are deleted; source files, saved outputs, and prior results are never removed.
- [ ] The controller reaches Cancelled exactly once despite races between process exit and cancellation.
- [ ] The selected source and processing settings remain ready for a new run after cancellation.
- [ ] Stale run-owned temporary directories left by a crash are removed safely at a later launch.
- [ ] Fixture-process tests spawn a descendant and prove process-tree termination, race handling, and scoped cleanup.

## Blocked by

- [Upscale one image at 2× to an unsaved result](04-upscale-2x-unsaved-result.md)
