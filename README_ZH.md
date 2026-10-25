# RUST CHESS

[English](README.md)

> 本地开发候选：**v0.2.2 Analysis Board**（未发布），详见[验收报告](RELEASE_CANDIDATE_v0.2.2.md)。[v0.2.1](https://github.com/Zaxaerith/rust-chess/releases/tag/v0.2.1) 已正式发布。

一个用 Rust 编写的本地窗口国际象棋游戏。支持双人对战、人机对战、5 档 AI
难度、走棋动画、AI 思考停顿、开局结束动画、设置记忆，以及多套仿 VSCode
主题。v0.1.4 增加棋钟终局结算、带时间恢复的悔棋、明确的 AI 错误处理及回归测试。

## 功能

- 双人对战 / 人机对战（人机执白或执黑）
- AI 难度 5 档：入门 / 简单 / 中等 / 困难 / 大师
- 合法走法提示、将军提示、升变选择、王车易位、吃过路兵
- 悔棋、新对局、棋谱记录
- 走棋滑动动画、AI 思考停顿、对局结束结算面板
- 游戏设置记忆（保存到 exe 同目录的 `settings.cfg`）
- 分辨率/刷新率选项自动读取系统显示模式
- 10 套主题：Dark+ / Light+ / Monokai / Solarized / Nord / Abyss / Kimbie Dark / Quiet Light / Tomorrow Night Blue / Red
- 11 种语言：简体中文、繁體中文、English、Français、Español、Latina、日本語、한국어、Deutsch、Italiano、Русский
- 5 套棋子：CBurnett / Merida / Chessnut / RhosGFX / Fantasy
- 6 套棋盘：Classic / Walnut / Ocean / Emerald / Marble / Graphite
- 玩家执黑时可选择自动翻转棋盘
- 对局中可请求基于 Pleco 搜索器的决策建议
- 左侧纵向主菜单与动态旋转齿轮 Logo
- 三级棋盘设置：界面设置 / 对局行为 / 棋钟
- 棋子拖拽、拖动放大、圆形或方形落点标记
- 预走棋、自动升后、着法确认和两种王车易位输入方式
- 本地 10 分钟棋钟、临界时间提示、十分之一秒显示和给对方加时

着法确认和升变选择期间继续计时；菜单和设置页面暂停计时。悔棋恢复目标历史局面保存的双方棋钟，人机回复后退两手。加时不能恢复已结束对局。当前所有本地对局均属休闲对局，因此“仅限休闲”和“总是”均允许相应操作。根目录 EXE 为历史构建，本候选版本请运行新编译的 target/release/chess_rust.exe。

## 直接运行

项目根目录的 `国际象棋.exe` 是完整可执行文件，棋子素材已嵌入 exe，
双击即可运行，不需要额外安装任何东西。

## 从源码编译

需要 [Rust 工具链](https://www.rust-lang.org/tools/install)（stable 即可）。

```powershell
cargo build --release
```

编译产物位于 `target/release/chess_rust.exe`，可以复制到任意位置直接运行。

## 发布打包

- 从 v0.2.2 起：本地只做源码验证、Release 构建及 `./package.ps1 -SkipBuild -VerifyOnly`，不生成正式 ZIP 或解压目录。获授权后由 GitHub Actions 生成并验证正式发行包。
- GitHub 自动发布：推送 `v*` 开头的 tag，Actions 会自动编译、打包并创建
  Release，详见 [RELEASE.md](RELEASE.md)

## VSCode 开发

1. 用 VSCode 打开项目根目录。
2. 安装推荐的 `rust-analyzer` 扩展。
3. 按 `Ctrl+Shift+B` 编译 release 版，按 `F5` 启动游戏。

也可以在终端运行：

```powershell
cargo run --release
```

## 操作说明

- 点击己方棋子选中，再点击目标格走棋
- 点击选中的棋子或空白格取消选择
- 兵到达底线时选择升变棋子（后/车/象/马）
- 右侧栏提供新对局、悔棋、返回主菜单
- 按 `Esc` 退出游戏

## 游戏设置

- 对战模式：双人 / 人机执白 / 人机执黑
- AI 难度：入门 / 简单 / 中等 / 困难 / 大师
- 窗口分辨率：自动读取系统支持的显示模式
- 刷新率：自动读取系统支持的刷新率
- 主题：10 套 VS Code 风格配色
- 语言：简体中文、繁体中文、英语、法语、西班牙语、拉丁语、日语、韩语、德语、意大利语、俄语
- 棋盘皮肤：Classic / Walnut / Ocean / Emerald / Marble / Graphite
- 棋子皮肤：CBurnett / Merida / Chessnut / RhosGFX / Fantasy
- 执黑时翻转棋盘：开启或关闭

设置会立即生效并保存到 `settings.cfg`，下次启动自动恢复。

## 技术栈

- `minifb`：本地窗口与输入
- `shakmaty`：国际象棋规则引擎
- `chasolver` / `chess`：超时将死可能性判定（MIT），不替换对弈 AI
- `pleco`：人机对战的 AI 搜索器（Rust 重写的 Stockfish）
- `png`：棋子 PNG 解码
- `ab_glyph`：文字渲染（使用系统中文字体）

## 项目结构

```text
src/
  main.rs       程序入口与对局逻辑
  game.rs       终局、超时规则判定与棋钟快照
  render.rs     棋盘、菜单、设置、动画渲染
  ai.rs         Pleco AI 难度映射与决策建议搜索
  i18n.rs       十一语言界面文本
  theme.rs      界面主题与棋盘配色
  display.rs    读取系统分辨率/刷新率
  settings.rs   设置保存与读取
  font.rs       文字渲染
  assets.rs     棋子图片解码
assets/
  piece-svg/    开源棋子 SVG 素材
  pieces/       运行时使用的 PNG（由 SVG 渲染生成）
  ATTRIBUTION.md 棋子素材版权说明
```

## 开源协议

本项目使用 **GPL-3.0-or-later** 协议，见 [LICENSE](LICENSE)。

协议选择原因：项目依赖的规则引擎 `shakmaty` 使用 GPL-3.0+ 协议，因此整个
项目以兼容的 GPL-3.0-or-later 发布。

第三方依赖、素材和版权声明详见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
与 [assets/ATTRIBUTION.md](assets/ATTRIBUTION.md)。

## 致谢

- [shakmaty](https://github.com/niklasf/shakmaty)：国际象棋规则引擎
- [Pleco](https://github.com/pleco-rs/Pleco)：Rust 重写的 Stockfish，提供 AI 搜索器
- [Lichess](https://github.com/lichess-org/lila)：开源棋子素材与许可证清单
- [Visual Studio Code](https://github.com/microsoft/vscode)：界面主题配色参考

## 棋谱、存档与复盘（v0.2.0）

- 主菜单“打开棋谱”通过原生文件对话框读取单盘 UTF-8 PGN，并进入外部只读复盘。限制为 4 MiB、4096 个半回合、变化嵌套 64 层。只载入主线；省略注释、NAG 和变化时会提示，再次导出也不包含这些内容。多盘文件、非标准变种与非法着法会明确报错。
- 对局顶部提供“导出 PGN / 复盘 / 复制 FEN / 导入 FEN”；终局后仍可导出和复盘。取消文件对话框不改变棋局。
- 复盘支持首步、上一步、下一步、末步按钮，可滚动并点击着法；Left/Right/Home/End 浏览首尾及前后局面。返回后继续原来的实时棋局或主菜单。复盘不改真实历史、不扣时间、不消费后台回复，也不覆盖自动存档。
- FEN 必须完整包含六字段。使用粘贴按钮或 Ctrl+V；Ctrl+A 清空输入，方向键和 Home/End 移动输入光标。FEN 建立独立局面并重置棋钟、历史和重复记录，不等同于完整存档恢复。
- 自动存档位于 `%LOCALAPPDATA%\RUST CHESS\autosave-v1.json`。格式版本为 1，保存初始 FEN、完整着法、双方棋钟及快照、模式/偏好和终局原因。使用 SHA-256 校验、临时文件、安全替换和 `.json.bak` 备份。坏存档或未知版本会报错并保留；人工修复前请先复制备份。失败写入留下可恢复 `.tmp` 文件，不删除用户数据。
- 每次正式落子、悔棋、加时、终局、菜单/正常退出时保存，进行中的棋局另每五秒保存一次。离线时间不扣钟。启动发现未完成对局时可选择“继续 / 新局 / 暂不恢复”；不会静默覆盖运行中的另一棋局。
- 文件对话框和 FEN 输入期间暂停棋钟。重复存档错误首次弹窗，关闭后保留失败标记。自动存档只保留最近棋局；长期归档请导出 PGN。
- 测试可用 `RUST_CHESS_DATA_DIR` 隔离存档；正常设置仍使用原 `settings.cfg` 路径与键名。
- 打包使用 `./package.ps1`，需 Python 3.11+ 收集并核实锁定依赖的许可证。已有发行目录/ZIP 不会被覆盖，测试工具、配置、用户存档和测试棋谱不进入发行包。

对于已发布的 v0.2.0，Pleco 0.5.0 的历史 MIT 原件已与发布源码提交核实，随包保留固定来源及校验记录。该版本的用户验收与包证据保留在历史 `RELEASE_CANDIDATE_v0.2.0.md`，不冒称为 v0.2.1 验收结果。报告不进入 ZIP，以免引用自身归档哈希。新的正式发布仍须单独授权。

## Analysis Board / 可选的外部 UCI 分析（v0.2.2）

- 打开“UCI 分析”，选择自行安装的本地 `.exe`，再手动“启用引擎”。支持空格和 Unicode 路径，以引擎所在目录作为工作目录，支持相邻 NNUE 文件。不下载、不捆绑引擎，加载偏好不会自动执行。
- “分析当前局面”针对实时棋盘或 Replay 游标，发送真实初始 FEN 和着法前缀。显示白方视角评分、深度、合法 SAN 最佳着法及完整编号 PV；`M+3` / `M-3` 分别表示白方 / 黑方可在 3 回合将杀，`M+0` / `M-0` 区分已将死黑方 / 白方，`>=` / `<=` 表示评分边界。不同引擎的 cp 标定不等同于统一胜率。
- 默认 2 线程、64 MiB Hash、1000 ms、MultiPV 1。按钮循环设置有界时间/线程/Hash（100–10000 ms、1–8、16–256 MiB）；改变这些资源参数会关闭当前引擎，需重新启用。MultiPV 可选 1/2/3，修改时安全停止、同步、配置并重新分析，不重启引擎；不支持 MultiPV 时明确退回单线，引擎声明的范围也可能限制实际条数。只设置引擎声明支持的 spin 参数，并按其范围限幅；不支持时显示降级提示并使用引擎默认值。
- “停止分析”取消真实搜索，“关闭引擎”回收进程。落子、悔棋、导入及复盘跳转使旧结果失效。必须消费 stop 对应的 bestmove 才可开始下一搜索，超时则关闭引擎。分析不落子、不暂停实时棋钟，也不替换 Pleco 五档对弈；Replay 的只读停钟规则保留。
- 独立 JSON 偏好文件 `analysis.cfg` 位于 `%LOCALAPPDATA%\RUST CHESS` 的存档旁（QA 可隔离），仅保存路径和资源设置，不保存启用许可，不进入自动存档 JSON。原 `settings.cfg` 保持兼容。
- 只选择并运行您信任的引擎；本功能不是第三方程序沙箱。没有引擎配置仍可正常对弈、存档和复盘。本轮真实引擎、窗口测试证据及限制单独记录在 `RELEASE_CANDIDATE_v0.2.2.md`；没有进行真实 Stockfish 或 v0.2.2 UI 人工验收。
- 验收采用 CLI / Rust 自动化优先，使用受控 Mock UCI 进程，不依赖 GUI 自动操作。分析测试运行 `cargo test --locked -j 2 uci_tests`；许可证/检出回归运行 `python -B -m unittest discover -s tools -p test_collect_licenses.py`。真实 Stockfish 与完整 UI 人工验收单独记录，不从无界面测试推断通过；长期约定保存在 `AGENTS.md`。

- 纵向评分条与当前候选箭头仅供阅读。正值始终有利白方，翻转棋盘也不改变含义。白色占比采用 `0.5 + atan(clamp(cp,-4000,4000)/400)/pi`，是有界可视化，**不是获胜概率**；将杀使用端点，边界分数保留 `>=` / `<=`。无有效分数时显示中性条及 `—`。
- 点击已收到的候选行切换其评分与箭头。每行显示排名、自身深度、评分及带回合编号的 SAN，已收到 / 实际配置数量不会虚构未到达的候选。长文本换行并明确省略，数据模型仍保留每线最多 128 半回合、最多 3 线。仅在 PV 更新时于临时局面逐手验证和缓存 SAN。非法 PV 停在错误半回合，显示错误并禁用该线的评分和箭头。
- PV 是预测，不能视为玩家已走棋谱。MultiPV 共享原有时间预算，多线可能降低搜索深度。落子、悔棋、导入、Replay 跳转/退出、取消及引擎变化会统一使所有候选失效。旧 `analysis.cfg` 无 `multipv` 字段时默认为 1。仅支持标准国际象棋，暂不提供 UCI_Chess960 或任意引擎参数编辑器。
