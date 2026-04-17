## Goal

Publish `video.minimax` as a provider-specific public mirror family using MiniMax's official HTTP video protocol and official paths, while keeping the shared `video.openai` `/v1/videos*` contract unchanged.

## Protocol Surface

MiniMax does not expose an OpenAI-compatible `/v1/videos*` transport. To remain base-URL-switch compatible with MiniMax's own clients, the gateway must publish the provider-native paths exactly as-is:

- `POST /v1/video_generation`
- `GET /v1/query/video_generation`
- `GET /v1/files/retrieve`

No wrapper routes such as `/video/minimax/*`, `/v1/video/minimax/*`, or `/files/minimax/*` should be added.

## Runtime Design

Reuse the existing provider-official JSON relay helper and the existing mirror-identity-constrained stateful planner:

- stateless mode only relays when `mirror_protocol_identity == "minimax"`
- stateful mode only selects providers whose mirror identity is `minimax`
- every successful stateful request persists a routing decision log entry

The capability remains `videos` for routing and usage storage, while the OpenAPI tag is `video.minimax`.

## Usage Recording

MiniMax's protocol is task-oriented and does not expose a stable completed-duration field on the query or file-retrieve responses that the gateway can trust for settlement. This slice therefore records lightweight usage for the three public routes instead of inventing media duration:

- `video.minimax.generate`
- `video.minimax.query`
- `video.minimax.files.retrieve`

Reference ids should be preserved when present:

- generation: `task_id`
- query: `task_id`
- file retrieve: `file.file_id`, falling back to provider-native `file_id` when needed

## OpenAPI

Publish a new `video.minimax` tag with three operations:

- `video_minimax_generation_create`
- `video_minimax_generation_query`
- `video_minimax_file_retrieve`

Request and response bodies can remain `serde_json::Value` because the compatibility target is MiniMax's native JSON transport, not a gateway-defined abstraction.

## Documentation Impact

Update English and Chinese gateway docs and compatibility docs so `video.minimax` moves from reserved-governance wording to active-public wording, while the remaining future video families stay reserved.
