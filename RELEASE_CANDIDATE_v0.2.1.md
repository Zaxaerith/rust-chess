# RUST CHESS v0.2.1 — 本地 Release Candidate 验收报告

日期：2026-10-25（Asia/Hong_Kong）。

**结论：GO（本地 RC，未发布）。** CLI/Rust 自动化、Mock UCI 子进程、严格 Clippy、Release 构建、许可与 ZIP 门禁均通过。真实 Stockfish 集成和完整 Windows UI 人工验收尚未执行；没有将历史窗口观察或无界面测试扩大为这些项目已通过。本轮严格未使用 Computer Use、截图、鼠标或自动图形窗口操作，没有 Push、Tag、PR、Release、远端部署或合入 main。

## A. 基线与工作区

- 项目：`C:\MYAPPLY\Code\Rust\chess`。
- v0.2.1 开发起点（2026-10-24）：main，HEAD `9520ac840e43421381e9986e69b827be59ac0bd0`，当时工作树干净；从已发布 v0.2.0 建立 `feat/v0.2.1-uci-engine`。
- 本轮恢复（2026-10-25）：同一开发分支及 HEAD，已有上一轮未提交 UCI 实现、测试、文档与行尾修复。先审阅实际修改，保留成果，未覆盖用户修改或重做已完成开发。
- 本地 main、origin/main 和 `v0.2.0^{commit}` 均核实为上述发布提交；本轮没有刷新远端引用，也没有改写已公开标签/Release。
- 保留 v0.1.4 的 Duration 棋钟、快照、终局与 generation，及 v0.2.0 的 PGN/FEN、存档与只读 Replay；原有 91 项测试文件未删减。Cargo 依赖未升级或新增，锁文件仅本项目版本从 0.2.0 改为 0.2.1。
- CHANGELOG 将 v0.2.0 正确记为 2026-10-23 已发布，新增 v0.2.1 记录。中英文 README、第三方说明同步更新；历史验收报告保留为当时记录。

## B. 技术实现

### 模块与边界

- `uci_protocol.rs`：身份、option、uciok/readyok、info、cp/mate/bounds、bestmove 解析；未知普通字段容忍。使用 shakmaty 解析合法 UCI 着法及 SAN，不复制规则。
- `uci.rs`：独立后台控制器、子进程、管道、握手、参数、搜索边界、超时与清理。
- `analysis.rs`：只读目标与请求 ID、局面标识、白方视角转换、结果验证、独立偏好与应用集成。
- `analysis_ui.rs`：沿用 minifb/字体/主题的侧栏；主要标签和状态覆盖既有 11 语言。选择/启用/关闭、资源设置、分析/停止及 Replay 导航不执行实际着法。
- `uci_tests.rs` 与 `tools/mock_uci_engine.rs`：直接调用 App/分析控制器的无界面断言、受控协议进程。唯一新增测试专用报告注入入口受 `#[cfg(test)]` 限制，不进入 Release；未开发第二套 CLI 游戏。
- `main.rs` 只添加状态、轮询、绘制及退出入口；`records_app.rs`/`records_ui.rs` 添加状态切换取消钩子；`native_files.rs` 复用原生文件选择适配。原 Pleco、规则与渲染框架不替换。

### 进程、线程与资源

仅对用户明确选择并启用的本地绝对 EXE 路径调用 `Command::new`，不拼 shell，不下载引擎。进程以 EXE 所在目录为工作目录，支持空格/Unicode 和相邻 NNUE。普通 UNC 网络引擎路径不支持。

UI 线程只读写短生命周期的最新值邮箱；控制线程拥有 Child，另有 stdin 写入、stdout 读取、stderr 排空线程。请求与报告覆盖旧值，不累计无界队列；stdout 队列最多 64 行，每行最多 16 KiB；stdin 队列 1 项、命令最多 32 KiB；最多保留 256 个 option。stderr 以固定 4 KiB 缓冲排空，不保留日志海量内容。

初始化 uci 握手期限 3 秒，isready 同步 2 秒，stdin 写入确认 200 ms；搜索期限为 movetime 加 1500 ms；stop 等待 bestmove 600 ms。退出先尝试 stop/quit，300 ms 后必要时 kill，再有界轮询 300 ms。清理异常会记录错误，已移除无超时 Child::wait；控制器 shutdown 最多等待 2 秒。UI 关闭/切换引擎通过后台退休对象处理；应用正常退出及状态析构均请求有界清理。

