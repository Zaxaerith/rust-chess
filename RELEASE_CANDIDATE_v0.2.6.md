# RUST CHESS v0.2.6 — Opening Explorer 本地候选

日期：2026-10-27（Asia/Hong_Kong）。**本地开发候选 GO，可交给用户亲自试玩；本轮未授权正式发行。**

## A. Git 基线与提交

- 起始分支 main；HEAD、main、本地远程跟踪引用 origin/main、`v0.2.5^{commit}` 均为 `391c499828b554d8ee982f713ebe432fcf5ec370`，初始工作树干净。已阅读 AGENTS.md、Cargo 文件和上一版候选/发行记录。
- 新建 `feat/v0.2.6-opening-explorer`，保留完整 v0.2.5；不移动 main，不改历史标签/报告。origin/main 为本地引用，本轮不进行远端同步/发布。
- 本报告与源码同次提交，准确 SHA 用 `git log -1 --format=%H -- RELEASE_CANDIDATE_v0.2.6.md` 查询，亦记于最终交付答复及忽略的 `target/v026-qa/delivery.json`，避免提交内容自引用其 SHA。
- 只修改本项目；无 Computer Use、截图、GUI 启动/输入注入、引擎下载、cargo clean、全局 Cargo 修改或其他项目清理。

## B. 官方 ECO 数据与 CC0

