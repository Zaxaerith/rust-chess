# RUST CHESS v0.2.5 — PGN Library 本地开发候选

日期：2026-10-26，Asia/Hong_Kong。**本地候选 GO，可交给用户亲自试玩；本报告不表示已授权正式发行。**

## A. Git 基线及最终提交

- 起始分支 `main`，HEAD、本地 main、远程跟踪引用 origin/main、`v0.2.4^{commit}` 均为 `84f2e9e099b89bc7eacb9e32e1d3a88b7f129bad`。初始工作树干净，正确继承完整 v0.2.4。
- 开发分支 `feat/v0.2.5-pgn-library`，不移动/合并 main。origin/main 是本地已存在的远程跟踪引用，本轮没有执行远端同步或写入。
- 显式提交本轮源码、测试、版本和文档。本报告同次提交；完整候选 SHA 可用 `git log -1 --format=%H -- RELEASE_CANDIDATE_v0.2.5.md` 取得，交付答复及忽略的 `target/v025-qa/delivery.json` 记录提交后的准确 HEAD/工作树状态，避免提交正文自引用其 SHA。
- 遵守 AGENTS.md：无 Computer Use、截图、GUI 输入/启动；无 Stockfish 下载或执行；无 cargo clean、全局 Cargo 修改或共享缓存清理。

## B. 多盘 PGN 解析

- `src/pgn.rs` 增加回调式 `import_collection`，避免返回并长期持有 200 棵棋盘树。复用原词法校验层 `prepare_part`，根据完整 token、注释、RAV 深度和顶层终止标记确定词法封装；**不按 `[Event` 字符串切割**，支持无标签棋局及包含类似标签的注释。
- 有界规范化文本交给**同一个 pgn-reader 0.26.0 BufferedReader** 连续 `read_game`，每盘新建原 TreeImporter。所有分支 SAN/FEN/结果仍由原 shakmaty/AnnotatedGame 验证，节点模型未改。
- 先做有界词法封装，再逐盘构建并完整验证一棵树、提交归档、释放该树。界面只保留索引元数据，不保存整个集合的 AnnotatedGame。
- 文件及规范化集合各最多 32 MiB、一次最多 200 盘；单盘仍最多 4 MiB、每路径 4096 半回合、64 层嵌套、8192 个着法节点及 v0.2.4 注释/NAG/模型预算限制。规范化输入和原输入各有独立边界，这不等于整个进程 RSS 上限。
- 嵌套 RAV、文字注释、NAG、Unicode、SetUp/FEN、不同标签和结果保留。结果标记后、下一盘开始前的注释按上一盘尾注保存；不承诺原始空格/换行的字节往返。
- 第一个非法棋局或归档错误停止，准确报告盘号、原因、先前成功/重复数量，后续明确未处理。UTF-8 损坏属于文件级错误，在任何归档前拒绝；超过 200 盘时保留已提交前 200 盘并报告第 201 盘限额错误。
- 原“打开棋谱”仍要求单盘，完整复杂单盘入口不变；多盘从 Library 导入，不偷偷只打开第一盘。

## C. 文件组织与轻量索引