默认 Threads 2、Hash 64 MiB、movetime 1000 ms、MultiPV 1；上限为 8 线程、256 MiB、10000 ms。只设置引擎声明支持的 spin 选项并按范围限幅，不支持则显示降级、采用引擎默认值；超过安全上限的强制最小值明确拒绝。这些是协议参数限制，不是第三方进程沙箱或 OS 内存配额。

### 状态机与异步隔离

状态包括未启用、初始化、就绪、搜索、完成、停止、失败。上下文改变才发送 ucinewgame 并 isready 同步；标准初始局面发送 startpos 与完整着法前缀，其他局面发送六字段 FEN。Replay 使用游标前缀。

每次分析有独立 ID，并绑定 Screen、游戏 generation、对局上下文、Replay 游标及当前完整 FEN。落子、悔棋、重开、模式/页面改变、PGN/FEN 导入、复盘跳转、停止或关闭都会取消或使结果失效。替换搜索先发送 stop，必须消费其 bestmove，之后才 isready 和新 position/go；readyok 本身不代表搜索完成。

bestmove/PV 首手均交给当前目标的 shakmaty 局面验证；非法返回明确失败，绝不选择第一项合法着法补救，绝不应用到棋盘。无合法着法的终局不请求不存在的着法。cp 和 mate 分开保存；黑方回合转换为白方视角时同时反转数值与上下界，不把非精确评分冒称精确值，也不换算统一胜率。

### 偏好与本轮必要修复

`analysis.cfg` 是存档目录旁的独立 JSON 偏好（默认 `%LOCALAPPDATA%\RUST CHESS`；QA 可隔离）。读取最多 16 KiB，仅保存路径、线程、Hash、时间；加载不会自动执行。原 settings.cfg 以及自动存档 schema 均不变，存档不含引擎 EXE 路径或线程对象。

1. 完整 FEN 与 startpos：shakmaty 的 Chess 相等比较忽略回合计数，曾将标准棋子布局但 `12 42` 计数的局面错发为 startpos；失败回归确认后改为比较完整六字段 FEN。
2. 旧请求失败：旧搜索停止失败时，Failed 状态曾绕过请求 ID 检查，将旧评分/PV 显示为新请求数据；本轮先建立失败回归，再仅保留传输错误，丢弃不匹配的评分、深度、PV 和 bestmove。
3. 有界退出：消除 kill 后无期限 wait，改为有界轮询；对握手/搜索中的子进程句柄直接断言退出，不再跳过无法获取句柄的检查。

## C. CLI 功能验收

| 场景 | 证据与结果 |
| --- | --- |
| 初始局面、握手、身份、参数、多行 info | Mock 子进程测试通过；深度 12、cp 52、e2e4，SAN 为 e4；选项命令按声明发送 |
| 黑方评分方向与边界 | cp +52/lowerbound 转为白方 cp -52/upperbound；mate 方向独立断言通过 |
| 非标准初始 FEN、黑方先行、计数 | `position_preserves_history_fen_and_black_start` 验证完整 FEN 和 prefix，包括 `12 42` 边界 |
| 着法合法性 | 普通、吃子、升变、双翼易位、吃过路兵、无合法着法、非法/空结果全部有断言 |
| 导入 PGN 后 Replay 分析 | 按游标仅发送正确着法前缀；快速首尾/指定步跳转使旧结果失效 |
| 真实对局隔离 | 分析不增加着法、不修改棋钟快照/棋谱；Replay 隔离与返回行为保留既有回归 |
| 落子/悔棋/重开/导入后的旧结果 | 原有 UCI 过期结果测试保留；本轮补充失败状态跨请求串入的负例并修复 |
| 连续停止、重分析、上下文同步 | 反复取消、stop→bestmove→isready→新 go 边界验证通过，readyok 不能替代 bestmove |
| 异常与重启 | 启动退出/无响应、ready 超时、超长行、阻塞 stdin、无效 EXE、非法结果、搜索中退出、缺少 bestmove 到期均明确失败；失败后重新启用可正常分析 |
| 生命周期与无残留 | 正常 quit、握手中关闭、搜索中强制清理均通过；Windows 原生进程句柄确认退出，CLI 核查本轮 target/v021-uci-tests 下 Mock 残留为 0 |

本轮未下载、查找后执行或安装真实 Stockfish；用户明确表示当前没有可提供的真实引擎。

## D. 兼容性

