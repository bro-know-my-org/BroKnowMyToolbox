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

## 后续修复与真实复验

上述失败保留为 Spark `0.1.2` 基线证据。第一次仅给栅格化副本设置 `left: 0` 的修复仍在真实 WebKit 失败：序列化 SVG 同时包含 `inset-inline: -12000px 12260px`。最终 Spark 提交 `ae6bb1db101a65518ea8a70918cdc7fa07de9b22` 将副本设为 `position: static`，同时忽略物理与逻辑 inset；活节点仍在屏幕外，成功或失败后均清理。回归覆盖浅色、深色、逻辑 inset 和失败清理，并通过真实 `toSvg` 检查副本。

- 临时验收 checkout 使用本地修补 tarball，SHA-256 为 `9f7019f9da383a4628449014aa2ffee53045c7e3cd2a9982eae80f95572e32bb`；该包仍标为 `0.1.2`，仅用于修复验证，不是 registry 发布包，未替换 Toolbox 正式依赖。
- 重新构建后的运行中桌面二进制 SHA-256 为 `b690c68f88b96d72bef4d1893644ba701ac6f8d9585110585cdf3202f73f0177`。无 CSS/序列化探针，通过真实导出按钮和原生保存对话框分别导出浅色、深色图片，并人工检查五节诊断正文可见。
- 浅色 `diagnosis-static-light.png`：1840×836、462 种颜色，SHA-256 `c2bc6ea69ced7da771419882f05978be02ced9be4ed972134bbe47c364437f98`。
- 深色 `diagnosis-static-dark.png`：1840×836、491 种颜色，SHA-256 `c968c9087408e0bccf2afdab5caa3bfe7b5e9e0c66c8211ff81f49edd94980f9`。
- 修复最终 OCR 会话 `9397c669-edef-4b9f-9a85-1b65ecadeea9` 对两个选中文件零意见；16 项前端测试及 SDK/独立前端构建通过。补丁版本准备提交 `afffddbcf4ed5180be1cb59b26f9d44a034daebb` 统一版本为 `0.1.3`，尚待发布和 Toolbox 正式依赖切换。

图片及脚本位于临时目录 `/tmp/bkmt-gui-ai.qTWpbF`，不保证长期保留。颜色数仅排除空白，不能代替完整视觉质量或导出标题对比度测量。未验证外部 AI、Windows/macOS GUI、FAT/exFAT 或 macOS 实机。全部剩余门禁见 [审查与发布进度](../review-release-status.md)。