- 来源：[lichess-org/chess-openings 固定提交](https://github.com/lichess-org/chess-openings/tree/a6189a30dc273ccb21fc2536a9a2fefd5592a67a)，`a6189a30dc273ccb21fc2536a9a2fefd5592a67a`。
- 本轮仅获取 a.tsv、b.tsv、c.tsv、d.tsv、e.tsv、COPYING.txt 六个官方原始文本；总计 **402,112 字节**，未获取大型棋谱库或可执行程序。
- 条目数 A=823、B=781、C=1250、D=644、E=367，合计 **3,865**。字段为 eco/name/pgn，原文英文名称不改写。
- `data/chess-openings/provenance.json` 记录每文件固定 URL、字节数和 SHA-256；原始文件使用 Git `-text` 属性，防止 CRLF 检出转换。
- 五个 TSV 通过 include_str! 编入 EXE；普通启动不解析，首次打开 Explorer 才后台构建并共享只读索引。
- 许可证为 **CC0-1.0**，署名 Lichess Contributors；全文原样复制为 `LICENSES/chess-openings-CC0-1.0.txt`，provenance 副本也在 LICENSES，THIRD_PARTY_NOTICES.md 增加独立数据说明。应用代码继续 GPL-3.0-or-later。
- `tools/verify_opening_data.py` 以独立固定哈希核对六个原始文件、来源、许可及分发副本。package.ps1 在 VerifyOnly/正式打包前执行此门禁；verify_package.py 白名单加入两个分发文件。发布工作流仍调用同一 package.ps1，无额外运行时 TSV 资源要求。

| 原始文件 | SHA-256 |
| --- | --- |
| a.tsv | `3282e4c9155289a29224f9a85fba0decb46efa35c2fa5e39662d2ac48fb0f793` |
| b.tsv | `1d5ed134ebbd87915ead5683416e37583bb3e5ae3582b4d8cb5cb4aa7ef4f623` |
| c.tsv | `45e843c7b7e79a0d04d5bda879c9bd23f5a10974488bd8615e4f4c5d0749986f` |
| d.tsv | `1cf4057a8aece94f27bb3f278ad410ee2844831a642364eeceaafa091dbcceda` |
| e.tsv | `48c6f6645b031a5937c8386839debab0e2e48837251687ba1afb6369677f8546` |
| COPYING.txt | `a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499` |

## C. 局面匹配与转置

- 新增 `opening_book.rs`，复用现有严格 PGN 解析器/shakmaty，逐手验证所有 SAN，不安装生成数据工具或复制棋规。官方两条将死路线没有结果 token，通过原有结果校验确定其合法终局，不放宽 GameRecord 检查。
- 索引使用原 `game::position_key()`：棋子位置、走棋方、易位权、Legal EP 四字段；忽略计数，以实际局面而非字符串前缀识别转置。
- 每条参考保存 ECO、官方名称、合法 Move/UCI 序列、终点键及稳定源序号。数据限额为五文件各 512 KiB、最多 5000 条、每行 8192 字节、参考路线最多 100 半回合；实际数据均通过。
- 整局分类只对标准初始六字段 FEN，取实际历史最后到达的已命名局面。未命中显示 Unknown，不默认 A00，也不信任输入 PGN ECO 标签。非标准 FEN 棋局不冒充有标准初始历史。
- 多名规则：优先完整实际 UCI 历史与参考路线精确相同，再按 a.tsv..e.tsv/原始行顺序稳定选择。UI 显示候选数，详情最多列八项，不宣称歧义唯一。
- 当前固定数据命名终点键实际唯一；歧义规则用合法受控多候选索引测试。实际 Ruy Lopez 不同走子顺序识别同一局面，馆藏转置也合并。

## D. 馆藏统计与分母

- `opening_stats.rs` 逐盘 `Library::load(id)`，通过原 import_annotated 获得合法主线；处理后释放完整树。只保留统计，不同时持有 2000 棵 AnnotatedGame。
- 默认 **40 半回合**：位置 P0..P40，后续着法只来自该前缀。短局/终点或 P40 可能没有下一着；假设变化分支不算真实比赛。
- 每盘每个规范局面只计**第一次到达**，其首次下一着为该盘在该节点的后续样本。节点保存按馆藏 ID 去重的半回合/结果/下一着/非标准标志、结果总计与着法结果总计。
- `1-0` 白胜，`1/2-1/2` 和棋，`0-1` 黑胜，`*` 未完成。W/D/L 百分比分母只含有明确结果的比赛，同时显示总数、未完成数和有结果 n。没有有结果样本时显示 `— (n=0)`。
- 后续着法频率以有下一着的样本总数为分母，可能小于节点棋局数；按出现次数降序、UCI 次序打破平局。
- 非标准 FEN 的实际相同局面可合并统计，但有独立样本计数，游戏开局分类仍未知。整局 ECO 分类可查看完整主线，与统计前缀 40 的范围不同。
- 空馆藏或未见局面显示无馆藏样本。参考路线仍可浏览，计数为 None，不虚构全球样本或 0% 胜率。**这些 W/D/L 是本地历史，不是引擎预测。**

## E. Explorer UI、导航与关联复盘

- 新增 `Screen::Opening`、`opening_app.rs`、`opening_ui.rs`；主菜单/Library 均有入口。使用原 Renderer 棋盘、主题、棋子和字体；局部 Layout 为路径/导航及棋盘外侧坐标预留空间。
- 独立位置序列、着法路径与游标，最多 160 半回合；上一着/下一着/初始、后续选择、ECO/英文名搜索、跳参考路线、关联棋局和滚动列表。左/右/Home 在搜索焦点外导航，搜索框消费输入，避免快捷键误触。
- 棋盘没有 PointerDown/Up 走棋动作，ViewState 清除真实选择、拖拽、AI/提示、PV、箭头、确认与终局覆盖。绝不调用真实 apply_move。
- 关联棋局用原 Library 读取/验证及 open_annotated_replay/Replay 主线定位，返回目标为 Opening，保留 Explorer 路径。不存在匹配时先拒绝，不先替换当前 Replay。
- 继续使用唯一的未保存退出保护，Intent 仅扩展 OpeningPage/OpeningGame；不会绕过注释确认。
- 30 个主要标签覆盖原 11 种语言，英文开局名保持官方原文。仅画可见行；错误有详情入口，缓存写错误也标记详情按钮。

## F. 缓存、失效和后台任务

- 独立 `%LOCALAPPDATA%\RUST CHESS\opening-stats-v1.json`，显式测试跟随既有数据目录；不写 autosave/Review Cache/Library schema。
- schema=1、payload SHA-256、最多 **32 MiB**；身份哈希含算法 1、ECO 来源提交、规范化版本 1、前缀 40、排序后的馆藏 ID/完整内容 SHA。馆藏 2000 盘约束使位置/样本最多 82,000。
- 添加、删除、保存内容使缓存失效；收藏/导入时间等索引显示属性不失效。完整注释内容哈希也会让单改注释保守重建；Game Review 身份不变。
- 首次进入后台加载参考和匹配缓存，无缓存时明确等候“构建统计”。用户构建使用一个线程、唯一完成通道（容量 1）、独立取消标志、进度和快照键，不启动 UCI，不用线程池。
- 所有 PGN 处理、缓存解析、序列化和暂存写入均在后台。UI 只轮询完成的线程，复核最新索引成员后，最终原子替换缓存。应用自己的 Library 变更事件立即使旧数据失效；没有每帧扫描 PGN 或全量哈希。
- 取消/过期/失败不发布新统计或覆盖原缓存；只回收任务自己生成的 pending 文件，不清理用户数据。成功统计但缓存写失败时保留内存结果并明确报错。
- 坏/未知缓存只影响统计加载，原文件保留；明确成功重建后才替换该缓存。缓存也校验节点规范键、合法后续、计数、成员/样本/分类范围。
- 正常退出合作取消并有界等待 join（1 秒），无后台引擎进程。已验证正常本地任务回收；极端 OS 文件 IO 卡住仍可能超时，明确记录，不能承诺任意磁盘故障下强制安全终止 Rust 线程。原子持久化不被绕过。
- 多进程同时写入仍遵守 Library 原单写入实例约定；没有新增数据库事务或持续文件监控。外部变化在进入/构建/完成复核时检查。

## G. 与原有系统隔离

- game/fen/pgn/AnnotatedGame/GameRecord/Library 存储/Replay、records_app、persistence、Review Cache/评分/调度、Pleco/UCI、settings/preferences 均无修改。只在已有 Library 事件加入统计身份失效通知、原入口加入新的界面动作。
- 不修改原 PGN 顺序或 ECO 标签，不改变 Library index/autosave schema，不迁移或清理 Review Cache。没有新 crate 或依赖升级。
- 回归验证真实棋局 record/棋钟不变、autosave 与旧 Review Cache 字节哨兵不变；Opening 不创建 UCI Controller。原 Mock UCI、Game Review、悔棋/超时、PGN/FEN、Advanced Replay 全部测试继续运行。

## H. 门禁、离屏与构建

最终命令均退出码 0，日志在忽略的 `target/v026-qa/`。

| 检查 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过，fmt.txt |
| `cargo test --locked -j 2` | **180/180，0 失败、0 忽略**，9.24 秒，tests.txt |
| `cargo clippy --locked -j 2 --all-targets -- -D warnings` | 通过，无源码 lint 豁免，clippy.txt |
| `cargo build --release --locked -j 2` | **通过，43.21 秒，本轮唯一一次 Release 构建**，build.txt |
| `python -B -m unittest discover -s tools -p test_collect_licenses.py -v` | **15/15**：原 12 项保留 + 3 项 CC0/字节/来源检查，license-tests.txt |
| `./package.ps1 -SkipBuild -VerifyOnly` | 87 个锁定 Windows crate、0 未解决项，六个数据原件及分发副本通过；无包生成，license-gate.txt |
| 静态白名单核对 | verify_package 声明与实际 LICENSES **280 个文件**集合相同；没有创建 ZIP 来测试 |

原 **170** 项 Rust 测试函数全部保留，新增 **10** 个综合回归覆盖：全部参考合法性/身份、转置与歧义、10 盘结果与分母、重复/40 前缀/非标准 FEN、空/未完成样本、缓存/收藏/成员变化、后台取消/回收/缓存写失败、旧快照与坏 PGN、关联复盘/未保存保护、11 语言离屏。

内存离屏验证 **640×480 / 1280×720**、全部语言、后续/参考/关联列表、长参考路径、棋盘及外侧坐标像素不被面板覆盖，Renderer 不产生落子动作。没有窗口、截图文件或图像识别。

开发期发现并修复：测试重复建立同一参考索引导致冷启动等待，改为共享生产只读索引；长路径文字与棋盘坐标边距拥挤，调整局部 Layout 并加像素断言；缓存序列化/写盘移到后台并保留最终快照提交边界。没有靠放宽等待、忽略测试或降低 Clippy 标准通过。

Cargo 在此环境报告增量缓存不能硬链接、退回复制，原日志保留；这是编译缓存环境警告，不是被豁免的源码 lint。未清理缓存或修改全局配置。

Release EXE：`C:\MYAPPLY\Code\Rust\chess\target\release\chess_rust.exe`

- 大小：**11,878,912 字节**。
- SHA-256：`7c19406a0472e8bb9936dad1e625c3272db824e411f374aa2b03a92ea09949cf`。
- Release 后仅整理报告，并将两份自建 provenance JSON 规范为 LF；未改 Rust 或内嵌 TSV/许可证原件，无需重新 Release 构建。无本地 v0.2.6 ZIP 或历史解压目录；正式云端 ZIP 尚未生成/验收。

## I. 未执行的验收

- **真实 Stockfish/NNUE/引擎质量与性能未测**，本功能也不依赖引擎；没有下载/运行真实外部引擎。既有 Mock 协议回归不能冒称真实引擎验收。
- **人工 GUI、原生文件对话框、窗口关闭/Alt+F4 未实测**。仅 Rust 状态/离屏/线程测试，不将它们冒称窗口操作证据。
- 2000 盘最大规模的真实磁盘性能、极端 OS IO 阻塞、多进程写入和未来其他 ECO 数据版本未做实机压力验收；有明确资源边界、错误和过期保护。

## J. 结论与本地交付

**本地候选 GO，可交给用户亲自试玩。** ECO 命名和局面转置、仅馆藏历史统计、独立导航/复盘、缓存与任务控制门禁通过，无剩余自动化失败。

提交后应为干净工作树，分支相对 main/本地 origin/main 领先 1 个提交、落后 0；main 和 v0.2.5 标签保留基线。完整 SHA/交付核对以最终答复及 delivery.json 为准。target 日志、模拟引擎和临时脚本未加入 Git。

**未 Push、未合并 main、未创建 Tag/PR/Release、未远端部署、未创建本地正式 ZIP。** 本地交付后停止，等待用户试玩及另行授权 Luna/GitHub Actions 发行，不自动开始下一版本。
