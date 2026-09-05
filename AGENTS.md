# AGENTS.md

本仓库实现 Bro Know My Toolbox。开始编码前先阅读 [CONTEXT.md](./CONTEXT.md)、[产品范围](./docs/product-scope.md) 和 [架构](./docs/architecture.md)。

## 不变量

- 工具是编译期内置模块；工具目录是唯一注册来源。
- 桌面端和 `bkmt` 通过共享核心获得业务行为，adapter 只处理输入输出和运行环境。
- 文件、网络、凭据等能力通过明确 Rust interface；授权检查必须位于 Rust 侧。
- 默认数据进入系统应用数据目录；只有显式 marker、参数或环境变量才启用便携配置。
- 展示层负责翻译；核心返回稳定的结构化结果与错误码。
- 首版范围以 [产品范围](./docs/product-scope.md) 为准；范围变化先更新该文档。

## 按任务读取

- 修改工具定义、注册或导航时，阅读 [工具开发约定](./docs/tool-development.md)。
- 修改 CLI 命令或输出时，阅读 [CLI 约定](./docs/cli.md)。
- 修改路径、配置、缓存、日志或凭据时，阅读 [数据与便携模式](./docs/portable-data.md) 和 [权限与安全](./docs/security.md)。
- 修改 CI、构建矩阵、版本或发布渠道时，阅读 [测试与发布](./docs/testing-release.md)。
- 迁移旧实现或 Spark 依赖时，阅读 [迁移清单](./docs/migration.md) 和 [风险](./docs/risks.md)。
- 改变难以逆转的架构选择前，阅读 `docs/adr/`，并用后续编号记录新的取舍。

## 完成标准

- 行为变化同时覆盖共享核心的接口测试，以及受影响 adapter 的集成测试。
- GUI 和 CLI 对同一行为返回等价结果；机器输出字段与错误码保持稳定。
- 新增能力同时定义声明、授权体验、Rust 强制检查和最小 capability。
- 文档只在其单一事实来源更新，链接校验通过，计划中的完成项具有可验证证据。

## 工作纪律

- 保留用户和其他代理的已有改动。
- 使用小步、可验证的变更；相关检查通过后再进入下一阶段。
- 不配置远程、不提交、不发布，除非用户明确要求。
