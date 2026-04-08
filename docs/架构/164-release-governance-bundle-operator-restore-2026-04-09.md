# Release Governance Bundle For Operator Restore

> Date: 2026-04-09
> Goal: reduce blocked-host restore friction by publishing one repository-owned governance bundle without changing governed truth or attestation semantics.

## 1. Problem

- `162` added restore into default latest paths.
- `163` made the real CLI consume those latest paths.
- operators still had to download five artifacts manually before restore.

## 2. Design

- keep the five existing latest governance artifacts as the only governed source files.
- add one derived bundle:
  - output dir: `artifacts/release-governance-bundle/`
  - contents:
    - `docs/release/release-window-snapshot-latest.json`
    - `docs/release/release-sync-audit-latest.json`
    - `docs/release/release-telemetry-export-latest.json`
    - `docs/release/release-telemetry-snapshot-latest.json`
    - `docs/release/slo-governance-latest.json`
    - `release-governance-bundle-manifest.json`

## 3. Validation Boundary

- bundle creation must validate all five inputs with the repository-existing validators first.
- bundle creation is not allowed to fabricate or recompute evidence.
- invalid latest artifacts must fail bundling.

## 4. Workflow Placement

- only `web-release` publishes the bundle artifact.
- bundle is created after the five governed latest artifacts already exist.
- bundle upload happens before the governance gate completes, matching the existing evidence-upload posture.

## 5. Manifest Contract

- fields:
  - `version`
  - `generatedAt`
  - `bundleEntryCount`
  - `artifacts[].id`
  - `artifacts[].relativePath`
  - `artifacts[].sourceRelativePath`
  - `restore.command`
- restore command stays repository-owned:
  - `node scripts/release/restore-release-governance-latest.mjs --artifact-dir <downloaded-dir>`

## 6. Honest Boundary

- bundle is a transport convenience layer only.
- attestation subjects remain the five governed latest files, not the bundle.
- release truth still comes from governed evidence or live collection, not from the existence of a bundle.
