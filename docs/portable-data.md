# 数据与便携模式

## 数据根目录

应用有两种明确配置：

1. 安装配置：使用 Tauri/操作系统推荐的应用数据和配置目录。
2. 便携配置：仅在检测到 `portable.bkmt`、显式 `--portable` 或约定环境变量时，使用便携根目录中的 `data/`。

建议结构：

```text
data/
  config.toml
  locales/
  tools/<tool-id>/
  cache/
  logs/
  exports/
```

Spark Analyzer 的 provider、base URL、模型和 temperature 等非敏感偏好存放在 `tools/spark-analyzer/preferences.json`，使用版本字段、原子替换和有界跨进程锁。嵌入 Toolbox 时不使用 WebView `localStorage` 保存这些设置；API Key 仍只进入系统 keyring 或 CLI 环境变量。

资源目录不等于数据目录。Windows Program Files、Linux AppImage 挂载内容和 macOS `.app/Contents` 都不能假定可写；macOS 便携 `data/` 必须位于 `.app` 外。

若便携目录不可写，程序回退到系统数据目录并清楚展示实际路径，不能静默丢失数据。

当前路径模块在选择便携目录前创建 `data/` 并用临时文件验证可写性；失败时返回 `system` 来源。CLI 的 `diagnostics data-dir` 与桌面设置中的“实际数据目录”均展示最终路径和来源。

若 Unix 路径含无法表示为 UTF-8 的字节，桌面诊断和 CLI JSON 诊断返回 `data_root_not_unicode`，不以替换字符伪造另一条路径；桌面设置会展示诊断错误。路径本身仍可用于文件系统操作；CLI 人类输出仅供展示，可能包含替换字符，不能用于还原原始路径。

## 解析优先级

1. CLI `--data-dir` 或等价显式调用参数。
2. 约定的数据目录环境变量。
3. 便携标记或 `--portable`。
4. 操作系统默认应用目录。

GUI 和 CLI 必须调用同一个路径解析模块。

## 文件格式与一致性

- 普通设置使用带 `schema_version` 的 TOML。
- 语言包和适合树状结构的工具数据使用 JSON。
- 写入采用临时文件、刷新和原子替换。
- GUI/CLI 同时运行时使用跨进程锁；配置与授权锁争用超过 500ms 会返回可诊断错误。配置测试用多个真实子进程验证最终文件保持完整可读。
- 缓存可删除，用户配置、模板和导出不可被清理任务删除。
- 备份与恢复以整个 `data/` 为基本单位，但缓存和日志可以排除。

用户模板以 UTF-8 字节计，最大 1 MiB（包含上限）；保存保留输入原文，不追加换行。GUI 保存、重新读取及预览，与 CLI 读取使用一致的大小上限，超限保存不会替换已有模板。

首版不引入 SQLite。只有出现历史搜索、关系查询或大量增量记录后，才通过 ADR 评估 SQLite 或事务型 KV 存储。

## 凭据

凭据不写入普通 TOML/JSON。桌面 Spark 集成使用启用真实平台 backend 的系统 keyring；`bkmt spark` 首版仅从 `BKMT_SPARK_API_KEY`（兼容 `BKMSA_API_KEY`）读取，不落盘。便携模式不会把系统 keyring 自动变成便携凭据，也不提供口令加密凭据文件；需要完全便携时由启动环境注入密钥。
