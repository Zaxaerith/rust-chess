# RUST CHESS

[简体中文](README_ZH.md)

> Local development candidate: **v0.2.1 UCI Engine Integration** (unpublished). See [validation report](RELEASE_CANDIDATE_v0.2.1.md). [v0.2.0](https://github.com/Zaxaerith/rust-chess/releases/tag/v0.2.0) is already published.

A local window chess game written in Rust. Supports two-player, human vs AI, 5 AI difficulty levels, move animations, AI thinking pause, opening/ending animations, settings persistence, and multiple VSCode-inspired themes. v0.1.4 adds clock-based game results, clock-aware undo, explicit AI search errors, and regression tests.

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

Clock time continues during move confirmation and promotion selection. Opening menus/settings pauses it. Undo restores the saved clocks for the previous position (two plies after an AI reply). Time gifts cannot reopen a finished game. All current local games are casual, so Casual and Always access policies both permit the action. The root executable is a historical build; use the newly built target/release/chess_rust.exe for this candidate.

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
- `chasolver` / `chess`: timeout mating-possibility adjudication (MIT); no change to the playing engine
- `pleco`: AI searcher for human vs AI (Rust port of Stockfish)
- `png`: PNG decoding for piece images
- `ab_glyph`: text rendering (uses system fonts, including Chinese)

## Project Structure

```text
src/
  main.rs       entry point and game logic
  game.rs       game results, timeout adjudication and clock snapshots
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

## Game records and replay (v0.2.0)

- Main menu **Open PGN** opens one UTF-8 game in read-only replay. File limit: 4 MiB, 4096 mainline plies, variation depth 64. Comments, NAGs and variations are intentionally omitted with a notice; exports contain only the mainline. Multiple games and unsupported variants are rejected.
- The live board toolbar provides **Export PGN**, **Replay**, **Copy FEN** and **Load FEN**. Export is available after game over as well. Native dialogs let you choose a path; cancellation leaves the game intact.
- Replay has First/Previous/Next/Last controls, a scrollable clickable move list, and Left/Right/Home/End shortcuts. Back returns to the untouched live game or menu. Clocks and AI replies pause during replay; queued replies remain valid only for their original live position.
- Load FEN accepts the complete six-field standard FEN. Paste with the button or Ctrl+V; Ctrl+A clears the field. Arrow/Home/End keys edit the caret while this field is focused. FEN starts an independent game with fresh clocks/history; it cannot reconstruct previous moves or repetition claims.
- Autosave: `%LOCALAPPDATA%\RUST CHESS\autosave-v1.json`. Schema 1 stores the initial FEN, full moves, clocks and clock snapshots, mode/preferences and terminal reason. A SHA-256 envelope detects corruption. Atomic replacement and `.json.bak` preserve the last valid save. Corrupt/unknown saves are reported and kept; copy them to safety before manually repairing the primary. Failed writes leave recovery `.tmp` files rather than deleting data.
- Save occurs after moves, undo, time gifts, game end, menu/exit, and every five seconds during active play. Offline time is not deducted. At startup choose Resume/New/Not now; no save silently replaces a running game. All current local games are casual.
- File dialogs and the FEN input pause the live clock. A recurring save failure remains visibly marked after dismissing the first error. Only the most recent game is autosaved; use PGN for a permanent archive.
- QA may set `RUST_CHESS_DATA_DIR` for isolated save files; normal storage remains the Windows user data directory. `settings.cfg` retains its prior location and keys.
- Release packaging requires Python 3.11+ to collect and verify the locked dependencies' license texts: `./package.ps1`. Existing release folders/ZIPs are never overwritten. Test tools, configuration, autosaves and PGNs are not distributed.

For the published v0.2.0, Pleco 0.5.0's historical MIT original is verified against its published source commit and bundled with pinned provenance. Its user acceptance and artifact evidence remain in the historical `RELEASE_CANDIDATE_v0.2.0.md`; they are not v0.2.1 acceptance results. Reports stay outside the ZIP to avoid self-referential archive hashes. A new public release requires separate authorization.

## Optional external UCI analysis (v0.2.1)

- Open **UCI analysis**, choose an independently installed local `.exe`, then explicitly enable it. Unicode/spaces are supported; the executable's directory is its working directory for adjacent NNUE files. No engine is downloaded or bundled, and nothing starts automatically when preferences load.
- **Analyze position** uses the live position or Replay cursor, including its initial FEN and move prefix. Results show White's perspective, depth, best move and first PV move in SAN. `#` means mate; `>=` / `<=` are score bounds. Centipawn calibration varies by engine and is not a universal win probability.
- Defaults: 2 threads, 64 MiB Hash, 1000 ms, MultiPV 1. Buttons cycle bounded time/threads/hash choices (100–10000 ms, 1–8, 16–256 MiB); changing options closes the engine and requires enabling again. Supported spin options are clamped to the engine's declared range. Missing optional settings use engine defaults and display a fallback warning.
- **Stop analysis** cancels the real search. **Close engine** shuts down the process. Moves, undo, imports and replay jumps invalidate results. A stop must receive `bestmove` before a replacement search; an unresponsive engine is closed. Analysis never plays a move, pauses the live clock, or replaces Pleco's five AI levels. Replay remains read-only and paused.
- `analysis.cfg` is a separate JSON preference file beside the autosave in `%LOCALAPPDATA%\RUST CHESS` (or the isolated QA data directory). It stores path and resource preferences, not enable permission. It is not part of autosave JSON; `settings.cfg` remains compatible.
- Only run engines you trust and explicitly choose. This is process integration, not a sandbox for third-party programs. With no engine configured, normal play, save and replay remain available. Current real-engine/GUI test evidence and limitations are recorded separately in `RELEASE_CANDIDATE_v0.2.1.md`.
- Acceptance is CLI/Rust first, with controlled Mock UCI processes; no GUI automation is required. Run `cargo test --locked -j 2 uci_tests` for the analysis suite and `python -B -m unittest discover -s tools -p test_collect_licenses.py` for license/checkout regressions. Complete UI acceptance and real Stockfish testing are recorded separately, not inferred from headless tests. `AGENTS.md` preserves this development convention.