- Pleco 五档对弈、shakmaty 规则和 CHA-Solver 终局判定保留；未改写这些组件。
- 棋钟、确认、升变、悔棋、旧 AI/提示 generation，以及 PGN/FEN、完整历史存档、Replay 的原 91 项回归全部保留并通过。
- 打开分析面板不暂停实时棋钟；Replay 仍只读停钟。UCI 仅分析，不自动落子。
- 旧 settings.cfg 和默认值兼容；未配置或未启用外部引擎仍可正常使用原游戏系统。
- 11 语言主要按钮/状态非空；自动渲染在 640×480 和 1280×720 的实时/复盘界面做内存像素断言，分析侧栏不改写棋盘关键区域。这不是屏幕截图，也不启动窗口。
- 字体回退、主题、棋子、棋盘及原资源保留。未引入 GUI 框架或新 crate。

## E. 质量门禁与许可证

| 门禁 | 本轮结果 |
| --- | --- |
| cargo fmt --all -- --check | 通过，退出码 0 |
| cargo test --locked -j 2 | **113 passed，0 failed，0 ignored**，8.17 秒；原 91 + UCI 22 |
| cargo clippy --locked -j 2 --all-targets -- -D warnings | 通过，退出码 0，1.45 秒；无 lint 豁免 |
| cargo build --release --locked -j 2 | 通过，退出码 0，32.68 秒；复用 Cargo 缓存，必要修复后仅一次最终构建 |
| 许可证回归 | **12 passed**，原 9 + 工作区/Git 行尾 3；不混称为 Rust 测试 |
| package.ps1 -SkipBuild -VerifyOnly | 通过，0 个许可未决项 |
| 正式 RC 打包/完整性 | 通过，未使用 QA_ONLY 或许可例外；286 个归档文件，285 个载荷文件逐项 SHA-256 及源文件/EXE 字节核验 |

本轮只新增两项有实质缺口的 Rust 用例，原先完成的 20 项 UCI 用例保留；其余场景复用已有测试，不为数量重复覆盖。新的受控报告注入只用于验证旧请求失败边界，仍调用真实 App 分析逻辑。

日志在忽略目录 `target/v021-qa/cli-final/`：fmt.txt、tests.txt、clippy.txt、build.txt、license-tests.txt、license-gate.txt、package.txt、license-audit.json/txt、package-verification.json。编译环境曾提示增量缓存 hard link 不可用、回退文件复制；原文保留，未清缓存/修改全局配置，也未通过关闭警告或放宽 Clippy 掩盖。

### CRLF/LF 修复证据

- 原属性只覆盖少量 Pleco 文件；Windows 自动转换曾使 267 个依赖文件与 manifest 字节不符。现在 `LICENSES/** -text`，明确保留上游原始行尾，而非一律转 LF。
- 从已锁定 Cargo 缓存恢复原始字节；绝大多数文件与原 Git blob 相同。`generic-array-0.14.7/LICENSE` 原有 20 个 CRLF，`pleco-0.5.0/README.md` 原有 118 个 CRLF，现均恢复并保留；对其原始 .crate 校验和核对 Cargo.lock，再直接读取 tar 成员确认，正文/版权完全未改，仅纠正此前 Git 丢失的行尾。
- manifest 中 87 个 Windows 锁定依赖、272 个许可/声明文件逐项 SHA-256 通过；重复收集的 LICENSES 全树哈希不变；Pleco MIT 原件及 provenance 保留。
- 新增回归验证工作区哈希、全树 Git 属性，以及启用 core.autocrlf=true 时 ab_glyph 的真实 Git 过滤检出字节。CI 在运行收集器之前执行这组测试，避免先重写工作区掩盖检出错误；不忽略任何字节差异。
- 另将已核验的 LICENSES 同步索引属性后，以 core.autocrlf=true 实际检出全部 278 个第三方文件到独立 target 目录，逐字节等于原件，272 个 manifest 文件哈希全部通过；证据在 checkout-proof.json/txt。没有覆盖当前工作区或修改全局 Git 配置，索引实际内容差异仅为上述两份原始 CRLF 恢复。
- `.github/workflows/build.yml` 使用锁定依赖与 -j 2，并执行许可证回归；release.yml 继续复用同一 package.ps1/verify_package.py。未执行远端 Actions。
- 通用 `git diff --check` 会将上述原件的 CRLF/上游已有尾空白报告为 whitespace；保留该诊断，不改写原件或关闭 SHA 校验。项目自身文件单独通过 diff 空白检查，第三方原件以锁定 .crate、manifest 和实际检出字节的严格核验为准。

UCI 使用标准库，未新增许可依赖；发行包无 Stockfish、其他引擎、NNUE、Mock 源码/EXE、analysis.cfg、用户存档或测试脚本。RUST CHESS 继续 GPL-3.0-or-later。

