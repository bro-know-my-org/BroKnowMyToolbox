# 实施计划

本计划按依赖顺序推进。阶段只有在完成条件有可重复证据时才结束；不得用“代码已写”代替验证。

## 当前进度（2026-09-05）

- Spark Analyzer 工作树已启用三平台 keyring backend，并通过本机跨 Entry、跨进程测试；三平台 CI 与新版本发布尚待完成。
- Phase 1 workspace、`bkmt --version`、Vue/Tauri 桌面壳、前后端测试和 CI 配置已落地。
- 文件生成器共享核心已实现 v1 JSON 模板、变量默认值/必填校验、安全生成计划、冲突/覆盖、逐文件结果和 capability-based 防越界写入；GUI 与 `bkmt file create` 已接入计划、执行、授权、dry-run、force、JSON 及分级结果。
- 产品壳已采用固定侧栏工作台：正式首页、工具页共用侧栏与顶栏，并实现工具目录、收藏、最近使用和键盘优先命令面板；临时方案切换代码已移除。
- 视觉设置支持明亮、深色和跟随系统主题，以及强调色、字号、界面密度和减少动画；设置修改会同步应用运行时 CSS token，避免页面与 Naive UI 主题不同步。界面文案支持内置中英文与用户语言覆盖。
- 工作台在 860px 最小窗口宽度仍保留侧栏导航文字，不把桌面导航退化为无文字的纯图标栏。
- 命令面板已支持方向键选择、Enter 打开和可见焦点；工具声明及文件模板的共享语义校验已收紧。
- 设置面板的语言、主题、强调色、字号、密度和减少动画控件具有稳定可访问名称；组件测试覆盖未保存视觉修改的即时运行时应用、语言切换、持久化以及手动检查/打开更新页。
- 桌面配置 IPC 使用 camelCase DTO 与 TypeScript 契约对齐，核心配置模型及磁盘 TOML 继续使用 snake_case，避免传输格式改动破坏已有配置文件。
- 配置和授权文件使用有界跨进程锁，锁争用会在 500ms 后返回诊断错误；配置测试通过多个真实子进程验证原子写入。
- CLI 已将原先集中在单个入口文件的参数、错误、运行时能力、文件命令和 Spark 命令按内部 seam 拆分；进程入口保持最小，现有 CLI 契约由端到端测试覆盖。
- Spark GUI 已嵌入 sibling 修复版 npm UI 和 Tauri plugin；`bkmt spark` 已直接复用 `bkmsa-core`/`bkmsa-agent` 实现工具目录、报告解析、确定性工具和 AI 分析。正式发布前两类依赖都必须切到已发布修复版本。
- Spark GUI 的网络、凭据和导出操作已通过上游 host authorizer seam 接入 Toolbox Rust 授权，并具有允许/拒绝/自动重试体验。
- Spark 宿主测试已覆盖 Toolbox authorizer 装入真实 Tauri plugin 后的 IPC 阻断/放行，以及页面授权弹窗、持久化和自动重试流程。
- Spark 的语言与主题由 Toolbox 设置控制，AI provider/model/temperature 等非敏感偏好通过显式 store seam 写入共享数据根；Toolbox 嵌入模式不再依赖 WebView `localStorage`。
- 手动版本检查、Apache-2.0 license、品牌图标、四目标 Release workflow、便携包、校验清单及 Scoop/Homebrew/AUR 元数据生成器已落地；workflow 已加入 staged CLI/桌面启动冒烟，渠道同步仍待 tag 后验证。
- 本机已通过 Vue 构建、Vitest、ESLint、TypeScript、Cargo check/test/clippy、Tauri debug no-bundle 构建，以及 Linux x86_64 release 构建。Linux release 已真实生成 DEB、RPM、AppImage 和带 `portable.bkmt` 的便携目录，并通过 staged CLI/桌面启动冒烟；Windows 与 macOS 仍需 tag workflow 证据，macOS 还缺少实机验证。
- 仓库内 Markdown 文件目标和标题锚点由默认测试中的链接检查器验证，当前检查通过。
- 三轮 OCR 收口已覆盖设置初始化与预览回滚、扁平 i18n key 解析、命令面板可访问性、文件生成计划快照与授权复核、模板及报告有界读取、原子文件安装、数据根解析、CLI JSON 错误、阻塞 IPC 隔离和发布资产安全；相关前端、Rust、发布脚本与文档检查均纳入本地门禁。
- 发布 workflow 会先用 tar 保留便携文件权限，再汇总校验和、revision 与渠道元数据；已发布 Release 不允许覆盖，已有 draft 会先清理旧资产再上传。

