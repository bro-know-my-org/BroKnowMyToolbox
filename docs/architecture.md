# 架构

## 总体形状

桌面应用和 `bkmt` 是两个产品入口，共享业务核心，不互相调用。

```text
Vue desktop UI                    bkmt CLI
      |                              |
desktop adapters                  CLI adapters
      |                              |
      +---------- shared cores ------+
                       |
          explicit system interfaces
                       |
       Tauri / filesystem / keyring / network
```

共享核心接受依赖、返回结构化结果，不创建窗口、不翻译文本、不读取隐式全局配置。adapter 负责把界面事件或 CLI 参数转换为核心输入，并把结果翻译成人类输出或稳定 JSON。

## 目标工作区

初始结构以下列职责为准，目录只在确有实现内容时创建：

```text
apps/
  desktop/              Vue 3 + Tauri 2 桌面应用
  cli/                  bkmt 可执行程序
crates/
  toolbox-core/         共享领域类型、配置路径和能力策略
  file-generator/       生成计划与执行行为
packages/
  tool-contract/        工具声明与目录类型
docs/
```

避免为了结构图创建只有转发代码的浅模块。Spark Analyzer 保留在独立仓库，正式发布通过已发布 npm 包和 Rust crates 集成；当前开发阶段的 sibling path/file 依赖只用于验证尚未发布的 host-authorizer 与 keyring 修复，不能作为发布依赖。

## 工具目录

工具目录是工具发现信息的唯一来源。每项工具声明：

- 稳定 ID、翻译键和搜索关键词；
- GUI 路由、路由名与 CLI 命令命名空间；
- 所需能力。

路由、首页卡片和搜索索引由目录派生。目录构建时拒绝空或非法 ID、非工具路由、非法路由名/CLI 命名空间、重复值、未知能力和重复能力。图标、工具级设置入口和生命周期尚未进入首版声明契约。

## Interface 与 seam

- 文件生成器的外部 interface 是“生成计划”和“执行计划”；执行使用 `cap-std` 目录 capability 限制真实文件系统写入，测试在临时真实目录中验证路径和竞态防护。
- Spark 的 seam 由其发布包定义；Toolbox 只编写宿主 adapter。
- Spark 原生 plugin 通过 `HostAuthorizer` seam 请求网络、凭据或导出写入，Toolbox adapter 将其映射到共享能力策略；上游不导入 Toolbox 类型。
- 配置根目录、授权存储和凭据存储各有明确 interface，GUI/CLI 使用相同实现。
- Tauri command 是桌面进程 seam，不是业务逻辑归属地。
- 桌面配置在 Tauri command 边界通过 camelCase DTO 与 TypeScript 交换；`toolbox-core` 的配置模型和磁盘 TOML 保持 snake_case。传输命名与持久化格式不能共用一次隐式 Serde 转换。
- `bkmt` 进程入口只负责解析参数、调用顶层命令调度和呈现统一错误；参数、运行时数据根/能力，以及文件与 Spark 命令分别位于内部模块。命令模块复用共享核心，不形成第二套业务实现。

## 状态

- 跨页面展示状态使用 Pinia。
- 主题、强调色、字号、密度和减少动画由设置 store 监听并同步应用到根节点运行时状态；控件修改与 Naive UI 主题计算必须在同一轮更新中生效。
- 业务状态由工具核心或工具拥有的持久化模块管理。
- 进程内状态不伪装成持久化数据。
- 模块级 `ref` 不承担跨页面单例状态。

## 国际化

核心只产生稳定错误码、参数和机器字段。GUI 从内置扁平 JSON 语言表加载文本，用户语言包可以覆盖部分键，回退顺序为：用户选择语言覆盖 → 对应内置语言 → 内置英文 → 可诊断的缺失键占位。CLI 首版保持稳定英文人类输出和与语言无关的 JSON；CLI 本地化属于后续兼容性设计。
