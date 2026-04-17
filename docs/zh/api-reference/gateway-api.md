# 网关 API

网关服务以镜像协议方式暴露公开 API。官方客户端路径保持不变，因此现有 SDK 和 CLI 理论上只需要切换 `base_url`。

## 基础地址与鉴权

- 默认本地基地址：`http://127.0.0.1:8080`
- 主要鉴权方式：`Authorization: Bearer skw_live_...`
- 健康检查：`GET /health`
- Metrics：`GET /metrics`
- OpenAPI JSON：`GET /openapi.json`
- API 清单页面：`GET /docs`

最小首个请求：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

在独立服务模式下，网关是一个依赖 admin store 的有状态实现。无状态 gateway runtime 仍以库和运行时形态存在，其覆盖范围通过兼容矩阵文档继续说明。

OpenAPI 直接从当前 `axum` 路由实现生成，因此 JSON 文档与浏览器中的 API 清单会始终跟随真实公开路由面。

## 镜像协议家族

- `code.openai`：OpenAI 与 Codex 的官方 `/v1/*` 镜像协议面
- `code.claude`：Claude 的官方 `/v1/messages*` 镜像协议面
- `code.gemini`：Gemini 的官方 `/v1beta/models/{model}:*` 镜像协议面，包含 Nano Banana 这类可出图 Gemini 模型
- `images.openai`：OpenAI 图片的官方 `/v1/images/*` 镜像协议面
- `images.kling`：可灵图片在共享 DashScope 官方 `/api/v1/services/aigc/image-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `images.aliyun`：阿里云图片在共享 DashScope 官方 `/api/v1/services/aigc/image-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `images.volcengine`：火山引擎图片在官方 `/api/v3/images/generations` 上的镜像协议面
- `audio.openai`：共享音频的官方 `/v1/audio/*` 镜像协议面
- `video.openai`：共享视频的官方 `/v1/videos*` 镜像协议面，包含 Sora 2 与 Sora 2 Pro
- `video.kling`：可灵视频在共享 DashScope 官方 `/api/v1/services/aigc/video-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `video.aliyun`：阿里云视频在共享 DashScope 官方 `/api/v1/services/aigc/video-generation/*` 与 `/api/v1/tasks/{task_id}` 上的镜像协议面
- `video.google-veo`：Google Veo 在 Vertex AI 官方 `/v1/projects/*/locations/*/publishers/google/models/*` 上的镜像协议面，包含通过官方模型路径选择的 Veo 3 类模型
- `video.minimax`：MiniMax 的官方 `/v1/video_generation`、`/v1/query/video_generation` 与 `/v1/files/retrieve` 视频镜像协议面
- `video.vidu`：Vidu 的官方 `/ent/v2/*` 视频镜像协议面
- `video.volcengine`：火山引擎官方 `/api/v1/contents/generations/tasks` 与 `/api/v1/contents/generations/tasks/{id}` 视频镜像协议面
- `music.openai`：共享音乐的官方 `/v1/music*` 镜像协议面
- `music.google`：Google 音乐在 Vertex AI 官方 `/v1/projects/*/locations/*/publishers/google/models/{model}:predict` 上的镜像协议面
- `music.minimax`：MiniMax 的官方 `/v1/music_generation` 与 `/v1/lyrics_generation` 音乐镜像协议面
- `music.suno`：Suno 的官方 `/api/v1/*` 音乐镜像协议面
- 公开契约不会额外发明 `/code/*`、`/claude/*`、`/gemini/*` 这类 wrapper 前缀

## 路由家族

下表中的 OpenAI 家族行默认使用官方 `/v1` 前缀。Claude 与 Gemini 保持各自官方路径，不会被改写到网关自定义命名空间中。

| 家族 | 路由 | 说明 |
|---|---|---|
| models | `GET /models`、`GET /models/{model_id}`、`DELETE /models/{model_id}` | 有状态模式下基于 catalog |
| chat completions | `GET /chat/completions`、`POST /chat/completions`、`GET/POST/DELETE /chat/completions/{completion_id}`、`GET /chat/completions/{completion_id}/messages` | 支持兼容 JSON 与流式转发 |
| completions | `POST /completions` | 传统文本补全接口 |
| responses | `POST /responses`、`POST /responses/input_tokens`、`POST /responses/compact`、`GET/DELETE /responses/{response_id}`、`GET /responses/{response_id}/input_items`、`POST /responses/{response_id}/cancel` | OpenAI 风格 responses 工作流 |
| embeddings | `POST /embeddings` | 基于请求模型做 provider 选择 |
| moderations | `POST /moderations` | OpenAI 兼容审核接口 |
| images | `POST /images/generations`、`POST /images/edits`、`POST /images/variations` | 当前公开镜像家族为 `images.openai`；provider 路由可隐藏在共享 OpenAI 图片契约之后 |
| images.kling | `POST /api/v1/services/aigc/image-generation/generation`、`GET /api/v1/tasks/{task_id}` | 当前已发布的 provider-specific 镜像 tag，面向可灵兼容客户端复用共享 DashScope 官方异步图片路径 |
| images.aliyun | `POST /api/v1/services/aigc/image-generation/generation`、`GET /api/v1/tasks/{task_id}` | 当前已发布的 provider-specific 镜像 tag，面向阿里云兼容客户端复用共享 DashScope 官方异步图片路径 |
| images.volcengine | `POST /api/v3/images/generations` | 当前已发布的 provider-specific 镜像家族，保持火山引擎官方图片生成协议路径不变 |
| audio | `POST /audio/transcriptions`、`POST /audio/translations`、`POST /audio/speech`、`GET /audio/voices`、`POST /audio/voice_consents` | 当前公开镜像家族为 `audio.openai`；provider 路由可隐藏在共享 `/v1/audio/*` 契约之后 |
| files | `GET/POST /files`、`GET/DELETE /files/{file_id}`、`GET /files/{file_id}/content` | 元数据与二进制内容获取 |
| uploads | `POST /uploads`、`POST /uploads/{upload_id}/parts`、`POST /uploads/{upload_id}/complete`、`POST /uploads/{upload_id}/cancel` | multipart 上传生命周期 |
| containers | `GET/POST /containers`、`GET/DELETE /containers/{container_id}`、`GET/POST /containers/{container_id}/files`、`GET/DELETE /containers/{container_id}/files/{file_id}`、`GET /containers/{container_id}/files/{file_id}/content` | 容器与嵌套文件流程 |
| assistants | `GET/POST /assistants`、`GET/POST/DELETE /assistants/{assistant_id}` | 兼容 assistants 管理 |
| threads | `POST /threads`、`GET/POST/DELETE /threads/{thread_id}`、嵌套 messages 和 runs 路由 | 包含 tool output 提交与 run steps |
| conversations | `GET/POST /conversations`、`GET/POST/DELETE /conversations/{conversation_id}`、嵌套 item 路由 | 面向 response 风格负载的 conversation 流程 |
| vector stores | `GET/POST /vector_stores`、`GET/POST/DELETE /vector_stores/{vector_store_id}`、嵌套 search、files、file batches | 检索与导入流程 |
| batches | `GET/POST /batches`、`GET /batches/{batch_id}`、`POST /batches/{batch_id}/cancel` | 异步批处理工作流 |
| fine tuning | `GET/POST /fine_tuning/jobs`，以及 retrieve、cancel、events、checkpoints、pause、resume、checkpoint permissions | 覆盖较完整的 fine-tuning 家族 |
| webhooks | `GET/POST /webhooks`、`GET/POST/DELETE /webhooks/{webhook_id}` | 兼容 webhook CRUD |
| realtime | `POST /realtime/sessions` | realtime session 创建 |
| evals | `GET/POST /evals`、`GET/POST/DELETE /evals/{eval_id}`、嵌套 runs 和 output item 路由 | 评估工作流 |
| videos | `GET/POST /videos`，以及 retrieve、delete、content、remix、edits、extensions、extend、character 路由 | 当前公开镜像家族为 `video.openai`；Sora 2 与 Sora 2 Pro 继续停留在共享官方 `/v1/videos*` 契约中，不拆成 provider wrapper 家族 |
| video.kling | `POST /api/v1/services/aigc/video-generation/video-synthesis`、`GET /api/v1/tasks/{task_id}` | 当前已发布的 provider-specific 镜像家族，面向可灵兼容客户端复用共享 DashScope 官方异步视频路径 |
| video.aliyun | `POST /api/v1/services/aigc/video-generation/video-synthesis`、`GET /api/v1/tasks/{task_id}` | 当前已发布的 provider-specific 镜像家族，面向阿里云兼容客户端复用共享 DashScope 官方异步视频路径 |
| video.google-veo | `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning`、`POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:fetchPredictOperation` | 当前已发布的 provider-specific 镜像家族，保持 Google Vertex AI Veo 官方长任务传输路径不变，并通过 `{model}` 覆盖 Veo 3 类模型 |
| video.minimax | `POST /v1/video_generation`、`GET /v1/query/video_generation`、`GET /v1/files/retrieve` | 当前已发布的 provider-specific 镜像家族，保持 MiniMax 官方异步视频协议路径不变 |
| video.vidu | `POST /ent/v2/text2video`、`POST /ent/v2/img2video`、`POST /ent/v2/reference2video`、`GET /ent/v2/tasks/{id}/creations`、`POST /ent/v2/tasks/{id}/cancel` | 当前已发布的 provider-specific 镜像家族，保持 Vidu 官方异步视频协议路径不变 |
| video.volcengine | `POST /api/v1/contents/generations/tasks`、`GET /api/v1/contents/generations/tasks/{id}` | 当前已发布的 provider-specific 镜像家族，保持火山引擎官方异步视频任务路径不变 |
| music | `GET/POST /music`、`GET/DELETE /music/{music_id}`、`GET /music/{music_id}/content`、`POST /music/lyrics` | 当前公开镜像家族为 `music.openai`；provider 路由可隐藏在共享 `/v1/music*` 契约之后 |
| music.google | `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` | 当前已发布的 provider-specific 镜像家族，保持 Google Vertex AI 官方音乐预测路径不变 |
| music.minimax | `POST /v1/music_generation`、`POST /v1/lyrics_generation` | 当前已发布的 provider-specific 镜像家族，保持 MiniMax 官方音乐生成协议路径不变 |
| music.suno | `POST /api/v1/generate`、`GET /api/v1/generate/record-info`、`POST /api/v1/lyrics`、`GET /api/v1/lyrics/record-info` | 当前已发布的 provider-specific 镜像家族，保持 Suno 官方协议路径不变 |

当前阶段将图片、音频、视频、音乐分别治理为 `images.openai`、`audio.openai`、`video.openai`、`music.openai` 这四个共享镜像家族，继续保留原有 `/v1/images*`、`/v1/audio/*`、`/v1/videos*`、`/v1/music*` 路径，不引入 `/images/openai/*`、`/audio/openai/*`、`/video/openai/*`、`/music/openai/*` 这类 wrapper 前缀。

当前图片能力同时发布了四个镜像 tag，并占用三组公开路径家族：共享 `images.openai` 继续使用 `/v1/images*`，provider-specific 的 `images.kling` 与 `images.aliyun` 共同复用 DashScope 官方 `/api/v1/services/aigc/image-generation/generation` 与 `/api/v1/tasks/{task_id}` 路径，provider-specific 的 `images.volcengine` 则直接暴露火山引擎官方 `/api/v3/images/generations` 路径。Nano Banana 继续使用 Google 官方 Gemini `/v1beta/models/{model}:generateContent` 协议面，因此归属 `code.gemini`，不再单独发布 `images.nanobanana` 家族。`images.midjourney` 仍未发布，因为 Midjourney 当前没有可通过只切换 `base_url` 镜像的官方 API 面。当前视频能力同时发布了七个镜像家族：共享 `video.openai` 继续使用 `/v1/videos*`，并覆盖 Sora 2 与 Sora 2 Pro，provider-specific 的 `video.kling` 与 `video.aliyun` 共同复用 DashScope 官方 `/api/v1/services/aigc/video-generation/video-synthesis` 与 `/api/v1/tasks/{task_id}` 路径，provider-specific 的 `video.google-veo` 直接暴露 Google Vertex AI 官方 `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predictLongRunning` 与 `:fetchPredictOperation` 路径，并通过 `{model}` 覆盖 Veo 3 类模型，provider-specific 的 `video.minimax` 直接暴露 MiniMax 官方 `/v1/video_generation`、`/v1/query/video_generation` 与 `/v1/files/retrieve` 路径，provider-specific 的 `video.vidu` 直接暴露 Vidu 官方 `/ent/v2/text2video`、`/ent/v2/img2video`、`/ent/v2/reference2video`、`/ent/v2/tasks/{id}/creations` 与 `/ent/v2/tasks/{id}/cancel` 路径，provider-specific 的 `video.volcengine` 则直接暴露火山引擎官方 `/api/v1/contents/generations/tasks` 与 `/api/v1/contents/generations/tasks/{id}` 路径。当前音乐能力同时发布了四个镜像家族：共享 `music.openai` 继续使用 `/v1/music*`，provider-specific 的 `music.google` 直接暴露 Google Vertex AI 官方 `/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict` 路径，provider-specific 的 `music.minimax` 直接暴露 MiniMax 官方 `/v1/music_generation` 与 `/v1/lyrics_generation` 路径，provider-specific 的 `music.suno` 则直接暴露 Suno 官方 `/api/v1/generate*` 与 `/api/v1/lyrics*` 路径。由于 OpenAI 已提供官方 Sora 协议面，网关不再单独发布 `video.sora` 家族。

## 网关语义

- 公开契约规则：保留官方客户端路径，只切换网关 `base_url`
- provider 选择由 models、route keys 和 routing policy 共同决定
- 在有状态模式下，usage 与 billing 绑定到已鉴权项目
- 创建类路由在记录 usage 时可同时保留 route-key 选择与创建后的资源 ID 关联
- chat、completions、responses、embeddings、moderations 这类生成接口即使上游返回资源 ID，计费仍保持以请求模型为主键

## 常用 Header

| Header | 用途 |
|---|---|
| `Authorization` | gateway API key |
| `Content-Type` | JSON、multipart 或兼容上游媒体类型 |
| `x-request-id` | 请求关联 |
| `x-sdkwork-region` | geo-affinity provider 选择的可选提示 |

## 相关文档

- 公开契约与执行真值：
  - [API 兼容矩阵](/zh/reference/api-compatibility)
  - [完整兼容矩阵](/api/compatibility-matrix)
- 控制平面：
  - [管理端 API](/zh/api-reference/admin-api)
## 2026-04-17 补充

- `images.volcengine` 已转为 active 的 provider-specific 镜像家族，对应公开路径 `POST /api/v3/images/generations`
- 图片当前 active mirror tags 为 `images.openai`、`images.kling`、`images.aliyun` 与 `images.volcengine`
- `Nano Banana` 继续归属 `code.gemini` 官方 `/v1beta/models/{model}:generateContent` 协议面，不再单独治理为 `images.nanobanana`
- 仍未发布为 images 家族的仅剩 `images.midjourney`
- Sora 2 与 Sora 2 Pro 归属于共享 `video.openai` `/v1/videos*` 契约，不再单独治理为 `video.sora`
