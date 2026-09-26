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

> **修订（2026-09-26）**：初版基于"Orca 只暴露 CLI 和 runtime RPC"的判断选了 CLI。
> 复查后发现 **Orca 有完整的插件系统**（`src/main/plugins/`，121 个文件），
> 这是更合适的主集成面。以下为修订后的结论。

### 3.1 Orca 插件 API（主路径）

插件根目录放 `orca-plugin.json`（manifest v1），由桌面端、`orca serve`、relay 和 CLI
**用同一份 zod schema 校验**，所以一个插件在本地、SSH 远程和无头服务里行为一致。

贡献点（`src/shared/plugins/plugin-manifest.ts`）：

| 贡献点 | 内容 | 对我们的意义 |
|---|---|---|
| `panels` | **HTML 入口**，渲染在沙箱 frame，右侧活动栏出 Lucide 图标（上限 64） | AetherCodex 的界面本来就是 HTML，可直接成为 Orca 面板 |
| `commands` | `global` / `worktree` 两种上下文（上限 256） | 从 Orca 命令面板触发 AetherCodex 动作 |
| `events` | 封闭集合：`worktree.created`、`worktree.removed`、`agent.status.changed` | **订阅式状态，不用轮询** |
| `agentProfiles` | 指向 profile 文件 | 这就是 agent adaptor |
| `keybindings` / `languagePacks` / `vmRecipes` | — | 暂不用 |

能力（`plugin-capabilities.ts`，需在 manifest 声明并获授权）：
`events:subscribe`、`notifications:show`、`secrets`、`settings:own`、`storage`、
`terminal:send`、`workspace:read`。

关键事实：`AgentType` 是**开放联合类型**
（`export type AgentType = WellKnownAgentType | (string & {})`，见 `agent-status-types.ts`），
所以插件可以引入全新的 agent 类型字符串，而不是只能复用内置的 `codex`。

### 3.2 但有一条明写的风险

`plugin-manifest.ts` 的注释原文：

> *Everything here is EXPERIMENTAL: no compatibility promises until pluginApi v1 freezes.*

插件 API 未冻结，Orca 升级可能直接破坏我们的插件。这不是推测，是他们自己标注的。

### 3.3 因此采用双轨

| 轨道 | 用途 | 稳定性 |
|---|---|---|
| **稳定轨：`orca` CLI + `--json`** | 配置协调、会话让路——即**不能坏**的部分 | 有 `cli-command-name-parity.test.ts` 保命令名 |
| **富能力轨：Orca 插件** | 面板、事件订阅、fan-out——**坏了只是降级**的部分 | 实验性，需跟版本 |

原则：**凡是坏了会导致数据错误或配置丢失的，走 CLI；凡是坏了只是少一块 UI 的，走插件。**

不做 runtime RPC 客户端（理由不变：混合版本常态 + 未知 opcode 静默丢弃）。

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

- 数据源：插件订阅 `agent.status.changed` 事件（`events:subscribe` 能力），**推送而非轮询**。
- 通道：插件收事件 → 写入 AetherCodex 的 helper bridge → 注入脚本渲染。
- 降级：插件不可用时回退到 `orca orchestration run-show --json` 轮询。
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

- 点击 → 取当前输入框内容 → 优先用插件的 `terminal:send` 能力投递；
  不可用则回退 `orca orchestration run-create` + `dispatch --prompt <内容>`。
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
| P3 | Orca 插件骨架（`orca-plugin.json` + 面板 HTML + 事件订阅） | P0 | 中 |
| P4 | 4.2 状态面板（插件事件 → bridge → 注入面板） | P3 | 中 |
| P5 | 4.4 fan-out 按钮 | P4 | 中 |

`OrcaClient` 放 `crates/aethercodex-core/src/orca/`，与 `zed_remote` 同构——那已经是
"探测外部工具 + 调用 CLI + 优雅降级"的现成范式，直接照搬，不另起炉灶。

**Orca 没装时全部能力静默隐藏**，不产生任何报错或性能开销，这是硬要求。

## 6. 明确不做

- **不 fork、不内嵌 Orca。** 23k 文件、Electron + pnpm workspace + 原生模块 + 移动端，
  维护成本远超收益，而且我们刚从上游 814 commit 的同步困境里出来，不该再造一个。
- **不实现 Orca 的 runtime RPC 客户端**（理由见 §3）。
- **不把关键路径押在插件 API 上**——它未冻结，见 §3.2 的双轨原则。
- **不碰 Orca 的 managed home 目录。** 只写规范 config.toml，镜像是 Orca 的职责。
- **不做 Codex App ↔ Orca 的双向会话同步。** 两边会话模型不同（单会话 vs run×worktree），
  强行映射会产生大量边界情况。

## 7. 待决策

1. ~~Orca 是已在用还是要新引入？~~ **已确认：本机已部署 Orca，P1 可立即验证。**
2. **P4 的 fan-out 要不要做？** 它最能体现联动价值，但也最侵入 Codex 页面，
   且行为依赖 Orca 的 run 模型，Orca 改版会直接影响它。
3. **AetherCodex 作为 config.toml 唯一作者这条契约，能否接受？**
   接受则 4.1 成立；若你希望 Orca 也能改写，需要另设仲裁机制（成本高得多）。
4. **AetherCodex 要不要注册成一个独立的 Orca agent 类型？**
   `AgentType` 开放联合允许这么做，意味着"带中转注入和页面增强的 Codex GUI"可以作为
   一种可被编排的 agent 出现在 Orca 的 run 里。潜力最大，但也最依赖未冻结的插件 API。
