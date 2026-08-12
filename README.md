# Rust 国际象棋

一个用 Rust 编写的本地窗口国际象棋游戏。支持双人对战、人机对战、5 档 AI
难度、走棋动画、AI 思考停顿、开局结束动画、设置记忆，以及多套仿 VSCode
主题。

## 功能

- 双人对战 / 人机对战（人机执白或执黑）
- AI 难度 5 档：入门 / 简单 / 中等 / 困难 / 大师
- 合法走法提示、将军提示、升变选择、王车易位、吃过路兵
- 悔棋、新对局、棋谱记录
- 走棋滑动动画、AI 思考停顿、对局结束结算面板
- 游戏设置记忆（保存到 exe 同目录的 `settings.cfg`）
- 分辨率/刷新率选项自动读取系统显示模式
- 5 套主题：Dark+ / Light+ / Monokai / Solarized / Nord

## 直接运行

项目根目录的 `国际象棋.exe` 是完整可执行文件，棋子素材已嵌入 exe，
双击即可运行，不需要额外安装任何东西。

## 从源码编译

需要 [Rust 工具链](https://www.rust-lang.org/tools/install)（stable 即可）。

```powershell
cargo build --release
```

编译产物位于 `target/release/chess_rust.exe`，可以复制到任意位置直接运行。

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
- 主题：Dark+ / Light+ / Monokai / Solarized / Nord

设置会立即生效并保存到 `settings.cfg`，下次启动自动恢复。

## 技术栈

- `minifb`：本地窗口与输入
- `shakmaty`：国际象棋规则引擎
- `pleco`：人机对战的 AI 搜索器（Rust 重写的 Stockfish）
- `png`：棋子 PNG 解码
- `ab_glyph`：文字渲染（使用系统中文字体）

## 项目结构

```text
src/
  main.rs       程序入口与对局逻辑
  render.rs     棋盘、菜单、设置、动画渲染
  ai.rs         Pleco AI 难度映射
  theme.rs      主题配色
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
- [chess-viewer](https://github.com/chessviewer-org/chess-viewer)：开源棋子素材来源
- [lichess](https://lichess.org)：棋子素材原始项目
