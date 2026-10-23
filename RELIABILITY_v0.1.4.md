# RUST CHESS v0.1.4 本地 Release Candidate 验收报告

交付日期：2026-10-23（Asia/Hong_Kong）。仅本地候选，尚未正式发布。

## A. 基线状态

- 目录：`C:\MYAPPLY\Code\Rust\chess`。
- origin：`https://github.com/Zaxaerith/rust-chess.git`。
- 原始分支：`main`；原始 HEAD：`4621edecf3d65d10359a9b06a74cdf6432a19d2b`。
- 初始工作树干净，没有覆盖或清理用户未提交修改；未发现适用的 AGENTS.md。
- 实际版本 0.1.3，Rust 2024；minifb 0.27、shakmaty 0.27、Pleco 0.5、ab_glyph 0.2，11 种界面语言，与任务基线一致。程序依赖 Windows 显示及资源 API，CI 保留 Windows 执行。
- 原有 10 个测试通过。先增加 5 个回归测试，在旧实现上全部失败，确认：超时不结束对局、悔棋不恢复时间、人机回复后少退一手、两格易位输入失败、不可合法吃过路兵的格子错误参与重复局面标识。
- 源码另确认：确认/升变暂停棋钟；AI FEN 转换失败会使用初始局面；着法转换失败会选第一项合法着法；后翼易位动画的王目标格错误；点击己方车可能变为重新选子，绕过易位逻辑；预走棋绕过着法确认；棋钟显示可能把 59.99 秒四舍五入为 0:60.0。
- 需要测试确认的异步边界现已覆盖：旧回复在重开、悔棋、模式切换、返回菜单、终局后不得污染当前局面。

## B. 修改内容与核心设计

1. `ClockState` 用 `Duration` 保存双方时间和警告状态，饱和扣减，按局面保存历史快照。菜单、设置和关闭棋钟时暂停；确认及升变期间继续计时；长帧一次结算全部实际经过时间。
2. `GameResult` 统一将死、逼和、子力不足、重复、75 回合和超时结果。终局只结算一次，清除输入与预走棋，使后台任务失效；加时不能恢复终局。终局面板按既有策略提供悔棋入口。
3. 明确事件顺序：落子前先结算当前时间；恰好归零时拒绝该步；未归零则验证并落子，再判定将死/规则终局。已经完成的将死优先于其后的超时及自动和棋规则。
4. 双人悔棋退一手；AI 思考中退玩家刚走的一手；AI 回复后退两手，恢复对应历史局面的双方时间。新局清空全部旧快照、动画与终局状态。
5. AI 返回 `Result`，捕获搜索 panic，明确处理空结果、非法着法和工作线程断开；取消初始局面和第一步合法着法的静默替代。通过 Pleco UCI 表示交给 shakmaty 验证，保持原五档搜索器及深度。
6. AI 与提示回复携带 generation；局面或模式变化时丢弃接收端并增加 generation；接收时再次检查 generation、终局和当前合法性。无须物理终止旧搜索线程。
7. 重复键使用规范局面字段和合法吃过路兵权利，保留行棋方与易位权，排除计数器。条件三次重复使用有权提和一方的时间，且仅在棋钟开启时生效。五次重复与 75 回合自动终局独立于三次重复偏好。
8. 修复两种易位输入、后翼动画、预走棋确认和整数十分之一秒显示；新增结果及错误文字覆盖全部 11 种语言，长状态文字按可用宽度适配。
9. 超时先使用规则库可靠的子力快速判定，再使用 CHA-Solver 3.0.0 判断是否存在合作将死序列，覆盖封闭兵墙、受限象和强制吃掉最后子力的局面。新增 `chess` 3.2.0 用于判定库的局面表示；不启动外部进程、不替换 Pleco 对弈 AI，也未新增 UCI 功能。
10. 检查中发现既有 Clippy 警告；以等值颜色字面量、简化条件、合并绘图尺寸/坐标参数修复，没有增加 lint 豁免。核对主题全部 190 个十六进制颜色值保持一致。

修改文件：

- 对局与规则：`src/main.rs`、新增 `src/game.rs`、新增 `src/reliability_tests.rs`。
- AI：`src/ai.rs`。
- 展示与机械 lint 修复：`src/render.rs`、`src/i18n.rs`、`src/assets.rs`、`src/font.rs`、`src/display.rs`、`src/theme.rs`。
- 设置兼容测试及纯解析入口：`src/settings.rs`。`src/preferences.rs` 默认值未修改。
- 构建与版本：`Cargo.toml`、`Cargo.lock`、`.github/workflows/build.yml`。
- 文档及版权：`CHANGELOG.md`、`README.md`、`README_ZH.md`、`THIRD_PARTY_NOTICES.md`、新增 `LICENSES/chasolver-MIT.txt`、`LICENSES/chess-MIT.txt` 和本报告。
- 打包声明随附：`package.ps1`、`.github/workflows/release.yml`；原 Windows Release 触发、编译、打包、发布步骤保留。

原本正常且保留：旧设置缺少新键时沿用默认值；自动升后已有 Never/Premove/Always 区分；关闭棋钟的无计时模式；普通/特殊着法的 Pleco 编码与 shakmaty 合法着法能够对应；旧异步任务已有丢弃接收端保护。本轮增加明确错误与接收校验，没有重写 Pleco 搜索算法。

