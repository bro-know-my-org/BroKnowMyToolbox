# Linux 真实 WebView 接入验收 · 2026-09-05

本记录是本机真实 Tauri WebView 的部分交互证据，不是完整 GUI 验收或正式发布。剩余门禁统一见 [验收进度](../review-release-status.md)。

## 构建与隔离

- Toolbox 产品源码：`617adb7`；本轮另修改 IPC 测试和文档，没有改变 Toolbox 产品运行代码。
- Spark：`f262fc5`，包含样式修复 `1b286b0`；版本仍为未发布的 `0.1.2`。
- 构建：`pnpm tauri build --debug --no-bundle`，Linux x86_64，嵌入实际前端资源，不使用开发服务器。
- 最终 debug 二进制 SHA-256：`493694d4ba34d5b3e734a70599af80b855871d17026631def3e6b9fe57941f77`。
- 驱动：`tauri-driver 2.0.6`，WebKitWebDriver `2.52.6-0ubuntu0.24.04.1`；WebDriver 返回 `wry 0.55.1`。使用实际元素点击、清空和键盘输入，JavaScript 仅用于只读 DOM/颜色断言，不直接调用 Vue 或 Rust 业务函数。
- 数据、XDG 目录和 DBus 会话均隔离于 `/tmp/bkmt-webview-validation.9IRDig`；驱动仅监听 `127.0.0.1:4447/4448`。未授予 Spark 网络或凭据能力，未使用用户 API key。
- 驱动和截图是本机临时验收资料，未加入仓库或 CI；临时目录不保证长期保留。

两次修复后均可恢复移走根 `node_modules`，再运行 `pnpm install --frozen-lockfile --offline`，避免 file 依赖复用旧快照。备份分别在 `/tmp/bkmt-style-refresh.SkKntg/node_modules` 和 `/tmp/bkmt-ipc-refresh.zaZTEs/node_modules`，未删除。最终实际安装版本与 sibling 均为 `0.1.2`，三个文件的两侧 SHA-256 相同：

| 文件             | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| `dist/index.js`  | `8ce433a491324b47806452d0b83aed02a172fc1d50b441ec98edb3df4d14973b` |
| `dist/style.css` | `d3a672619a0c7ba5e48a2a2fac8d6ad4844b39e56e05220444da89e60958659c` |
| `dist/tauri.js`  | `dcadcfd3891e99fa252980805f1ddddde8eb6e9c8de6ffda3219d5f850264935` |

## 复现与修复

1. 原 Spark 全局 CSS 的 `:root`、`h2` 和通用布局类污染宿主。真实文件生成器标题为 `rgb(220,228,234)`、背景为 `rgb(244,245,247)`，对比度仅 `1.17895:1`，断言两次失败。隔离到组件、全屏和导出根的 `.bkmsa-scope` 后，浅色标题为 `rgb(23,26,31)`，同背景对比度 `15.98925:1`；深色为 `rgb(240,242,245)` / `rgb(16,18,22)`，对比度 `16.71615:1`，均超过本次要求的 `4.5:1`。仓库样式回归也先用旧 CSS 验证失败，再用新 CSS 通过。
2. 点击 Spark 的 `Load Text` 原先稳定返回 `Plugin not found`。运行时与前端使用 `bkmsa`，而生成的权限命名空间为 `bkmsa-tauri`。统一为 `bkmsa-tauri`，权限范围不变；Rust 插件名称和 npm adapter 回归均先红后绿。Toolbox IPC 测试改用 `generate_context!()` 的真实权限配置，移除 `__allow_command` 手工放行，也先红后绿。

## 实际操作结果

- 初次英文会话（Spark `ee6e71e`）验证文件生成授权：拒绝后没有写入且显示 `consent_denied`；重新预览并允许后，预览仍不写文件，执行才生成预期 README。
- 最终构建再次通过“填写目标与变量 → 预览 → 执行”，确认预览前后 README 不存在，执行后内容精确为 `# FinalWebViewAcceptance\n`。
- 最终构建勾选覆盖、产生计划后，在磁盘改写目标为 `# EditedAfterPreview\n`；执行返回 `plan_changed`，保留磁盘编辑，并移除旧执行按钮，要求重新预览。这不证明最终检查与替换之间的竞态已解决。
- Spark 凭据提示选择拒绝后，本地粘贴文本并点击 `Load Text` 成功，显示 `Text loaded`、类型 `text`、来源 `pasted text`，无需网络或凭据授权。
- 在设置中切换深色、简体中文并保存，宿主和 Spark 同步变化；销毁 WebDriver 应用会话后启动新进程，两者仍为深色、中文。设置显示的数据位置与隔离目录一致。
- 完成文件生成器与 Spark 页面往返导航。截图保存在临时目录：`file-heading-fixed.png`、`spark-scoped.png`、`spark-text-loaded.png`、`file-dark-zh.png`。

## 验证边界

- Spark CSS 批次 OCR：`600b87c7-1a0e-4d26-80ed-ea9336eb64dc`，2 个选中文件零意见；IPC 批次：`84b31077-394c-4625-b411-575136f770b7`，3 个选中文件零意见。测试与直接相关文档一并人工检查。
- Spark 最终 13 项前端测试、UI 包构建、插件 15 项 Rust 测试及 Clippy 通过；2 项真实 keyring 测试本批默认忽略，此前的隔离执行证据见进度文档。样式批次还通过独立前端构建。
- Toolbox 最终 73 项前端/脚本测试、Rust 工作区全 target 测试、桌面 Clippy、格式与嵌入资源桌面构建通过；lint 在 CSS 刷新后通过。Rust 唯一默认忽略的配置 helper 由父测试显式执行。Toolbox IPC 回归 OCR 会话 `0d0eadd8-8bb6-4774-ae60-533fdf2adacd`，1 个选中文件零意见；文档另经人工核对与链接检查。
- 未执行真实 AI 请求、诊断全屏/图片导出完整流程、全部视觉偏好组合、Windows/macOS 交互或 FAT/exFAT 实测。全屏与导出样式只有样式级回归，不能等同于真实导出验收。
- 未生成本轮正式安装包，未推送、开 PR、合并、创建 tag 或发布；旧 `552030d` 发布产物不改写为本轮证据。
