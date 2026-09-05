# 测试与发布

## 测试层次与当前证据

- Rust/TypeScript 单元测试覆盖纯计算、解析和校验。
- 接口集成测试穿过共享核心的公开 interface；文件生成使用临时真实文件系统，Spark 授权使用 host-authorizer adapter 测试。
- Vitest 组件测试覆盖固定侧栏工作台、工具路由壳、首页工具发现、全局搜索与命令面板键盘导航、视觉设置修改的即时 CSS 状态同步、手动更新检查/打开下载页、授权、文件生成和 Spark 宿主授权/重试流程；Rust 测试覆盖桌面配置 IPC 的 camelCase 往返、磁盘 TOML 格式隔离，以及 Toolbox authorizer 装入 Spark plugin 后使用真实生成权限配置的 IPC 门禁。[Linux WebView 验收](./validation/linux-webview-20260905.md)另记录本机真实元素交互、标题对比度、文本报告及设置重启持久化证据；临时驱动尚未成为仓库或 CI 的端到端套件。
- 860px 最小窗口宽度已用无头 Chromium 检查侧栏文字、主题 token 和正文前景/背景色；该项目前是人工可重复视觉检查，尚未纳入 CI 浏览器测试。
- CLI 契约测试覆盖人类输出基本行为、JSON schema、stderr 和退出码。
- 发布 workflow 在每个目标完成 staging 后执行 CLI 版本、工具目录、文件生成 dry-run、便携数据根和桌面进程启动冒烟。本机已使用与 workflow 相同的 staging/smoke 脚本验证 Linux x86_64 release 产物；Windows 与 macOS 仍只有流程定义，须由真实 tag workflow 形成对应平台证据。
- Linux 本机桌面冒烟使用当前图形会话直接启动并存活五秒。workflow 的 `xvfb-run` 分支因本机未提供该 wrapper 而未形成额外证据；release runner 会在构建 job 中显式安装 `xvfb` 后执行该分支。当前源码的具体 revision、产物校验和与验证限制记录在 [Linux 本地验收](./validation/linux-release-552030d.md)。

测试不依赖真实用户凭据。keyring 使用独立测试凭据并在测试后清理；便携测试使用临时目录和 marker。

## CI 门禁

`pnpm run test:docs` 使用固定版本的 CommonMark 解析器提取 Markdown 链接、图片及引用定义，用 GitHub 标题 slug 规则检查本地锚点，诊断保留链接起始行号。代码块、跨行代码跨度和转义按语法处理；外部 URL 不做连通性探测，原始 HTML 的链接不在此检查器范围内。

合并前要求 format、lint、类型检查、文档链接、Rust check、单元测试和接口集成测试。发布 workflow 执行质量门禁、四目标构建、staged CLI/桌面启动冒烟和发布资产整理；它尚未驱动真实 WebView 内部的完整 GUI 操作。校验和及发布清单生成器已有脚本测试；Linux x86_64 已有本机真实 release staging/smoke 证据，其余目标仍须由真实 tag run 产生。

没有 macOS 实机。GitHub macOS runner 可以验证编译和自动冒烟，但真实交互、Gatekeeper、签名、公证及升级必须标为未验证，直到获得设备和凭据。

## 产物

- 目标是 Windows、Linux、macOS 均提供 CI 已验证的未签名产物；当前本机已生成并验证 Linux x86_64 的 DEB、RPM、AppImage 和便携目录，但尚无 tag workflow 产物，Windows/macOS 也尚无 release 产物证据。
- CPU 架构只在对应 runner 和冒烟证据存在时列为支持。
- 便携包包含 `portable.bkmt`，更新包不能覆盖 `data/`。
- 所有产物提供校验和，并记录源码 revision、工作区版本和构建目标。

## 渠道与更新

首版渠道为 GitHub Releases、Scoop、Homebrew Tap 和 AUR。应用可以检查版本并让用户点击下载或打开发布页，但不自动替换自身。WinGet、Homebrew Core、apt/rpm 官方仓库和签名/公证属于后续路线。

桌面端和 `bkmt` 使用同一工作区 SemVer、changelog 和发布批次；Spark Analyzer 保持独立版本。

`.github/workflows/release.yml` 在 `v*` tag 上先执行完整质量门禁，再构建 Windows x64、Linux x64、macOS x64 和 macOS arm64。构建结果先进入 draft release；脚本从同一批二进制生成便携包、`SHA256SUMS`、release manifest 及三个渠道文件。维护者核对未签名说明和冒烟证据后才手动发布，详见 [发布操作](./releasing.md)。

当前 Spark npm UI 使用 sibling file 依赖，Rust crates 使用 sibling path，因此 CI 会把 Spark Analyzer checkout 到相邻目录并先构建 UI 包。该接法只服务于修复发布前的集成验证；正式 Toolbox release 必须先把 npm 与 Rust 依赖一起切到已发布的修复版本。
