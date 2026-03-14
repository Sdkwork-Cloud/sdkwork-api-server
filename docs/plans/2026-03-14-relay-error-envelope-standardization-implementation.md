# Relay Error Envelope Standardization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Standardize relay-failure responses across the remaining HTTP route surface so upstream failures consistently return OpenAI-style JSON error envelopes.

**Architecture:** Add representative regression tests for non-core relay families, then sweep the remaining `502` relay branches in `sdkwork-api-interface-http` to route through the shared OpenAI error helper. Keep status codes and successful responses unchanged.

**Tech Stack:** Rust, Axum, serde_json, cargo test

---

### Task 1: Add representative failing route tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/assistants_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/audio_speech_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/files_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/evals_route.rs`

**Step 1: Write the failing tests**

Add tests that assert relay failures return:

- HTTP `502`
- JSON `error.message`
- JSON `error.type`
- JSON `error.code`

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test -p sdkwork-api-interface-http --test assistants_route stateless_assistants_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http --test audio_speech_route stateless_audio_speech_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http --test files_route stateless_file_content_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http --test evals_route stateless_eval_run_route_returns_openai_error_envelope_on_upstream_failure -q
```

Expected: failures because the current handlers still return plain-text `502` bodies.

### Task 2: Standardize remaining relay-failure branches

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Route remaining `BAD_GATEWAY` relay branches through the helper**

Replace remaining plain-text relay-failure branches with `bad_gateway_openai_response(...)`.

**Step 2: Keep semantics unchanged**

- do not change success paths
- do not change status codes
- do not change local fallback routing decisions

**Step 3: Run focused tests**

Run:

```powershell
cargo test -p sdkwork-api-interface-http --test assistants_route stateless_assistants_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http --test audio_speech_route stateless_audio_speech_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http --test files_route stateless_file_content_route_returns_openai_error_envelope_on_upstream_failure -q
cargo test -p sdkwork-api-interface-http --test evals_route stateless_eval_run_route_returns_openai_error_envelope_on_upstream_failure -q
```

Expected: PASS

### Task 3: Re-run broader verification

**Files:**
- Review: `sdkwork-api-interface-http`
- Review: workspace

**Step 1: Run package verification**

Run:

```powershell
cargo test -p sdkwork-api-interface-http -q
```

Expected: PASS

**Step 2: Run workspace verification**

Run:

```powershell
cargo test --workspace -q -j 1
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS

### Task 4: Commit the batch

**Files:**
- Include updated tests, implementation, and plan docs

**Step 1: Commit**

Run:

```powershell
git add crates/sdkwork-api-interface-http docs/plans/2026-03-14-relay-error-envelope-standardization-design.md docs/plans/2026-03-14-relay-error-envelope-standardization-implementation.md
git commit -m "fix: standardize relay error envelopes"
```
