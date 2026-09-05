# Linux 本地构建与启动验收：552030d

验证日期：2026-09-05。此记录是本地集成证据，不是正式 Release 或三平台验收结论。

## 来源和范围

- Toolbox revision：`552030d04ed474b0846599341ca555ce383bd00d`，构建前后工作区干净。
- 工作区版本：`0.1.0`；目标：`x86_64-unknown-linux-gnu`。
- Tauri CLI 锁定版本：`2.11.4`。
- Spark 仍使用尚未独立发布的 sibling 工作树；不能仅凭 Toolbox revision 重现全部依赖，也不得发布此批产物。
- 产物目录：`release-stage/`（已被 Git 忽略）；本次 installer 数量恰好为 3。

## 已执行的检查

```sh
cargo build --locked --release -p bkmt-cli --target x86_64-unknown-linux-gnu
pnpm tauri build --target x86_64-unknown-linux-gnu
node scripts/stage-release.mjs --target x86_64-unknown-linux-gnu --platform linux --arch x86_64 --version 0.1.0 --output release-stage
node scripts/smoke-release.mjs --root release-stage --platform linux --version 0.1.0 --direct-desktop true
```

以上均退出 0。启动冒烟使用现有 `DISPLAY=:12.0`，桌面应用使用临时数据目录，存活五秒后由脚本终止；没有安装 DEB/RPM，也没有执行包管理器脚本。

冒烟覆盖 staged CLI 的版本、两个工具目录项、文件生成 dry-run、marker 便携数据根，以及 staged 桌面程序启动。它不驱动 WebView 内部操作，也不证明 AppImage 在其他发行版可启动。

同一源码状态下，完整 Rust 工作区 145 项测试通过；默认忽略的 `config_child_writer` 由父测试显式调用。完整工作区 Clippy、格式检查、前端及脚本 73 项测试、lint、类型检查、生产前端构建与文档链接检查通过。工作区版本与候选字符串 `v0.1.0` 的一致性检查通过；没有创建或推送该 tag。

## 产物校验和

以下路径相对于 `release-stage/`，大小单位为字节。

| 路径                                                               |     大小 | SHA-256                                                            |
| ------------------------------------------------------------------ | -------: | ------------------------------------------------------------------ |
| `portable/bkmt`                                                    |  6627752 | `889425fd2aaf8318e96c001aa54a9aa61ddb70c621cc5e8501d0a8a2fbe45e84` |
| `portable/bro-know-my-toolbox`                                     | 17648360 | `cc23dd9cf67d8de7f9b694337252608bed05933e7fda164b7b92daea97d8057c` |
| `installers/linux-x86_64-Bro Know My Toolbox-0.1.0-1.x86_64.rpm`   |  6930763 | `780b8b54dbe119b0c6d7c9f38337f343a85bfac66bef24eb15546cc0e48e899b` |
| `installers/linux-x86_64-Bro Know My Toolbox_0.1.0_amd64.AppImage` | 81611256 | `b1e53b774e9337cbf1014cbc2aebea85eb6742b6669a1d1d170eb524a82ddcdd` |
| `installers/linux-x86_64-Bro Know My Toolbox_0.1.0_amd64.deb`      |  6929900 | `0bff169cf1bf1a5347ef41acfb23c3d0ebe0f1417fa0876f110802de9d3ed3be` |

DEB 内部元数据为 `bro-know-my-toolbox` / `0.1.0` / `amd64`。RPM 已校验 lead/signature/main header，并读取内部 `NAME=bro-know-my-toolbox`、`VERSION=0.1.0`、`RELEASE=1`、`ARCH=x86_64`，不是仅按文件名推断。本机没有 `rpm` 或 `rpmbuild`；实际生成并解析 RPM 反证了基线意见 #17 的必需系统工具判断。

## 未完成的门禁

- Spark 修复独立发布及依赖固定。
- 覆盖操作最终检查与替换之间的竞态处理。
- Windows/macOS 真实 CI、四目标汇总与完整 GUI 关键流程验收。
- PR、合并、tag、draft release、安装包交付和三个渠道发布。

不能用本记录替代[完整发布进度](../review-release-status.md)的完成证据清单。
