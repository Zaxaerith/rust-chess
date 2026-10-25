# RUST CHESS v0.2.3 — Game Review 本地开发候选

验收日期：2026-10-25（Asia/Hong_Kong）。**本地开发候选 GO，可以交给用户亲自试玩；没有执行正式发行。**

## A. Git 基线与分支

- 目录：`C:\MYAPPLY\Code\Rust\chess`。
- 起始分支 `main`，工作树干净；HEAD、本地 main、缓存 origin/main 及现有 `v0.2.2` 标签解引用均为 `2a04694aa69947ac9aa8af695026be8bb1c1f3e7`。
- 从该已发布版本新建 `feat/v0.2.3-game-review`，继承 v0.2.2、v0.2.1、v0.2.0 及 v0.1.4 成果，未强制移动或覆盖已有分支。
- 已阅读 AGENTS.md，保留 CLI / Rust 自动化优先、无 Computer Use、无本地 ZIP 的规则；本轮无须修改这些约定。
- 阅读 chess-tui 的 [PGN 状态](https://github.com/thomas-mauran/chess-tui/blob/main/src/pgn_viewer.rs)、[状态集成测试](https://github.com/thomas-mauran/chess-tui/blob/main/tests/pgn_viewer_integration.rs)、[测试数据隔离](https://github.com/thomas-mauran/chess-tui/blob/main/tests/common/mod.rs)、[曲线坐标组织](https://github.com/thomas-mauran/chess-tui/blob/main/src/ui/menu/rating_chart.rs)、[事件轮询](https://github.com/thomas-mauran/chess-tui/blob/main/src/app/run.rs)。只借鉴组织思路，未复制代码，未引入 Ratatui/ruci 或新增第三方源码。

## B. GameReviewJob 架构、调度、搜索次数与控制

新增模块：

| 文件 | 职责 |
| --- | --- |
| `src/game_review.rs` | 固定 ValidatedRecord 快照、Job/Point/Options、状态和最终报告消费 |
| `src/review_app.rs` | 与 App/现有 Controller 的互斥、轮询调度及检查点 |
| `src/review_scoring.rs` | 纯函数优势指标、损失、分类和证据限制 |
| `src/review_chart.rs` | 曲线坐标、缺口、标记与点击命中 |
| `src/review_ui.rs` | Replay 整盘复盘页签、进度、预算和 11 语言文字 |
| `src/review_cache.rs` | 小型版本化、内容标识、校验和原子缓存 |
| `src/review_tests.rs` | 复用现有 App/Mock 的无窗口回归 |

Job 状态包括 Idle、Running、Paused、Completed、Cancelled、Failed；未开始时 UI 显示 Idle。Running/Paused 独占现有 UCI Controller。单点搜索、引擎参数变更不能覆盖整盘任务；关闭引擎会先取消任务，完成/取消/失败后恢复原 MultiPV 偏好和单点可用性。

默认每局面 500 ms、MultiPV 1、最多前 160 半回合；线程/Hash 沿用当前启用引擎配置，默认 2/64 MiB。UI 时间可选 100/500/1000/2000 ms，上限范围可选 40/80/160/320/512 半回合。默认最多 161 局面、约 80.5 秒搜索预算，协议/调度/磁盘开销另计。开始前显示局面数量、范围和估算预算；长棋谱显示“仅指定范围”和 `0..limit / N`，Completed 只指该范围处理结束，不指全棋谱或每手均可分类。

对 P0..PN 的每个历史节点，先处理普通规则终局，否则发送完整初始 FEN + 正确历史前缀的 `position` 及有界 `go movetime`。每次只有一个 pending 请求，绑定共享唯一 ID、索引和六字段 FEN。只有匹配请求的最终 Complete 才保存 Point，之后才调度下一局面；Searching、旧 ID、重复报告都不能计入完成。普通 4/10/20 半回合分别验证了 5/11/21 次搜索，同一个引擎进程、没有重复提交已完成节点。

重复棋盘但不同历史的节点保留各自历史前缀，以免丢失重复规则上下文；这里避免的是把每手前后局面重复搜索，不把历史不同的节点强行按棋盘合并。将死、逼和、子力不足、五次重复和 75 回合等复用现有 GameResult；例如 4 半回合 Fool's Mate 处理 5 个节点，只发 4 次搜索，最后保存将死原因，不伪造 cp。PGN 中的认输、超时等棋赛结果与“棋盘规则终局”保持区别，不用引擎评分改写原比赛结果。

暂停清空未完成 pending，向同一 Controller 发 stop，由既有 `stop → bestmove → isready` 边界收尾，已完成 Point 不变。继续从首个缺失节点以新 ID 重试。取消保留部分结果但释放引擎；新棋谱、退出 Replay、关闭引擎也会取消。Replay 游标跳转只清单点显示，不停止或改变整盘任务目标。GUI 不等待引擎消息，未启动第二个引擎客户端或进程。

搜索异常退出、非法 bestmove/PV、缺少最终评分等显式 Failed，保留此前成果，未完成节点不计为完成；没有吞错、替代着法或假完成。

## C. 着法质量模型

所有评分先转换为白方视角，并反转黑方的上下界。Point 记录 FEN、白方评分/边界、合法标准 UCI/SAN 最佳着法、深度、请求到消费的耗时、引擎报告耗时及可选终局原因。没有序列化 Instant、线程或窗口。

内部优势指标为：

`A(cp) = 2 * atan(clamp(cp, -4000, 4000) / 400) / pi`

白棋损失 `max(0, A(before) - A(after))`，黑棋为 `max(0, A(after) - A(before))`。这是单调有界指标，不是经校准的真实胜率；饱和区不直接用巨大 cp 差制造大量严重失误。

| 分类 | 条件 |
| --- | --- |
| Best | 实战与合法引擎建议相同，且损失 ≤0.05，证据门槛通过 |
| Good | 与建议不同，损失 ≤0.05 |
| Inaccuracy | 0.05 < 损失 ≤0.15 |
| Mistake | 0.15 < 损失 ≤0.35 |
| Blunder | 损失 >0.35 |
| Unrated | 证据不足、不完整或不可比较 |

证据门槛：相邻两个已完成节点均有精确 cp，深度均 ≥10、深度差 ≤6。Mate/被将杀/丢失将杀机会/被迫将杀、规则终局、上下界、缺少评分/节点/深度、低深度、深度差过大均保守 Unrated；不把 mate 距离当 cp。匹配建议但相邻评分损失矛盾时也为 Unrated。非有限的分类输入不会变成 Blunder。

MoveReview 保存半回合序号、执棋方、实际 SAN/UCI、前后评分、推荐着法、前后深度、损失、质量、是否具备分类证据、匹配事实与原因。UI 保留 `=` 匹配事实、显示深度/损失和低深度等提示；Best 是预算受限的启发式标签，不宣称绝对最优。没有 Brilliant/Great/Book 等标签。

## D. 曲线与 Replay 交互

- 新增整盘复盘页签，保留原记录页和 UCI/PV 页签；分析期间仍可导航 Replay。
- X 映射完整 0..N 半回合，Y 为有界白方优势，正分在零线上方、负分在下方，零线与曲线颜色区分。
- 精确评分绘点，只连接相邻已知节点；未知、失败、未分析和非精确评分保持缺口。Mate 用 M，普通规则终局采用结果标记而非 cp；Mistake/Blunder 用 `?`/`??` 标记。
- 曲线竖线直接读取 Replay 游标，点击接近合法已分析点才发既有 Jump 动作，不点击空白来伪造评分。较长棋谱压缩到有限横坐标，密集重合点可能需键盘/原棋谱列表精确定位。
- 两种尺寸均采用独立 Review 侧栏，不与 PV 固定内容叠放。Review 中原评分条/箭头读取游标对应的已保存 Point，不显示正在分析的另一局面。切回 UCI 保留原单点功能，运行/暂停中的整盘任务仍独占搜索。
- 11 种语言的主要标签、状态和质量名非空。没有修改主题、字体、棋子或棋盘资源。

## E. 缓存格式、隔离、失效和恢复

- 位置：`%LOCALAPPDATA%\RUST CHESS\review-cache-v1\<key>.review.json`，或现有显式 QA 数据目录下同名子目录。与自动存档及 PGN 分离，不改变 GameRecord/Snapshot schema。
- 外层包含 payload 字符串和 SHA-256；内层 schema=1、algorithm=1、棋谱/缓存标识、引擎名称、预算/范围、任务状态、可选 Point 数组。质量由同版评分函数重建，不永久写入棋谱。
- 标识覆盖六字段初始 FEN、完整 UCI 历史、比赛结果、引擎 EXE SHA-256/名称/作者/路径、线程/Hash、实际请求的 MultiPV 1、搜索预算、范围和算法版本。只处理当前棋谱和设置，没有全盘扫描或数据库。
- EXE 指纹在原 UCI 后台线程计算，限 256 MiB，不在 UI 线程扫描大引擎文件。
- 加载检查大小、校验和、schema/算法/身份、数量、逐节点 FEN、合法最佳着法/SAN 和终局原因；完成标记不得掩盖缺失节点。完整缓存再次开始不发 go，部分缓存只处理缺失节点。
- 每完成 8 个局面批量保存，并在暂停/取消/完成/失败/安全退出补写，复用 atomic_write 临时文件、flush 和安全替换，不逐帧或每节点刷新。检查点是小型同步写，慢磁盘可能短暂停顿。
- 单文件最多 1 MiB，目录最多 32 个缓存；数量满或不可写时提示缓存错误，但内存分析继续。没有自动淘汰、清理历史或覆盖坏缓存。损坏/不兼容文件保留，任务可重新在内存分析，须用户手动移动相应坏缓存后才恢复该缓存写入。
- 清理方式：关闭应用后，用户自行移动/删除所需 `.review.json`；不删除 autosave 或 PGN。改变预算/范围形成独立缓存，不跨范围合并旧成果。

## F. 自动化验收

最终门禁均退出码 0；日志只保存在忽略的 `target/v023-qa/`。

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过，`fmt.txt` |
| `cargo test --locked -j 2` | **133 通过，0 失败，0 忽略**；7.74 秒；`tests.txt` |
| `cargo clippy --locked -j 2 --all-targets -- -D warnings` | 通过，无 lint 豁免；`clippy.txt` |
| `cargo build --release --locked -j 2` | 通过，13.18 秒；本轮唯一一次 Release 构建；`build.txt` |
| `python -B -m unittest discover -s tools -p test_collect_licenses.py -v` | **12/12 通过**；`license-tests.txt` |
| `./package.ps1 -SkipBuild -VerifyOnly` | **87 个锁定 Windows 依赖、0 未解决许可项、不生成包**；`license-gate.txt` |
| `git diff --check` | 通过 |

原有 122 项 Rust 测试全部保留，本轮新增 11 项参数化场景测试，复用原有 App、fixture/ready/wait 和 Mock 引擎，不建第二套 CLI 游戏或棋规。

- 正常 4/10/20 半回合 N+1 请求、单进程/相同 PID、非标准黑方起始、规则终局及有限范围。
- 白黑损失方向、所有阈值边界、胜败饱和区、Best 匹配、Mate/边界/缺失/低深度/差距/矛盾证据、未完成或重复报告。
- 暂停→继续→完成、取消→缓存重开、Paused 时单点/预算/参数互斥、批量结束恢复同一进程的单点分析。
- 搜索中崩溃、缺少最终评分、非法 bestmove 的明确失败；旧请求不能进入新棋谱，游标跳转不重定向任务，关闭引擎取消任务。
- 跨 App/子进程重启恢复部分缓存、完成缓存零搜索、引擎/参数/棋谱键变化、schema/算法不兼容、坏缓存保留且可继续内存分析、数量/大小/IO 错误。
- 原自动存档字节哨兵、实时 FEN/历史/棋钟不变；正常任务只有一次 uci 握手，关闭后 worker 完成且 PID 清除。原有 Windows 进程句柄回收、stderr 洪流、stop 边界与超时测试继续通过，未另建进程回收机制。
- 离屏曲线正负/Mate/缺口、0..N 坐标、有效点点击跳转、Replay 游标、800 半回合长棋谱映射；11 语言×2 尺寸×5 状态的内存渲染，侧栏不改棋盘像素。没有生成图片或使用截图识别。

开发期先暴露了测试驱动时序不足：需确认 Mock 真正进入目标搜索再暂停，且等待 Completed 必须匹配新请求 ID，不能接受 latest-value 邮箱中上一请求的完成报告。已修正为受控条件断言，没有延长引擎期限、忽略失败或降低门禁。开发期缓存硬链接回退复制警告保留在日志中，未清理缓存或修改全局 Cargo 设置。

本地 EXE：`C:\MYAPPLY\Code\Rust\chess\target\release\chess_rust.exe`，**10,838,528 字节**，SHA-256：`d47b94bbe3c9756c2b8f7c61413548f9576ae8aa43d8d79ec2fbb6f35cef8eb6`。

没有新增 crate 或依赖升级，Cargo.lock 仅根版本更新。GPL-3.0-or-later、既有第三方声明、Pleco MIT provenance、LICENSES 原始字节属性和 12 项许可证测试均保留。未改 Pleco、CHA-Solver、棋钟/悔棋核心、PGN/FEN、settings/preferences 或自动存档实现。

## G. 真实 Stockfish 与人工 GUI 边界

- **真实 Stockfish：未执行**。用户没有提供本地 EXE；没有搜索下载、安装、运行或捆绑其他来源的引擎。
- **v0.2.3 GUI 人工验收：未执行**。本轮无 Computer Use、截图、鼠标模拟、键盘注入或自动窗口操作；所有渲染为内存离屏。
- Mock 证明协议、合法性、调度、互斥、状态和生命周期，不证明真实棋力。其深度/评分由受控脚本生成，不能称为 Stockfish 搜索质量或硬件性能验收。
- 真实引擎的 NNUE、性能/资源占用、评分稳定性和 UI 玩家体验仍需用户后续验证；v0.2.1 历史窗口观察也不作为本版验收。

## H. 工作树、提交与交付状态

- 开发分支 `feat/v0.2.3-game-review`。本报告与源码同次本地提交；最终完整 SHA 可由 `git log -1 --format=%H -- RELEASE_CANDIDATE_v0.2.3.md` 取得，提交后准确值写入 `target/v023-qa/delivery.json` 并在交付答复列出，避免提交内容自引用其 SHA。
- 提交后检查工作树干净、相对 main/缓存 origin/main 为 1 ahead/0 behind，main 和 `v0.2.2` 保持基线。仅显式暂存本轮审查文件，不使用 `git add -A`；target/Mock/用户配置及历史产物不入 Git。
- **未合并 main，未 Push、未创建 Tag/PR/GitHub Release 或远端部署。没有本地 v0.2.3 ZIP 或对应解压目录。**
- 既有 Actions/白名单打包流程不需新增资源；VerifyOnly 是许可证门禁，不等于云端 ZIP 已构建或验收。正式包仍由用户单独授权后的 Luna / GitHub Actions 生成验证。

## I. 结论及遗留限制

**本地开发候选 GO。** 纯逻辑、Mock 协议/调度、缓存和离屏门禁均通过，可交给用户运行新 EXE、自行选择可信 UCI 引擎试玩。

限制：分类是有预算的启发式；Mate/非精确/弱证据保守未评价；长棋谱只分析显示的有限前缀；密集曲线点需原导航辅助；当前重开缓存需先启用同一引擎校验身份；外部 NNUE/配置文件不做指纹，变更后须用户清理受影响缓存；慢磁盘可能有短暂检查点停顿；不做变例编辑、数据库或实时棋力统计。

没有剩余自动化门禁失败。真实引擎和人工 UI 仍明确未验证，不伪装为已通过。完成本地提交后停止，等待用户试玩及后续正式发行授权，不自动开发下一版本。
