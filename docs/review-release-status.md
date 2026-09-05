# 重构审查与发布验收进度

目标是完成重构 PR 合并、正式发布和约定验收；本地提交或单批 OCR 清零不等于目标完成。产品验收范围以 [产品范围](./product-scope.md) 为准，发布步骤以 [发布操作](./releasing.md) 为准。

## 当前外部前置条件

2026-09-05 复核：GitHub API 身份验证仍返回 401，需维护者在本机恢复登录。Spark 必需修复已在独立分支 `fix/toolbox-host-integration` 提交为 `e31a366`，版本准备另提交为 `ee6e71e`；尚未推送或发布。公开 npm 包与 crates sparse index 中的 `bkmsa-tauri` 最新均为 `0.1.1`，不包含这些修复；本地已统一补丁版 `0.1.2` 的版本及内部依赖约束，没有创建 tag。维护者已授权自行安排本轮所需处理；只推进 Toolbox 发版必需修复及最小补丁版本，Spark 未来的大重构明确不在本轮范围。不能把本地 sibling 依赖直接当成可发布版本。技术阻断详见 [风险](./risks.md)。

另以禁止交互的 `git push --dry-run origin HEAD:refs/heads/refactor/rebuild` 验证 Git HTTPS 传输：返回无法读取 Username（exit 128），没有真实推送或远程变更；不能绕过 GitHub API 的认证问题直接完成 Git 推送。

## 已完成批次

- 真实 WebView 接入验收：修复 SA 全局样式污染宿主（Spark `1b286b0`）及运行时/前端插件名与生成权限命名空间错配（Spark `f262fc5`）。两批 OCR 均零意见，回归先红后绿；Toolbox IPC 测试改用真实生成 ACL，不再手工放行。最终 debug 构建已实测文件预览/写入、过期覆盖拒绝、Spark 本地文本加载、凭据拒绝、深色/中文同步和重启持久化；标题对比度由 `1.18:1` 恢复到浅色 `15.99:1`、深色 `16.72:1`。[完整证据](./validation/linux-webview-20260905.md)记录 revision、依赖/二进制哈希、测试、隔离环境与未验证范围。SA 大重构仍明确延期；这不是完整 GUI 验收或发布完成，GitHub 认证复查仍为 401。
- Toolbox 本地联动验证：根 Cargo 锁文件中的三个 sibling Spark 包跟随 `ee6e71e` 升为 `0.1.2`，仍是开发期 path/file 依赖。冻结安装曾复用旧 UI 快照；将根 `node_modules` 可恢复移到 `/tmp/bkmt-dependency-refresh.okWXhS/node_modules` 后，干净离线冻结安装重新导入 443 个包。实际安装版本为 `0.1.2`，`package.json`、`dist/index.js`、`style.css`、`tauri.js`、`adapter.d.ts` 均与 sibling SHA-256 相同；主 JS 为 `43c1b6ad9f9f7578ab64ec9d58078f3860af59224159b12c6fdbaba4ebac5091`。ReBuild 的 145 项 Rust 测试和 Clippy 通过，刷新 UI 后的 73 项前端/脚本测试、构建、lint、类型检查通过。默认忽略的 Rust helper 仍由父进程测试显式执行。没有更新安装包或完成真实 WebView 交互验收，原 Linux 产物证据仍对应其记录的旧 revision。
- Spark 依赖警告补查：在 Spark 仓库运行 `cargo tree --locked --target all -i chacha20` 及追加 `--all-features` 的查询均成功退出并报告没有活动路径。因此当前证据是 `chacha20 0.10.1` 存在于锁文件但未在该依赖树激活；不将 registry 撤回警告直接等同于产品使用了该算法，也不声称已查明撤回原因。
- Spark 版本准备 `ee6e71e`：workspace、独立 Tauri、两个 npm manifest、Tauri 配置、内部 crate 依赖约束及两套锁文件统一为 `0.1.2`；历史发布资产示例不改写。OCR 会话 `acb756f5-3495-4cec-8b4d-441fe7fe8ab1` 对 5 个选中文件零意见。锁定 metadata 和版本一致性检查、97 项 Rust 测试、10 项 Vue 测试、两套 Clippy、UI 包构建、npm pack dry-run 和离线冻结安装通过。提交后在干净工作树重新运行 `bkmsa-core` 的 publish dry-run，以及 agent/tauri/cli 的 `cargo package --locked`（内部依赖使用 CI 同款本地 patch），四个包均验证通过；本机镜像配置要求核心包预检额外指定 `--registry crates-io`，没有上传。Cargo 提示既有锁定依赖 `chacha20 0.10.1` 已被 registry 撤回，本批未升级依赖，撤回原因及发布影响仍需核实；通过打包不等于已排除该风险。后续步骤是 Spark PR/跨平台 CI、正式发布并验证 npm/Cargo 包，再固定 Toolbox 依赖与完成剩余验收。
- Spark 宿主接入修补 `e31a366`：Rust 宿主授权、真实平台凭据后端、两套 Cargo 锁文件、受控主题/语言及宿主配置存储已独立提交。OCR 共四轮，修复延迟加载覆盖编辑、保存错配密钥/地址、交叉保存、模型/温度编辑漏载凭据，并补齐各 AI 命令的双能力拒绝测试；最终会话 `bde9c9e9-e69c-4f20-9daf-d6cd5991d76a` 对 10 个选中文件零意见。97 项 Rust 测试、10 项 Vue 组件回归、工作区和独立 Tauri Clippy、格式、UI 包与独立前端构建、npm pack dry-run、离线冻结安装通过。两个默认忽略的真实凭据测试已另在隔离 DBus/XDG 环境显式执行，通过重新打开与新进程持久化验证；不构成 Windows/macOS 实测证据。未重构 SA 原有大组件，新增测试文件分别为 283 行与 178 行。后续版本准备及发布门禁见上方批次，不能仅凭这个提交关闭发布门禁。
- 当前源码 `552030d` 的 Linux 本地集成构建、三个包的 staging 与 CLI/桌面启动冒烟通过；[验收记录](./validation/linux-release-552030d.md)包含确切 revision、范围、产物 SHA-256 和未完成门禁。这些产物含未发布的 Spark 工作树依赖，不可作为正式 Release。
- 基线 `0289b0a`：以 ReBuild 源码接续正式仓库历史，未推送。基线 OCR 会话 `b687718d-7760-416b-940f-088c506ba124` 提出 39 条意见，尚未全部收口。
- CLI 修复 `b1a409d`：统一参数解析的 JSON 错误，保留 help/version 输出；模板目录读取错误不再被忽略；补齐报告大小、符号链接和 FIFO 输入测试。OCR 两轮，最终会话 `21e11040-129c-4004-8449-eb2c5a1aa4df` 零意见；本机 CLI 32 项测试、Clippy、格式及文档检查通过。Windows/macOS 实测仍须 CI 补齐。

