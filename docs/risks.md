# 风险与阻断项

## 阻断项

### Spark keyring backend

Spark Analyzer 工作树已为 Windows Credential Manager、macOS Keychain 和 Linux Secret Service/keyutils 显式启用 backend，并新增跨 Entry、跨进程凭据测试。该修复仍需通过三平台 CI 并发布新版本；在此之前 Toolbox 只能使用本地路径依赖，不能完成可发布的 Spark 集成。

当前 Toolbox 还直接依赖 sibling Spark Analyzer 工作树中的未提交 host-authorizer、API 和 keyring 改动；其已提交 HEAD `7a04edc56a8ff6422ae8519dbeee367afc82c409` 本身不足以构建本项目。不得把这些 sibling 改动混入 Toolbox 提交。正式发布前必须先在 Spark 仓库独立审查、提交并发布，再把 Toolbox 的 Cargo crates、npm package、CI checkout 和 release workflow 统一固定到同一可追溯 revision/version。

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
