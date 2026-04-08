# SDKWork API Router Release 与 Change Log 管理规范

## 1. 目录定位

本目录用于管理 `sdkwork-api-router` 的版本、发布说明、迭代变更日志、阶段放行记录与商业化交付相关 release 文档。

它服务的对象包括：

- 研发与重构执行者
- 验证与放行负责人
- 运维与私有化交付团队
- 商业化与产品交付相关人员

## 2. 必备文档

本目录至少维护以下文件：

- `README.md`
  - 发布与 changelog 的管理规则

- `CHANGELOG.md`
  - 累计变更总表，按版本倒序记录

- `YYYY-MM-DD-vX.Y.Z-迭代变更日志.md`
  - 每一轮重要迭代的详细专业记录

## 3. 版本号规则

建议采用语义化版本：

- `Major`
  - 协议不兼容变化
  - 重大架构变更
  - 重大交付模型变化
  - 正式商业化大版本里程碑

- `Minor`
  - 完成一个重要 step 或完整波次
  - 完成一个可对外感知的重要能力闭环

- `Patch`
  - 同一能力上的缺陷修复、测试补强、性能优化、文档对齐、脚本修补

## 4. Change Log 写作规范

每次迭代的 release / changelog 文档必须至少说明：

- 日期
- 版本号
- 所处波次与 step
- 本轮主执行模式
- `Top 3` 候选动作与优先级判断
- 已兑现的架构能力
- 代码、测试、文档、脚本变更摘要
- 验证结果
- 已知风险与限制
- 本轮主假设、实际结果、被证伪项、下一轮决策入口
- 下一轮迭代焦点
- 对商业化交付的影响

要求：

- 专业、克制、可审计
- 区分已完成与未完成
- 不使用模糊表述替代真实交付信息
- 明确本轮执行模式、是否发生回归、是否提升商业化完备度
- 写清楚为什么当前轮选择这个动作，而不是其它动作
- 写清楚是否处于“未闭环执行中 / 当前阶段完美收敛 / 持续优化”中的哪一状态

### 4.1 未成功发布版本的合并规则

如果某一轮版本说明已经写入 `docs/release/`，但以下任一事实无法被证明：

- 真实 GitHub release 已成功发布
- 对应 tag / release 资产可被远端历史验证
- 发布所需依赖仓库版本与本地基线一致

则该轮版本不得被视为“已正式发布”，而应视为“待并入下一次成功 release 的候选记录”。

要求：

- `CHANGELOG.md` 顶部必须保留 `Unreleased` 聚合区，汇总从“上一次已验证成功发布”到当前的全部未正式发布变更
- 下一次成功发布时，必须把此前所有未成功发布的 changelog 和 release 文档合并进该次正式 release
- 如果后续证明历史上其实已经发布成功，可以在具备证据后再拆分回真实发布边界
- 没有远端证据时，禁止把本地 version 文档直接当成既成发布事实

## 5. 与 step / review / 架构文档的关系

- 版本升级必须与 `docs/step/` 当前 step 推进一致
- 每次重要迭代必须和 `docs/review/` 审计结果对齐
- 若当前事实变化，应同步回写 `docs/架构/`

## 5.1 与持续执行 Prompt 的关系

当使用 `docs/prompts/反复执行Step指令.md` 持续推进项目时：

- 每一轮迭代完成后都必须更新本目录
- 不能只在“正式发布”时才补 changelog
- 本目录既是发布记录，也是持续收敛记录

## 6. 发布放行要求

在正式生产或商业化交付前，release 文档至少要能支撑以下判断：

- 当前事实分与目标分
- `stateful standalone` 与 `stateless runtime` 的独立结论
- 已知限制
- 是否允许预发 / 正式放行 / 继续优化
- 协议兼容与客户端行为差异证据是否充分
- 性能 / 容量 / SLO 证据是否充分
- 安全 / 审计 / 凭据治理证据是否充分
- 部署 / 回滚 / 备份恢复 / 值守手册是否充分
- quota / usage / billing / pricing / settlement 证据是否充分
- 当前商业化放行证据包还缺什么

### 6.1 远程 Git 与依赖仓库同步闸门

在执行 `commit -> push -> GitHub release` 之前，必须额外验证：

- `sdkwork-api-router` 的远端 `origin` 可访问，且目标分支 / tag 的真实状态可被验证
- `sdkwork-core`、`sdkwork-ui`、`sdkwork-appbase`、`sdkwork-im-sdk` 的本地仓库边界清晰、工作区干净，且与各自 GitHub 目标版本一致
- 如果任一依赖仓库处于 dirty、ahead / behind 未确认、或远端不可验证状态，则不得提交和 push 当前 release
- 必须明确区分“本地开发依赖模型”和“release 环境依赖模型”，不能为了发布而破坏本地相对路径开发体验

### 6.2 Release 环境的 SDK 依赖策略

