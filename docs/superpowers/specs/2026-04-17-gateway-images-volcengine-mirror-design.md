# Gateway Images Volcengine Mirror Design

## Superseded Status

This slice is implemented, but one detail is outdated: `images.nanobanana` is no longer treated as a reserved future image family. Nano Banana is governed under `code.gemini` because it stays on Google's official Gemini `generateContent` protocol surface.

## Goal

Publish `images.volcengine` as an active provider-specific mirror family on Volcengine Ark's official image-generation transport without adding wrapper routes.

## Scope

This slice activates the official Volcengine image-generation surface on:

- `POST /api/v3/images/generations`

The public contract must remain a strict mirror: official Volcengine image clients should only need to switch `base_url`.

## Design

`images.volcengine` is a provider-specific top-level family because the official Volcengine Ark public transport does not reuse the gateway's shared `/v1/images/*` contract verbatim. Current official examples show an `api/v3` base URL and image generation through the official `images.generate` surface, so the gateway should publish the matching HTTP path exactly as-is.

Stateful behavior:

- parse the request JSON and require a non-empty `model`
- select an `images` provider by request model and mirror identity `volcengine`
- relay the request as passthrough JSON to the official upstream path
- record usage/billing on a dedicated route key for the provider-specific family

Stateless behavior:

- relay only when `upstream.mirror_protocol_identity() == "volcengine"`
- preserve the official path and JSON body unchanged

## Route Key

- `images.volcengine.generate`

## OpenAPI and Docs

OpenAPI should publish a new `images.volcengine` tag and expose `POST /api/v3/images/generations` under that tag. Public docs should move `images.volcengine` from reserved-only to active, keep `images.midjourney` unpublished, and document that Nano Banana belongs to `code.gemini` on the official Gemini `generateContent` surface instead of a separate image family.

## Deferred

This slice does not invent `images.volcengine` wrapper routes and does not guess at extra edit, variation, or async task-query paths until those protocols are confirmed as stable public surfaces.
