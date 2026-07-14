# Establish release identity and signing channel

Status: ready-for-human
Type: HITL

## Parent

[Image Upscaler v1 PRD](../PRD.md)

## What to build

Establish the human-owned release identity required for trusted packages and opt-in update notifications. This includes the public source/release repository, final reverse-DNS package identifier, release download page, update-manifest location, and usable macOS and Windows signing identities. Development builds must remain network-silent until this contract exists.

## Acceptance criteria

- [ ] A public source and release repository is established and recorded as the authoritative release channel.
- [ ] The final package identifier and ASCII executable/artifact slug are approved under an owned namespace.
- [ ] The update-manifest URL and release download page are stable, HTTPS-served, and owned by the project.
- [ ] Apple Developer signing/notarization credentials and Windows Authenticode credentials are available through documented secure CI secrets.
- [ ] Credential ownership, renewal, revocation, and release-authority responsibilities are assigned to humans.
- [ ] A non-secret signing and publication runbook is approved without embedding credentials in the repository.

## Blocked by

None - can start immediately