## 基线意见跟踪

覆盖版本批次完成了 #35 的版本识别部分：共享 `revision` 模块通过受约束的单个 no-follow 句柄计算内容 SHA-256 与文件身份/状态，固定缓冲区并限制到打开时长度加一字节，读取前后检查元数据。回归先复现旧实现对同尺寸、恢复 mtime 的编辑给出相同版本值，再验证修复后拒绝过期覆盖；补齐同内容替换文件、GUI `plan_changed` 及 GUI/CLI 共享的目录约束测试。首轮 OCR 指出需重试 `Interrupted`，已修复；最终会话 `4e9e74f3-5bb8-4c13-bc59-e69caf79efe3` 对六个选中文件零意见。相关三个 Rust crate 的 115 项测试、Clippy、文档和格式检查通过；Windows 文件生成器及其测试的交叉 Clippy 通过，非实机执行证据。最终检查与替换之间的竞态尚未解决，#35 不标完成。

新建文件安装批次已完成实现：平台差异集中在私有 `install` 模块，Linux/macOS 使用父目录句柄相对的 no-replace rename，Windows 使用禁止替换的 `FileRenameInfo`；不支持该 Unix 系统调用时保留安全硬链接回退。八个并发创建者恰好一个成功、目标目录在预览后出现时不被替换，均有真实文件系统测试。Linux 三个相关 crate 共 110 项测试及 Clippy 通过；OCR 会话 `5dcd0ba2-3e07-471c-b3c6-951ce862c957` 对五个选中文件零意见。Linux `strace` 故障注入分别令 `linkat` 返回 `EOPNOTSUPP`、令 `renameat2` 返回 `ENOSYS`，两项新测试在原生安装和硬链接回退路径均通过；不将此证据等同于 FAT/exFAT 实测。本机没有可用的 FAT/exFAT FUSE 驱动，实际可移动文件系统及 Windows/macOS 执行仍待补验。覆盖操作的版本识别和最终替换竞态（#35）未在此批次解决。

