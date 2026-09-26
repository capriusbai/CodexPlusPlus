# AetherCodex

<p align="center">
  <img src="docs/images/aethercodex.png" alt="AetherCodex 图标" width="160">
</p>

<p align="center">
  中文 | <a href="README_EN.md">English</a>
</p>

<p align="center">
  <img alt="Release" src="https://img.shields.io/github/v/release/capriusbai/CodexPlusPlus">
  <img alt="Stars" src="https://img.shields.io/github/stars/capriusbai/CodexPlusPlus">
  <img alt="License" src="https://img.shields.io/github/license/capriusbai/CodexPlusPlus">
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.85%2B-orange">
  <img alt="Tauri" src="https://img.shields.io/badge/tauri-2.x-24C8DB">
</p>

AetherCodex 是面向 Codex App 的外部增强启动器和管理工具。它不修改 Codex App 原始安装文件，而是通过外部 launcher 启动 Codex，并使用 Chromium DevTools Protocol 注入增强脚本。

## 快速使用

从 [GitHub Releases](https://github.com/capriusbai/CodexPlusPlus/releases) 下载最新版安装包：

- Windows：`AetherCodex-*-windows-x64-setup.exe`
- macOS Intel：`AetherCodex-*-macos-x64.dmg`
- macOS Apple Silicon：`AetherCodex-*-macos-arm64.dmg`
- Linux x86_64：`AetherCodex-*-linux-x64.deb` 或 `AetherCodex-*-linux-x64.tar.gz`
- Linux arm64：`AetherCodex-*-linux-arm64.deb` 或 `AetherCodex-*-linux-arm64.tar.gz`

安装后会有两个入口：

- `AetherCodex`：静默启动入口，不显示管理界面，只负责启动 Codex 并注入增强功能。
- `AetherCodex 管理工具`：Tauri 控制面板，用于启动、检查、修复、更新、配置中转注入、管理增强功能和用户脚本。

Windows 安装包会创建桌面和开始菜单快捷方式。macOS DMG 会安装 `/Applications/AetherCodex.app` 和 `/Applications/AetherCodex 管理工具.app`。Linux `.deb` 会安装到 `/usr/lib/aethercodex/`，并在应用菜单中注册 `AetherCodex` 与 `AetherCodex Manager` 两个入口。

### Ubuntu 26.04 / 24.04、Debian 13+

`.deb` 适用于 Ubuntu 24.04 及以上（含 26.04）和 Debian 13 及以上，依赖这些发行版自带的 WebKitGTK 4.1：

```bash
sudo apt install ./AetherCodex-1.3.0-linux-x64.deb
```

`apt` 会自动补齐 `libwebkit2gtk-4.1-0`、`libgtk-3-0t64` 等运行时依赖。如果用 `dpkg -i` 安装后提示缺少依赖，执行 `sudo apt -f install` 补齐即可。

卸载：

```bash
sudo apt remove aethercodex
```

### 其他发行版（tar.gz）

`.tar.gz` 不依赖包管理器，解压后按当前用户安装到 `~/.local`：

```bash
tar -xzf AetherCodex-1.3.0-linux-x64.tar.gz
cd AetherCodex-1.3.0-linux-x64
./install.sh            # 也可用 PREFIX=/opt/aethercodex ./install.sh
```

先确认发行版已提供 WebKitGTK 4.1 运行库（Ubuntu/Debian 为 `libwebkit2gtk-4.1-0`、Fedora 为 `webkit2gtk4.1`、Arch 为 `webkit2gtk-4.1`）。卸载执行同目录下的 `./uninstall.sh`。

`install.sh` 会调用 `aethercodex-manager --install-entrypoints` 写入用户级 `.desktop` 入口；也可以随时手动执行该命令重建入口，或用 `--uninstall-entrypoints` 移除。

## 关于本分支与上游项目

AetherCodex 是 [Codex++](https://github.com/BigPizzaV3/CodexPlusPlus) 的重新品牌化分支，由 Archai 维护。上游
项目由 BigPizzaV3 以 MIT 协议发布，核心的外部启动与 CDP 注入思路、增强脚本和
大量功能实现都来自上游，版权声明按 MIT 要求保留在安装包的 `copyright` 中。

本分支相对上游的改动：

- 品牌、命名、安装包标识和界面视觉改为 AetherCodex / Archai CI-VI；
- 新增 Linux 发布支持（Ubuntu 24.04 及以上，含 26.04）；
- 自动更新指向本分支的 Release，不再拉取上游安装包。

本分支移除了上游的推荐/广告功能和赞赏入口：不再拉取远端广告列表，界面里也没有
赞赏码。上游的赞助商、交流群和赞赏渠道属于上游项目，需要支持原作者请直接访问
[上游仓库](https://github.com/BigPizzaV3/CodexPlusPlus)。

## 交流与支持

本分支的问题与建议请提到 [Issues](https://github.com/capriusbai/CodexPlusPlus/issues)。

上游 Codex++ 的社区（QQ 群、微信群、Telegram 频道）与本分支无关，入口在
[上游仓库](https://github.com/BigPizzaV3/CodexPlusPlus)。

## 品牌与界面

界面遵循 Archai CI/VI 工作标准（`ARCHAI-CIVI-001`）：方形几何、发丝线分隔、
克制的语义化用色，不使用装饰性阴影、渐变和毛玻璃。

| 色彩 | sRGB | 语义 |
| --- | --- | --- |
| Archai Orange | `#ED9527` | 品牌锚点、章节标记。**不可**在浅色背景上作为小号正文（白底仅 2.35:1） |
| Aether Blue | `#2F6FED` | 连接、交互、链接、可编辑字段 |
| Evidence Green | `#2F855A` | 仅用于有证据支撑的「已验证 / 已发布 / 已关闭」状态 |
| Engineering Graphite | `#0B0D10` | 主要文字与技术权威色 |
| Secondary Ink | `#626A78` | 元数据与次要文案 |
| Soft Surface | `#F7F9FC` | 非语义的表面分层 |
| Hairline | `#DDE3EC` | 分隔线与表格边界 |

蓝色和绿色在深色主题下提亮、浅色主题下加深，保证两种主题都达到 WCAG AA 正文
对比度（4.5:1）。橙色在浅色主题下另有一个加深的 ink 变体用于文字和图标。

### 更换 Logo

全部图标由一个母版生成：

```bash
# 替换 assets/brand/aethercodex-mark.svg（或 .png，>= 1024x1024），然后：
bash scripts/brand/generate-icons.sh          # 重新生成 Windows .ico / macOS .icns / Linux PNG / 应用图标
bash scripts/brand/generate-icons.sh --check  # 校验图标与母版是否一致（CI 会跑）
```

母版是项目方提供的 Aether Logo，已用 `potrace` 矢量化后置于石墨底上；
来源、哈希和使用边界见 [`assets/brand/README.md`](assets/brand/README.md)。

## 主要功能

- Rust 后端和静默 launcher，启动时不依赖额外运行时。
- Tauri + React 管理工具，支持深色/浅色切换。
- 外部 CDP 注入，不改 `app.asar`，不向 Codex 安装目录写入 DLL。
- 中转注入模式：支持多个中转配置，写入 `AetherCodex` provider，并可切回官方 ChatGPT 登录态。
- 传统增强模式：插件入口解锁、特殊插件强制安装、会话删除、Markdown 导出、项目移动、Timeline 等。
- 用户脚本独立管理，可在启动时注入自定义脚本。
- Provider 同步：启动前同步本地会话 metadata，切换供应商后旧会话仍可见。
- Zed 打开入口：识别远程 SSH 上下文后，可从 Codex 直接打开对应文件到 Zed Remote Development。
- Upstream worktree 创建：可从 `upstream/<base-branch>` 创建新 worktree，创建前自动 fetch 远端分支，降低从陈旧本地 HEAD 派生导致的冲突风险。
- GitHub Release 自动更新，管理工具和静默启动器都会检测可用更新。
- Windows 单实例、无黑框启动、管理员权限清单、系统桌面路径识别。
- macOS x64/arm64 分架构 DMG，静默入口隐藏 Dock 图标。
- Linux x64/arm64 分架构 `.deb` 与便携 `.tar.gz`，遵循 XDG 规范注册 `.desktop` 入口和 hicolor 图标。

## 痛点与解决

API Key 登录模式下，Codex 原生插件入口会提示需要登录 ChatGPT，导致插件功能无法正常使用：

![API Key 模式下插件入口不可用](docs/images/pain-plugin-disabled.png)

Codex 原生会话列表只有归档入口，没有真正的删除按钮：

![原生会话列表缺少删除能力](docs/images/pain-no-delete-button.png)

AetherCodex 启动后会解锁插件入口，并在会话列表悬停时显示删除按钮：

![AetherCodex 解锁插件入口并添加删除按钮](docs/images/solution-plugin-and-delete.png)

顶部菜单栏会出现 `AetherCodex`，可以查看后端状态并打开设置面板：

![AetherCodex 后端状态指示灯](docs/images/backend-status-indicator.png)
![AetherCodex 设置面板](docs/images/settings-panel.png)

## 中转注入

中转注入适合已经在 Codex/ChatGPT 中完成官方账号登录，同时希望把模型请求转到自定义兼容 API 的场景。

这种混合模式的边界是：

- 官方 ChatGPT/Codex 登录态继续负责 Codex App 的账号能力和插件入口。
- 中转配置只接管模型请求使用的 Base URL、Key 和模型名称。
- 兼容 API 供应商不需要固定为某一家；只要上游协议和 Codex 配置匹配即可。
- 清除 API 模式后应能回到官方登录态，继续使用官方账号和插件。

应用中转注入前建议先做一次最小检查：

1. 先确认 Codex 已检测到 ChatGPT 登录状态，插件入口可用。
2. 确认自定义 Base URL 可访问，并且支持所选上游协议（例如 Responses 兼容接口）。
3. 用目标 Key 做一次最小认证测试，例如模型列表或很短的消息请求。
4. 只记录 Key 是否存在和认证结果，不要把真实 Key 写入日志、截图或 issue。
5. 确认 `~/.codex/config.toml` 已有备份，便于清除 API 模式后回滚。

在管理工具的“中转注入”页面：

1. 确认已经检测到 ChatGPT 登录状态。
2. 添加一个或多个中转配置，填写 Base URL 和 Key。
3. 选择当前配置并应用中转注入。
4. 启动 `AetherCodex`。

AetherCodex 会在 `~/.codex/config.toml` 中写入类似配置：

```toml
model_provider = "AetherCodex"

[model_providers.AetherCodex]
name = "AetherCodex"
wire_api = "responses"
requires_openai_auth = true
base_url = "https://example.com/v1"
experimental_bearer_token = "sk-..."
```

如果需要回到官方登录态，在“中转注入”页面点击清除 API 模式即可移除 `OPENAI_API_KEY` 相关配置并切回官方 ChatGPT 登录模式。

## 增强功能

增强功能在管理工具中统一开关。默认开启增强注入；关闭后不会注入 AetherCodex 菜单和脚本。

如果启用中转注入模式，插件入口解锁和强制安装不再需要，界面会提示“中转注入模式下无需开启”。会话删除、导出、移动、Timeline 和用户脚本等增强仍可继续使用。

## 自动更新与安装包

AetherCodex 通过 GitHub Release 发布安装包。Windows 会生成 NSIS 安装程序，macOS 会生成 Intel x64 和 Apple Silicon arm64 两个 DMG，Linux 会生成 x64 和 arm64 的 `.deb` 与 `.tar.gz`。

管理工具的“关于”页可以检查并启动更新。静默启动器发现新版本时会拉起管理工具并进入更新提示。更新时只会挑选与当前系统和 CPU 架构匹配的安装包：Linux 优先 `.deb`，没有匹配的 `.deb` 时回退到同架构的 `.tar.gz`。

## 数据位置

- Codex 配置：`~/.codex/config.toml`
- Codex 登录状态：`~/.codex/auth.json`
- Codex 本地数据库：`~/.codex/state_5.sqlite`
- AetherCodex 状态与日志：`~/.aethercodex/`
- Provider 同步备份：`~/.codex/backups_state/provider-sync`

## 常见问题

### AetherCodex 菜单没出现

确认是从 `AetherCodex` 入口启动，而不是原版 Codex。也可以打开管理工具的“诊断”和“日志”页面查看注入状态。

### 插件内显示后端连不上

先在浏览器或 PowerShell 里测试：

```powershell
Invoke-RestMethod -Method Post -Uri http://127.0.0.1:57321/backend/status -Body "{}" -ContentType "application/json"
```

如果接口正常，但插件仍显示超时，通常是 Codex 页面里的 CDP bridge 或脚本缓存问题。重启 AetherCodex，或在管理工具里查看日志中的 `renderer.script_loaded`、`bridge.request`、`bridge.response`。

### Upstream worktree 和 Codex 原生创建有什么区别

AetherCodex 的 Upstream worktree 功能等价于先更新远端分支，再执行：

```bash
git worktree add -b <new-branch> <worktree-path> upstream/<base-branch>
```

这样新 worktree 从最新的远端跟踪分支开始，而不是从当前会话所在的本地 HEAD 开始。如果 AetherCodex 无法安全识别当前 Codex 版本的原生 worktree 创建表单，请从 AetherCodex 菜单中手动填写仓库路径、分支名、worktree 路径、remote 和 base branch。

### macOS 提示无法打开或已损坏

当前安装包未签名/未公证时，macOS Gatekeeper 可能拦截，出现“已损坏，无法打开”的提示：

![macOS 提示 AetherCodex 管理工具已损坏](docs/images/macos-damaged-warning.png)

如果遇到该提示，可以在终端执行下面两条命令，解除苹果系统的安全隔离限制：

```bash
sudo xattr -rd com.apple.quarantine /Applications/AetherCodex\ 管理工具.app
sudo xattr -rd com.apple.quarantine /Applications/AetherCodex.app
```

执行后重新打开 `AetherCodex` 或 `AetherCodex 管理工具` 即可。

### macOS Intel 能用吗

可以。Release 会分别提供 `macos-x64.dmg` 和 `macos-arm64.dmg`。Intel Mac 下载 x64 包，Apple Silicon 下载 arm64 包。

### Linux 上管理工具启动后是空白窗口

这是 WebKitGTK 的渲染问题，不是注入失败。先尝试关闭硬件加速：

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 aethercodex-manager
```

Ubuntu 24.04 起内核默认开启 `kernel.apparmor_restrict_unprivileged_userns=1`，会阻止依赖 bubblewrap 沙箱的进程创建 user namespace。AetherCodex 管理工具自身不使用该沙箱，但被拉起的 Codex App（Electron/Chromium）可能受影响。如果 Codex 本体起不来，可加载 Ubuntu 自带的 `bwrap-userns-restrict` 配置：

```bash
sudo apt install apparmor-profiles
sudo install -m 0644 /usr/share/apparmor/extra-profiles/bwrap-userns-restrict /etc/apparmor.d/bwrap-userns-restrict
sudo apparmor_parser -r /etc/apparmor.d/bwrap-userns-restrict
```

### Linux 上找不到 Codex App

Linux 没有 MS Store / `/Applications` 这样的固定安装位置，AetherCodex 会在 `/opt`、`/usr/lib`、`/usr/share`、`~/.local/share`、`~/Applications` 等目录下查找包含 Codex 可执行文件（`Codex`、`codex`、`codex-app`）的目录，以及名字含 `codex` 的 `.AppImage`。

只有确实包含可执行文件的目录才会被采纳，因此 `PATH` 上的 `codex` CLI 不会被误认成桌面版。自动识别失败时，在管理工具的设置里手动填写 Codex App 路径（可以直接填 `.AppImage` 文件路径）。

## 开发

```bash
# 前端检查
cd apps/aethercodex-manager
npm install
npm run check
npm run vite:build

# Rust 检查
cd ../..
cargo fmt --check
cargo test
cargo build --release
```

主要结构：

```text
apps/
  aethercodex-launcher/          静默启动入口
  aethercodex-manager/           Tauri 管理工具
assets/inject/
  renderer-inject.js            注入到 Codex 渲染端的增强脚本
crates/
  aethercodex-core/              启动、注入、配置、更新、安装、桥接等核心逻辑
  aethercodex-data/              会话数据、导出、Provider 同步
scripts/installer/
  windows/AetherCodex.nsi     Windows NSIS 安装包
  macos/package-dmg.sh          macOS DMG 打包
  linux/package-linux.sh        Linux .deb 与 .tar.gz 打包
```

Linux 本地打包（需要先 `cargo build --release`）：

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev desktop-file-utils
cargo build --release
bash scripts/installer/linux/package-linux.sh 1.3.0     # 架构默认取本机，可显式传 x64 / arm64
```

产物在 `dist/linux/`。CI 固定在 `ubuntu-24.04` 上构建：它的 glibc 与 WebKitGTK 4.1 soname 是需要支持的最低版本，更新的 Ubuntu（25.10、26.04）向下兼容。

## 友情链接

- [上游项目 Codex++](https://github.com/BigPizzaV3/CodexPlusPlus)
- [LINUX DO](https://linux.do)

## 说明

AetherCodex 是外部增强工具，不修改 Codex App 原始文件。Codex App 更新后，如果页面结构变化，可能需要更新注入脚本。
