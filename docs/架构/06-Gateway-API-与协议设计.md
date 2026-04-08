# Gateway API 与协议设计

## 1. 文档目标

定义对外 API 面、协议兼容模型、统一执行上下文、版本治理与协议红线。

## 2. 对外接口面

平台对外暴露三类接口面：

- Gateway：`/v1/*` 与兼容协议面
- Admin：`/admin/*`
- Portal：`/portal/*`

其中：

- Gateway 面关注请求执行
- Admin 面关注治理与运维
- Portal 面关注客户自助与工作区运营

## 3. 协议兼容模型

当前主协议族为：

- `OpenAI Compatible`
- `Anthropic Messages Compatible`
- `Gemini Compatible`

兼容原则：

- 兼容层只做协议翻译，不重造第二套路由内核
- 所有兼容协议都必须回到同一条身份、路由、配额、计费、审计链路
- 执行真值以兼容矩阵中的 `native / relay / translated / emulated / unsupported` 标签为准

## 4. 能力家族

当前主能力家族包括：

- `models`
- `chat/completions`
- `messages`
- `completions`
- `responses`
- `embeddings`
- `containers`
- `files`
- `uploads`
- `audio/*`
- `images/*`
- `assistants`
- `threads`
- `conversations`
- `vector_stores`
- `batches`
- `fine_tuning/jobs`
- `webhooks`
- `evals`
- `videos`
- `realtime/sessions`

补充能力口径：

- `music` 采用统一 `/v1/music*` 资源家族思路，路由、计费、扩展治理与图像、视频保持同一能力模型

## 5. 统一执行上下文

任意网关请求进入系统后，最少要形成以下上下文：

- `request_id`
- 调用主体：`admin user / portal user / gateway api key / service subject`
- `tenant / project / api key group`
- 目标模型与能力意图
- `routing policy / routing profile`
- `quota / pricing / billing / settlement` 上下文
- 审计与诊断上下文

## 6. 认证与鉴权输入

当前兼容输入必须受控支持：

- `Authorization: Bearer ...`
- `x-api-key`
- `x-goog-api-key`
- `?key=`

要求：

- 兼容原生客户端输入，不绕过统一身份解析
- 协议差异不能绕过租户、项目、计费与审计边界

## 7. 版本与兼容治理

- 路由与协议演进必须有显式版本边界与弃用路径
- 新协议必须先补充兼容矩阵，再落实现有主链路
- 新能力家族必须先定义“能力语义”，再定义“Provider 适配”
- 不允许直接把上游私有传输协议暴露为平台长期公共契约

## 8. 设计红线

- 禁止每种协议各自维护一套计费逻辑
- 禁止 Admin、Portal 接口直接混入 Gateway 执行语义
- 禁止以“兼容”为名旁路统一身份、路由、配额与审计

## 9. 评估标准

- 是否能用同一网关解释 OpenAI、Anthropic、Gemini 三类入口
- 是否能让新增能力家族复用统一上下文模型
- 是否能用兼容矩阵准确回答每个 API 家族在不同运行模式下的执行真值
