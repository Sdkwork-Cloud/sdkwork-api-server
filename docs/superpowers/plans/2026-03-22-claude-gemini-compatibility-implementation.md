# Claude And Gemini Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class Claude Code and Gemini CLI gateway compatibility to `sdkwork-api-router` without creating a second routing system, so both clients can use existing model routing, provider selection, quota enforcement, and usage recording.

**Architecture:** Implement protocol translation only at the HTTP gateway boundary. Anthropic Messages and Gemini GenerateContent requests are converted into the existing OpenAI-compatible chat completion execution flow, and responses or SSE chunks are translated back into Anthropic or Gemini wire formats. Preserve provider compatibility by extending the existing chat-completions request DTOs to pass through protocol-specific options such as tools, system prompts, token limits, and sampling fields.

**Tech Stack:** Rust, Axum, serde, serde_json, futures-util, bytes, sqlx, existing sdkwork-api router crates

---

## Chunk 1: Request Surface And TDD Coverage

### Task 1: Add failing Anthropic compatibility tests

**Files:**
- Create: `crates/sdkwork-api-interface-http/tests/anthropic_messages_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/support/mod.rs`

- [ ] **Step 1: Write failing stateless tests for `POST /v1/messages` and `POST /v1/messages/count_tokens`**
- [ ] **Step 2: Write failing stateful tests that authenticate with `x-api-key`, route by request model, and record usage for `chat_completion`**
- [ ] **Step 3: Write failing streaming tests that require Anthropic SSE event framing for text responses**
- [ ] **Step 4: Run `cargo test -p sdkwork-api-interface-http anthropic_messages_route -- --nocapture` and verify the failures are missing routes or missing translation**
- [ ] **Step 5: Keep the assertions focused on protocol shape, auth, and routing evidence**

### Task 2: Add failing Gemini compatibility tests

**Files:**
- Create: `crates/sdkwork-api-interface-http/tests/gemini_generate_content_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/support/mod.rs`

- [ ] **Step 1: Write failing stateless tests for `POST /v1beta/models/{model}:generateContent`, `:streamGenerateContent`, and `:countTokens`**
- [ ] **Step 2: Write failing stateful tests that authenticate with `x-goog-api-key` and `?key=` query support**
- [ ] **Step 3: Write failing streaming tests that require Gemini SSE chunks with `alt=sse`**
- [ ] **Step 4: Run `cargo test -p sdkwork-api-interface-http gemini_generate_content_route -- --nocapture` and verify the failures are route or translation gaps**
- [ ] **Step 5: Lock assertions to request translation, response translation, and routing evidence**

## Chunk 2: Gateway Translation Implementation

### Task 3: Extend chat-completions request DTOs for passthrough fields

**Files:**
- Modify: `crates/sdkwork-api-contract-openai/src/chat_completions.rs`
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs` (serialization should continue to work unchanged)

- [ ] **Step 1: Add `serde(flatten)` support to `CreateChatCompletionRequest` for extra top-level fields**
- [ ] **Step 2: Add `serde(flatten)` support to `ChatMessageInput` for per-message protocol fields such as `tool_call_id`, `name`, and `tool_calls`**
- [ ] **Step 3: Re-run the new compatibility tests and confirm the failures move from serialization loss to missing routes or translators**
- [ ] **Step 4: Re-run existing `chat_route` and `chat_stream_route` tests to ensure no OpenAI regression**

### Task 4: Add Anthropic compatibility handlers and translators

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/compat_anthropic.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

- [ ] **Step 1: Define Anthropic request and response DTOs only for the gateway boundary**
- [ ] **Step 2: Implement request translation from Anthropic Messages into OpenAI chat-completions payloads, including system prompts, tool definitions, tool results, and text content**
- [ ] **Step 3: Implement stateful and stateless handlers that reuse the existing chat-completions relay flow and local fallback flow**
- [ ] **Step 4: Implement non-stream and SSE response translation back into Anthropic message and streaming event shapes**
- [ ] **Step 5: Re-run `anthropic_messages_route` until green**

### Task 5: Add Gemini compatibility handlers and translators

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/compat_gemini.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

- [ ] **Step 1: Define Gemini request and response DTOs only for the gateway boundary**
- [ ] **Step 2: Implement request translation from Gemini `contents` and `tools` into OpenAI chat-completions payloads**
- [ ] **Step 3: Implement stateful and stateless handlers that reuse the existing chat-completions relay flow and local fallback flow**
- [ ] **Step 4: Implement non-stream and SSE response translation back into Gemini `GenerateContentResponse` and `CountTokensResponse` shapes**
- [ ] **Step 5: Re-run `gemini_generate_content_route` until green**

## Chunk 3: Auth, Streaming, And Documentation Polish

### Task 6: Add compatibility-specific auth extraction and stream adaptation

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/compat_anthropic.rs`
- Modify: `crates/sdkwork-api-interface-http/src/compat_gemini.rs`

- [ ] **Step 1: Add compatibility auth helpers that accept `Authorization: Bearer`, `x-api-key`, `x-goog-api-key`, and Gemini `key` query parameters without weakening the existing OpenAI route contract**
- [ ] **Step 2: Add shared SSE parsing and re-emission helpers for OpenAI chunk streams to Anthropic or Gemini event streams**
- [ ] **Step 3: Verify usage recording, quota checks, and route-key logging still point at the selected model**
- [ ] **Step 4: Run compatibility tests plus existing `chat_route`, `chat_stream_route`, and `gateway_auth_context` tests**

### Task 7: Update repository docs and compatibility matrix

**Files:**
- Modify: `README.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 1: Document the new Anthropic and Gemini protocol surfaces with exact paths and auth headers**
- [ ] **Step 2: Document that compatibility uses the same routing policies and project routing preferences as the OpenAI surface**
- [ ] **Step 3: Add notes for Gemini CLI gateway mode and Claude Code LLM gateway setup**
- [ ] **Step 4: Rebuild docs only if the touched pages participate in the docs workspace checks**

## Chunk 4: Verification And Integration

### Task 8: Run focused and broader verification, then integrate safely

**Files:**
- Modify: any touched files as needed

- [ ] **Step 1: Run the new compatibility test targets and confirm they pass**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-interface-http chat_route chat_stream_route responses_route gateway_auth_context -- --nocapture`**
- [ ] **Step 3: Run `cargo test -p sdkwork-api-interface-http -- --nocapture` if targeted coverage is clean enough to justify the longer pass**
- [ ] **Step 4: Run `cargo fmt --all` and `cargo check -p sdkwork-api-interface-http`**
- [ ] **Step 5: Review `git diff`, commit the feature branch cleanly, and only then decide the safest path to move the work onto `main` without overwriting unrelated dirty changes in the source checkout**
