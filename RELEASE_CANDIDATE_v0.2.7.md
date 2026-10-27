# RUST CHESS v0.2.7 — 本地开发候选验收报告

日期：2026-10-27（Asia/Hong_Kong）
结论：**GO — 本地开发候选，可交由用户亲自试玩；不是正式发行授权。**

## A. Git 基线与交付

- 起始分支 main，起始 HEAD、main、缓存 origin/main、v0.2.6 解引用均为 `16c3eb9f4b2319f1b7e11f66f3c125fbd78bcc4a`；初始工作树干净。
- 创建 `feat/v0.2.7-offline-puzzles`，严格继承已发布 v0.2.6，未移动 main、历史标签或远端。
- Cargo.toml / Cargo.lock 根包版本 0.2.7；没有新增 crate、依赖升级或锁定依赖图变化。
- 本报告与最终源码一同提交。提交后工作树应为空，相对本地 main / 缓存 origin/main 为 ahead1 / behind0。同提交报告不能包含自己的哈希；最终准确 SHA 见交付消息，亦可运行 `git log -1 --format=%H -- RELEASE_CANDIDATE_v0.2.7.md`。
- 不运行 Push、Tag、PR、Release、远端部署、main 合并、本地 ZIP 或发行解压操作。日志、原始采样响应、Mock 和测试数据仅在忽略的 target/。

## B. 题库来源、数量、筛选与许可

