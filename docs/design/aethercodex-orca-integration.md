# AetherCodex × Orca 联动设计计划

状态：**草案，待决策**　·　日期：2026-09-26　·　范围：内部使用

调研对象：[`stablyai/orca`](https://github.com/stablyai/orca) @ `f0a3610`（v1.4.197，MIT，
23,053 个 TS/TSX 文件）。本文关于 Orca 机制的结论均来自该 commit 的源码，不是文档转述；
§1.1 另有用户本机（**v1.4.210**）的实测部署情况，两者不一致时以实机为准。

> 内部使用，不做商业化，按指示不展开 License 讨论。仅记录一个事实以备将来：Orca 是 MIT，
> 若某天需要对外分发，保留其 LICENSE 是单文件成本。

---

## 0. 硬约束：不得影响现有的官方 Codex / Claude / Orca

这是本计划的最高优先级约束，高于任何功能目标。任何一条联动能力若与它冲突，砍能力。

具体到可执行的规则：

1. **绝不写 `~/.codex/`。** 官方 Codex App 和用户 shell 读的就是它。AetherCodex 改为
   使用自己的 `CODEX_HOME`（见 §2.1），`~/.codex` 全程只读。
2. **绝不写 Orca 的任何目录**——不碰 managed home、不改 Orca 安装目录、不动它的配置与数据库。
3. **绝不改 Orca 进程状态**：不 kill、不重启、不抢它的端口。
4. **对 Orca 只做只读探测**（`--json` 查询类命令），写类命令仅在用户显式点击时执行。
5. **插件装进用户插件目录**，不注入 Orca 的 app bundle。
6. **Orca 缺席或版本不匹配时全部能力静默隐藏**，不报错、不降级官方功能。

验收方式：联动功能全开的情况下，把 AetherCodex 完全退出，官方 Codex / Claude / Orca
的行为必须和联动前逐字节一致（配置文件 diff 为空）。

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

### 1.1 本机实测部署（2026-09-26 采集）

以上是源码结论。下面是 `scripts/orca/collect-orca-info.sh` 在**用户本机**采到的实况，
它把 §8 的第 1 条从"待确认"变成了已知，并且直接影响 §4.5 的做法。

| 项 | 实测值 | 对设计的影响 |
|---|---|---|
| CLI | `~/.local/bin/orca-ide`，**v1.4.210** | 比本文调研的 v1.4.197 新 13 个版本。插件 API 未冻结（§3.2），必须按实机版本验证，不能按源码想当然 |
| 安装 | 用户级，非系统包（`orca-ide` 无 deb 记录） | 升级由用户自己控制，我们的探测不能假设固定路径 |
| 同名包 | `orca 50.2-0ubuntu0.1` = GNOME 屏幕阅读器，**无关** | 探测必须按 §0 第 3 条区分，绝不对它发命令 |
| 插件目录 | **不存在**，一个插件都没装 | P4 会是本机第一个 Orca 插件：没有冲突对象，也没有先例可抄 |
| 项目 | 2 个 git 项目（`摄政王`、`光胪寺`） | 都是真实在用的仓库，不是试验环境 |
| 编排 run | 3 个，其中 1 个 legacy（inspect only） | 见下方「摄政王」 |
| Claude 账号 | `claude.accounts: []`，且 rateLimits 报 `OAuth access token has been revoked` | **既有故障，与本计划无关。按 §0 一律不碰**，不"顺手修" |

#### Orca 现在读的就是官方 `~/.codex`

这是最重要的一条实测发现：

```
codex.accounts:      []
codex.activeAccountId: null
codex.systemDefault: { hasAuth: true, authKind: "oauth", workspaceLabel: "Personal (Pro)" }
```

Orca 没有建任何 per-account managed home，而是走 `systemDefault` —— 也就是**直接读
`~/.codex`**。`CODEX_HOME` 环境变量未设置，`find` 也没在 Orca 数据目录下找到任何
`codex*home*` 目录。

所以 §2.1 不是预防性设计，而是**已经踩在线上**：如果 AetherCodex 按旧行为往
`~/.codex/config.toml` 写中转配置，用户那两个项目、那个 coordinator 下面所有 Codex agent
读到的配置会当场被改写。已实现的 `CODEX_HOME` 隔离（官方/中转二选一 + 官方只读）正是
这条的解法，且是唯一不需要用户信任我们"改得对"的解法。

#### 「摄政王」：实例级 Coordinator 已经存在

`run_b07e3bee8874` 的 objective 写得很明确：

> 摄政王：本机 Orca 实例级总 Coordinator，管理本实例所有项目和 agents，直接向 Owner 负责；
> 不隶属于 AetherWorks 或任何业务项目。

这改变了 §4.5 的做法：本机**已经有**一个实例级调度层，AetherCodex 不应该另起一个平行的
编排入口，而应该作为一种 agent **注册进现有 coordinator 管的池子里**。§4.4 的 fan-out
同理——从 Codex GUI 发起的并行任务应该落进既有 run，而不是新建一个孤立的 run。

#### 用户自建的 adaptor：形态是「状态上报」

采集到两处，且只有这两处：

```
~/.config/orca/omp-managed-status-extension/orca-agent-status.ts
~/.orca/agent-hooks/                        # 权限 700，建于 09-16
```

两处都不是 Orca 官方插件目录（官方是 `<root>/plugins/`），和"我自己让 Orca 搭的"一致。
命名指向 **agent status**，与源码里 hook server 单一持有 agent status store 的模型
（§4.5 第 1 条）吻合：这套东西很可能已经是一个 status producer 的接入点。

**若确实如此，§4.5 应当复用它，而不是另定一套上报协议。** 在读到源码之前这仍是推断——
采集器已补上 `--with-sources`，见 §8 第 2 条。

## 2. 两个真实碰撞点

调研中发现两处二者写/读同一份状态，这是设计必须先解决的：

### 2.1 `~/.codex/config.toml` —— 改用隔离，不做仲裁

**决策（用户）：不接受"AetherCodex 唯一作者"契约，两边的配置都要保留。**

这个判断是对的，而且解法比仲裁干净：**Codex 认 `CODEX_HOME` 环境变量。**

Orca 源码已经在用这个机制（`src/main/codex-usage/codex-session-file-discovery.ts`）：

> `// Why: Orca-launched Codex processes receive an Orca-owned CODEX_HOME`

```ts
// src/main/ai-vault/session-scanner-agent-sources.ts
export const DEFAULT_CODEX_HOME_DIR = join(homedir(), '.codex')
resolveAbsoluteDirOverride(process.env.CODEX_HOME, DEFAULT_CODEX_HOME_DIR)
```

所以三方各持一份，互不写入：

| 谁 | CODEX_HOME | 谁来写 |
|---|---|---|
| 官方 Codex App / shell | `~/.codex` | **只有用户自己** |
| Orca 编排的 agent | Orca managed homes | 只有 Orca |
| AetherCodex 启动的 Codex | `~/.aethercodex/codex-home` | 只有 AetherCodex |

写冲突从根上消失，§0 的硬约束自动满足。

#### 这要求改 AetherCodex 现有行为

必须说清楚：**AetherCodex 目前确实直接改写 `~/.codex/config.toml`**，这条路要求把它改成
写自己的 home。这不只是"接个 Orca"，是产品行为变更，代价是：

- **登录态**：`auth.json` 不在新 home 里。首次启用时从 `~/.codex` **复制**一份（只读源），
  之后两边独立。官方那边改了密码/换了账号，AetherCodex 这边不会自动跟。
- **历史会话**：`state_5.sqlite` 同理。会话管理页需要同时读两个 home 并标明来源，
  否则用户会觉得"我的会话不见了"。
- **切换成本**：用户得理解"我现在开的是哪一个 Codex"。UI 上必须显著标注。

**决策（用户）：每次启动时显式选，不做隐式模式。已实现。**

- 概览页两个按钮：「启动（官方配置）」/「启动（中转配置）」，hover 说明各自读哪个目录。
- 静默启动器：`--profile official|proxy`（别名 `--codex-home`）。
- 设置里记住上次选择作为默认；不带参数时沿用。
- 中转注入在官方 profile 下**拒绝执行并报错**，不会改写 `~/.codex`。
- 中转 home 首次启动自动创建并从官方复制一份 `auth.json`，之后两边独立。

实测（Linux）：`--profile official` 给 Codex 的环境里 `CODEX_HOME` **未设置**，
官方目录零改动；`--profile proxy` 导出 `CODEX_HOME=~/.aethercodex/codex-home`
且登录态已 seed。§0 的验收标准在这条路径上成立。

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

## 4. 五项联动能力

按"先拿价值、后担风险"排序。

### 4.1 配置导出：把中转配置显式推给 Orca（不再是自动继承）

隔离之后，"配一次全体继承"不再自动发生——这是③决策的直接代价，得讲清楚。

替代方案是**显式导出**，用户点了才发生：

- AetherCodex 供应商页加「导出到 Orca」按钮，列出 Orca 账号（`orca account list --json`）
  让用户勾选。
- 导出内容只有 provider 段，**不覆盖整个文件**，且导出前先展示 diff。
- 走 Orca 自己的写入通道（CLI 命令），不直接写它的 managed home，符合 §0 第 2、4 条。

这比自动继承慢一步，但符合你"怕搞坏"的顾虑：每一次跨系统写入都是你按下去的。

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

### 4.4 从 Codex GUI 发起 fan-out ⭐ 已决策：做

在 Codex App 的输入框旁注入一个按钮：**「分发到 N 个 agent」**。

- 点击 → 取当前输入框内容 → 优先用插件的 `terminal:send` 能力投递；
  不可用则回退 `orca-ide orchestration dispatch --prompt <内容>`。
- **默认投进既有 run**（本机是「摄政王」那条，§1.1），`run-create` 只在用户明确要求
  开新 run 时才用。凭空建 run 会在用户的调度视图里堆出一批孤立记录。
- 效果：在熟悉的 Codex GUI 里写提示词，一键让 Claude Code / OpenCode / 另一个 Codex
  在各自 worktree 里并行跑，结果回到 4.2 的面板。

这是两边强项的真正相乘：AetherCodex 的 GUI 体验 + Orca 的并行编排。
但它依赖 4.2 已经跑通，排在最后。

### 4.5 注册成独立的 Orca agent 类型 ⭐ 已决策：做

`AgentType = WellKnownAgentType | (string & {})` 是开放联合，插件可贡献新类型。
注册 `aethercodex` 作为一种 agent，让"带中转注入和页面增强的 Codex GUI 会话"
出现在 Orca 的 run 里，和 CLI agent 并排被编排。

需要解决的四件事：

0. **挂进「摄政王」，不另起编排。** 本机已有实例级 coordinator（§1.1），AetherCodex 注册
   为 agent 后应受它调度，出现在它管的池子里。不新建平行 run，不绕过它直接调度。
1. **状态上报**：Orca 的 agent status 由执行宿主的 hook server 单一持有。我们作为新的
   producer 要写进那个 store，而不是自己维护一份。按
   `docs/reference/agent-status-store.md` 的规矩来。**优先复用用户自建的
   `orca-agent-status.ts` / `~/.orca/agent-hooks/`**（§1.1）——那已经是本机在用的
   producer 接入点，另定一套协议等于把本机现有状态链路劈成两条。
2. **生命周期**：Orca 期待 agent 能被启动、观察、停止。AetherCodex 的静默启动器天然能做
   前两件；"停止"要映射到关闭那个 Codex 窗口，且**不能误杀用户自己开的官方 Codex**
   —— 只能停我们自己启动的、记录了 PID 的实例。这是 §0 第 3 条的具体落地。
3. **worktree 绑定**：Orca 的 run 以 worktree 为单位。AetherCodex 要能在指定 worktree
   目录下启动 Codex 会话，这一条现有的 upstream worktree 能力可以复用。

风险：这是**最依赖未冻结插件 API** 的一项，排在最后做，且必须能一键关掉退回普通模式。

## 5. 实施阶段

| 阶段 | 内容 | 前置 | 规模 |
|---|---|---|---|
| P0 | Orca 探测（有没有装、版本、CLI 路径）+ `OrcaClient`（子进程 + JSON 解析 + 超时 + 降级）。**必须认 `orca-ide` 而非 `orca`，并拒绝自述为屏幕阅读器的二进制**；基线版本按实机 v1.4.210 | — | 小 |
| ~~P1~~ | ~~独立配置模式~~ → **已实现**：启动时选择 + CODEX_HOME 隔离 + auth 种子 + 写入保护 | — | 已完成 |
| P2 | 4.1 配置导出（显式、带 diff 预览） | P1 | 小 |
| P3 | 4.3 会话让路 | P0 | 小 |
| P4 | Orca 插件骨架（`orca-plugin.json` + 面板 HTML + 事件订阅） | P0 | 中 |
| P5 | 4.2 状态面板（插件事件 → bridge → 注入面板） | P4 | 中 |
| P6 | 4.4 fan-out 按钮 | P5 | 中 |
| P7 | 4.5 注册为 Orca agent 类型 | P5 | 大 |

`OrcaClient` 放 `crates/aethercodex-core/src/orca/`，与 `zed_remote` 同构——那已经是
"探测外部工具 + 调用 CLI + 优雅降级"的现成范式，直接照搬，不另起炉灶。

**Orca 没装时全部能力静默隐藏**，不产生任何报错或性能开销，这是硬要求。

## 6. 明确不做

- **不 fork、不内嵌 Orca。** 23k 文件、Electron + pnpm workspace + 原生模块 + 移动端，
  维护成本远超收益，而且我们刚从上游 814 commit 的同步困境里出来，不该再造一个。
- **不实现 Orca 的 runtime RPC 客户端**（理由见 §3）。
- **不把关键路径押在插件 API 上**——它未冻结，见 §3.2 的双轨原则。
- **不碰 Orca 的 managed home 目录**，也**不写 `~/.codex/`**。AetherCodex 只写自己的
  `CODEX_HOME`；要把配置送进 Orca 时走它自己的写入通道，且由用户显式触发（§4.1）。
- **不做 Codex App ↔ Orca 的双向会话同步。** 两边会话模型不同（单会话 vs run×worktree），
  强行映射会产生大量边界情况。

## 7. 决策记录

| # | 问题 | 决策 |
|---|---|---|
| ① | Orca 是否在用 | **已在用**：v1.4.210，2 个真实项目，实例级 coordinator 在跑（§1.1） |
| ② | P4 fan-out 做不做 | **做** |
| ③ | config.toml 唯一作者契约 | **不接受**。两边配置都保留 → 改为 `CODEX_HOME` 隔离（§2.1） |
| ④ | 注册成独立 Orca agent 类型 | **做**（§4.5） |
| — | 硬约束 | **不得影响现有官方 Codex / Claude / Orca**（§0） |

## 8. 待补充的信息

1. ~~本机 Orca 的实际部署情况。~~ **已补**：见 §1.1。v1.4.210、用户级安装、零插件、
   Orca 直读官方 `~/.codex`、已有实例级 coordinator「摄政王」。
2. **用户自建 agent adaptor 的源码。** 路径已定位（§1.1），内容还没读到。采集器原来只带回
   文件名，现已补上 `--with-sources`：

   ```bash
   bash scripts/orca/collect-orca-info.sh --with-sources > orca-info.txt
   ```

   它会把 `orca-agent-status.ts` 和 `~/.orca/agent-hooks/` 下的文本源码一并带出，
   token 形态的值（`sk-*`、`gh*_*`、JWT、`Bearer`、以及 `*token`/`secret`/`password`
   等键名的值）自动打码，二进制跳过。**打码是安全网不是保证，发之前请过一眼。**

   在读到这份源码之前，§4.5 第 1 条的"复用"只是推断。
3. ~~会话管理的双 home 读取标注。~~ **已补**：会话页和供应商页都带 profile 徽标
   （官方配置 `~/.codex` / 中转配置 `~/.aethercodex/codex-home`），中转下的空列表
   会明说"官方那边的记录没有丢失"。
