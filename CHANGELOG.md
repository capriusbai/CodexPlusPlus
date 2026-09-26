# 更新日志

## 2.0.0 - 2026-09-26

重大变更：项目更名为 **AetherCodex**，采用 Archai CI/VI 视觉规范。升级会自动迁移
现有配置，但二进制名、安装路径和包名全部改变。

### 品牌与命名

- 产品名 `Codex++` → `AetherCodex`；二进制 `codex-plus-plus` / `codex-plus-plus-manager`
  → `aethercodex` / `aethercodex-manager`；bundle id → `com.archai.aethercodex`。
- Rust crate 与目录同步改名：`codex-plus-core/data` → `aethercodex-core/data`，
  `apps/codex-plus-*` → `apps/aethercodex-*`。
- 界面改用 Archai CI/VI：方形几何、发丝线、语义化用色，移除全部装饰性阴影与渐变。
  蓝绿在深浅主题分别提亮/加深以满足 WCAG AA 正文对比度；橙色在浅色主题使用加深的
  ink 变体，避免低对比度文字。
- 新增 `assets/brand/` 单一母版与 `scripts/brand/generate-icons.sh`，一条命令生成
  Windows `.ico`、macOS `.icns`、Linux PNG 和应用图标；CI 用 `--check` 防止图标漂移。
- 采用项目方提供的 Aether Logo：原始位图经 `potrace` 矢量化后置于 Engineering Graphite
  底上，界面、安装包和三端图标共用同一母版。

### 升级兼容

- 状态目录 `~/.codex-session-delete` → `~/.aethercodex`，首次读取时自动迁移设置、
  用户脚本和日志（`codex-plus.log` → `aethercodex.log`），新目录已存在时不覆盖。
- 用户脚本配置目录 `Codex++` → `AetherCodex`，同样自动迁移；启动器与管理工具改为
  共用 `aethercodex_core::paths` 的同一实现。
- Windows：清理旧的卸载注册表项（`CodexPlusPlus`、`Codex++`）、桌面/开始菜单快捷方式
  和 `CodexPlusPlusWatcher` 自启动项；NSIS 安装包会移除旧安装目录。
- macOS：安装与卸载时移除 `Codex++.app`、`Codex++ 管理工具.app`。
- Linux：`.deb` 声明 `Conflicts`/`Replaces`/`Provides: codex-plus-plus`，apt 会干净替换
  旧包；入口安装会清理旧的 `.desktop` 与图标。
- 自动更新的产物匹配改为识别 `aethercodex`，同时保留对旧命名产物的识别；更新源指向
  本分支 Release，不再拉取上游安装包。

### 分支与归属

