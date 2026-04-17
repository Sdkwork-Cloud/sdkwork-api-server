## Goal

Publish `video.vidu` as a provider-specific public mirror family using Vidu's official HTTP video protocol and official paths, while keeping the shared `video.openai` `/v1/videos*` family and the existing `video.minimax` family unchanged.

## Protocol Surface

Vidu does not use the OpenAI `/v1/videos*` transport. To preserve real base-URL-switch compatibility for Vidu-native clients, the gateway should publish Vidu's official routes directly:

- `POST /ent/v2/text2video`
- `POST /ent/v2/img2video`
- `POST /ent/v2/reference2video`
- `GET /ent/v2/tasks/{id}/creations`
- `POST /ent/v2/tasks/{id}/cancel`

No wrapper routes such as `/video/vidu/*`, `/v1/video/vidu/*`, or `/ent/v2/video/*` should be introduced.

## Auth Compatibility

The existing generic provider mirror relay assumes upstream `Authorization: Bearer ...`. Vidu's official protocol instead uses `Authorization: Token {api_key}`. This slice therefore must generalize provider mirror auth formatting so provider-specific mirror families can choose the correct upstream authorization style without forking the relay flow.

For this slice:

- `vidu` uses `Authorization: Token {api_key}`
- existing provider-specific mirror families remain on `Authorization: Bearer {api_key}`

## Runtime Design

Reuse the existing provider-official JSON relay helper and stateful mirror-identity-constrained planner:

- stateless mode only relays when `mirror_protocol_identity == "vidu"`
- stateful mode only selects providers whose mirror identity is `vidu`
- every successful stateful request persists a routing decision log entry

The routing capability remains `videos`, while the OpenAPI tag is `video.vidu`.

## Usage Recording

This slice records lightweight usage for:

- `video.vidu.text2video`
- `video.vidu.img2video`
- `video.vidu.reference2video`
- `video.vidu.creations.get`
- `video.vidu.cancel`

Reference ids should follow the official task lifecycle:

- create routes: `task_id` from the response
- `GET /ent/v2/tasks/{id}/creations`: path `id`
- `POST /ent/v2/tasks/{id}/cancel`: path `id`

## OpenAPI

Publish a new `video.vidu` tag with the five official operations. Request and response schemas can remain `serde_json::Value` because the compatibility target is Vidu's native JSON protocol.

## Documentation Impact

Update English and Chinese gateway docs and compatibility docs so `video.vidu` moves from reserved-governance wording to active-public wording, while the remaining future video families stay reserved.
