# RUST CHESS v0.2.4 — Advanced PGN Replay 本地候选

日期：2026-10-26（Asia/Hong_Kong）。**本地开发候选 GO，适合交给用户亲自试玩；未授权或执行正式发行。**

## A. Git 基线及提交

- 目录 `C:\MYAPPLY\Code\Rust\chess`；起始 main、HEAD、缓存 origin/main、现有 v0.2.3 标签解引用均为 `397c62e876b002f20f8a14d28543caedbfde1a03`，开发开始时工作树干净。
- 新建 `feat/v0.2.4-advanced-pgn`，未移动 main。限额中断后保留本轮已有的未提交模型/导航草稿继续完成，没有清理或覆盖用户数据。
- 遵守 AGENTS.md：CLI/Rust/Mock/离屏优先，无 Computer Use，无本地发行 ZIP，无 cargo clean，无引擎下载。
- 本报告随源码同次本地提交。完整候选 SHA 由 `git log -1 --format=%H -- RELEASE_CANDIDATE_v0.2.4.md` 取得，提交后准确值和工作树检查写入忽略的 `target/v024-qa/delivery.json` 并列于交付答复，避免提交内容自引用其 SHA。

## B. PGN 树模型与严格分支解析

- `src/annotated_game.rs` 新增 AnnotatedGame/Node/稳定 usize NodeId；root 为 0。节点保存父节点、子节点（首个为续着，其余为替代变化）、合法 Move/UCI、规范 SAN、后局面、半回合、前/后注释、NAG 及可选 RAV 结果。前局面由父节点唯一确定，不复制棋规。节点 ID 在内存编辑/导航期间不变化。
- GameRecord/ValidatedRecord 仍保存原线性主线。树验证检查关系、可达性、合法着法、SAN/UCI、六字段局面与线性主线一致性；元数据编辑不改节点着法/局面。
- `src/pgn.rs` 使用本机锁定 **pgn-reader 0.26.0** 的 Visitor API，读取其源码确认签名。每次 begin_variation 回退到被替代着法的父局面，栈保存恢复节点；所有分支逐手使用 shakmaty 验证，不再 Skip(true)。非法分支或将军/将死后缀明确报错。
- 新生产入口 `import_annotated`/`export_annotated`，应用只走完整树导入。普通 `export(GameRecord)` 保留；原线性 import 投影及 Replay 便捷构造仅作为保留回归套件的测试适配器，仍复用同一严格解析器，没有跳过旧测试。
- 严格词法预检补齐库的宽容行为：UTF-8、完整括号/标签/结果、NAG 范围与符号、未知 token、所有节点与注释限额。分号注释转为可保留的花括号注释，保留文字内容；移动文本空白规范化，避免库把空行误当棋局终止。多盘仍明确拒绝。

资源限制：输入/输出 4 MiB，每路径 4096 半回合、RAV 嵌套 64、累计 8192 着法节点；单注释 4096 UTF-8 字节、总文字 1 MiB、注释条目 16384（包括空条目）；每节点 NAG 64、累计 65536。树入场预算按每节点 2048 字节、注释文本/条目与 NAG 的保守费用计费，上限 24 MiB；这是模型预算，**不等于整个进程 RSS 上限**。数量限制也防止大量空注释放大分配。

## C. 注释/NAG 展示、编辑及保存

- 新增 `advanced_app.rs` 和 `advanced_ui.rs`，复用现有字符输入、文本绘制、剪贴板及文件对话框，不引入框架。
- 注释归属：游戏开头归 root.before；RAV 首着前归该首节点 before；其他文字归当前前一着 after。采用明确且可往返的标准 PGN 约定，不为任意着前内容发明私有扩展。
- 当前节点着后注释可编辑，游戏/RAV 起始也可切换着前注释。未编辑的多条注释保留条目划分；修改时以新的单条文字替换所选区域。清除操作删除当前节点文字，NAG 模式清除其 NAG。
- Unicode 字符边界的光标、Backspace/Delete、Home/End、Ctrl+A、Ctrl+V；Enter 应用、Escape 取消。拒绝花括号、不安全控制符或超长输入，粘贴可含 CR/LF/tab，不静默截断。输入错误保留原数据和编辑缓冲。
- NAG `$0`–`$255` 与 `! ? !! ?? !? ?!` 可添加/移除，保留顺序及重复；其他合法值以数字显示。注释视口明确标为片段，使用上下控制浏览；底层全文保留。
- 编辑只改内存树，不改来源 PGN，不进入 autosave schema。未导出标志及退出/替换确认支持取消、导出后离开、明确放弃；未应用的已修改编辑缓冲也受保护。只打开未修改编辑器不会制造虚假脏状态。
- 文件选择取消或原子写失败不清 dirty、不放行 pending 离开；成功写入才标记已导出。导出默认使用新 `-annotated.pgn` 名称，不自动写回来源。
- Windows 标题栏关闭经 `close_guard.rs` 的 comctl32 子类回调转为应用请求，在 minifb 标为已关闭前走同一确认。没有修改其 GWLP_USERDATA；RAII 先移除回调再释放上下文，失败明确记录并保留极小上下文以防 UAF。这里只做了无窗口回调单测，真实标题栏仍待人工验收。

