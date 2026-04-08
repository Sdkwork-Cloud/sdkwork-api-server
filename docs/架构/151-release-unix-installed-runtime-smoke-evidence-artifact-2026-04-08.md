# Release Unix Installed Runtime Smoke Evidence Artifact

> Date: 2026-04-08
> Goal: close the remaining gap from `/docs/架构/150-*` by turning Unix installed-runtime smoke from step output into persisted release evidence.

## 1. Problem

- `/docs/架构/150-*` already moved Unix installed-runtime proof into the native release workflow.
- That proof still disappeared into stdout/stderr after the step finished.
- Release governance therefore lacked a retrievable artifact for audit, triage, and later comparison.

## 2. Design

1. The Unix smoke script accepts `--evidence-path`.
2. The script writes JSON evidence under `artifacts/release-governance/`.
3. Evidence is written for both success and failure.
4. Native Unix release jobs upload the evidence with `actions/upload-artifact`.
5. Governance evidence stays separate from `release-assets-*` so publish does not ship internal proof as customer-facing binaries.

## 3. Evidence Contract

- Producer: `scripts/release/run-unix-installed-runtime-smoke.mjs`
- Required workflow input:
  - `--evidence-path artifacts/release-governance/unix-installed-runtime-smoke-${{ matrix.platform }}-${{ matrix.arch }}.json`
- Evidence payload contains:
  - `ok`
  - `platform`
  - `arch`
  - `target`
  - relative `runtimeHome`
  - relative `evidencePath`
  - `healthUrls`
  - `failure.message` when not successful
  - available log excerpts when present

## 4. Workflow Rule

- Order must remain:
  1. native service build
  2. native desktop build
  3. Unix installed-runtime smoke
  4. Unix smoke evidence upload
  5. release asset packaging
- Upload must use `if: ${{ always() && matrix.platform != 'windows' }}` so failed smoke runs still preserve evidence.

## 5. Non-Goals

- Do not claim Windows parity.
- Do not merge governance evidence into release assets.
- Do not claim the current sandbox executed a full built-artifact release smoke locally.

## 6. Remaining Closure

- Windows installed-runtime evidence still needs a stable release-host implementation.
- `release-slo-governance` still blocks on missing live evidence.
- `release-window-snapshot` and `release-sync-audit` still block on host Git child-process policy.