- 官方来源：[Lichess 数据库](https://database.lichess.org/#puzzles)；[官方 Hugging Face 镜像](https://huggingface.co/datasets/Lichess/chess-puzzles)。Lichess Contributors，CC0-1.0；不暗示官方背书。
- 镜像固定仓库提交：`461725966ea46898ec6481372a6326de3281bfbd`，数据卡注明 2026-10-07 更新。保存该提交原始数据卡和标准 CC0 正文，未改版权内容。
- **正式内置 512 道真题，213,265 字节**，没有 AI 生成题或伪造 PuzzleId。每道保留源 PuzzleId、源行、FEN、UCI 序列、评分/偏差/热度/主题和 URL。
- 正式采样为 32 个分散偏移 `i*190000`（i=0..31），每批100行，共3200候选；另有2行 schema 探针。超时请求只重试原偏移，复用成功响应。未下载 Parquet、完整 CSV、zst、比赛库或可执行引擎。
- 筛选 Popularity>=80、RatingDeviation<=100、2–16答案半回合、Rating400–3000。先对指定12主题各选最多16题，再按五难度桶循环补足512，去重后按源行排序；具体稳定排序、请求 URL/哈希见 provenance 和 data/chess-puzzles/README.md。
- 镜像实际使用 GameId，而非 CSV GameUrl：保留其 side/ply 后缀，转换为 `https://lichess.org/` + GameId；主题数组原样保留。

| 主题（可重叠） | 实际数量 | 主题 | 实际数量 |
| --- | ---: | --- | ---: |
| mateIn1 | 77 | mateIn2 | 78 |
| mateIn3 | 26 | fork | 59 |
| pin | 37 | skewer | 16 |
| discoveredAttack | 22 | backRankMate | 17 |
| hangingPiece | 19 | sacrifice | 51 |
| promotion | 18 | endgame | 264 |

难度桶 [400,1000)/[1000,1400)/[1400,1800)/[1800,2200)/[2200,3001) 分别为127/131/97/83/74。这是有意均衡的小样本，不是全球题库分布或用户等级分。

- 题库 SHA-256：`c372fd943e5ad910deee03a25b73b2a58a0ef2f2c6b2a20a41ba1a1d7c27c1f5`。
- 固定数据卡 SHA-256：`080193c64118ba9e01ebe04dc96797d595ec57994fd8bb04629e123a498b8eed`。
- CC0 正文 SHA-256：`a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499`；复用 Lichess 已分发的标准 CC0 原件，题库 dedication 独立见固定数据卡及官方数据库页。
- **版本证明边界**：Viewer 接受 revision 查询，但没有 revision 响应头；不把该参数当作独立快照证明。实际核对提取前/后仓库 HEAD，保存每份响应哈希和最终不可变题库/源行。重取时任何 HEAD 或响应哈希变化都停止，不混用未来版本。
- `tools/fetch_puzzle_sample.py --download` 为显式 opt-in 有界复现工具，只输出 target/。本轮用32份真实已取得响应离线复算，最终字节/hash完全一致，没有重复联网获取。
- `tools/verify_puzzle_data.py` 固定数据/数据卡/CC0 哈希、来源、源行范围及覆盖数；Git 属性禁止这些文件换行转换。LICENSES 中附许可正文及 provenance；程序仍 GPL-3.0-or-later。

## C. 全题库合法性校验

- puzzle_data.rs 使用既有六字段 FEN 校验和 shakmaty 标准棋规，逐手转换 UCI、生成规范 SAN、保留合法局面序列。
- 512个唯一ID、每条完整答案线、对手第一着执行及玩家从第二着开始，全部验证并实际通过状态机走到参考线末端。
- 内存预验证按需首次进行并共享结果，普通启动不解析题库。正式题库 include_str! 嵌入 EXE，运行时无需文件、网络或 Python。
- 内置选择2–16半回合；用户格式2–32且偶数。ID/字段/主题/字符串长度、题数、字节数均有界；mateIn1 参考线必须两半回合且真实将死。
- 四种升变、易位、吃过路兵及其他合法立即将杀使用专用规则/状态测试；专用合成测试明确使用 QAfixture，未混入官方正式题库。

## D. 解题状态机与自动应答

- puzzle_session.rs：无 Session 表示未开始；Waiting / Responding / Solved / Answer / Skipped 明确分离。持有自身 Chess、solver、答案索引、实际已走棋、错误、提示、独立 Instant 和固定结果。
- 正确玩家着法仅推进训练局面，下一 UI tick 执行预设对手应答；没有搜索、sleep 或后台自动落子线程。
- 非法着法保持局面且不记战术错误。合法但不匹配参考线只计错误并允许重试，不宣称客观差着。
- mateIn1 允许另一合法立即将杀，直接由实际局面 is_checkmate 判定。
- Puzzle 的 PointerDown / PointerUp / Promote 在 App 中独立分发，不调用真实 App::apply_move。复用 Renderer 坐标、棋子、主题与既有易位目标转换。
- 棋盘自动朝向解题方，支持点击、拖拽、合法落点、上一步高亮和四种升变选择；不受真实对局默认升后控制。升变面板限制在棋盘区域，避免小窗口侧栏覆盖。

## E. 提示、答案、结果与界面

- 每个玩家回合提示先起始格、再目标格；回合间重置提示阶段，但整题提示使用事实保留。
- 揭示答案进入独立答案游标，支持上一步/下一步及方向键，详情显示完整参考 SAN。重试回到执行对手第一着后的出题局面。
- FirstTry = 本次尝试无提示/无错误；Assisted = 本次提示或错误后完成；Revealed / Skipped 分列。FirstTry 是尝试内的状态，不证明玩家从未见过此题。已定结果不会因答案回放或再次揭示被重写。
- 主题支持混合及12个主要指定主题筛选，其余官方标签完整保留并可显示；难度五档、未做优先、历史错题重练、有限去重轮换，无题时显示空状态。
- 主要操作/反馈36个标签覆盖11语言；主题数据标识保持原文。只有当前题目/有限控件绘制，成绩汇总在进度变化时计算，不每帧扫描全部历史。
- 修复最终审查发现的边界：空训练视图中，导入后自动选题曾清掉新增/重复报告。先以失败测试复现，再将报告赋值移到选题之后；新导入先清旧报告，避免误认上次结果。回归最终通过。

## F. 训练记录及用户 CSV 题包

- 独立 schema1：`%LOCALAPPDATA%\RUST CHESS\puzzle-progress-v1.json` 与 `puzzles-v1\user.json`；尊重 RUST_CHESS_DATA_DIR，测试仅用 target/ 隔离目录。
- 记录ID、尝试次数、首次/辅助/揭示/跳过次数、提示/错误、累计秒数、最近时间/结果。重启重新加载；重试增加尝试数，不增加独立已完成题数。`#` 独立尝试ID，`✓#` 独立完成ID。
- 正确率 = 无提示无错误完成次数 / 全部已记录尝试次数（含跳过、揭示）；没有 Glicko、用户等级分、账户或遥测。训练会话时长来自独立 Instant。
- 进度上限16MiB/50000ID；写入前再次检查旧主文件，损坏/未知 schema 原样保留且报错、停写，不覆盖。每个最终结果只结算一次；不能写入时界面明确提示，不能声称已保存。
- CSV 原生打开适配器复用既有 Windows 实现；UTF-8/BOM/CRLF、RFC4180引号/逗号/双引号转义均处理。必需官方基本列，额外可选列可存在；无 zst 支持。
- 单输入16MiB/20000题；用户总集合20000题/32MiB。单后台线程进行读取、CSV解析、逐题验证、去重和原子写入，提供进度及取消；不创建线程池或引擎进程。
- 所有输入先验证，包括重复ID行；已有ID跳过不替换。非法记录报告ID/序号，整次零归档、在第一处错误停止；不是虚报全部错误已扫描。磁盘失败与验证失败信息分别保留，原始CSV不改。
- 取消在提交检查点之前生效；原子提交已开始/完成时允许成功结果如实返回。退出协作取消并有界等待1500ms，正常路径测试回收；极端OS I/O卡死不保证在期限内 join，明确打印超限而不无限阻塞。
- 删除用户题包文件不会连带删除独立训练历史；未提供自动清理、在线同步或完整数据库装载功能。

## G. 既有系统隔离与发行资源

- ai、game、fen、pgn、annotated_game、game_record、library、opening_app/book/stats、replay、persistence、review_app/cache、uci、settings、preferences 共17个核心模块源码与基线一致。
- 不改 autosave-v1、Library index、Review Cache schema，不写用户原PGN或偏好。训练状态转换用真实局面/历史/棋钟/记录/Opening局面及磁盘字节哨兵断言隔离。
- 页面进入沿用既有策略停止旧分析任务，不启动新引擎、不生成训练应答搜索；已有Review算法/缓存身份不变。未保存Replay编辑经原有统一Intent确认，不能绕过。
- Renderer 新 Screen::Puzzle；Game/Replay/Opening 原输入路径保留。records_ui 仅在 Replay 页面绘制遗留 Replay 面板，防止打开其他页面后残留面板覆盖训练。
- package.ps1 与 verify_package.py 同时加入题库来源门禁及两个许可/来源文件白名单。现有 GitHub Actions 已调用这些脚本及许可证测试，无需新运行时文件；正式数据嵌入 EXE。
- 静态比对发行许可白名单与实际目录一致：282个许可/声明/provenance文件。没有生成ZIP，未声称执行真实ZIP完整性测试。

## H. 自动化与构建证据

| 门禁 | 最终结果 |
| --- | --- |
| cargo fmt --all -- --check | PASS |
| cargo test --locked -j 2 | 195 passed / 0 failed / 0 ignored；22.73秒 |
| 原有Rust测试 | 全部180项保留，新增15个综合场景 |
| cargo clippy --locked -j 2 --all-targets -- -D warnings | PASS，无源码 lint 豁免 |
| cargo build --release --locked -j 2 | PASS，修复后最终构建1分04秒 |
| python -B -m unittest discover -s tools -p test_collect_licenses.py -v | 18/18，原15项保留，新增3项题库门禁 |
| ./package.ps1 -SkipBuild -VerifyOnly | PASS：87个锁定Windows依赖，0 unresolved，无包生成 |
| 题库合法性及来源 | 512/512；32份真实采样响应离线复现SHA一致 |
| 离屏 | 11语言、640×480 / 1280×720、双方朝向、空/解题/提示/答案状态、升变四按钮及棋盘像素不被侧栏覆盖 |
| Mock UCI | 原有生命周期/过期结果/整盘调度等回归通过；Puzzle本身不使用引擎 |

- Git差异检查：自有代码/文档无空白错误；固定上游数据卡第49行原有尾随空格按原始字节保留，SHA严格验证，未改写来源原件。
- 构建告警仅为当前文件系统的增量缓存 hard-link 不可用而改复制；未降低 -D warnings、禁用测试或吞源码错误。
- Release 首次构建20.82秒；收尾发现并复现导入报告缺陷后，必要地重建变更应用以保证交付EXE一致，复用依赖缓存；没有 cargo clean、全局配置修改、缓存清理或反复重建依赖。
- 最终 EXE：`target/release/chess_rust.exe`，12,310,528 字节；SHA-256：`ca3623ef1b31508df5b2fe29a3422515036549d7f6c7d79d64a77e99930a71f1`。它对应最终源码，不使用根目录历史EXE。
- 日志、第一次导入报告失败回归及最终成功结果保留于忽略的 `target/v027-qa/`。正式来源数据只保留512题，候选原始响应不加入Git或发行目录。

## I. 未执行项目及用户试玩清单

- **未执行** Computer Use、截图、GUI自动点击、图形窗口启动或真实人工UI验收。
- **未执行**原生CSV文件对话框实际打开/取消、真实拖拽手感、窗口关闭/Alt+F4人工操作及极端系统I/O阻塞测试；离屏动作/状态断言不能替代这些。
- **未执行**真实Stockfish/NNUE/引擎性能验证，未下载或捆绑任何外部引擎；该功能运行不需要引擎。
- 未做20,000题完整压力/峰值RAM基准或多进程同时写进度保证；限额、失败保留和单工作线程路径有自动化覆盖。
- 请用户试玩：白黑各一题，点击/拖拽及升变，起点/终点提示，合法非参考着法重试，揭示/前后回放，难度/主题/错题过滤，CSV导入/取消/错误报告，退出重开后成绩，以及返回原对局后的状态。
- 已知范围：12个主要主题为独立筛选项，其他标签保留显示；答案只提供官方参考线，不评估其他非将杀着法的棋力。Viewer证明边界见B，不能隐去后宣称完全可从未来最新镜像重新下载旧快照。

## J. 最终结论

**GO：v0.2.7 本地开发候选满足已执行的源码、规则、状态机、持久化、Mock回归、离屏与许可门禁，可交由用户亲自试玩。**

这不代表真实人工GUI或公开发行已经验收。剩余人工项目与Viewer版本证明范围如实保留。完成本地提交后停止，等待用户试玩及单独授权 Luna / GitHub Actions 正式发行；不自动开始下一版。