## Phase 0：解除上游阻断

- 在 BroKnowMySparkAnalyzer 启用 keyring 的 Windows、macOS、Linux 平台 backend。
- 为真实持久化和旧 mock 行为编写回归测试。
- 发布包含修复的新版本，并确认 npm、Rust crate 和 Tauri capability 兼容。

完成条件：三平台后端选择可验证，新进程能够读取先前保存的测试凭据，Toolbox 记录明确依赖版本。

## Phase 1：工作区与质量骨架

- 建立 pnpm workspace、Cargo workspace、Tauri 2/Vue 3 桌面端和 `bkmt` CLI。
- 配置 TypeScript、Rust、lint、format、单元测试和基础 CI。
- 实现统一版本读取、结构化错误和 JSON 输出约定。

完成条件：空壳桌面端与 CLI 在本地构建，CI 在三个操作系统运行基础检查。

## Phase 2：产品壳与基础模块

- 建立工具目录及重复/无效声明校验。
- 从工具目录派生路由、首页、搜索、收藏和最近使用。
- 实现主题 token、键盘导航和国际化加载/回退。
- 建立固定侧栏、顶栏、正式首页、设置抽屉和命令面板组成的统一工作台，让工具路由保持完整产品壳。
- 实现数据根目录解析、便携标记、原子写入和跨进程锁。
- 实现能力声明、授权存储、Rust 检查和系统权限引导。

完成条件：一个虚拟工具可以通过目录完整出现在 GUI/CLI，安装与便携模式测试覆盖路径、并发进程争用和锁失败。

## Phase 3：文件生成器纵向切片

- 定义模板 schema、变量校验和路径安全规则。
- 实现生成计划、冲突检测和逐文件执行结果。
- 完成 GUI 编辑/预览/执行体验。
- 完成 `bkmt file create`、`--dry-run`、`--force` 和 `--json`。

完成条件：GUI/CLI 通过同一接口测试；遍历路径、覆盖冲突、部分失败、编码和跨平台路径均有验证。

## Phase 4：Spark Analyzer 集成

- 集成修复后的 Spark npm 包、Tauri plugin 和 Rust crates。
- 将宿主主题、语言、数据根目录、授权和错误模型接入 Spark adapter/store seam。
- 实现 `bkmt spark` 命令族，保持稳定的人类输出、JSON 和退出码。
- 验证系统 keyring；CLI 首版通过环境变量注入凭据，不实现口令加密便携凭据文件。

完成条件：GUI/CLI 对相同输入产生等价分析结果；凭据跨进程持久化；不复制 Spark 分析实现。

## Phase 5：发布闭环

- 完成三平台构建、GUI 关键流程和安装包启动冒烟。
- 产出安装包与带 `portable.bkmt` 的便携包。
- 实现版本检查和点击下载。
- 建立 GitHub Releases、Scoop、Homebrew Tap、AUR 发布流程。
- 完成中英文 README、用户说明、贡献说明和 changelog。

完成条件：[产品完成条件](./product-scope.md#首版完成条件) 全部有链接到 CI、产物或测试的证据。

## 后续路线

- 基于真实需求评估 SQLite、可安装模板、主题包和更多工具。
- 有签名凭据后启用 Windows 签名与 macOS 签名/公证。
- 评估 WinGet、Homebrew Core、apt/rpm 官方仓库。
- Windows/Linux ARM64 在具备 runner 和真实验证环境后进入支持矩阵。
