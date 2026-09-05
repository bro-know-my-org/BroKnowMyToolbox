# 发布操作

## 前置条件

1. Spark Analyzer 的必需接入修复必须先在其独立仓库提交，在 Windows、Linux、macOS CI 通过，并发布包含修复的新版本；本地提交不等于已发布，当前状态统一见 [审查与发布验收进度](./review-release-status.md)。
2. Toolbox 必须从临时 sibling path/file 依赖切换到该固定版本，并让 Cargo crates、npm package、CI checkout 与 release workflow 指向同一个可追溯 revision/version；锁文件不得依赖本地工作树状态。
3. `Cargo.toml`、根 `package.json`、桌面 package、Tauri config 和 tag 使用同一 SemVer。
4. 本地及 CI 的格式、lint、类型、测试、构建和 clippy 门禁全部通过。

## 流程

1. 将 `[Unreleased]` 内容整理为目标版本，创建并推送 `vX.Y.Z` tag。
2. Release workflow 在四个目标构建未签名安装包和便携包。
3. 各平台先把 staged 目录打包为 tar 以保留 Unix mode；汇总 job 解包后生成 `SHA256SUMS`、带构建 revision 的 `release-manifest.json` 以及 Scoop、Homebrew Tap、AUR 元数据，并创建 draft GitHub Release。
4. 每个构建 job 对 staged CLI 执行版本、工具目录、文件生成 dry-run 和便携数据根冒烟，并要求桌面进程在虚拟/runner 会话中存活启动窗口；维护者再核对 revision、校验和、日志和 macOS 限制。
5. workflow 拒绝覆盖已发布 Release；重跑既有 draft 时会先删除旧资产再上传，避免新旧文件混杂。维护者核对后手动发布 draft，再把生成的三个渠道文件同步到各渠道仓库。

应用内更新只请求 GitHub 的 latest release 接口，并在用户点击后打开 release 页面；它不会替换正在运行的程序。

## 未签名说明

首版产物不签名。Windows SmartScreen、macOS Gatekeeper 和 Linux 发行版策略可能显示警告。macOS runner 结果只能证明构建和自动冒烟，不能代替真实设备、签名或公证验证。
