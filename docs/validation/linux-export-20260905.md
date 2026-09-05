# Linux 诊断与图片导出验收 · 2026-09-05

本轮是此前 [WebView 接入验收](./linux-webview-20260905.md)的补充，不是正式发布或完整 GUI 验收。

## 环境与范围

- 运行中的 debug 二进制从 `/proc/512199/exe` 核对 SHA-256 为 `493694d4ba34d5b3e734a70599af80b855871d17026631def3e6b9fe57941f77`，与此前构建相同；不能用后续 Cargo 测试覆盖过的磁盘路径代替运行中二进制证据。
- Spark UI 对应 `f262fc5`；正式 npm `0.1.2` 的主 JS、CSS、Tauri adapter 与该构建逐文件哈希相同。Toolbox `3cfb453` 已将 npm/Cargo 切为已发布精确版本。
- 数据根及 XDG data/config/cache/runtime 均位于 `/tmp/bkmt-gui-ai.qTWpbF`。driver、WebKit、app 和 Secret Service 四个进程的独立 DBus/XDG 环境一致，凭据库使用私有 control/data 目录，不使用用户正常凭据库。
- `127.0.0.1:18791` 假 OpenAI 服务只接受固定假 key、`bkmt-fixture` 模型和本机请求，返回固定五节 Markdown。不访问外部 AI，也不验证分析质量或真实 provider 兼容性。
- GUI 使用真实元素操作；原生保存对话框核对为该 app PID 后，定向激活并复核活动窗口才输入临时文件路径。诊断时临时 CSS 探针单独标注，不计为修复后的产品验收。

## 已通过

- 在真实 WebView 的 860px viewport 中，文件生成器和 Spark 页面均有 `innerWidth = scrollWidth = 860`，没有整页横向溢出；浅色标题对比度保持 `15.98925:1`。
- 粘贴固定文本报告，允许隔离环境下的凭据/网络能力后，实际点击分析并完成报告清单、overview、evidence_gaps 工具链，最终显示 `Agent complete` 和固定诊断内容。
- 1180×860 viewport 中，全屏诊断根位于 teleport modal，稳定布局为约 1133×791，正文颜色 `rgb(32,40,48)`、背景白色，五节文字可见。

## 未通过：PNG 空白

- 全屏和普通诊断面板均能经过 Rust 导出授权及原生保存对话框写出 PNG，但图片仅有均匀背景，无诊断文字。
- 初次导出、两次全屏重现和普通面板重现均为 1840×836、`identify` 颜色计数为 1。初次及普通面板 PNG 的 SHA-256 同为 `da14ece496dc1e847f8984e04e55b84747fe7ed145878ed2c8e8f0079c820f25`。
- 可重复脚本 `/tmp/bkmt-gui-ai.qTWpbF/check-export.mjs` 实际点击、保存并断言颜色数大于 10；基线失败，不以“文件存在”冒充导出成功。临时脚本和截图不保证长期保留。
- 单变量探针：仅临时覆盖导出节点的 `left: -12000px` 为 `left: 0`，同一真实导出流程得到 462 种颜色；撤除探针后不作为产品修复。原因是离屏节点位置被复制进栅格化画布，文字位于画布外。
- 后续需要仅重置导出副本位置、回归测试及新构建真实复验；当前不关闭图片导出门禁，也不把已发布 Spark `0.1.2` 宣称为完整验收通过。

未验证深色图片导出、外部 AI、Windows/macOS GUI、FAT/exFAT 或 macOS 实机。全部剩余门禁见 [审查与发布进度](../review-release-status.md)。