Windows 编译证据补充：现有 `+stable` 同为 Rust 1.96.1 且装有 `x86_64-pc-windows-msvc` 标准库，联网取得缺失缓存后，文件生成器全部测试通过目标平台 check 与 Clippy（`-D warnings`）。CLI 的同目标 check 停在 `ring` 构建依赖：本机没有 MSVC `lib.exe`；不能据此声称 CLI 或 Windows 运行测试通过。没有安装或修改工具链。

模板目录批次已完成：GUI/CLI 共用 `file-generator::templates`，有效模板与逐文件警告分开返回；损坏模板不阻断其他模板且不被修改，显式加载仍严格失败，CLI JSON stdout 保持原数组。统一单句柄、非阻塞、拒绝文件链接和 1 MiB 有界读取。首轮 OCR 指出悬空目录链接被误判为目录不存在，已修复并覆盖直接及嵌套路径；最终会话 `52525ac3-6bcf-4fbd-9477-b1a6167ef7c8` 对 16 个选中文件零意见。三个相关 Rust crate 共 108 项测试与 Clippy、前端及脚本 73 项测试、lint/typecheck/build 通过。Windows 测试尚未执行；交叉编译离线检查因缺少缓存依赖 `winx 0.36.4` 未完成，不计为 Windows 验证。

报告 Windows 边界批次已完成实现：同一打开句柄的元数据若含 reparse 属性则在读取前拒绝；新增有效/悬空文件链接及目录链接回归，要求 Windows runner 具备符号链接权限或 Developer Mode，不静默跳过。OCR 会话 `ac9480f3-b67e-4b0c-a3b5-4e2ec04da78c` 零意见，本机 CLI 38 项测试与 Clippy 通过；当前只有 Linux target，新增 Windows 测试仍须真实 CI 验证。

文档批次后另补跑整个 Rust 工作区：129 项通过；唯一默认忽略项 `config_child_writer` 是跨进程测试辅助入口，由父测试带 `--ignored --exact` 和专用环境变量显式执行，不属于漏跑的产品验收。

文档解析批次已完成：以固定开发依赖 `mdast-util-from-markdown 2.0.3` 和 `github-slugger 2.0.0` 替代手写行级正则，覆盖转义、括号标题、重复/跨行引用定义、跨行代码、Setext 与分隔线、嵌套标题及完整 fragment。10 项文档回归和仓库链接检查通过；`pnpm test` 共 72 项、lint/typecheck/build/format 及离线冻结安装通过。OCR 会话 `2091e1ad-dddc-4d15-a4d4-4c9676dc7691` 对五个选中文件零意见；未改变应用运行时依赖。

模板大小批次已完成：保存不再追加换行，原始 UTF-8 输入可达到完整 1 MiB；边界以下、恰好边界、超限拒绝且保留旧文件均有 GUI adapter 回归，CLI `@boundary` 同样验证读取与超限错误。两轮 OCR 均零意见，最终会话 `15cf4112-44aa-4e1d-b350-2205fff3750d`；相关测试、Clippy、格式和文档检查通过。

RPM 意见已取得实际反证：锁定 Tauri CLI `2.11.4` 在本机没有 `rpmbuild` 的情况下成功构建 RPM，内部 header 的名称、版本和架构已校验，详见上述 Linux 验收记录。不添加不需要的系统工具；该结论不替代正式发布的四平台门禁。

生成路径批次已完成：有序完整路径索引替代全表扫描；父子冲突与重复路径使用一致的大小写键。所有平台拒绝 Windows 保留名、非法字符、尾随点/空格及原始 `.`/空路径段；覆盖仅限普通文件，目录和链接保持冲突。新增共享核心及 GUI/CLI adapter 测试，包括 10,000 文件和 100 KB 深路径。首轮 OCR 指出父路径副本造成二次方内存增长，已改为有序区间查找和借用祖先切片；最终会话 `faf7c652-ea3a-4492-9c13-26a0261ed084` 对四个源码/测试文件零意见。本机三个相关 Rust crate 全部 97 项测试、Clippy、格式和文档检查通过；Windows/macOS 执行证据仍待 CI。

