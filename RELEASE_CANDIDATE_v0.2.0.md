# RUST CHESS v0.2.0 — 本地开发候选与验收报告

日期：2026-10-23（Asia/Hong_Kong）。

**结论：GO，建议接受为 v0.2.0 正式本地 Release Candidate。** Pleco 0.5.0 独立 MIT 许可阻塞已解除；本轮 91 项 Rust 测试、9 项许可证回归、格式检查、严格 Clippy 和 Windows Release 构建全部通过。用户亲自完成人工试玩，反馈“实际试玩没有发现问题”；这是用户人工验证，不是 Codex 自动执行的窗口测试。正式 ZIP 已生成并通过完整性、白名单、逐文件 SHA-256 及 Release EXE 一致性核验。逐项窗口操作证据与极端边界仍有限，保留说明，不编造截图或操作日志。本建议不代表正式发布授权；本轮未执行 Push、Tag、PR、Release 或远端部署。

## A. 基线

- 项目目录：`C:\MYAPPLY\Code\Rust\chess`。
- 起始分支：`fix/v0.1.4-reliability`。
- 原始 HEAD：`77f9df5fb00d7cbeeb2b777ebe065b8b13ce4195`，候选提交存在；初始工作树干净。
- 本地 main 为 `4621edecf3d65d10359a9b06a74cdf6432a19d2b`，不包含 v0.1.4；因此直接从核实的候选建立 `feat/v0.2.0-game-records-replay`，没有基于远端旧版开发，也未移动 main。
- 已阅读 `RELIABILITY_v0.1.4.md`。继承 Duration 棋钟、快照、终局、generation 与 CHA-Solver。上一轮实际发行验收仍有未完成项目，本轮没有把它误记为完整验收通过。
- `src/reliability_tests.rs`、原资源目录、GPL 总许可证和构建 CI 门禁均保留；原有全部 56 项测试仍在完整测试集中。
- 上轮 Final QA 起点：分支 `feat/v0.2.0-game-records-replay`，HEAD `97396fbf61bb2a40b08d70fef386a26fdda58630`，当时工作树干净。许可证修复和测试工具留在工作区未提交。
- 本轮最终收尾从同一 HEAD 开始，工作区已有上一轮 9 个已修改文件及明确列出的未跟踪许可/工具/文档；全部逐项审阅，没有覆盖用户修改，也未使用 `git add -A`。已阅读本报告、人工清单和 v0.1.4 报告；未修改 `src/`、Cargo.toml 或 Cargo.lock，游戏版本仍为 0.2.0。

## B. 架构与修改

### 独立模块

- `game_record.rs`：正式棋谱模型，含初始 FEN、UCI 着法序列、标签、结果、创建时间与省略注释标记；验证后生成局面、Move 和 SAN 序列，不依赖界面文字或尺寸。
- `pgn.rs`：使用 `pgn-reader = 0.26.0`。已检查其实际发布源码依赖 `shakmaty = 0.27`，依赖树仅有一个 `shakmaty 0.27.3`。结构扫描补充解析器的容错行为，避免坏文本静默变成截断主线。
- `fen.rs`：六字段、计数器及标准局面合法性验证；输出保留完整回合编号、半回合计数、易位及吃过路兵状态。
- `persistence.rs`：版本化快照、校验、受限读取、备份与安全替换。不存在 Instant、线程、接收器或窗口序列化。
- `replay.rs`：独立的已验证棋谱、局面序列、游标及滚动位置。
- `records_app.rs`：与实时 App 集成，负责快照、恢复、初始局面重建和文件操作；先完整验证再替换状态。
- `records_ui.rs`：保持现有 minifb/字体/主题，提供主要新操作的 11 语言文字、复盘面板和 FEN 输入；文本光标导航与复盘快捷键分离。
- `native_files.rs`：轻量 Windows 原生打开/保存对话框、Unicode 剪贴板、本地日历日期及安全文件替换，不引入 GUI 框架或外部运行程序。
- `records_tests.rs`：PGN/FEN、存档、恢复、复盘及多语言回归。

### 棋谱与局面

PGN 含 Event/Site/Date/Round/White/Black/Result 七标签和规范 SAN。日期取本地日历；实时对局人类名称默认来自可用 Windows 用户名，AI 名称取决于真实模式，导入棋谱保留原始玩家标签。普通、将军/将死、升变、双翼易位和吃过路兵都经 shakmaty 逐手验证。非标准初始局面输出 SetUp/FEN，黑方起始使用正确的 `42...` 等编号。终局棋盘不得导入或导出为未完成 `*`；超时、和棋和未知判定分别保留结果/原因。

