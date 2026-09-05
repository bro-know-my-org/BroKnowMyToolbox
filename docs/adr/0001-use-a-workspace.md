---
status: accepted
---

# 使用单仓库工作区

桌面应用、`bkmt`、共享 Rust 核心和必要的 TypeScript 包放在同一 workspace，并使用统一产品版本发布。拆成多个仓库会放大跨入口行为漂移；单体目录又会隐藏共享核心与 adapter 的职责差异。