前端构建清理批次已完成：确认 `__BKMT_VERSION__` 无消费者后删除 Vite 注入、ESLint 全局声明及单行类型文件；原生版本检查和产品版本不变。OCR 会话 `88c9b43e-cd06-4311-9217-15b12e3bcf6a` 零意见，生产构建（含 vue-tsc）和 lint 通过。

生成计划界面批次已完成：执行返回 `plan_changed` 时清除旧计划及其请求快照，必须重新预览才能再次执行；测试验证旧列表/按钮消失、重新预览后的 `targetRevision` 随重试发送。OCR 两轮均零意见，最终会话 `efaf263f-7733-4062-aa3b-7959c5219074`；桌面全部 43 项测试、核心 execution 7 项、桌面 file_generator_commands 7 项及 lint/typecheck/格式/文档检查通过，未改变 Rust 写入策略。

工具注册批次已完成：目录构造逐项验证定义并拒绝关键词/能力数组空洞；两个桌面 loader 显式返回默认导出的 Vue 组件。OCR 会话 `783f425e-8178-4b38-b85e-6a935965941d` 对两个源码文件零意见；目录契约 15 项、桌面 42 项、发布 4 项、文档 5 项测试及 lint/typecheck/build 通过。基线将 loader 问题判断为构建失败并不成立，本次修正的是返回值与声明契约的一致性。

诊断批次已完成：GUI 与 CLI JSON 对非 UTF-8 路径统一返回 `data_root_not_unicode`，不改变实际数据位置；数据源使用封闭枚举，GUI 显示结构化诊断错误。Rust 诊断回归 4 项、设置 UI 9 项、lint/typecheck/Clippy/格式与文档检查通过。OCR 会话 `a3c3f47a-8863-4304-ba08-78b016ed14f6` 的唯一意见称缺少 UI 回归测试；已核实 `settings-ui.test.ts` 中 `structured runtime diagnostic failures remain visible in settings` 正是该场景且运行通过，故为误报，无剩余可执行意见。

数据路径批次已完成：显式 DesktopState 构造复用共享路径校验，discovery 与 resolver 对空覆盖目录返回同一错误；优先级测试的候选目录全部来自绝对临时路径。OCR 会话 `b4c5e10f-1d37-476e-be34-f4e3b7227d7c` 零意见；toolbox-core、CLI、桌面所有 Rust target 测试及 Clippy 通过。

更新检查批次已完成：正式仓库为 `bro-know-my-org/BroKnowMyToolbox`，公开 API 返回 200；原连字符 slug 返回 404。已统一 API、Cargo 元数据、opener 白名单和发布测试引用；下载 URL 必须匹配同一仓库和 tag，application identifier 不变。OCR 会话 `ac6d87bc-d197-43f5-9520-0ada9111b63d` 零意见；更新检查 4 项、设置 UI 8 项及发布脚本 4 项测试与桌面 Clippy 通过。

前端状态批次已完成：每次加载语言包从内置文案重建，失效请求不再回写；系统语言变化可实时响应。设置保存期间禁止重复提交、编辑和关闭，失败时保留草稿。OCR 会话 `b4fdae93-73f2-46f0-8174-aa9da7d510c7` 对三个源码文件零意见（默认规则排除了两个测试文件）；回归测试人工检查并执行，完整前端 39 项、目录契约 12 项、发布脚本 4 项和文档 5 项测试，以及 lint/typecheck/build/format 均通过。

语言包读取批次已完成：使用 cap-std/cap-fs-ext 目录句柄拒绝目录与文件链接，非阻塞打开后验证普通文件和大小。7 项本机语言包测试、桌面 Clippy、格式及文档检查通过；OCR 会话 `f530fb8b-9768-488d-91fb-b6765dde6fa5` 零意见。Windows 链接测试已编写，仍待 Windows CI 形成证据。

授权读取批次已完成：单次 no-follow/nonblocking 打开后检查同一文件句柄，Windows 额外拒绝 reparse 属性。核心和 GUI/CLI adapter 异常输入测试通过；OCR 两轮，最终会话 `9e7efb96-40b2-4f1a-986a-7cb5d24d79c2` 零意见。本机相关 Rust 全量测试、Clippy、格式及文档检查通过；新增 Windows 有效/悬空链接测试尚待 Windows CI 执行。

