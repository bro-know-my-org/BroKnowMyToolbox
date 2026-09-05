# 旧仓库迁移清单

旧仓库 `/home/halo/Desktop/gitRepo/BKM/BroKnowMyToolbox` 只作为行为和资产参考。新实现不得批量复制旧目录后再整理。

| 资产                       | 处理方式               | 说明                                  |
| -------------------------- | ---------------------- | ------------------------------------- |
| Tauri 2 + Vue 3 基础选择   | 保留技术选择，重新搭建 | 旧构建配置可用于核对平台细节          |
| 工具路由挂在 RootLayout 下 | 保留行为               | 新路由从工具目录派生                  |
| `ToolPage` / `ToolPanel`   | 仅作视觉参考           | 新 UI 直接使用 Naive UI 与主题 token  |
| 文件创建器                 | 重新实现               | 升级为生成计划、模板和批量写入        |
| 剪贴板                     | 不迁移                 | 旧功能属于测试，不在产品范围          |
| Spark Vue 页面             | 使用发布包             | 升级到兼容的 0.1.x 修复版本           |
| 旧本地 `spark.rs`          | 不迁移                 | 分析逻辑继续归 Spark 独立仓库         |
| i18n 文案                  | 可作为翻译参考         | 新格式使用扁平稳定键和覆盖回退        |
| 图标                       | 视觉评审后选择性复用   | 不默认继承旧品牌表现                  |
| localStorage 设置          | 不迁移                 | 使用新应用 identifier 和新配置 schema |
| release workflow           | 只作平台命令参考       | 新流程必须加入测试、冒烟和渠道发布    |

## 外部 Spark 资产

Spark Analyzer 仓库位于相邻的 `BroKnowMySparkAnalyzer`。GUI 复用 `@bro-know-my/spark-analyzer` 与 Tauri adapter；CLI 复用 `bkmsa-core`、`bkmsa-agent`，不依赖 npm UI，也不启动 `bkmsa` 子进程。

Toolbox 所需的 host-authorizer、API、keyring、样式隔离和插件命名修复已随 Spark `v0.1.2` 独立发布。npm 与 Cargo 精确固定 `0.1.2`，CI/release 不再检出 sibling；发布与本轮验证证据见 [审查与发布验收进度](./review-release-status.md)。不得把 Spark 源码复制或提交进 Toolbox；Spark 未来大重构不在本轮迁移范围。

集成前先完成 [keyring 阻断项](./risks.md#阻断项)。