- 明确 AetherCodex 是上游 [Codex++](https://github.com/BigPizzaV3/CodexPlusPlus) 的
  重新品牌化分支，`copyright` 保留 BigPizzaV3 的 MIT 版权声明。
- 移除推荐/广告功能：删除 `ads` 模块、`/ads` bridge 路由、管理工具的「推荐内容」页面和
  注入菜单的推荐页签，不再请求任何远端广告列表。
- 移除赞赏入口：删除注入菜单的赞赏码页签与随脚本内联的二维码图片，仓库内相关图片一并删除。
- 移除上游的赞助商列表与交流群入口，避免把上游的商业合作与社区表述成本分支的。
- 脚本市场与推荐列表仍指向上游维护的数据源，界面中已注明其归属。

## 1.3.0 - 2026-09-26

- 新增 Linux 发布支持：Release 会同时产出 x64 / arm64 的 `.deb` 与便携 `.tar.gz`，适用于 Ubuntu 24.04 及以上（含 26.04）和 Debian 13 及以上。
- 新增 `scripts/installer/linux/package-linux.sh`，构建 `.deb`（含依赖声明、`postinst`/`postrm`、`/usr/bin` 软链接、copyright 与 changelog）和便携压缩包（含 `install.sh` / `uninstall.sh`），并在打包后校验产物结构。
- 新增 Linux 入口安装能力：按 XDG 规范写入 `aethercodex.desktop` / `aethercodex-manager.desktop` 与 hicolor 图标，支持安装、修复、卸载，并能识别 `.deb` 安装在 `/usr/share/applications` 下的入口。
- 管理工具新增 `--install-entrypoints` / `--uninstall-entrypoints` 命令行参数，可在无桌面会话下管理入口，供 `.tar.gz` 安装脚本使用。
- 新增 Linux 下的 Codex App 自动识别：扫描 `/opt`、`/usr/lib`、`/usr/share`、`~/.local/share`、`~/Applications` 等目录中真正包含 Codex 可执行文件的目录，并支持把 `.AppImage` 当作 app 路径；`PATH` 上的 `codex` CLI 不会被误判为桌面版。
- 自动更新在 Linux 上按架构挑选安装包，优先 `.deb`，回退同架构 `.tar.gz`，并通过 `xdg-open` 交给系统包管理器安装。
- 修复管理工具在 Linux 上打开外部链接时调用 macOS 的 `open` 而失败的问题，改为 `xdg-open`。
- CI 新增 Linux 打包任务（`ubuntu-24.04` / `ubuntu-24.04-arm`），在 PR 构建和 Release 流程中都会构建、安装并验证 `.deb` 入口注册。
- 版本号更新到 `1.3.0`，同步 Rust workspace、Tauri、前端 package 和后端展示版本。

## 1.2.4 - 2026-06-08

- 新增 Zed 远程项目记录能力，支持维护 AetherCodex 可识别的远程项目最近列表，并为远程工作区打开提供更稳定的回退策略。
- 修复供应商同步在存在多条 `session_meta` 记录时只处理部分会话元数据的问题。
- 修复 Windows 单实例启动保护，在默认端口被异常占用时改用更稳健的锁与端口回退逻辑，降低无法启动的概率。
- 限制 Codex 快速服务档位只对支持的模型生效，避免不兼容模型收到无效配置。
- 修复 macOS DMG 打包和 bundle 结构，恢复 launcher / manager 二进制重命名逻辑。
- 补充混合登录中继模式文档说明。
- 版本号更新到 `1.2.4`，同步 Rust workspace、Tauri、前端 package 和后端展示版本。

## 1.1.8 - 2026-05-26

- 新增上游分支 worktree 支持，可从上游仓库/分支创建和选择独立工作区。
- 新增上游分支列表获取、默认值处理、远端解析和 worktree 创建相关接口与测试。
- 优化供应商同步逻辑，保留 rollout 文件 mtime，减少同步后不必要的会话状态变化。
- 新增独立的「工具与插件」页面，用于统一管理 AetherCodex / Codex 的 MCP、skills、plugins，不再绑定到单个供应商。
- 切换供应商时会合并当前启用的工具与插件配置，同时避免把供应商专属配置误写入通用配置。
- 工具与插件列表改为从当前 Codex 配置实时读取启用状态，支持直接开关和删除条目。
- 调整通用配置提取逻辑，改为手动提取，减少自动覆盖和配置污染。
- 修复供应商切换隔离问题，避免 `model_catalog_json`、旧 `model_provider`、历史 provider 表和旧 `auth.json` 被带到新供应商。
- 修复纯 API 模式下 `auth.json` 没有写入 API Key 的问题，并固定供应商 provider 名称为 `AetherCodex`。
- 优化模型目录写入方式，支持与原始模型目录合并，并在预览中显示真实路径。
- 供应商配置页新增模型插入方式、模型列表、上下文大小、压缩上下文大小、目标功能等配置项。
- 官方模式下隐藏仅混入 API Key 场景使用的模型列表和模型插入方式。
- 将 Base URL、API Key、上游协议移动到模型列表之前，测试模型和上下文选项收进「更多选项」。
- 修复 `model_reasoning_effort`、`plan_mode_reasoning_effort` 重复写入导致 TOML 解析失败的问题。
- 修复重复插件表、空配置体、布尔值解析等导致配置文件解析失败的问题。
- 优化供应商详情页布局，保持顶部返回和提示区域固定，增大默认窗口尺寸并减少顶部缝隙。
- 移除脚本安装时的 checksum 阻断，避免市场脚本校验不一致导致安装失败。
- 清理关于页和状态页中不需要展示的登录、当前供应商、配置文件路径等信息。
- 调整提示信息居中显示，避免遮挡重启按钮。
- 更新讨论群二维码、README 说明和 macOS DMG 打包脚本。