## F. 历史窗口证据与未执行项

以下仅是 **2026-10-24 早期构建的历史观察**，本轮没有恢复 Computer Use：

- 原生打开对话框选中含空格和中文的本地 Mock EXE；未启用前未启动，手动启用后显示 Local Mock UCI 0.1。
- 初始局面显示 +0.52、深度 12、SAN e4；实际 e4 后结果清空，黑方局面显示白方 -0.52、SAN e5，实时棋钟换边。
- Replay 首步按钮回到初始局面，日志 position startpos；实时局面后台仍为 e4，回到实时棋盘恢复 e4。
- 搜索中停止操作被观察，协议日志含 stop、isready；Replay 停留期间测试存档 SHA-256 保持不变。
- 当时单次资源采样：游戏工作集 169,607,168 字节，Mock 4,624,384 字节；累计 CPU 时间分别 120 秒和 0.015625 秒，不是 CPU 百分比，也不代表真实 Stockfish 资源消耗。
- 曾独立观察关闭 Mock 后残留为 0；最后游戏关闭步骤因用户物理 Escape 停止 Computer Use 中断，不能声称完整自动退出流程通过。键盘注入的 Home 跳转未获得有效窗口证据，不据此宣称通过或认定为游戏缺陷；鼠标导航与 CLI 导航断言分别记录。

最终 CLI 修复后的 EXE 没有重新执行窗口操作，完整 Windows UI 人工验收尚未执行。真实 Stockfish/其他第三方引擎的版本、实际搜索结果、NNUE 加载、实际资源与退出残留验证也尚未执行。以上缺失不否定已经通过的纯逻辑、协议和 Mock 生命周期断言，也不被这些断言替代。

## G. Git 与本地发行候选

- 分支：`feat/v0.2.1-uci-engine`，由已发布 v0.2.0 的 `9520ac840e43421381e9986e69b827be59ac0bd0` 继承。
- 本报告随 `feat: add UCI engine integration and basic analysis (v0.2.1)` 本地提交保存。报告不能内嵌自身提交哈希；精确 Commit SHA、最终工作树与产物关联记录在交付消息和 ZIP 外 `dist/rust-chess-0.2.1.verification.json`。
- 仅按审阅后的明确文件清单暂存/提交；target/、dist/、日志、Mock EXE、配置和用户数据不加入 Git。不使用 git add -A，不合并 main，不改 v0.2.0 包/Tag/Release。
- `AGENTS.md` 保存长期约定：CLI/Rust 自动化优先，GUI 人工补充；未经再次明确允许，不主动使用 Computer Use、截图或图形窗口操作。

| 产物 | 路径、大小及 SHA-256 |
| --- | --- |
| Release EXE | `target/release/chess_rust.exe`；10,664,960 字节；`ba6fca6e3213045388124432192abc105af8b1227bceb4918e37f2570b767a91` |
| RC ZIP | `C:\MYAPPLY\Code\Rust\chess\dist\rust-chess-0.2.1.zip`；2,805,495 字节；`cb042b57175d047dbe86705ed95f25c12f3eec980ee5033cf36b3d8fb0198da1` |

正式脚本复用刚验证的 Release 构建，完整检查 ZIP CRC、唯一安全路径、精确白名单、依赖闭包、许可清单、285 个载荷哈希及原始源文件字节。checksums.sha256 本身由整个 ZIP SHA-256 覆盖，不自引用。验收报告、AGENTS 和测试工具不进入 ZIP。本轮未 Push、创建/推送 Tag、创建 PR、发布 GitHub Release 或远端部署。

## H. 最终建议与后续

**GO：接受为已完成 CLI/Mock 验收的 v0.2.1 本地 RC，停止在本地。** 当前没有未通过的本地自动化、构建、许可或归档门禁。公开发布仍须另行授权；GO 不代表真实 Stockfish 或完整 UI 已验收。

后续可由用户提供可信引擎路径，优先以 CLI 验证真实握手/分析/NNUE 与退出，再由用户进行少量 UI 人工补充；不会擅自下载引擎或恢复 Computer Use。第三方引擎的内部资源实现、特殊命令行启动需求、全部品牌/版本及最坏响应耗时未全面验证。

v0.2.2 可在明确需求后扩展更多可选 UCI 参数、完整主变化展示、CLI 真实引擎兼容性矩阵与更好的错误说明；本版不增加 UCI 实际对弈、引擎捆绑、复杂分析标注或在线功能。若最终源码/打包输入再改变，应按变化重新运行相应门禁，不复用失效证据。
