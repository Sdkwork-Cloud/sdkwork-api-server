# API 兼容矩阵

SDKWork 使用五种执行真值标签来描述网关接口的真实实现方式。

## 执行真值标签

| 标签 | 含义 |
|---|---|
| `native` | 由 SDKWork 直接实现 |
| `relay` | 透明转发到兼容上游 |
| `translated` | 在本地接受请求后，映射到不同的上游原语 |
| `emulated` | 本地返回兼容形状的响应 |
| `unsupported` | 当前运行时下不可用 |

这些标签描述的是运行时真相，不是公开 URL 的分类方式。

## 公开契约规则

- 保留官方 provider 路径，只切换网关 `base_url`
- 如果某个能力已经存在 OpenAI 标准路由，则优先复用该标准作为共享公开契约
- 如果不存在共享标准，则暴露 provider 官方协议路径作为镜像协议面
- 不发明 `/code/*`、`/claude/*`、`/gemini/*` 这类 wrapper 前缀

## 镜像协议家族

- `code.openai`：OpenAI 与 Codex 的 `/v1/*`
- `code.claude`：Claude 的 `/v1/messages` 与 `/v1/messages/count_tokens`
- `code.gemini`：Gemini 的 `/v1beta/models/{model}:*`，包含 Nano Banana 这类可出图 Gemini 模型
- `images.openai`：OpenAI 图片协议 `/v1/images/*`
- `images.kling`：可灵图片协议，共享 DashScope `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}`
- `images.aliyun`：阿里云图片协议，共享 DashScope `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}`
- `images.volcengine`：火山引擎图片协议 `/api/v3/images/generations`
- `audio.openai`：共享音频协议 `/v1/audio/*`
- `video.openai`：共享视频协议 `/v1/videos*`，包含 Sora 2 与 Sora 2 Pro
- `video.kling`：可灵视频协议，共享 DashScope `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}`
- `video.aliyun`：阿里云视频协议，共享 DashScope `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}`
- `video.google-veo`：Google Veo 协议，对应 Vertex AI `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` 与 `:fetchPredictOperation`，并通过 `{model}` 覆盖 Veo 3 类模型
- `video.minimax`：MiniMax 视频协议 `/v1/video_generation`、`/v1/query/video_generation` 与 `/v1/files/retrieve`
- `video.vidu`：Vidu 视频协议 `/ent/v2/text2video`、`/ent/v2/img2video`、`/ent/v2/reference2video`、`/ent/v2/tasks/{id}/creations` 与 `/ent/v2/tasks/{id}/cancel`
- `video.volcengine`：火山引擎视频协议 `/api/v1/contents/generations/tasks` 与 `/api/v1/contents/generations/tasks/{id}`
- `music.openai`：共享音乐协议 `/v1/music*`
- `music.google`：Google 音乐协议，对应 Vertex AI `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict`
- `music.minimax`：MiniMax 音乐协议 `/v1/music_generation` 与 `/v1/lyrics_generation`
- `music.suno`：Suno 音乐协议 `/api/v1/generate*` 与 `/api/v1/lyrics*`

## 高价值 API 家族

当前已覆盖的主要网关家族包括：

- `/v1/models`
- `/v1/chat/completions`
- `/v1/messages`
- `/v1/completions`
- `/v1/responses`
- `/v1beta/models/{model}:generateContent`
- `/v1beta/models/{model}:streamGenerateContent`
- `/v1beta/models/{model}:countTokens`
- `/v1/embeddings`
- `/v1/files`
- `/v1/uploads`
- `/v1/audio/*`
- `/v1/images/*`
- `/api/v1/services/aigc/image-generation/generation`
- `/api/v3/images/generations`
- `/api/v1/services/aigc/video-generation/video-synthesis`
- `/api/v1/contents/generations/tasks`
- `/api/v1/contents/generations/tasks/{id}`
- `/api/v1/tasks/{task_id}`
- `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning`
- `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:fetchPredictOperation`
- `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict`
- `/v1/music`
- `/v1/music_generation`
- `/v1/lyrics_generation`
- `/v1/video_generation`
- `/v1/query/video_generation`
- `/v1/files/retrieve`
- `/ent/v2/text2video`
- `/ent/v2/img2video`
- `/ent/v2/reference2video`
- `/ent/v2/tasks/{id}/creations`
- `/ent/v2/tasks/{id}/cancel`
- `/api/v1/generate`
- `/api/v1/generate/record-info`
- `/api/v1/lyrics`
- `/api/v1/lyrics/record-info`
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

`audio` 能力当前以共享 `audio.openai` 镜像家族的形式发布在 `/v1/audio/*` 上，公开契约不会引入 `/audio/openai/*` 这类 wrapper 前缀。

`music` 能力当前以共享 `music.openai` 镜像家族的形式发布在 `/v1/music*` 上，继续采用资源化路由，而不是绑定单一上游厂商的私有传输路径，这样可以与图片、视频一样复用统一的路由、计费和插件适配架构。