## D. Replay 导航与自动播放

- `Replay.game/cursor/identity` 保留主线语义；新增 tree/node/selected_path，棋盘和 FEN 使用 `position()`，高亮用当前节点 Move 并正确映射易位国王落点。
- First/Previous/Next/Last/Jump、Left/Right/Home/End 沿所选路径；候选变化列表、上级变化、回主线、路径滚动和当前节点高亮。回主线恢复进入分支前记录的主线位置。
- 播放速度 0.5×/1×/1.5×/2×/3×/4×，默认 1×。用现有帧 Instant 增量和整数纳秒积分，单帧最多接收 2 秒、补进 8 着；无 sleep、后台落子线程或真实对局着法。
- 手动导航、选择变化及编辑暂停，路径终点停止，退出 Replay 释放状态。原生选择框返回后重置时间基准，取消选择不会因停留时间快进。活跃播放不持久化。
- 输入编辑或未保存模态优先消费按键，不把光标操作误当复盘快捷键；新局/恢复/FEN 等替换先验证并确认，再清理旧编辑状态。

## E. PGN 导入/导出语义往返

完整导出包含七基本标签、原有其他合法标签、Result、必要 SetUp/FEN、原主线、嵌套 RAV、注释、NAG 和正确白/黑方回合编号。省略的必要标签在首次导入规范为标准未知值，默认初始局面不重复保留无意义 SetUp/FEN。

测试对 `Import → Export → Import` 比较主线 UCI、每节点路径/子关系、六字段局面、前后注释、NAG 顺序、分支结果、标签和比赛结果；覆盖嵌套、Unicode、分号注释、黑方第 42 回合起始、易位、升变、吃过路兵。保证语义，不保证原空白、缩进、glyph 拼写或原 NodeId 序列；应用内节点 ID 不重排。

不支持的 `%` 逃逸行、无法安全括号表示的注释、非法 token 明确拒绝。注释内 `[%cal ...]` 等指令保留为文字，不解释图形。多盘、变种、复杂分支创建/删除/提升不在本版范围内。

## F. UCI、Game Review、真实棋局和存档兼容

- 分支节点的单点 UCI **明确禁用**，使用 11 语言提示，不会误送主线着法前缀。返回主线可继续原分析。分析 stamp 包含 NodeId，切换或播放推进清除旧 PV/分数/箭头。
- Game Review 固定原 ValidatedRecord/identity，同一 Controller 持续工作。注释编辑与分支浏览不改固定目标或缓存身份，不启动分支引擎。
- 分支隐藏主线曲线游标、质量与评分覆盖，标明仅主线分析。共享祖先虽然属于主线，其已选后续仍可能是分支：曲线点击现在显式先切回主线路径再 Jump，避免错误映射。回主线恢复联动。
- 回归实际发现并修复旧 MultiPV 配置报告竞态：Controller 报告被确认的 requested_multipv；新单线设置排队时，旧 Stopped/MultiPV=3 不再被误判为引擎不支持单线。保留原 stop/bestmove/isready 边界。
- **Review Cache 算法/版本/文件格式未改**，原缓存不因注释变化重算或删除。Mock 集成验证修改注释、进入分支期间任务继续，完成后再次开始命中缓存、go 数不增，自动存档字节哨兵保持不变。
- Pleco、CHA-Solver、规则/棋钟核心、普通 GameRecord、自动存档实现、settings/preferences 未改；所有 v0.2.3 回归保留，真实局面、历史和棋钟不被 Replay 修改。

