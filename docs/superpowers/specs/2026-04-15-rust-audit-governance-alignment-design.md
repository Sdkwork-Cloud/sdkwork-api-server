# Rust Audit Governance Alignment Design

**Date:** 2026-04-15

## Goal

Close the last commercial-readiness gap in the Rust supply-chain hardening slice by aligning three things that are currently out of sync:

- the real `cargo audit` result
- the repository's dependency-audit policy
- the CI gates that are supposed to detect governance drift

The goal is not to add new runtime behavior. The goal is to make the already-completed hardening work trustworthy, repeatable, and non-regressing.

## Current Evidence

- `cargo audit --json --no-fetch --stale` is now down to zero vulnerabilities and zero warnings in the active workspace lockfile.
- `Cargo.lock` no longer contains `rand 0.8.5`, `paste 1.0.15`, or `daemonize 0.5.0`.
- vendored `pingora-*` crates and `vendor/sqlx-postgres-0.8.6` now resolve `rand = "0.10.1"`.
- `scripts/check-rust-dependency-audit.policy.json` still carries an allowlist entry for `RUSTSEC-2026-0097`.
- `node --test scripts/check-rust-dependency-audit.test.mjs` currently fails because the test expects `allowedWarnings` to be empty while the policy still contains the stale exception.
- `.github/workflows/rust-verification.yml` triggers on changes to the dependency-audit tests, but the workflow does not execute those tests. It only runs `node scripts/check-rust-verification-matrix.mjs --group ...`.

## Problem Statement

The repository has already removed the previously tracked supply-chain warnings from the shipping dependency graph, but the governance layer is still inconsistent:

- policy says there is a remaining exception
- the test suite says there must be no remaining exception
- CI does not fully enforce the governance tests that would catch this mismatch

This is a commercial-delivery problem even though the runtime dependency graph is now clean. A release process is not trustworthy when the audit result, policy file, and CI gate can disagree without the main PR workflow failing.

## Options Considered

### Option A: Only delete the stale policy exception

Pros:

- smallest code change
- fixes the immediately failing Node test

Cons:

- leaves the CI workflow blind to future governance-test regressions
- does not fix the gap between "workflow watched these files" and "workflow actually ran the assertions"
- still leaves stale security narrative in design and plan documents

### Option B: Align policy, CI, and documentation together

Pros:

- resolves the current contradiction at the source-of-truth level
- makes future policy drift fail in CI instead of relying on manual vigilance
- keeps the security documentation consistent with the real dependency graph
- stays narrowly scoped to governance, not runtime behavior

Cons:

- slightly broader than a one-line policy cleanup
- requires touching workflow and review docs in addition to the policy

### Option C: Expand the scope into release-pipeline hard gates now

Pros:

- strongest long-term release governance
- would make release workflows independently re-validate the same rules

Cons:

- broadens the current slice beyond the actual blocking defect
- mixes immediate PR-gate correctness work with release-pipeline enhancement
- risks turning a governance-alignment fix into a larger delivery stream

## Recommendation

Choose Option B.

The runtime-facing dependency problem is already solved. The remaining defect is governance inconsistency, so the fix should directly align governance artifacts rather than reopen dependency surgery or expand into a larger release-program refactor.

Option A is too narrow because it fixes the current test failure but still permits the same class of drift to return silently. Option C has merit, but it should be a later enhancement after the core PR gate becomes truthful again.

## Design

### Source Of Truth Boundary

- `cargo audit` is the authoritative statement of active RustSec status for the workspace lockfile.
- `scripts/check-rust-dependency-audit.policy.json` may only contain exceptions for warnings that still exist in the active `cargo audit` result.
- once `cargo audit` reports zero warnings, the policy file must carry an empty `allowedWarnings` array rather than stale historical entries.

### CI Gate Boundary

- `rust-verification` must validate both:
  - the operational audit command path
  - the governance tests that assert workflow and policy correctness
- the dependency-audit lane should continue running `node scripts/check-rust-verification-matrix.mjs --group dependency-audit`
- that same lane must also run:

```bash
node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs
```

This closes the current blind spot where CI watches governance test files but does not execute them.

### Documentation Boundary

- design and plan documents created during the `paste`, `daemonize`, and `rand` hardening slices must reflect that the active workspace graph is now clear.
- any statement that still describes `paste`, `daemonize`, or `rand 0.8.5` as an unresolved current warning must either:
  - be updated to historical wording, or
  - be explicitly marked as pre-closure implementation context

The point is not cosmetic consistency. The point is to stop future engineering decisions from being made on stale security assumptions.

### Verification Boundary

The governance-alignment change is only acceptable if all of the following are true afterward:

- `node --test scripts/check-rust-dependency-audit.test.mjs` passes
- `node --test scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs` pass
- `node scripts/check-rust-verification-matrix.mjs --group dependency-audit` passes
- `cargo audit --json --no-fetch --stale` reports zero vulnerabilities and zero warnings
- the PR workflow definition now executes the governance tests it claims to protect

## Risks And Mitigations

### Risk: clearing the policy hides a still-active warning in another lockfile or platform lane

Mitigation:

- verify against the workspace root `Cargo.lock`, because that is the audit subject for the current hardening lane
- rerun `cargo audit --json --no-fetch --stale` after policy cleanup
- keep the verification evidence in the implementation notes and final handoff

### Risk: CI remains green while governance tests silently stop running again later

Mitigation:

- add the governance test execution explicitly to `rust-verification.yml`
- keep `scripts/rust-verification-workflow.test.mjs` aligned with the workflow content so future drift becomes review-visible

### Risk: old plan/spec text causes engineers to resurrect already-closed dependency work

Mitigation:

- update the affected `docs/superpowers/specs/` and `docs/superpowers/plans/` entries in the same slice
- use factual wording that distinguishes "historical implementation context" from "current residual risk"

## Success Condition

This work is successful when the repository's Rust supply-chain governance becomes internally consistent:

- the active audit result is clean
- the policy file reflects that clean state
- CI actually runs the governance tests that protect the policy and workflow
- the project documentation no longer reports already-closed RustSec items as current unresolved debt
