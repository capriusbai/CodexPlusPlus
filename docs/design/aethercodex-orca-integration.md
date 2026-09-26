# AetherCodex × Orca 联动设计计划

状态：**草案，待决策**　·　日期：2026-09-26　·　范围：内部使用

调研对象：[`stablyai/orca`](https://github.com/stablyai/orca) @ `f0a3610`（v1.4.197，MIT，
23,053 个 TS/TSX 文件）。本文所有关于 Orca 的结论均来自该 commit 的源码，不是文档转述。

> 内部使用，不做商业化，按指示不展开 License 讨论。仅记录一个事实以备将来：Orca 是 MIT，
> 若某天需要对外分发，保留其 LICENSE 是单文件成本。

---

## 1. 两个系统到底是什么

| | AetherCodex | Orca |
|---|---|---|
| 形态 | Rust + Tauri 桌面工具 + 静默启动器 | Electron 桌面应用 + CLI + 无头服务 + 手机端 |
| 对象 | **Codex 桌面 App**（GUI） | **Agent CLI 集群**（Codex / Claude Code / OpenCode / Pi / Cursor / Gemini …） |
| 手段 | 外部启动 + CDP 注入渲染进程 | `codex-app-server` JSON-RPC + PTY 编排 |
| 单位 | 一个 Codex 会话 | 一次 run，横跨 N 个 git worktree |
| 强项 | 供应商/中转注入、页面增强、会话治理 | 并行 worktree、跨 agent 编排、远程/SSH/手机 |

**结论：两者不重叠，是同一条栈的上下两层。** AetherCodex 管"一个 Codex GUI 会话用得爽不爽"，
Orca 管"十个 agent 并行跑得清不清楚"。这决定了联动方式是**分层协作**，不是功能合并。

## 2. 两个真实碰撞点

调研中发现两处二者写/读同一份状态，这是设计必须先解决的：

### 2.1 `~/.codex/config.toml`

- AetherCodex：中转注入直接改写它（三种模式 × 两种协议，写 `model_providers.custom`）。
- Orca：`src/main/codex-accounts/codex-config-mirror.ts` 把**规范配置镜像进每个账号的
  managed `CODEX_HOME`**，并有 `codex-trust-config-mutation-queue.ts` 串行化写入。

两边同时写 = 互相覆盖。但换个角度看，Orca 的 managed home 模型恰好给出了干净的分层：

> **AetherCodex 是 config.toml 的唯一作者，Orca 是消费者。**
> AetherCodex 写规范配置 → Orca 镜像进各 managed home → 所有 Orca 托管的 Codex agent
> 自动继承中转设置。配一次，十个 worktree 全都走你的中转。

这不是妥协，这是本次联动**最大的即得价值**。

### 2.2 会话与 rollout 文件

- AetherCodex：读 `~/.codex/state_5.sqlite`，删除会话时连带删 rollout 文件。
- Orca：`codex-usage/codex-rollout-file-parse.ts`、`codex-session-file-discovery.ts`
  解析同一批文件做用量归因。

AetherCodex 删一个会话，Orca 的用量统计就少一条且无从知晓。需要删除前的让路协议（见 4.3）。

## 3. 接口选型

Orca 暴露三个面，按耦合度从低到高：

| 接口 | 形态 | 耦合 | 适用 |
|---|---|---|---|
| **A. `orca` CLI** | 子进程 + `--json` | 低。命令名稳定，JSON 输出，跨版本容错 | 编排动作、查询 |
| **B. `orca serve` 运行时 RPC** | WebSocket + 版本化 envelope | 高。受 `RUNTIME_PROTOCOL_VERSION` 约束 | 实时状态流 |
| **C. 共享状态文件** | 文件系统 | 零协议耦合，但有竞态 | 配置、会话 |

**选 A 为主，C 为辅，B 暂不做。**

理由：Orca 的 `docs/reference/remote-wire-compatibility.md` 明确说"混合版本是常态"，
新 opcode 发给老 peer 会被**静默丢弃**，表现为功能挂起而非报错。我们是 Rust 侧的第三方
客户端，跟着他们的协议版本走性价比极低。CLI 的 `--json` 输出足够覆盖 90% 场景，且
Orca 自己的 `cli-command-name-parity.test.ts` 在保命令名稳定。

可用的编排命令（`src/cli/specs/orchestration.ts`）：
`run-create` / `run-list` / `run-show` / `send` / `dispatch` / `ask` / `inbox` /
`check` / `reply` / `task-*` / `gate-*` / `coordinator-*`。

## 4. 四项联动能力

按"先拿价值、后担风险"排序。

### 4.1 配置下行：一次中转配置，全体 agent 继承 ⭐ 优先做

AetherCodex 已经是 config.toml 的作者，Orca 已经会镜像。**这一项几乎不用写代码，
只需要把契约固定下来并验证。**

- 在 AetherCodex「供应商配置」加一行状态：检测到 Orca 存在时显示
  "已生效于 N 个 Orca managed home"。
- 实现：`orca account list --json` 读账号列表，比对各 managed home 的 config.toml
  是否已含我们写入的 provider。
- 写入时机：AetherCodex 应用配置后，调 Orca 触发一次镜像（或提示用户重启 Orca）。

**风险低，价值最高。** 这是唯一一条"不联动也在发生、联动后才可见可控"的链路。

### 4.2 状态上行：Codex GUI 里看见 agent 集群

Codex App 页面里注入一块 Orca 面板，显示当前 run 的 agent 状态。

- 数据源：`orca orchestration run-current --json` + `run-show --json`。
- 通道：复用现有 helper bridge（`/orca/*` 路由），注入脚本轮询后端，后端调 CLI。
- 注意：Orca 的 agent status 由**执行宿主的 hook server 单一持有**
  （`docs/reference/agent-status-store.md`），读方只做展示策略，不要自己缓存或做优先级判断。
  我们是第 N 个读方，照这个规矩来。

轮询而非订阅，是 4.1 里选 CLI 的直接代价，可接受（状态秒级更新足够）。

### 4.3 会话让路：删除前问一句

解决 2.2 的竞态。

- AetherCodex 删除会话前，调 `orca orchestration run-list --json` 检查该 rollout
  是否属于活跃 run；是则提示"该会话正被 Orca run X 使用，仍要删除吗"。
- 纯读操作，不改 Orca 任何状态，风险可控。

### 4.4 从 Codex GUI 发起 fan-out ⭐ 最有意思

在 Codex App 的输入框旁注入一个按钮：**「分发到 N 个 agent」**。

- 点击 → 取当前输入框内容 → `orca orchestration run-create` +
  `orca orchestration dispatch --prompt <内容>`。
- 效果：在熟悉的 Codex GUI 里写提示词，一键让 Claude Code / OpenCode / 另一个 Codex
  在各自 worktree 里并行跑，结果回到 4.2 的面板。

这是两边强项的真正相乘：AetherCodex 的 GUI 体验 + Orca 的并行编排。
但它依赖 4.2 已经跑通，排在最后。

## 5. 实施阶段

| 阶段 | 内容 | 前置 | 规模 |
|---|---|---|---|
| P0 | Orca 探测（有没有装、版本、CLI 路径）+ `OrcaClient`（子进程 + JSON 解析 + 超时 + 降级） | — | 小 |
| P1 | 4.1 配置下行 + 供应商页状态显示 | P0 | 小 |
| P2 | 4.3 会话让路 | P0 | 小 |
| P3 | 4.2 状态面板（bridge 路由 + 注入面板） | P0 | 中 |
| P4 | 4.4 fan-out 按钮 | P3 | 中 |

`OrcaClient` 放 `crates/aethercodex-core/src/orca/`，与 `zed_remote` 同构——那已经是
"探测外部工具 + 调用 CLI + 优雅降级"的现成范式，直接照搬，不另起炉灶。

**Orca 没装时全部能力静默隐藏**，不产生任何报错或性能开销，这是硬要求。

## 6. 明确不做

- **不 fork、不内嵌 Orca。** 23k 文件、Electron + pnpm workspace + 原生模块 + 移动端，
  维护成本远超收益，而且我们刚从上游 814 commit 的同步困境里出来，不该再造一个。
- **不实现 Orca 的 runtime RPC 客户端**（理由见 §3）。
- **不碰 Orca 的 managed home 目录。** 只写规范 config.toml，镜像是 Orca 的职责。
- **不做 Codex App ↔ Orca 的双向会话同步。** 两边会话模型不同（单会话 vs run×worktree），
  强行映射会产生大量边界情况。

## 7. 待决策

1. **Orca 是已在用还是要新引入？** 若已在用，P1 可以立刻验证；若没用，整个计划的优先级要重排。
2. **P4 的 fan-out 要不要做？** 它最能体现联动价值，但也最侵入 Codex 页面，
   且行为依赖 Orca 的 run 模型，Orca 改版会直接影响它。
3. **AetherCodex 作为 config.toml 唯一作者这条契约，能否接受？**
   接受则 4.1 成立；若你希望 Orca 也能改写，需要另设仲裁机制（成本高得多）。