## C. 测试证据

环境：rustc 1.99.0，cargo 1.99.0，`x86_64-pc-windows-msvc`。

| 检查 | 本地结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过，退出码 0 |
| `cargo test -j 2` | 56 passed；0 failed；0 ignored，最终测试执行 13.08 秒 |
| `cargo clippy -j 2 --all-targets -- -D warnings` | 通过，退出码 0 |
| `cargo build --release -j 2` | 通过，退出码 0；最终增量构建 6.63 秒 |
| `git diff --check` | 通过 |
| 打包脚本 PowerShell AST 语法检查 | 通过；未执行脚本 |

`-j 2` 仅限制并行编译数量，不改变测试或检查标准。原始本地证据在忽略目录 `target/qa/fmt-final.txt`、`test-final.txt`、`clippy-final.txt`、`build-final.txt`；报告本身随源码提交。

覆盖：准确扣时及换边、暂停、长帧超时、双方冻结、超时胜负与不可将死和棋；确认/升变/AI 思考中超时；双人及双方颜色的人机悔棋和时间恢复；将死、逼和、三次/五次重复及 75 回合规则；合法/非法/空 AI 结果、panic 和断开；过期 AI/提示；旧配置及默认值；四种升变、双翼易位、吃过路兵；11 种语言的各类终局和错误渲染。

窗口证据：在接入最终 CHA-Solver 判定前，通过 Windows 原生窗口观察过双人 e4/e5、拖拽及点击落子、棋钟换边、悔棋恢复时间、人机 e4/c6 回复、两手悔棋、提示 d4。用户随后反馈“人工测试过可以”，并要求停止 Computer Use，已遵守。最终判定库接入后未再次执行窗口测试，新增规则通过自动化测试验证。没有把构建成功等同于全部窗口功能通过。

未实测：最终二进制的逐项窗口复测、真实等待十分钟的窗口超时、全部语言/主题组合的人工作业、远端 GitHub Actions、ZIP 打包和正式发布。打包脚本未运行，避免其既有清理步骤影响历史产物。

候选程序：`target/release/chess_rust.exe`，10,046,464 字节。

SHA-256：`145e57bddbce80c6c3b17f8dad68fc8814280a6d83232c6c932ffa83dc86514d`。

重要边界：CHA-Solver 的完整搜索仍有资源预算，理论上可能返回未决；转换或搜索异常也可能失败。此时显示“超时：胜负判定失败”，双方停钟、禁止继续落子，不猜测胜负；允许按策略悔棋或新局。测试注入了未决及 panic 验证该路径。判定目前在超时结算调用中同步执行，复杂局面可能暂时延迟窗口响应；本轮未做最坏耗时/峰值内存保证。不能宣称对所有可能局面都已证明覆盖。

判定依据及 API：

- [FIDE Laws of Chess，6.9](https://handbook.fide.com/chapter/e012023)
- [CHA-Solver 官方说明与覆盖边界](https://github.com/miguel-ambrona/chasolver)
- [CHA-Solver 3.0.0 API](https://docs.rs/chasolver/3.0.0/chasolver/)

## D. 兼容性

- `settings.cfg` 原键名和默认值保持不变；旧配置、无配置、未知键与非法值回退均有测试。测试通过纯解析，不覆盖用户设置。
- 当前全是本地休闲对局，Casual 与 Always 均允许操作；Never 禁止。认输/提和确认偏好原有但完整交互仍未实现，未扩展 UI 流程。
- 11 种语言、10 套主题、6 套棋盘、5 套棋子保留；资源目录、根目录历史 EXE、旧发布文件及原 GPL 许可证未修改或删除。新增 MIT 组件附带完整版权与许可文件。
- AI 难度、算法和深度保留；异常时现在明确报错而不是随机替代着法。过期结果不生效，超时后不再应用 AI 着法。废弃搜索线程仍可能短暂消耗 CPU，完成后自行退出。
- 复用 Cargo 缓存，未删除缓存或项目外 target，未修改全局 Rust/Cargo 配置。

## E. Git 与交付状态

- 开发分支：`fix/v0.1.4-reliability`，由干净 main 创建，没有覆盖同名分支。
- 本地提交信息：`fix: improve chess clock and game state reliability (v0.1.4)`。确切提交 SHA 和最终工作树状态由交付消息提供（本报告随该提交保存，无法内嵌自身哈希）。
- 仅本地提交；未执行 Push、未创建或推送 Tag、未创建远程 PR、未发布 GitHub Release。
- 历史二进制未覆盖；候选程序保留在 Cargo Release 输出目录，未作为源码提交。

## F. 后续计划

- 正式发布前复测最终候选，尤其是复杂超时判定耗时与依赖打包；如需进一步隔离判定耗时，可将其移入带 generation 校验的独立后台任务。
- v0.2.0：PGN 导入导出、FEN 支持、棋局自动存档、对局历史回放和复盘界面。
- 另行设计认输、提和、50 回合/预期重复申诉交互及测试，不在本轮扩展。
- 未实现在线对战、UCI 外部引擎或通用持久化系统。

本轮结束后等待用户明确授权正式发布。