- `src/library.rs` 负责存储和查询，`library_app.rs` 管理页面/导入线程，`library_ui.rs` 绘制独立页面；没有新 GUI 框架、数据库或 crate。
- 默认 `%LOCALAPPDATA%\RUST CHESS\library-v1\`；显式测试使用已有 `RUST_CHESS_DATA_DIR/library-v1`（应用已解析的 save_path 数据目录也沿用）。测试额外注入隔离 root，所有回归数据均在忽略的 target 下。
- `index.json`：schema=1、稳定 ID、内容 SHA-256、White/Black/Event/Site/Date/Round/Result、半回合数、收藏、导入/修改时间。时间使用 Unix 毫秒并在同实例连续导入中保持递增排序；文件名由 ID/哈希推导，无任意路径字段。
- `games/<64位稳定ID>-<64位SHA256>.pgn`：标准 UTF-8 完整注释 PGN，普通文本工具可读取。索引最多 2000 盘/4 MiB。ID/哈希、范围、重复条目和 schema 均严格校验。
- 启动不加载 Library；进入页面只读索引。选定 Open 才读取该盘文件、校验内容哈希、调用 `import_annotated`。损坏索引/PGN 不导致启动失败，也不自动重建或删除。

## D. 搜索、筛选与收藏

- 主菜单增加“棋谱档案馆 / PGN Library”，独立 `Screen::Library`，不把集合列表塞进 Replay 侧栏。
- 列表展示棋手、结果、日期、赛事、半回合和收藏/选择标志；空状态提供导入说明，长文本显式省略。仅绘制可见行，滚轮/上下按钮翻动。
- 普通文本以 Unicode lowercase 比较，匹配 White/Black/Event/Date/Result；可叠加收藏及结果筛选。四种排序：最近导入、PGN 日期文本、棋手姓名、收藏优先；日期是 PGN 字符串排序，不伪装为完整日期解释器。
- 搜索框支持 Unicode、Backspace、Ctrl+A 清空，Enter 结束输入。搜索/筛选/排序只在事件改变时重算，不逐帧扫描 PGN/哈希。
- 收藏只原子更新索引，重启后保留；不改 PGN 或引擎缓存。
- 主要 30 个标签覆盖原 11 种语言。页面错误摘要可点击 `! …` 查看完整原因；后台导入只用一个线程、一个容量 1 的完成通道和原子进度计数。

## E. Replay、编辑和导出

- 选定稳定 ID 后读取/验证原 AnnotatedGame，调用原 `open_annotated_replay`/`Replay::from_annotated`。使用同一棋盘、自动播放、节点导航、注释/NAG 编辑，不复制 Replay。
- 来源 Library ID 仅存在运行时 RecordsState。Back 回 Library，保留搜索、筛选、排序、选择和滚动；普通外部/实时 Replay 保留原返回目标。
- 馆藏 Replay 的“保存到馆藏”应用编辑缓冲，序列化完整树，更新受管理 PGN 及索引，稳定 ID/收藏/导入时间不变。成功才清 dirty；失败保留树、编辑成果及可见错误。
- 普通 Replay 的“加入档案馆”保存副本并建立关联；原“导出 PGN”与 Library“导出副本”继续另存独立文件。原生打开/保存对话框复用现有适配器；本版每次选择一个文件，可反复追加，没有新增多文件选择器。
- 导出禁止写入应用数据目录，路径比较处理大小写、父路径和可解析的链接目标；馆藏文件必须通过显式保存操作更新，不绕过索引。
- 原唯一未保存确认入口继续处理返回、切换馆藏、替换棋谱、退出/关闭：扩展 Intent，不创建第二套弹窗。成功导出副本也满足原退出保护；若要更新馆藏仍需明确保存到馆藏。
- 编辑不自动覆盖外部导入源。没有将 Library ID 或注释树写进 autosave-v1 schema。

## F. 去重、保存与磁盘异常

- 对 `export_annotated` 的完整规范化 PGN 做 SHA-256 去重，主要查索引，不扫描馆藏。完全相同跳过；不同标签/注释/NAG/变化保留为独立条目，不覆盖旧编辑。
- 新增/保存先通过原 atomic_write 写入新 PGN，再原子切换索引。索引提交前保留旧文件；保存成功提交后只移除其旧受管理修订，不做历史清理。写索引失败时内存索引不变，旧馆藏可继续读取。
- 索引内容指纹检测已观察到的外部变更并拒绝覆盖，不宣称拥有数据库式多进程事务；建议一个写入实例。发现变化需重启重载，不能继续用陈旧索引覆盖。
- 移除要求确认，只修改该条索引及对应受管理 PGN。缺失文件可明确移除残留索引；非普通文件或磁盘错误明确失败。如果索引已提交而后续删除失败，错误准确说明部分提交状态，不报告全成功。原始 PGN、自动存档和 Review Cache 均不删除。
- “检查/恢复”只在明确点击时有限检查（最多 4000 目录条目），报告缺失/损坏，重新索引合法未认领 PGN。冲突 ID/哈希或旧修订保留，不自动删除。原子写中断留下的临时文件也保留供人工检查。
- 损坏/未知 schema 的 index 不自动覆盖。文档说明：退出、人工备存/重命名 index、重启后 Check/recover；仅凭 PGN 无法恢复收藏与原导入时间。慢磁盘检查可能耗时，但只在停止真实棋钟的独立 Library 页面执行。

## G. 原有系统兼容性

- `annotated_game.rs`、`replay.rs`、`game_record.rs`、`game.rs`、`ai.rs`、`persistence.rs`、`review_cache.rs`、`review_app.rs`、UCI Controller、settings/preferences 均无修改；无依赖升级。
- 相同主线继续得到原 Review Cache 身份，馆藏 ID、收藏、列表搜索和注释保存不产生新引擎任务。切换棋局经原 cancel_review/analysis_context 清理旧任务与 PV/评分/箭头；分支单点 UCI 仍明确禁用，Review 仍只评原主线。
- Mock 集成验证：从 Library 打开完成 Review；收藏、修改并保存注释、重新打开仍命中同一缓存，缓存字节不变；切换馆藏后无旧分析覆盖。源 autosave 字节哨兵保持不变。
- Library/Replay 时真实对局棋钟暂停，真实历史与对局记录不被浏览、搜索、收藏或回放改动；Pleco、超时、悔棋、PGN/FEN、自动存档与引擎生命周期原测试继续保留。
- 原生文件选择取消的代码路径不启动导入或修改馆藏；真实原生对话框没有在本轮自动操作，未将静态/状态证据冒称为窗口实测。

## H. 自动化门禁与 Release

日志均在忽略的 `target/v025-qa/`，最终命令退出码均为 0。

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过，fmt.txt |
| `cargo test --locked -j 2` | **170/170，0 失败、0 忽略**；最终 8.29 秒，tests.txt |
| `cargo clippy --locked -j 2 --all-targets -- -D warnings` | 通过，无 lint 豁免或标准降低，clippy.txt |
| `cargo build --release --locked -j 2` | **通过，15.35 秒，本轮唯一一次 Release 构建**；build.txt |
| `python -B -m unittest discover -s tools -p test_collect_licenses.py -v` | **12/12 通过**；license-tests.txt |
| `./package.ps1 -SkipBuild -VerifyOnly` | **87 个锁定 Windows 依赖，0 未解决许可项，没有生成包**；license-gate.txt |
| 基线测试保留检查 | 原 153 个测试函数全部存在，新增 17 项，未禁用/忽略旧测试 |

新增覆盖包括：1/2/10/50 盘、逐盘标签/结果/注释树往返、黑方 FEN、非法盘号/资源限额、去重、收藏重启、搜索排序无需 PGN IO、稳定 ID 保存、索引损坏/路径校验、原子写失败、外部索引变更、移除失败、孤儿恢复、后台部分导入、原 Replay/未保存保护、Game Review/UCI 隔离。

离屏使用内存缓冲：**640×480、1280×720、11 语言、长棋手/赛事、列表选择点击、移除模态、搜索输入焦点及 Library 屏蔽分析侧栏**。未启动 GUI、生成截图或大量图片文件。测试的删除/故障操作仅针对独立 target 测试目录。

环境出现“incremental cache 无法硬链接、改用复制”的 Cargo 警告；它不是 Clippy 源码 lint。保留原始日志，没有修改全局配置、清缓存或忽略检查。

Release EXE：`C:\MYAPPLY\Code\Rust\chess\target\release\chess_rust.exe`

- 大小：**11,151,872 字节**。
- SHA-256：`fff0afc384be5f2fa427116417554e1906cd9c68663bb36b196561a0cbb033c4`。
- 构建后没有再改 Rust/Cargo 源码；仅补充文档/报告。新版本无新增运行时资源或 crate，GPL-3.0-or-later、第三方声明与原始许可证字节保持。
- 没有生成本地 v0.2.5 ZIP 或解压目录。package.ps1/工作流无需改动；VerifyOnly 不等于云端最终 ZIP 已验证。

## I. 未执行的验收

- **真实 Stockfish/NNUE/真实搜索质量与性能：未执行**，没有用户提供的 EXE，未下载、安装或运行外部真实引擎。
- **完整人工 UI、原生打开/保存/覆盖确认、真实标题栏/Alt+F4：未执行**。这些路径沿用现有入口并有状态/离屏回归，但仍需要用户亲自试玩。没有使用 Computer Use。
- 不承诺多进程并发写入事务、海量档案库性能、任意变种、在线同步、一次选择多个文件或自动清理。旧冲突修订/临时文件可能需人工检查；明确保留，避免误删恢复价值。

## J. 结论和本地交付

**本地开发候选 GO，可交给用户亲自试玩。** 必需自动化门禁通过，无剩余自动化失败；多盘归档、完整复杂 PGN、列表/收藏、Replay 编辑保存及原系统隔离已有直接测试证据。

提交后检查工作树干净，开发分支相对 main/缓存 origin/main 领先 1 个提交、落后 0；main 与 v0.2.4 标签保持基线。准确 HEAD 与提交后核验见交付答复及 delivery.json。target 日志/Mock EXE/临时脚本、用户数据、历史产物未入 Git。

**未 Push、未创建 Tag/PR/Release、未合并 main、未远端部署、未创建本地正式 ZIP。** 完成本地交付后停止，等待用户试玩及另行授权 Luna/GitHub Actions 正式发行。
