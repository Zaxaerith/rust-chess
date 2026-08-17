# 第三方声明

本项目在 GPL-3.0-or-later 协议下发布，同时使用以下第三方组件。各组件仍保留
其自身的版权与许可证。

| 组件 | 版本 | 许可证 | 来源 |
| --- | --- | --- | --- |
| shakmaty | 0.27 | GPL-3.0+ | https://github.com/niklasf/shakmaty |
| pleco | 0.5 | MIT | https://github.com/pleco-rs/Pleco |
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