本版本只处理单盘主线。注释、NAG 与变化省略会明确提示，重新导出不冒称已保留这些内容。多盘文件、非标准变种、损坏或非法着法明确拒绝。上限：UTF-8 文件 4 MiB，4096 个主线半回合，变化嵌套 64 层；标签键和值另有限制。支持 BOM、LF/CRLF。非法合法性着法报告半回合位置，格式错误给出具体原因。

FEN 从实时或复盘游标复制。导入是独立新局面：历史、重复记录与棋钟重新初始化，之后悔棋从该 FEN 重建，不回到标准初始局面。它不被伪装成完整对局恢复。

### 自动存档

默认路径：`%LOCALAPPDATA%\RUST CHESS\autosave-v1.json`。`schema_version = 1`，外层保存原始 JSON payload 的 SHA-256；内层保存初始 FEN、完整着法、当前 FEN、双方 Duration 棋钟及全部快照、警告状态、AI 颜色/级别、棋盘行为偏好、终局原因和创建/保存时间。

先校验完整历史、当前局面、结果和快照数量，再恢复；重新创建运行时状态、增加 generation、清空旧线程接收器，必要时重新请求 AI。不会扣除程序关闭期间的时间。

保存时机：正式落子、悔棋、加时、模式变化、终局、返回菜单及正常退出；活动对局另每五秒保存。临时文件先 flush/sync，Windows `MoveFileExW` 安全替换；覆盖有效存档前写 `.json.bak`。坏存档、未知版本、IO 错误和校验不符会显示错误，绝不静默删除或覆盖坏主存档。写入失败留下临时文件供恢复。首次保存错误弹窗，反复失败保留可见失败标记，避免持续阻断操作。

启动有效未完成存档可选择继续、新局、暂不恢复；只在明确操作后恢复。主菜单也可继续当前内存对局。自动保存只维护最近一盘及备份，不是多盘档案馆；永久归档使用 PGN。测试使用 `target/v020-tests` 和 `RUST_CHESS_DATA_DIR` 隔离，未改全局设置或 Cargo 配置。

### Replay 隔离

实时 pos/history/clocks 保留；ViewState 显示独立游标的局面。复盘期间停钟、不轮询 AI/提示接收器、不周期写盘，不改变实时 generation；返回后重新校准时刻，再消费仍对应真实局面的回复。支持首尾/前后按钮、Left/Right/Home/End、滚动和点击着法跳转。外部只读棋谱与当前对局复盘明确区分，返回目的地保留。

### 依赖、版权及打包

新增 pgn-reader（GPL-3.0-or-later）、serde/serde_json、sha2（MIT OR Apache-2.0）及其轻量传递依赖；未升级规则/AI 核心、未新增第二套 shakmaty。程序静态使用这些库，不要求 Python 运行。

`tools/collect_licenses.py` 收集 87 个锁定 Windows 依赖的可用许可、作者、清单、NOTICE/COPYRIGHT、原始 Cargo 声明及校验和，包括 Unicode 的 AND 条件。详情见 `LICENSES/dependencies/INDEX.md` 和 `manifest.json`。failure_derive 采用同版本工作区许可，mucow 以其发布清单的 Apache-2.0 选项附标准文本，并清楚标注来源；不冒充缺失的上游原件。

**Pleco 0.5.0 许可阻塞已解除。** 最初发布包缺少独立许可文件，本轮通过 GitHub contents API 以 base64 读取正确的 `pleco/LICENSE` 路径，核实如下：

