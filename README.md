# Bro Know My Toolbox

[English](./README.en.md)

Bro Know My Toolbox 是一个跨平台桌面工具箱，同时提供一等的 `bkmt` 命令行程序。项目以少量内置工具为起点，通过共享核心模块保证桌面端和 CLI 的行为一致。

> 当前状态：首版功能与发布自动化正在收口；正式发布仍由 Spark Analyzer 三平台 keyring CI 与修复版本阻断。

## 首版范围

- 文件生成器：模板、变量、批量生成、执行预览和安全覆盖策略。
- Spark Analyzer：复用独立发布的 Spark Analyzer npm 包和 Rust crates。
- Windows、Linux、macOS 构建与启动冒烟验证。
- 安装模式和显式便携模式。
- 简体中文、英文及用户语言包覆盖。

剪贴板、运行时插件安装、遥测和静默自动更新不属于首版。

## 设计原则

- 工具是编译进应用的内置模块，不是运行时插件。
- 业务能力只实现一次；桌面端和 CLI 是共享核心的 adapter。
- 原生能力经过明确的 Rust command、最小 Tauri capability 和持久化授权检查。
- 默认使用系统数据目录；只有显式便携标记才写入便携包的 `data/`。
- 机器接口使用稳定错误码和字段，翻译只发生在展示层。

## 文档入口

- [领域语言](./CONTEXT.md)
- [产品范围](./docs/product-scope.md)
- [架构](./docs/architecture.md)
- [实施计划](./docs/plan.md)
- [工具开发约定](./docs/tool-development.md)
- [CLI 约定](./docs/cli.md)
- [数据与便携模式](./docs/portable-data.md)
- [权限与安全](./docs/security.md)
- [测试与发布](./docs/testing-release.md)
- [发布操作](./docs/releasing.md)
- [旧仓库迁移清单](./docs/migration.md)
- [风险与阻断项](./docs/risks.md)
- [架构决策记录](./docs/adr/)

## 许可证

本项目使用 [Apache License 2.0](./LICENSE)。