图片当前同时激活了共享 `images.openai`，以及 provider-specific 的 `images.kling`、`images.aliyun` 与 `images.volcengine`；其中 `images.kling` 与 `images.aliyun` 共同复用 DashScope 官方 `/api/v1/services/aigc/image-generation/generation` 和 `/api/v1/tasks/{task_id}` 路径，`images.volcengine` 直接复用火山引擎官方 `/api/v3/images/generations` 路径。Nano Banana 继续使用 Google 官方 Gemini `/v1beta/models/{model}:generateContent` 协议面，因此归属 `code.gemini`，不再单独发布 `images.nanobanana` 家族。`images.midjourney` 仍未发布，因为 Midjourney 当前没有可通过只切换 `base_url` 镜像的官方 API 面。视频当前同时激活了共享 `video.openai`、provider-specific 的 `video.kling`、`video.aliyun`、`video.google-veo`、`video.minimax`、`video.vidu` 与 `video.volcengine`；其中 `video.openai` 继续覆盖 Sora 2 与 Sora 2 Pro，`video.kling` 与 `video.aliyun` 共同复用 DashScope 官方 `/api/v1/services/aigc/video-generation/video-synthesis` 和 `/api/v1/tasks/{task_id}` 路径，`video.google-veo` 直接复用 Google Vertex AI 官方 `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` 与 `:fetchPredictOperation` 路径，并通过 `{model}` 覆盖 Veo 3 类模型，`video.minimax` 直接复用 MiniMax 官方 `/v1/video_generation`、`/v1/query/video_generation` 和 `/v1/files/retrieve` 路径，`video.vidu` 直接复用 Vidu 官方 `/ent/v2/text2video`、`/ent/v2/img2video`、`/ent/v2/reference2video`、`/ent/v2/tasks/{id}/creations` 和 `/ent/v2/tasks/{id}/cancel` 路径，`video.volcengine` 直接复用火山引擎官方 `/api/v1/contents/generations/tasks` 与 `/api/v1/contents/generations/tasks/{id}` 路径。音乐当前同时激活了共享 `music.openai`、provider-specific 的 `music.google`、provider-specific 的 `music.minimax` 与 provider-specific 的 `music.suno`；其中 `music.google` 直接复用 Google Vertex AI 官方 `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` 路径，`music.minimax` 直接复用 MiniMax 官方 `/v1/music_generation` 和 `/v1/lyrics_generation` 路径，`music.suno` 直接复用 Suno 官方 `/api/v1/generate*` 和 `/api/v1/lyrics*` 路径。由于 OpenAI 已定义官方 Sora 协议面，网关不再单独发布 `video.sora`。

## Agent 客户端兼容面

网关现在提供两组一等镜像协议面，方便现有 agent 客户端直接接入：

- Claude 镜像协议面：`POST /v1/messages` 与 `POST /v1/messages/count_tokens`，适用于 Claude Code 等客户端
- Gemini 镜像协议面：`POST /v1beta/models/{model}:generateContent`、`POST /v1beta/models/{model}:streamGenerateContent?alt=sse`、`POST /v1beta/models/{model}:countTokens`，适用于 Gemini CLI gateway mode 以及 Nano Banana 这类可出图 Gemini 客户端

这些接口不会绕开 SDKWork 的路由系统，而是转换到现有 OpenAI 兼容执行链路，因此 provider 选择、项目路由偏好、配额控制、计费和 usage 记录都会与 `/v1/*` 网关保持一致。

有状态网关部署除了 `Authorization: Bearer ...` 之外，还支持官方协议原生认证入口：

- Claude 面：`x-api-key`
- Gemini 面：`x-goog-api-key` 或 `?key=...`

## 如何使用这份信息

- 如果你需要快速判断某一类接口是否能在当前运行时执行，先看完整矩阵
- 如果你需要了解公开镜像协议家族、基地址与鉴权方式，再看 [网关 API](/zh/api-reference/gateway-api)
- 如果你需要理解真实执行语义，再看这份兼容文档

## 进一步阅读

- [API 参考总览](/zh/api-reference/overview)
- [网关 API](/zh/api-reference/gateway-api)
- [完整兼容矩阵](/api/compatibility-matrix)
## 2026-04-17 补充

- `images.volcengine` 已转为 active 的 provider-specific 镜像家族，对应公开路径 `/api/v3/images/generations`
- `Nano Banana` 继续归属 `code.gemini` 官方 `/v1beta/models/{model}:generateContent` 协议面，不再单独治理为 `images.nanobanana`
- 仍未发布为 images 家族的仅剩 `images.midjourney`
- Sora 2 与 Sora 2 Pro 归属于共享 `video.openai` `/v1/videos*` 契约，不再单独治理为 `video.sora`
