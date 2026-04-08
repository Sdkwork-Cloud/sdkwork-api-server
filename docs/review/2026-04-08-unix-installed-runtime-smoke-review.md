# 2026-04-08 Unix 安装态运行时 Smoke Review

## 1. 范围

- `bin/tests/router-runtime-tooling.test.mjs`
- `bin/start.sh`
- `bin/stop.sh`
- `docs/架构/137-安装部署发布回滚设计-2026-04-07.md`

目标：把“脚本/描述符已生成”推进到“安装态运行时具备可执行 smoke 资产”，并把宿主限制与产品缺陷分离。

## 2. Findings

### P0 已补：Unix 安装态 `start.sh` / `stop.sh` 缺少回归 smoke

- 现象：
  之前测试覆盖了 `systemd / launchd / windows-task` 描述符与 helper 脚本生成，也覆盖了 `start.sh --dry-run` 路径回退，但没有覆盖“已安装 home”场景下 `bin/start.sh` 与 `bin/stop.sh` 的完整生命周期。
- 风险：
  这会把验证停留在“文件存在”和“dry-run 可输出”层，无法证明安装包落地后能稳定启动、写入 pid/state、再被 stop 脚本回收。
- 对标：
  行业成熟产品会把“安装态启动/停止/回滚 smoke”纳入发布门禁，而不是仅验证描述符生成。

### P0 已补：新增两层验证，避免把宿主限制误报为产品失败

- 新增静态契约测试：
  `unix runtime entrypoints default to the installed home beside the packaged scripts when binaries are colocated`
  作用：验证 `start.sh` / `stop.sh` 在安装态优先解析脚本同级 home，而不是退回仓库默认目录。
- 新增能力感知 smoke：
  `installed unix runtime start.sh and stop.sh manage an installed home end-to-end`
  作用：在宿主具备 Unix shell 子进程能力时，执行安装态 start/stop smoke；宿主不具备能力时自动 skip，而不是把环境限制误判为产品失败。

### P1 当前环境边界已确认：本机不能把 Node 子进程稳定拉起到 Unix shell

- 证据：
  `spawnSync('C:/Program Files/Git/bin/bash.exe', ['-lc', 'exit 0'])`
  在当前宿主返回 `EPERM`。
- 直接执行证据：
  `bash.exe` 报 `couldn't create signal pipe, Win32 error 5`。
- 结论：
  当前 Windows 宿主不适合把 Unix shell smoke 当作本机强制门禁；该 smoke 应在 Linux/macOS 车道，或具备放行策略的 Windows 车道执行。

## 3. 本轮改动

- 修改 `bin/tests/router-runtime-tooling.test.mjs`
  - 增加 Git Bash / Unix shell 能力探测
  - 增加安装态 home 静态契约测试
  - 增加安装态 start/stop smoke 测试，并在宿主不具备能力时自动 skip

## 4. 验证

- `cargo check -p sdkwork-api-interface-http`
  - pass
- `node --test --experimental-test-isolation=none bin/tests/router-runtime-tooling.test.mjs`
  - `46 tests, 0 fail, 8 skip`

## 5. 下一步

1. 在 Linux 与 macOS 发布车道执行 `bin/tests/router-runtime-tooling.test.mjs`，把当前 skip 的 Unix 安装态 smoke 转成真实执行证据。
2. 在发布门禁中增加“安装包级 start/stop smoke 必跑”要求，避免只靠 helper/descriptor 测试放行。
3. 后续再把“安装后健康检查、回滚、依赖泄漏阻断”并入同一条发布验收链。
