# CLI 约定

`bkmt` 是独立可安装的一等产品，与桌面端共享核心，不依赖正在运行的桌面进程。

## 命令空间

```text
bkmt file create --template <FILE> --destination <DIR> [--var NAME=VALUE]...
bkmt file templates [--json]
bkmt spark tools [--json]
bkmt spark inspect <REPORT> [--text] [--json]
bkmt spark tool <REPORT> <TOOL> [--text] [--args JSON | --arg KEY=VALUE] [--json]
bkmt spark analyze <REPORT> [--text] [--base-url URL] [--model MODEL] [--temperature VALUE] [--max-rounds 1..64] [--json]
```

命令默认适合人类阅读；自动化调用使用 `--json`。所有核心操作必须能够非交互执行，交互确认必须有显式参数替代。

## 稳定接口

- JSON 字段使用稳定英文标识，不参与翻译。
- 错误包含稳定 code、可选结构化 details 和展示层消息。
- 参数解析失败（包括空目标目录）在 `--json` 模式下同样输出 JSON 错误（`invalid_arguments`、退出码 2）；`--help` 和 `--version` 保持人类可读输出。
- 成功输出写 stdout，诊断写 stderr。
- `0` 表示完整成功；输入错误、授权拒绝、部分执行失败和内部错误使用不同的文档化退出码。
- 新增 JSON 字段允许向后兼容；删除、改名或改变语义需要主版本决策。

## 文件生成器

`bkmt file create` 读取带版本的 JSON 模板。`--var` 可重复；值按第一个 `=` 分隔。`--dry-run` 只输出生成计划，`--force` 将已有普通文件从 `conflict` 计划改为 `overwrite`，目录和链接等非普通目标仍为冲突；`--json` 选择机器输出。路径遵循共享的[便携命名规则](./security.md)。

`bkmt file templates` 列出共享核心的内置模板，以及数据根目录 `tools/file-generator/templates/` 中文件名与模板 ID 一致的用户模板。`file create --template @basic-readme` 通过 ID 使用内置模板；`@<id>` 也可以解析同一数据根中的用户模板。普通文件路径继续有效。

首版模板 schema：

```json
{
  "schemaVersion": 1,
  "id": "basic-project",
  "title": "Basic project",
  "variables": [{ "name": "name", "required": true, "default": null }],
  "files": [{ "path": "README.md", "content": "# {{name}}\n" }]
}
```

模板路径和内容使用 `{{name}}` 变量。声明为 required 且没有默认值的变量必须显式提供。渲染后的路径必须是数据目标内的相对路径，重复路径、绝对路径、`..` 和通过 symlink 越界的写入会被拒绝。

实际写入前需持久化产品授权：

```text
bkmt consent allow file-generator filesystem:write
bkmt consent deny file-generator filesystem:write
```

dry-run 不需要写入授权。执行 JSON 的 `status` 为 `complete`、`conflict`、`partial_failure` 或 `failed`；逐文件 `outcome` 为 `created`、`overwritten`、`skipped_conflict` 或 `failed`。

## Spark Analyzer

`spark tools` 输出编译进当前 Spark 核心的确定性工具目录；`spark inspect` 解析报告摘要；`spark tool` 执行一个确定性报告工具。工具参数可以作为完整 JSON object 通过 `--args` 传入，也可以重复使用 `--arg KEY=VALUE`。

读取本地报告前必须允许 `spark-analyzer` 使用 `filesystem:read`。AI 分析还必须分别允许 `network:spark` 和 `credentials:ai`：

```text
bkmt consent allow spark-analyzer filesystem:read
bkmt consent allow spark-analyzer network:spark
bkmt consent allow spark-analyzer credentials:ai
```

首版 CLI 从 `BKMT_SPARK_API_KEY` 读取 API Key，并兼容 Spark Analyzer 的 `BKMSA_API_KEY` 环境变量；前者优先。凭据授权不会把环境变量内容复制进普通配置文件。`--base-url`、`--model`、`--temperature` 和 `--max-rounds` 控制本次分析，其中最大轮数必须为 1 到 64。

Spark 命令直接调用 `bkmsa-core` 和 `bkmsa-agent`，不会启动外部 `bkmsa` 进程。JSON 模式下，授权和输入错误写入 stderr，并沿用下表退出码。

## 退出码

| 退出码 | 含义                      |
| -----: | ------------------------- |
|      0 | 完整成功，或 dry-run 成功 |
|      1 | 内部错误                  |
|      2 | 参数、模板或生成输入无效  |
|      3 | 存在未允许覆盖的文件冲突  |
|      4 | 缺少授权或授权已拒绝      |
|      5 | 执行部分失败或完全失败    |

## 配置

CLI 默认与桌面端共享数据根目录。`--data-dir` 优先级最高，之后是项目环境变量，再之后是便携标记和系统默认目录。最终路径必须可通过诊断命令查看，日志不得输出凭据。

## 国际化

首版人类输出使用稳定英文，没有语言参数或配置覆盖。JSON、错误码、参数名和脚本可见行为与语言无关；后续若加入本地化，必须保持默认自动化行为兼容。