## G. 自动化门禁和构建

日志位于忽略的 `target/v024-qa/`，最终命令均退出码 0。

| 检查 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过，fmt.txt |
| `cargo test --locked -j 2` | **153 通过，0 失败，0 忽略**，7.81 秒；tests.txt |
| `cargo clippy --locked -j 2 --all-targets -- -D warnings` | 通过，无新增 lint 豁免；clippy.txt |
| `cargo build --release --locked -j 2` | 通过，13.69 秒，本轮唯一一次 Release 构建；build.txt |
| `python -B -m unittest discover -s tools -p test_collect_licenses.py -v` | **12/12 通过**；license-tests.txt |
| `./package.ps1 -SkipBuild -VerifyOnly` | **87 个锁定 Windows 依赖、0 未解决许可项、不生成包**；license-gate.txt |
| `git diff --check` | 通过 |

原 133 项 Rust 测试全部保留，新增 20 项：5 项 PGN 树/严格往返与资源上限、5 项路径/播放纯状态、8 项应用/编辑/导出/Review/离屏集成、2 项无窗口关闭保护单测。通过条件没有依赖长 sleep 或截图。

离屏验证覆盖 640×480 与 1280×720、11 语言主要文字非空、侧栏不改棋盘像素、Unicode 编辑模态及曲线/分支状态。关闭保护测试直接调用回调处理 AtomicBool，不创建 HWND、不发送/注入消息。原 Mock 生命周期、异常、过期结果及进程回收测试继续通过。

开发期曾出现增量缓存硬链接回退复制的环境警告，原始日志保留；未删除缓存、修改全局 Cargo 或降低 Clippy 标准。

Release EXE：`C:\MYAPPLY\Code\Rust\chess\target\release\chess_rust.exe`。

- 大小：**10,938,880 字节**。
- SHA-256：`1d0c2e239b573ed1478abf6aec9d62c132cbdb044406f3c6eb8cd52321579fb8`。

没有新增 crate、依赖升级或重复 shakmaty；Cargo.lock 仅根版本改为 0.2.4。GPL-3.0-or-later、既有声明、Pleco MIT provenance、LICENSES 原始字节属性均保留。Windows comctl32 是系统 DLL，不捆绑第三方程序。

## H. 验证边界与交付状态

- **真实 Stockfish、NNUE、真实搜索质量和资源性能：未执行**；用户未提供 EXE，未擅自下载、安装或执行。
- **完整人工 UI、原生文件/剪贴板对话框、真实标题栏/Alt+F4 操作：未执行**；本轮严禁 Computer Use，未做截图、鼠标/键盘注入或自动窗口验收。状态/回调/离屏通过不等于这些人工路径已经实测。
- 本地提交后检查工作树干净、相对 main/缓存 origin/main 1 ahead/0 behind，原 v0.2.3 标签及 main 保持基线；准确 SHA 见 delivery.json/交付答复。
- 显式暂存本轮源码、测试、文档；target 日志、Mock EXE、临时脚本、用户数据、历史产物不入 Git。
- **未合并 main、未 Push、未创建 Tag/PR/Release、未远端部署；未创建本地 v0.2.4 ZIP 或历史解压目录。** 未 cargo clean 或修改其他项目。
- 发布脚本/白名单无需新增运行时资源。VerifyOnly 是许可证门禁，不是云端 ZIP 已验收；后续正式包等待用户授权 Luna/GitHub Actions。

## I. 结论及限制

**本地候选 GO，可交给用户亲自试玩。** 完整单盘树、元数据保存与往返、交互路径、播放、未保存保护及原有系统兼容门禁通过，无剩余自动化失败。

限制：仅标准国际象棋/单盘；着法树只浏览不创建/重排；着前注释遵循游戏/RAV 起始约定；长文本使用视口且单条编辑限 4096 字节；密集/长路径需滚动；分支单点 UCI 禁用，Game Review 仍只评原主线。未导出编辑只在内存，崩溃/强制结束不能由 autosave 恢复。真实引擎与人工 UI 仍须分别记录，不能将本报告当作已完成这些验收。

本地提交和交付检查完成后停止，不自动开展 v0.2.5 或正式发行。