本仓库允许本地开发继续使用相对路径 / workspace SDK 依赖，以减少频繁发布到 npm 中央仓库的成本。

但在 release 环境中，必须通过 GitHub 仓库地址或等价的 Git 仓库物化方式管理这些依赖，并满足：

- 不修改本地开发用依赖声明的主设计目标
- release 构建前先物化对应 GitHub 仓库，或用明确 Git 引用完成等价安装
- 发布时要能说明所使用的依赖仓库 URL、ref、与本地基线差异
- 如果 GitHub 版本与本地版本不一致且无法完成对齐，则 release 必须保持阻塞状态
- 当前 release-app 覆盖性审计必须证明：admin / portal 的 `package.json`、`pnpm-workspace.yaml`、`tsconfig.json` 中所有仓库外 sibling 引用都已被 `scripts/release/materialize-external-deps.mjs` 覆盖
- 按当前仓库事实，release 构建图中被审计到的仓库外 sibling 依赖仅为 `sdkwork-ui`；如果未来引入新的 sibling 仓库引用而未同步物化脚本，release 合同必须直接转红

### 6.3 Release Governance 执行入口

发布治理必须优先走仓库脚本，而不是临时拼接命令：

- `node scripts/release/materialize-external-deps.mjs`
  - 先执行 release 外部依赖覆盖审计，再进行 GitHub 仓库物化
- `node scripts/release/compute-release-window-snapshot.mjs --format json`
  - 计算最近 release tag、距该 tag 的提交增量、当前工作区条目数，用于保持 release 决策账本中的快照事实不过期
  - 当当前主机阻塞 Node -> Git 时，可通过以下 governed input 直接提供已审计的 release-window 事实：
    - `--snapshot <path>`
    - `--snapshot-json <json>`
    - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH`
    - `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
  - governed input 可接受两种形态：
    - 带 `generatedAt`、`source`、`snapshot` 的 artifact envelope
    - 仅包含 `latestReleaseTag`、`commitsSinceLatestRelease`、`workingTreeEntryCount`、`hasReleaseBaseline` 的 raw snapshot
- `node scripts/release/run-release-governance-checks.mjs --format json`
  - 统一执行 release contract checks 与 live sync audit
- `node scripts/release/verify-release-sync.mjs --format text`
  - 面向人工排查的文本输出
- `node scripts/release/verify-release-sync.mjs --format json`
  - 面向审计记录与脚本消费的结构化输出
  - 当当前主机阻塞 Node -> Git 时，可通过以下 governed input 直接提供已审计的 release-sync 事实：
    - `--audit <path>`
    - `--audit-json <json>`
    - `SDKWORK_RELEASE_SYNC_AUDIT_PATH`
    - `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
  - governed input 可接受两种形态：
    - 带 `generatedAt`、`source`、`summary` 的 artifact envelope
    - 仅包含 `releasable`、`reports` 的 raw summary

在当前沙箱中，涉及这些 release 脚本的 `node --test` 默认隔离模式会触发 `spawn EPERM`。此时必须使用：

- `node --test --experimental-test-isolation=none scripts/release/tests/release-sync-audit.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-workflow.test.mjs`
- `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`

如果 live sync audit 返回 `command-exec-blocked`、`missing-path`、`dirty-working-tree`、`branch-not-synced`、`remote-unverifiable`、`head-mismatch` 或 `remote-url-mismatch`，都不得执行 `commit -> push -> GitHub release`。

当前沙箱下，所有依赖 Node 子进程执行 Git 且未提供 governed input 的脚本都可能命中 `spawn EPERM`，其中包括：

- `verify-release-sync.mjs`
- `run-release-governance-checks.mjs` 的 live sync audit 车道
- `compute-release-window-snapshot.mjs`

因此在当前沙箱中：

- 若没有提供 governed release-window input，`release-window-snapshot` 仍应保持 `command-exec-blocked`
- 若已提供 governed release-window input，则应以该受治理输入作为 release-window 事实来源，再回写 `/docs/release`
- 若没有提供 governed release-sync input，`release-sync-audit` 仍应保持阻塞
- 若已提供 governed release-sync input，则应以该受治理输入作为 release-sync 事实来源，再回写 `/docs/release`

## 7. 推荐版本推进节奏

建议：

- 大范围文档与执行系统强化：`Patch`
- 一个重要 step 完整闭环：`Patch` 或 `Minor`
- 一个完整波次通过：`Minor`
- 正式商业化发布或重大架构里程碑：`Major`

## 8. 反回归要求

更新 changelog 时，必须额外说明：

- 本轮是否引入回归风险
- 如果发现回归，是否已在本轮修复
- 是否影响既有版本结论或放行结论
- 本轮是否切换了执行模式或策略，以及触发证据是什么
- 本轮决策账本是否完整，是否足以支撑下一轮无歧义续跑
