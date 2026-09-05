# 风险与阻断项

## 阻断项

### Spark keyring backend

Spark Analyzer 已为 Windows Credential Manager、macOS Keychain 和 Linux Secret Service/keyutils 显式启用 backend；跨 Entry、跨进程凭据测试已通过三平台 CI，修复已随 `v0.1.2` 发布，上游发布阻断解除。这不代替 Toolbox 自身的跨平台验收或 macOS 实机验证。

Toolbox 的 npm 与 Cargo 依赖已切为精确版本 `0.1.2`，移除 CI/release 的 sibling checkout。正式依赖切换后的完整门禁及剩余文件覆盖安全问题统一记录在 [审查与发布验收进度](./review-release-status.md)；依赖可下载不等于 Toolbox 已可发布。

## 已知风险

### macOS 缺少真实设备

GitHub runner 只能覆盖自动构建和冒烟，不能证明真实设备交互、Gatekeeper、公证、系统 keychain 或便携 `.app` 布局完全正确。首版发布说明必须如实标注。

### 便携目录可写性

Windows 受保护目录、Linux AppImage 和 macOS `.app` 内容可能不可写。便携 marker 不能成为数据丢失触发器；路径模块必须检测写入并回退，同时向用户展示实际目录。

### 跨进程配置竞争

GUI 与 CLI 共享数据会产生写入竞争。配置模块完成原子写与跨进程锁之前，任何共享持久化功能都不能宣称完成。

### 分发渠道维护成本

Scoop、Homebrew Tap 和 AUR 的元数据、校验和与发布节奏各不相同。发布自动化必须从同一 release manifest 生成渠道输入，避免手工版本漂移。

### 内置工具模型被误读为插件平台

贡献者可能把“开发者可扩展”理解为运行时安装第三方代码。产品文档、工具指南和 ADR 明确限定为编译期内置模块；若未来改变，必须单独设计签名、隔离、权限、兼容和分发模型。
