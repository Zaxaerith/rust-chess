# 第三方声明

本项目在 GPL-3.0-or-later 协议下发布，同时使用以下第三方组件。各组件仍保留
其自身的版权与许可证。

| 组件 | 版本 | 许可证 | 来源 |
| --- | --- | --- | --- |
| shakmaty | 0.27 | GPL-3.0+ | https://github.com/niklasf/shakmaty |
| pleco | 0.5 | MIT | https://github.com/pleco-rs/Pleco |
| chasolver | 3.0.0 | MIT | https://github.com/miguel-ambrona/chasolver |
| chess | 3.2.0 | MIT | https://github.com/jordanbray/chess |
| minifb | 0.27 | MIT OR Apache-2.0 | https://github.com/emoon/minifb |
| png | 0.17 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| ab_glyph | 0.2 | Apache-2.0 | https://github.com/alexheretic/ab-glyph |
| winapi | 0.3 | MIT/Apache-2.0 | https://github.com/retep998/winapi-rs |

## 素材

棋子素材来自开源项目：

- CBurnett 棋子：Colin M.L. Burnett，GPLv2+，来源为
  [lichess-org/lila](https://github.com/lichess-org/lila)
- Merida 棋子：Armando Hernandez Marroquin，GPLv2+
- Chessnut 棋子：Alexis Luengas，Apache-2.0
- RhosGFX 棋子：RhosGFX，CC0-1.0
- Fantasy 棋子：Maurizio Monge，MIT
- 完整版权表见 [assets/ATTRIBUTION.md](assets/ATTRIBUTION.md)

## 特别说明

- `pleco` 本身采用 MIT 协议，其算法来自 Stockfish，感谢 Stockfish 作者。
- v0.1.1 的建议高亮交互参考了 Lichess Chessground 的开源棋盘设计思路；本项目未复制或链接其代码。
- v0.1.2 的新增界面主题参考 Microsoft Visual Studio Code 官方内置主题；VS Code 源码采用 MIT 许可证。
- 六套棋盘皮肤为本项目自行定义的纯色调色板，未复制 Lichess 的 AGPL 棋盘纹理。
- 运行时使用 Windows 系统字体渲染拉丁、西里尔及中日韩文字，字体不随项目分发。


v0.1.4 使用 CHA-Solver 判定超时后是否存在合作将死序列；它只参与规则结算，AI 对弈仍由 Pleco 提供。新增组件版权及 MIT 许可全文见 [CHA-Solver](LICENSES/chasolver-MIT.txt) 与 [chess](LICENSES/chess-MIT.txt)。


## v0.2.0 棋谱与存档依赖

- `pgn-reader 0.26.0`：GPL-3.0-or-later；直接依赖既有 `shakmaty 0.27`，未引入第二套 shakmaty。
- `serde` / `serde_json`：MIT OR Apache-2.0；`sha2`：MIT OR Apache-2.0。实际锁定版本、作者许可全文及传递依赖见 [依赖索引](LICENSES/dependencies/INDEX.md) 和 `LICENSES/dependencies/manifest.json`。
- 索引包含锁定的 Windows 构建/运行依赖；保留所有可用 LICENSE、NOTICE、COPYRIGHT 文件，Unicode 的 AND 许可条件亦随包保留。构建工具只在打包时使用，运行程序不依赖 Python。
- 上游缺失文件的处理在索引逐项标记：failure_derive 使用同版本 failure 工作区许可；mucow 原发布包只带 MIT OR Apache-2.0 声明，本包选择 Apache-2.0 并附标准许可全文和原始声明，不冒称为上游独立文件。
- **Pleco 0.5.0 已核实**：锁定发布包的源码提交为 `7122ae552507abd3853c5ad6ed37cc36637a0831`，该提交的 `pleco/LICENSE` 与 2017 年历史原件逐字节相同，版权为 `Copyright (c) 2017 Stephen Fleischman`。原件见 [MIT 全文](LICENSES/dependencies/pleco-0.5.0/LICENSE-MIT)，来源 URL、提交、Git blob、SHA-256 及适用性见 [核实记录](LICENSES/dependencies/pleco-0.5.0/pleco-0.5.0-provenance.json)。2022 年更新只改版权行，未用于替换此版本原件。当前上游仓库根目录 GPL 与此历史库版本的 MIT 分开核实，RUST CHESS 的 GPL-3.0-or-later 不变。

## v0.2.1 外部分析

UCI 通信使用 Rust 标准库，不新增 Rust 依赖。发行包不包含 Stockfish 或其他外部引擎、NNUE 文件、模拟测试引擎及用户引擎配置；用户自行取得外部引擎并遵循其许可。RUST CHESS 保持 GPL-3.0-or-later。第三方文件按原始字节保留，Git 禁止 LICENSES 全树的行尾转换，严格 SHA-256 校验继续生效。
