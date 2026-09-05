# 工具开发约定

## 工具的形状

工具是围绕一个用户目标形成的完整内置模块。新增工具必须同时说明 GUI 入口、CLI 入口、能力需求、持久化需求和可观察结果；只有页面或菜单项不构成工具。

建议声明包含稳定 ID、翻译键、搜索关键词、图标、GUI loader、CLI namespace、能力和设置入口。具体 TypeScript 类型在实现阶段由最小可用 interface 驱动，不预先固化 manifest 格式。

## 注册流程

1. 在工具目录加入唯一声明。
2. 由目录派生路由、发现页面、搜索索引和 CLI 命令。
3. 为声明校验、核心 interface 和两个 adapter 添加测试。
4. 若新增系统能力，同时更新授权体验、Rust 检查、安全文档和 capability。
5. 若改变首版范围，先更新 [产品范围](./product-scope.md)。

## 设计规则

- 将复杂行为藏在小 interface 后面；调用者不应了解文件布局、Tauri command 名或 Spark 内部状态。
- 只有存在真实生产与测试 adapter 时才建立 seam。
- 核心接受依赖并返回结果；展示、翻译和进程退出属于 adapter。
- 测试穿过与调用者相同的 interface，断言可观察结果。
- 工具不能绕过数据根目录、授权检查或结构化错误约定。

## 首版工具 ID

- `file-generator`
- `spark-analyzer`

这些 ID 一旦随用户配置发布即视为稳定标识，显示名称通过语言键变化。