| 证据 | 核实结果 |
| --- | --- |
| Cargo.lock 精确版本 | `pleco 0.5.0`，registry crates.io；未升级或删除依赖 |
| 发布包 SHA-256 | `28a8c8ab569c544644c468a63f4fe4b33c0706b1472bebb517fabb75ec0f688e`；本地 Cargo 原始 `.crate` 实测相符 |
| 发布包 `.cargo_vcs_info.json` | 源码提交 `7122ae552507abd3853c5ad6ed37cc36637a0831` |
| 发布源码清单 | [pleco/Cargo.toml](https://github.com/pleco-rs/Pleco/blob/7122ae552507abd3853c5ad6ed37cc36637a0831/pleco/Cargo.toml) 明确为 0.5.0、MIT、Stephen Fleischman；include 列表确实未包含许可文件 |
| 历史原件 | [2017 年 pleco/LICENSE](https://github.com/pleco-rs/Pleco/blob/d8db0d4e94c68334ec51c36dd99001efeda8640c/pleco/LICENSE)，提交 `d8db0d4e94c68334ec51c36dd99001efeda8640c` |
| 同版本实际许可 | [发布源码 pleco/LICENSE](https://github.com/pleco-rs/Pleco/blob/7122ae552507abd3853c5ad6ed37cc36637a0831/pleco/LICENSE)；与历史原件相同 base64 字节、相同 Git blob `6adc426716168a2d3066000613a9d61f023a6eeb` |
| 原件 SHA-256 | `42679ebf22772f1a2901273259e9396a14c7866afd0a0b4564091cefee5a9962` |
| 原样版权行 | `Copyright (c) 2017 Stephen Fleischman`；未重写作者、年份或 MIT 条款 |
| 后续变化 | [2022 年提交 dac34fad](https://github.com/pleco-rs/Pleco/commit/dac34fadbf0eddca1608a65a80e3afb6e8ccddf4) 的 diff 和完整更新文件均已读取；逐字节比较确认只改为 `Copyright (c) 2022 Pleco`，未将它替换到 0.5.0 的原件 |

这不是仅凭年代推测适用性：历史原件就是已发布 0.5.0 源码提交中的同一个库许可文件。因此采用原始 2017 MIT 文本。当前上游根目录 [LICENSE](https://github.com/pleco-rs/Pleco/blob/main/LICENSE) 是 GPL，但 0.5.0 当时的根文件仍是 MIT；当前根 GPL 不能替代这一历史库版本的独立 MIT。RUST CHESS 根 GPL-3.0-or-later 及原版权声明未改。

原始字节固定在 `LICENSES/upstream/pleco-0.5.0-LICENSE-MIT`，分发副本为 `LICENSES/dependencies/pleco-0.5.0/LICENSE-MIT`；两份使用 Git `-text` 保持 LF 原始字节。相邻 `pleco-0.5.0-provenance.json` 与依赖 manifest 记录固定 URL、提交、blob、SHA-256 和适用性。收集脚本离线验证锁定版本/registry/checksum、VCS 提交、原件两种哈希、版权及固定来源；不符即报错，不能靠 QA 开关跳过。9 项测试覆盖成功、字节变化、缺失、版本/许可/源码/包校验不符、来源漂移与 provenance 不一致。修正依赖索引表头缺失说明列，并排序文件收集、固定输出换行；重复执行的 LICENSES 全树哈希完全相同。

`package.ps1` 使用文件白名单，附中英文 README、CHANGELOG、项目 LICENSE、第三方声明、所有 LICENSES、棋子版权表及 SHA-256 清单；不包含配置、测试源码/工具、测试 PGN、用户存档或调试符号，不覆盖已有发行目录/ZIP。正式打包会阻止未决许可，另执行许可回归测试。`-VerifyOnly` 仅运行门禁；本轮正式打包未使用 QA_ONLY 模式或许可例外。新增 `tools/verify_package.py` 在正式打包后强制检查 ZIP CRC、白名单、依赖闭包、许可清单和每个文件的 SHA-256/源码字节；失败报错，不吞错误。release.yml 复用同一脚本，固定 Python 3.12，构建使用 `--locked -j 2` 并核对标签版本。本地与 Actions 的打包及核验入口一致；本轮未运行远端流程。

本报告记录归档自身 SHA-256，因此保留在仓库、排除在 ZIP 外，避免循环校验；人工清单、开发测试工具及日志也不进入 ZIP。许可原始字节由 `-text` 保持，provenance 与生成清单固定 LF。旧版 dist 完整保留；首次核验工具的平铺布局假设已修正，首次产物保留在 target 测试目录，随后正式重跑脚本通过，没有覆盖历史发布文件。

## C. 功能验收

| 功能 | 自动化证据 | 窗口逐项证据范围 |
| --- | --- | --- |
| PGN 导入导出 | 七标签、往返、Unicode/转义、黑方起始、特殊着法、结果一致性、非法/多盘/超限均通过 | 用户未提供该项独立记录；Codex 历史上观察过打开对话框 |
| FEN | 六字段及合法性、权利、计数、往返、独立初始历史、失败不改局面通过 | 用户未提供该项独立记录 |
| 自动存档 | 保存恢复、双方时间、AI 模式、快照、悔棋、终局、损坏/未知版本、IO 失败、旧回复失效通过 | 用户未提供该项独立记录 |
| Replay | 前后/首尾/指定着法、显示同步、停钟、不写盘、不消费回复、回到真实局面通过 | 用户未提供该项独立记录 |
| 小窗口/多语言 | 全部 11 语言的 640×480、1280×720 复盘及对话界面渲染通过 | 用户未提供全部语言/尺寸的人工矩阵 |

**用户人工验收记录（2026-10-23，Asia/Hong_Kong）：** 用户明确表示已由本人完成 RUST CHESS v0.2.0 人工验收，“实际试玩没有发现问题”，并授权继续本地最终收尾。该整体反馈作为用户人工验收接受；用户未提供 A–E 逐项操作、截图、导出文件或特定二进制哈希，不据此补写逐项通过，也不推断用户没有测试这些流程。

Codex 本轮没有重新启动 Computer Use 或发送窗口输入。历史停止操作记录保留，不把它冒称为最终自动窗口测试。复测步骤保留于 [MANUAL_QA_v0.2.0.md](MANUAL_QA_v0.2.0.md)；隔离启动及短棋钟工具作为开发工具保留，不进入正式包。极端竞争窗口、求解器失败、全部语言组合仍主要依赖自动化或没有逐项窗口证据，不夸大覆盖率。

## D. 回归测试与证据

- 本轮重新执行 `cargo test --locked -j 2`：**91 passed；0 failed；0 ignored**，18.76 秒（原 56 项完整保留，v0.2.0 新增 35 项）。
- 本轮 `python -B -m unittest discover -s tools -p test_collect_licenses.py -v`：**9 passed**。两套合计 100 项通过，其中 Rust 测试仍为 91，不能混称为 100 项 Rust 测试。
- `cargo fmt --all -- --check`：通过。
- 本轮 `cargo clippy --locked -j 2 --all-targets -- -D warnings`：通过，0.49 秒，未降低检查标准或新增 lint 豁免。
- 本轮 `cargo build --release --locked -j 2`：通过，0.50 秒，复用现有缓存；本轮无 Rust 源码变化，无需重复完整优化编译。
- Windows 人工验证：用户亲自完成实际试玩并反馈无异常；本轮没有 Codex 自动窗口测试，逐项证据范围见 C。
- 历史 QA-only ZIP：上一轮已验证，详见 `target/v020-qa/package-verification-final.json`。它不包含本轮修正后的许可资料，不是新的正式发行包，禁止用其哈希代替本轮正式 ZIP 哈希。
- `./package.ps1 -SkipBuild -VerifyOnly` 及正式 `./package.ps1 -SkipBuild` 均通过；SkipBuild 复用本轮刚验证的构建。87 个锁定 Windows 依赖，272 个 manifest 文件校验通过，0 个许可未决项，未使用 QA 例外。
- 本轮日志在忽略目录 `target/v020-release-final/`：`tests.txt`、`fmt.txt`、`clippy.txt`、`build.txt`、`license-tests.txt`、`license-gate.txt`、`package-final.txt`、`package-verification.json`。完整依赖闭包与 manifest 集合一致，逐文件 SHA-256 相符；重复收集所有 LICENSES 文件逐字节一致。
- 正式 ZIP 的 286 个文件通过 CRC 与精确白名单；其中 285 个载荷文件均与 checksums.sha256 及源码/构建文件匹配，校验清单本身由 ZIP 整体 SHA-256 覆盖，避免自引用。无 QA_ONLY 标记、配置、测试存档/工具、临时脚本、调试符号，且依赖目录无清单外文件。ZIP 内 EXE 与验证过的 Release EXE 字节完全相同，其余分发文件与最终源码工作区相同。另用四个变造包确认核验会拒绝额外存档、缺失许可、EXE 篡改和校验清单篡改，产物仅在 target/。短钟工具的历史脚本冒烟证据在 `target/v020-final-qa/manual-tools.txt`，不计作窗口验收或新增 Rust 单元测试。
- 测试工具在 tools/，运行产物在忽略的 target/；package 白名单不复制这两个目录，也不复制人工验收清单及测试存档。最终 EXE 在 `target/release/chess_rust.exe`，本轮构建成功且相对候选无 Rust/Cargo 源码修改，哈希与上一候选一致。

| 本轮产物 | 文件名 / 大小 / SHA-256 |
| --- | --- |
| Windows Release EXE | `target/release/chess_rust.exe`；10,406,912 字节；`7f5bf21e5c7d3c0495aa18ca319247bbebb271ef76e30c6746f243c706587bc9` |
| 正式发行 ZIP | `C:\MYAPPLY\Code\Rust\chess\dist\rust-chess-0.2.0.zip`；2,701,065 字节；`5ff7df764a6f9c010ef3620a6f6a9b0b237534cb1d30cb05a6d1dc3c06dde574` |

本轮未发现需要更改游戏 Rust 实现的新增缺陷。实际修复限定于许可原件缺失、来源核实、收集可复现性及许可证索引格式，未扩展 v0.3.0 功能。

仍有限制/未验证：用户窗口试玩无逐项证据；真实磁盘耗尽、长时间运行/崩溃电断、并发多进程写同一存档、远端 CI/发布、最坏 CHA-Solver 耗时及峰值内存未实测。CHA-Solver 同步调用在复杂局面可能延迟窗口响应；预算未决或异常时冻结对局、明确显示判定失败，不猜测胜负。单盘主线 PGN、4 MiB/4096 半回合限制、注释/变化省略提示、仅最近对局自动保存等版本范围保持。基于通过的自动化、许可与归档核验和用户人工验收，目前未识别出阻止本地候选交付的实质性缺陷；这些限制不因此删除。

## E. 兼容性

- v0.1.4 棋钟、终局、悔棋、AI generation 与 CHA-Solver 保留；全部旧测试通过。
- 旧 `settings.cfg` 路径、键名、默认值兼容，存档另存用户数据目录；存档中的偏好只在明确恢复时应用。
- 原有 AI 五档策略与深度、主题、11 语言、棋子及棋盘素材保留；无大型 GUI 框架、无外部引擎、无在线功能。
- 正常棋局原操作保留。新增 FEN 输入和文件对话框暂停实时棋钟；文本输入期间不应用 AI 回复，返回真实对局后才继续。
- 根目录历史 EXE、旧 dist、用户数据、素材和 GPL 许可证未删除或替换。

## F. Git 状态

- 分支：`feat/v0.2.0-game-records-replay`。
- 继承提交：`77f9df5fb00d7cbeeb2b777ebe065b8b13ce4195`；main 未修改。
- 上一候选：`97396fbf61bb2a40b08d70fef386a26fdda58630`。本报告随 `chore: finalize v0.2.0 release candidate and license compliance` 本地收尾提交保存；该提交的精确 SHA 见交付消息与 `dist/rust-chess-0.2.0.verification.json`，报告不内嵌自身提交哈希。
- 通过明确文件清单提交许可证、文档、工作流和开发工具，未加入 target/、dist/ 或用户数据；本轮未合并/移动 main，提交后核对工作树。最终提交和最终 ZIP 的关联记录保存在上述 ZIP 外 JSON，不形成循环引用。
- 通过只读 GitHub API 核实远端 main 为 `4621edecf3d65d10359a9b06a74cdf6432a19d2b`，与本地 main/origin/main 相同。最终开发分支领先 3 个本地提交、落后 0 个：v0.1.4 可靠性、v0.2.0 功能、本轮收尾。全部继承关系由本地 Git 检查确认；无 fetch/push 或远端写入。
- 未执行 Push、未创建或推送 Tag、未创建 PR、未发布 GitHub Release、未远端部署。

## G. 后续计划

本地候选达到 GO，等待用户单独授权 GitHub 正式发行。正式发布前仍需确定是否以及如何合入 main、推送已核实的提交、建立版本 Tag、运行远端 CI/发布工作流并核对远端产物；这些远端/分支操作本轮均未执行，也未获本轮授权。若之后源码或打包输入变化，应重新执行相应门禁并生成新的候选包。

v0.3.0 可扩展外部 UCI 引擎、Stockfish 局面分析、实时评分、最佳着法可视化、失误与妙手标注；本轮均未实现，也未引入这些依赖。完整变化树、多盘档案馆和并发实例存档协调亦需独立设计。

本轮交付的是已通过本地门禁、结合用户人工验收的正式本地候选；没有将总体试玩反馈伪造为逐项窗口日志。达到 GO 后停止，等待正式 GitHub 发行授权。