编号对应基线 OCR 会话的原始意见顺序。“待核实”不等于接受 OCR 判断，也不等于已排除风险。

| 编号   | 内容                                                 | 状态                                                       |
| ------ | ---------------------------------------------------- | ---------------------------------------------------------- |
| 1      | DesktopState 显式构造路径校验                        | 已修复，见数据路径批次                                     |
| 2      | 未消费的前端版本常量                                 | 已清理，见前端构建清理批次                                 |
| 3–4    | CLI Unix 依赖作用域、JSON 参数错误                   | 已修复，见 CLI 批次                                        |
| 5      | Spark 报告异常输入测试                               | 已补齐 Windows reparse 用例及检查，实测待 CI               |
| 6      | CLI 模板目录探测错误                                 | 已修复，见 CLI 批次                                        |
| 7      | CLI 空目标目录被接受                                 | 已排除该判断：Clap 原本拒绝；JSON 错误格式已修复并覆盖测试 |
| 8      | 模板 ID 诊断遗漏 Windows 保留名                      | 已修复，见 CLI 批次                                        |
| 9      | 缺少根 Cargo.lock                                    | 误报：基线已跟踪根锁文件                                   |
| 10、16 | CI/release 使用未固定的 Spark checkout               | 待上游修复发布后统一固定依赖                               |
| 11–15  | 文档链接解析的转义、标题、代码跨度和片段边界         | 已修复，见文档解析批次                                     |
| 17     | Linux RPM 构建依赖                                   | 已排除必需 rpmbuild 的判断；实际 RPM 构建及头部校验通过    |
| 18     | 非 UTF-8 数据路径诊断                                | 已修复，见诊断批次                                         |
| 19     | 更新下载 URL 限制到正式仓库                          | 已修复，见更新检查批次                                     |
| 20–21  | 运行时数据源类型、诊断错误展示                       | 已修复，见诊断批次                                         |
| 22–23  | 语言包符号链接越界和 FIFO 阻塞                       | 已修复，见语言包读取批次                                   |
| 24     | 设置保存期间关闭、重新打开和重复提交竞态             | 已修复，见前端状态批次                                     |
| 25–26  | 系统语言变化和语言包覆盖累积                         | 已修复，见前端状态批次                                     |
| 27     | 懒加载组件类型                                       | 已修正返回值契约，见工具注册批次                           |
| 28     | 工具目录稀疏数组校验                                 | 已修复，见工具注册批次                                     |
| 29     | 授权文件检查后打开的竞态                             | 已修复，见授权读取批次                                     |
| 30–31  | 数据根空路径错误、Windows 绝对路径测试夹具           | 已修复，见数据路径批次                                     |
| 32     | 损坏用户模板恢复                                     | 已修复，见模板目录批次                                     |
| 33     | 用户模板 1 MiB 边界                                  | 已修复，见模板大小批次                                     |
| 34     | plan_changed 后保留过期 GUI 预览                     | 已修复，见生成计划界面批次                                 |
| 35     | 覆盖 revision/最终替换竞态                           | 已加强版本识别；最终替换竞态仍未解决                       |
| 36     | 无 hard-link 文件系统的新建文件安装                  | 已实现 no-replace 安装，跨平台及实际文件系统验证待补齐     |
| 37–39  | 生成计划路径冲突复杂度、Windows 文件名、已有目录冲突 | 已修复，见生成路径批次                                     |

## 完成证据清单

- [ ] 所有审查意见有可核实的处理结论，最后一轮代码复审无剩余可执行问题。
- [ ] Spark 修复独立发布，Toolbox npm/Cargo/CI 依赖全部可追溯并固定。
- [ ] 最终工作树完整质量检查通过，GUI 关键流程验收完成。
- [ ] 重构分支推送，PR 门禁通过并合并到 master，记录 PR 和 merge revision。
- [ ] 目标 tag 对应统一版本，四个目标构建和 staged 冒烟通过。
- [ ] 检查 release revision、安装包、便携包、校验和及 manifest，发布 draft。
- [ ] Scoop、Homebrew Tap、AUR 渠道输入生成并完成约定交付，发布说明如实标注未签名及 macOS 实机限制。
