# RUST CHESS

[简体中文](README_ZH.md)

> Current stable version: **v0.1.3** — source code and releases are available on `main`.

A local window chess game written in Rust. Supports two-player, human vs AI, 5 AI difficulty levels, move animations, AI thinking pause, opening/ending animations, settings persistence, and multiple VSCode-inspired themes. v0.1.3 refactors board-related preferences into three-tier settings and adds drag-and-drop, premoves, move confirmation, and a local chess clock.

## Features

- Two-player / Human vs AI (AI plays white or black)
- 5 AI difficulty levels: Beginner / Easy / Medium / Hard / Master
- Legal move hints, check alerts, promotion selection, castling, en passant
- Undo, new game, move history
- Smooth sliding move animations, AI thinking delay, game-end summary panel
- Settings persistence (saved to `settings.cfg` in the same directory as the executable)
- Resolution/refresh rate options automatically detected from system display modes
- 10 themes: Dark+ / Light+ / Monokai / Solarized / Nord / Abyss / Kimbie Dark / Quiet Light / Tomorrow Night Blue / Red
- 11 languages: 简体中文, 繁體中文, English, Français, Español, Latina, 日本語, 한국어, Deutsch, Italiano, Русский
- 5 piece sets: CBurnett / Merida / Chessnut / RhosGFX / Fantasy
- 6 board skins: Classic / Walnut / Ocean / Emerald / Marble / Graphite
- Optional auto-flip board when playing black
- In-game move suggestions based on the Pleco search engine
- Left vertical main menu with animated rotating gear logo
- Three-tier board settings: Interface / Game Behavior / Chess Clock
- Piece drag-and-drop, enlarged dragging, circular or square landing markers
- Premoves, auto-queen promotion, move confirmation, and two castling input methods
- Local 10-minute chess clock, critical time alerts, tenth-second display, and time-add for opponent

## Run Directly

The `国际象棋.exe` file in the project root is a fully self-contained executable – piece assets are embedded, so double-click to run without any additional installation.

## Build from Source

You need the [Rust toolchain](https://www.rust-lang.org/tools/install) (stable is fine).

```powershell
cargo build --release
```

The build output is `target/release/chess_rust.exe`, you can copy it anywhere and run.

## Packaging for Release

- Local manual packaging: run .\package.ps1, output goes to dist/
- Automatic GitHub release: push a tag starting with v*, and Actions will automatically compile, package, and create a Release – see RELEASE.md

## VSCode Development

1. Open the project root in VSCode.
2. Install the recommended rust-analyzer extension.
3. Press Ctrl+Shift+B to build the release version, or press F5 to run the game.

You can also run from the terminal:

```powershell
cargo run --release
```

## How to Play

- Click your own piece to select it, then click a target square to move.
- Click the selected piece or an empty square to deselect.
- When a pawn reaches the last rank, choose the promotion piece (Queen / Rook / Bishop / Knight).
- The right panel provides New Game, Undo, and Return to Main Menu.
- Press `Esc` to exit the game.

## Settings

- Game mode: Two-player / AI plays white / AI plays black
- AI difficulty: Beginner / Easy / Medium / Hard / Master
- Window resolution: automatically reads supported display modes
- Refresh rate: automatically reads supported refresh rates
- Theme: 10 VSCode-style colour schemes
- Language: 简体中文, 繁體中文, English, Français, Español, Latina, 日本語, 한국어, Deutsch, Italiano, Русский
- Board skin: Classic / Walnut / Ocean / Emerald / Marble / Graphite
- Piece set: CBurnett / Merida / Chessnut / RhosGFX / Fantasy
- Flip board when playing black: On / Off

Settings take effect immediately and are saved to `settings.cfg`, they will be restored on next launch.

## Technology Stack

- `minifb`: local window and input
- `shakmaty`: chess rule engine
- `pleco`: AI searcher for human vs AI (Rust port of Stockfish)
- `png`: PNG decoding for piece images
- `ab_glyph`: text rendering (uses system fonts, including Chinese)

## Project Structure

```text
src/
  main.rs       entry point and game logic
  render.rs     board, menu, settings, animation rendering
  ai.rs         Pleco AI difficulty mapping and suggestion search
  i18n.rs       interface texts for 11 languages
  theme.rs      UI themes and board colours
  display.rs    reads system resolution / refresh rate
  settings.rs   settings save/load
  font.rs       text rendering
  assets.rs     piece image decoding
assets/
  piece-svg/    open‑source SVG piece assets
  pieces/       PNGs used at runtime (generated from SVGs)
  ATTRIBUTION.md  piece asset licence notices
```

## License

This project is licensed under GPL-3.0-or-later – see LICENSE.

We chose this license because the rule engine shakmaty is GPL‑3.0+, so we release the entire project under the compatible GPL‑3.0‑or‑later.

Third‑party dependencies, assets and copyright notices are detailed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [assets/ATTRIBUTION.md](assets/ATTRIBUTION.md).

## Acknowledgements

- [shakmaty](https://github.com/niklasf/shakmaty): chess rule engine
- [Pleco](https://github.com/pleco-rs/Pleco): Rust port of Stockfish, provides AI search
- [Lichess](https://github.com/lichess-org/lila)Lichess: open‑source piece assets and licence information
- [Visual Studio Code](https://github.com/microsoft/vscode)Visual Studio Code: UI theme colour references