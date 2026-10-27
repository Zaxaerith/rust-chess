# RUST CHESS

[简体中文](README_ZH.md)

> Local development candidate: **v0.2.7 Offline Chess Puzzles** (unpublished). See [validation report](RELEASE_CANDIDATE_v0.2.7.md). [v0.2.6](https://github.com/Zaxaerith/rust-chess/releases/tag/v0.2.6) is published.

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

- From v0.2.2: validate source, Release build and `./package.ps1 -SkipBuild -VerifyOnly` locally; do not create a local release ZIP or extracted release directory. Authorized releases are built and verified by GitHub Actions.
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

- Main menu **Open PGN** opens one UTF-8 annotated game in Replay. Since v0.2.4 all nested variations, comments and NAGs are validated and retained; moves remain read-only while metadata can be edited in memory. Limits: 4 MiB, 4096 plies per path, 64 nested RAV levels, 8192 move nodes in total. Use PGN Library to import multiple games; Open PGN still requires one game. Unsupported variants are rejected.

### Advanced PGN Replay (v0.2.4)

- First/Previous/Next/Last and Left/Right/Home/End follow the selected path. At a fork, **Choose variation** lists the current position's continuations; return to its parent position to choose an alternative to a move. Scroll the candidate/path list for longer records. **Parent variation** returns to the nearest fork; **Mainline** restores the mainline position remembered when entering a branch. The current node, SAN, move numbers, NAGs and comments identify the selected path.
- **Play/Pause** navigates Replay only. Speed cycles 0.5×/1×/1.5×/2×/3×/4×, default about one ply/second. Manual jumps, branch selection and editing pause playback; an endpoint stops it. Delayed frames catch up at most 2 seconds/8 plies. There is no move-playing thread, clock advance or persisted active playback.
- **Comment** edits the current move's after-comment; **Before/after** can also edit a game's or variation's initial comment. Root comments precede the game; RAV-leading comments precede its first move; other imported comments belong to the preceding move. This explicit convention survives export without private extensions. Each editable entry is limited to 4096 UTF-8 bytes. Braces and unsafe controls are rejected; CR/LF/tab are supported when pasted. Enter applies, Escape cancels the editor, Ctrl+A selects all, Ctrl+V pastes, arrows/Home/End move the Unicode-safe caret. **Clear** removes the current node's comments (or its NAGs in NAG mode).
- **NAG** edits the ordered space-separated list, with `$0`…`$255` or `! ? !! ?? !? ?!`. At most 64 NAGs/node and 65536 overall are retained; duplicates retain their order. The comment viewport uses up/down controls and is marked as an excerpt; the underlying comments are never clipped. The source PGN is unchanged until an explicitly chosen export is successfully written.
- An unexported marker and a confirmation protect edited comments/NAGs when leaving, replacing a replay or closing the Windows title bar: cancel, export and leave, or explicitly discard. Cancelling a save dialog or a write failure keeps pending edits. Changes are memory-only until export; a forced process termination cannot recover them from autosave.
- Replay **Export PGN** includes the whole annotated tree, nested RAV, comments, NAGs, headers, FEN/SetUp and results. Import→export→import preserves semantics, not whitespace or original glyph spelling. Plain live-game export remains linear. Semicolon comments are normalized to braces; unsafe brace-containing comment text and percent escape lines are explicitly rejected. Graphical directives inside comments are retained as literal text, not interpreted.
- Resource limits also cap total comment text at 1 MiB and comment count at 16384, including empty comments. A conservative tree admission budget charges 2048 bytes/node plus comment text/entries and NAGs, capped at 24 MiB; this is not a promise of total process RSS. Use the v0.2.5 Library for multiple games; branch creation/reordering is not provided.
- Single-position UCI analysis is explicitly disabled on variation nodes in this version. Game Review continues on the original linear mainline in the same engine; branch views hide its curve cursor/quality/score overlays and are labelled mainline analysis only. Returning to the mainline restores linkage. Editing metadata does not change Review cache identity or trigger new searches. No new engine is started for a branch.
- Historical v0.2.4 CLI/Mock/offscreen evidence and its native-dialog/title-bar/real-engine UI boundaries are in [the v0.2.4 report](RELEASE_CANDIDATE_v0.2.4.md). No Computer Use or local ZIP is part of this workflow.
- The live board toolbar provides **Export PGN**, **Replay**, **Copy FEN** and **Load FEN**. Export is available after game over as well. Native dialogs let you choose a path; cancellation leaves the game intact.
- Replay has First/Previous/Next/Last controls, a scrollable clickable move list, and Left/Right/Home/End shortcuts. Back returns to the untouched live game or menu. Clocks and AI replies pause during replay; queued replies remain valid only for their original live position.
- Load FEN accepts the complete six-field standard FEN. Paste with the button or Ctrl+V; Ctrl+A clears the field. Arrow/Home/End keys edit the caret while this field is focused. FEN starts an independent game with fresh clocks/history; it cannot reconstruct previous moves or repetition claims.
- Autosave: `%LOCALAPPDATA%\RUST CHESS\autosave-v1.json`. Schema 1 stores the initial FEN, full moves, clocks and clock snapshots, mode/preferences and terminal reason. A SHA-256 envelope detects corruption. Atomic replacement and `.json.bak` preserve the last valid save. Corrupt/unknown saves are reported and kept; copy them to safety before manually repairing the primary. Failed writes leave recovery `.tmp` files rather than deleting data.
- Save occurs after moves, undo, time gifts, game end, menu/exit, and every five seconds during active play. Offline time is not deducted. At startup choose Resume/New/Not now; no save silently replaces a running game. All current local games are casual.
- File dialogs and the FEN input pause the live clock. A recurring save failure remains visibly marked after dismissing the first error. Only the most recent game is autosaved; use PGN for a permanent archive.
- QA may set `RUST_CHESS_DATA_DIR` for isolated save files; normal storage remains the Windows user data directory. `settings.cfg` retains its prior location and keys.
- Release packaging requires Python 3.11+ to collect and verify the locked dependencies' license texts: `./package.ps1`. Existing release folders/ZIPs are never overwritten. Test tools, configuration, autosaves and PGNs are not distributed.

For the published v0.2.0, Pleco 0.5.0's historical MIT original is verified against its published source commit and bundled with pinned provenance. Its user acceptance and artifact evidence remain in the historical `RELEASE_CANDIDATE_v0.2.0.md`; they are not v0.2.1 acceptance results. Reports stay outside the ZIP to avoid self-referential archive hashes. A new public release requires separate authorization.

## Analysis Board / optional UCI engine (v0.2.2)

- Open **UCI analysis**, choose an independently installed local `.exe`, then explicitly enable it. Unicode/spaces are supported; the executable's directory is its working directory for adjacent NNUE files. No engine is downloaded or bundled, and nothing starts automatically when preferences load.
- **Analyze position** uses the live position or Replay cursor, including its initial FEN and move prefix. Results show White's perspective, depth, best move and full validated PVs in numbered SAN. `M+3` / `M-3` distinguish White / Black mating in 3 moves; `M+0` / `M-0` identify the mated side at distance zero; `>=` / `<=` are score bounds. Centipawn calibration varies by engine and is not a universal win probability.
- Defaults: 2 threads, 64 MiB Hash, 1000 ms, MultiPV 1. Buttons cycle bounded time/threads/hash choices (100–10000 ms, 1–8, 16–256 MiB); changing these resource options closes the engine and requires enabling again. MultiPV cycles 1/2/3 and safely stops/synchronizes the search before reconfiguration and reanalysis, without restarting the engine. An unsupported MultiPV option explicitly falls back to 1; declared engine limits may reduce the count. Supported spin options are clamped to the engine's declared range. Missing optional settings use engine defaults and display a fallback warning.
- **Stop analysis** cancels the real search. **Close engine** shuts down the process. Moves, undo, imports and replay jumps invalidate results. A stop must receive `bestmove` before a replacement search; an unresponsive engine is closed. Analysis never plays a move, pauses the live clock, or replaces Pleco's five AI levels. Replay remains read-only and paused.
- `analysis.cfg` is a separate JSON preference file beside the autosave in `%LOCALAPPDATA%\RUST CHESS` (or the isolated QA data directory). It stores path and resource preferences, not enable permission. It is not part of autosave JSON; `settings.cfg` remains compatible.
- Only run engines you trust and explicitly choose. This is process integration, not a sandbox for third-party programs. With no engine configured, normal play, save and replay remain available. Current real-engine/GUI test evidence and limitations are recorded separately in `RELEASE_CANDIDATE_v0.2.2.md`. No real Stockfish or v0.2.2 manual UI acceptance is claimed.
- Acceptance is CLI/Rust first, with controlled Mock UCI processes; no GUI automation is required. Run `cargo test --locked -j 2 uci_tests` for the analysis suite and `python -B -m unittest discover -s tools -p test_collect_licenses.py` for license/checkout regressions. Complete UI acceptance and real Stockfish testing are recorded separately, not inferred from headless tests. `AGENTS.md` preserves this development convention.

- The vertical evaluation bar and selected-line arrow are view-only. Positive scores always favor White, including on a flipped board. The white fraction is `0.5 + atan(clamp(cp,-4000,4000)/400)/pi`, a bounded visualization, **not** a win probability; mate scores use endpoints and bounds retain `>=` / `<=`. With no valid score it is neutral and marked `—`.
- Click a received candidate row to choose its score/arrow. Rows show rank, their own depth and score, then numbered SAN; the received/effective count never invents missing lines. Long text wraps with an explicit ellipsis; the model retains up to 128 plies per PV and 3 lines. SAN is validated on a temporary position only when the PV changes. Invalid PVs stop at the failing ply, show an error and suppress that line's arrow/score.
- Predictions are not played game moves. MultiPV shares the same time budget, so more candidates may mean less depth. Live moves, undo, import, Replay navigation/exit, cancellation and engine changes invalidate every candidate together. `analysis.cfg` files without `multipv` load as 1. Only standard chess is supported, not UCI_Chess960 or arbitrary engine-specific option editors.

## Game Review (v0.2.3)

1. Open a single PGN, or enter Replay for a local game. In the **UCI** tab, explicitly select and enable your own trusted external engine.
2. Open **Game Review**. Check the range and approximate search budget, then start. Default: 500 ms per position, the first 160 plies (up to 161 positions, about 80.5 seconds plus overhead), MultiPV 1. Time choices are 100/500/1000/2000 ms and range choices 40/80/160/320/512 plies. Threads/Hash retain the enabled engine's configuration (defaults 2/64 MiB). Longer games clearly show a limited prefix, not whole-game completion.
3. **Pause** safely stops the current search; completed positions remain. **Resume** retries the first unfinished position. **Cancel** keeps partial results and releases the engine for manual analysis. A paused job still owns the controller. There is one engine process, never one per position or a second one for manual analysis. Completed means the requested range has been processed, not that every move has enough evidence for a quality rating.
4. Move through Replay while the task runs; its fixed game target does not follow the cursor. Click an analyzed curve point to jump there. Missing/non-exact scores stay gaps; mate and major errors have explicit markers. Switch to UCI for the original PV sidebar; manual search/configuration is available after cancellation/completion. Leaving Replay cancels the job. The live game, clock snapshots and original PGN are unchanged.

The curve is a **bounded White-view advantage indicator, not a win-probability curve**. Move review compares `A(cp) = 2*atan(clamp(cp,-4000,4000)/400)/pi`: White loss is `max(0,A(before)-A(after))`, Black loss reverses the subtraction. Loss ≤0.05 is Good, ≤0.15 Inaccuracy, ≤0.35 Mistake, above 0.35 Blunder. Best additionally requires matching the engine's recommended legal move and loss ≤0.05. Exact cp scores at depth ≥10 on both sides and depth gap ≤6 are required. Mate transitions, terminal results, bounds, low/missing depth, missing positions and inconsistent matching-best evaluations are Unrated. Matching the recommendation remains a fact (`=`), even when not rated. These are budget-limited heuristics, not objective chess theorems; no Brilliant/Book labels are assigned.

Review cache files live in `%LOCALAPPDATA%\RUST CHESS\review-cache-v1\<key>.review.json` (or beneath an explicitly isolated data directory). They are separate from `autosave-v1.json`, `settings.cfg`, `analysis.cfg` and PGN files. Version/checksum, full initial FEN+move history+result identity, engine EXE SHA-256/name/author/path, search settings, range and scoring algorithm prevent unrelated results from mixing. Checkpoints are written atomically every 8 completed positions and on pause/cancel/completion/safe exit, not each frame or each move; each file is capped at 1 MiB and the directory at 32 cache files, with no automatic eviction. Reopen the same PGN, enable the same engine and start review again: completed caches submit no searches, partial caches resume only missing positions. Terminal board results use rule markers instead of fabricated cp scores.

To clear cache, close the app and manually move/delete only the intended `.review.json` files in `review-cache-v1`; do not delete autosaves or PGNs. Corrupt caches are reported and preserved, with in-memory analysis still usable. A full/unwritable cache reports an error rather than deleting history. Small checkpoints use batched synchronous atomic writes; slow disks may briefly delay a checkpoint. External NNUE/configuration files are not fingerprinted: after changing them, manually clear affected caches. Executable fingerprinting runs on the existing UCI worker (256 MiB EXE limit). There is no downloaded/bundled engine, real Stockfish quality/NNUE/resource validation or v0.2.3 manual GUI acceptance in this candidate. See [the report](RELEASE_CANDIDATE_v0.2.3.md) for separate Mock and offscreen evidence.

### PGN Library (v0.2.5)

- Open **PGN Library** from the main menu and choose **Import PGN**. One native file picker accepts one file containing one or many games; repeat to append other files. Import runs on one bounded worker, displays processed count, and reports imported/duplicate/failed counts. On the first invalid game or archive failure, earlier committed games remain; later games are explicitly unprocessed. Invalid UTF-8 is an input-file error before any archive writes.
- Limits: 32 MiB input/normalized collection, 200 games per import, 2000 indexed games and 4 MiB index. Comments between a termination token and the next game start are retained as the preceding game's trailing comments. Each game retains all v0.2.4 limits (4 MiB, 4096 plies per path, 64 RAV levels, 8192 nodes and annotation limits). Only one complete validated tree is archived at a time; the list holds metadata rather than all boards.
- Stored in `%LOCALAPPDATA%\RUST CHESS\library-v1\` (or `RUST_CHESS_DATA_DIR/library-v1` for explicit tests). `index.json` contains metadata/favorites/content hashes; `games/<stable-id>-<sha256>.pgn` contains a normal annotated PGN. Nothing is written to the imported source, autosave schema, settings or Review Cache. No parsing/scanning runs at startup; the index loads when Library opens, and an individual PGN loads on Open.
- Search matches White, Black, Event, Date or Result without case sensitivity. Favorites-only and result filters combine with search. Cycle Sort through recent import, PGN date text, players and favorites first. Only visible rows are drawn; use the wheel or up/down controls. Long fields are shortened for display. Click search to type Unicode, Backspace to remove a character and Ctrl+A to clear; Enter ends input, then Enter opens the selection.
- **Toggle favorite** persists only the index. **Open Replay** retains all variations, notes, NAGs, autoplay and the existing analysis restrictions. Back returns to the same query/filter/selection/scroll. Game Review still uses the original mainline and its existing cache identity; Library IDs and favorites do not trigger engine searches.
- In Replay, **Save to Library** updates the managed copy and index under the same stable ID. **Add to Library** archives an external/live Replay and then associates it with the saved copy. **Export PGN/Export copy** writes an independently selected file; it is not a library update. Export inside the application data directory is refused to protect managed files. Changes are marked saved only after success, and the existing unsaved-exit guard remains in force. Exporting a copy also satisfies that guard; save to Library explicitly if its copy should change.
- Identical canonical annotated PGN is skipped by SHA-256. The same mainline with different tags, comments, NAGs or variations is retained separately. A save that would duplicate another entry reports a conflict instead of replacing it.
- **Remove** requires confirmation and removes only the selected managed PGN/index entry. The external source, autosave and Review Cache remain. There is no automatic eviction, sync, online service or bulk cleanup.
- Writes stage a new PGN revision then atomically switch the index. The previous revision is removed only after index success. An interrupted write may leave an unindexed revision/temp file; no recovery value is silently deleted. **Check/recover** explicitly scans at most 4000 managed directory entries, reports missing/invalid files and conflicting old revisions, and reindexes valid unclaimed copies. Files with conflicting IDs/hashes are retained for manual inspection. Check may take time on slow storage, on the Library screen where live clocks are paused.
- A corrupt/unknown index is never overwritten automatically. Quit, preserve/rename `index.json` yourself, restart and run Check/recover to rebuild from valid PGN files. Favorites and original import timestamps cannot be recovered from PGNs alone. External index changes are rejected on the next write; reopen the application to reload. Run one writing instance at a time; multi-process database transactions are outside scope.
- Standard chess only; no online sync or multi-file picker yet. The current tests use isolated directories and Mock UCI, not real Stockfish or native-window automation. See the v0.2.5 candidate report for exact evidence and remaining manual UI checks.

### Offline Opening Explorer (v0.2.6)

- Open **Opening Explorer** from the menu or PGN Library. The existing board is read-only; Previous/Next/Start (or Left/Right/Home) navigate its independent path. Choose a continuation, search an ECO code/English name and click a reference route, or switch to related games. Opening a related Library game uses Advanced Replay at its first matching mainline position; Back restores the Explorer path. Unsaved annotation guards still apply.
- Reference names come from [Lichess chess-openings](https://github.com/lichess-org/chess-openings/tree/a6189a30dc273ccb21fc2536a9a2fefd5592a67a), **3,865 CC0-1.0 entries**, pinned to the linked commit. Five TSVs are embedded; no external runtime dataset, engine or network is needed. The original license/provenance is distributed under LICENSES. This is a name/classification reference, not engine analysis or a global match database.
- Legal SAN is validated with the existing parser/shakmaty, then indexed by placement/turn/castling/**legally capturable en-passant** position fields. Move counters are excluded so transpositions merge. Exact reference history is preferred if several names share a position; otherwise source order a.tsv..e.tsv, then original row order, decides stably. Candidate count is shown and Details lists up to eight candidates; ambiguity is not claimed as unique truth. The pinned dataset currently has unique named terminal keys; controlled fixtures test the multi-name rule.
- Game classification chooses its last reached named position, and is Unknown without a match. PGN ECO tags are not trusted. Games starting from nonstandard six-field FEN remain unclassified as standard opening histories, although their real positions may contribute samples to identical local positions; those samples are counted separately as nonstandard FEN.
- **Build statistics** scans your current Library, one validated game at a time, using only its **mainline first 40 plies / positions 0..40**. Variations are not match samples. A game contributes once per position, at its first arrival, and only that arrival's next move; terminal/prefix-boundary samples may have no continuation. The move frequency denominator is samples with a next move, which can differ from the total games at the position.
- W/D/L percentages use only `1-0`, `1/2-1/2`, `0-1`; `*` is unfinished and remains visible separately. Counts, total and decided denominator are shown. This is **local historical W/D/L, not engine-predicted win probability**. Empty Library or absent positions say no samples; reference continuations still work and never invent global totals or 0% win rates.
- The first Explorer visit builds the small reference index off the UI thread and attempts a matching cache load; normal program startup does neither. A single cancellable worker builds statistics, with per-game progress. Serialization and staging writes also run in the worker; only a final atomic rename happens after the UI rechecks the snapshot. Cancel/Back stops publication; previous valid cache is retained on cancellation/failure. A changed Library snapshot cannot overwrite current statistics as if fresh. Threads are cooperatively stopped/joined during normal exit with a 1-second deadline; unusually stalled OS file IO can exceed that deadline and is logged, not blocked forever.
- Cache: `%LOCALAPPDATA%\RUST CHESS\opening-stats-v1.json` (explicit tests use `RUST_CHESS_DATA_DIR`). A checksum-protected schema-1 JSON, capped at 32 MiB, records algorithm/source/normalization/prefix identity and sorted Library ID/content hashes. Add/remove/save invalidates; favorite-only changes do not. Full annotated-content hashes conservatively mean comment-only saves also invalidate statistics, while Game Review's own identity remains unchanged. Rebuild explicitly after changes. Damaged/unknown cache reports an error and is retained; an explicit successful rebuild replaces only this statistics cache. Cache write errors leave completed results available in memory.
- Neither Explorer navigation nor statistics alters PGNs, Library schema, live clocks/history, autosave or Review Cache; no UCI engine is started. Statistics are not an online database or Stockfish opening recommendation. Standard chess only, 160 navigable plies per Explorer path, at most the existing 2,000 Library entries. Large local libraries/slow disks take time; cancel is cooperative between bounded work units. Native/manual UI acceptance remains separate from CLI/Mock/offscreen evidence in the candidate report.


## Offline Chess Puzzles (v0.2.7)

- Main menu → **Chess Puzzles**. The executable embeds 512 validated Lichess CC0 puzzles;
  no account, network connection, Python or external engine is needed for training.
- Choose Theme, Rating range and Mixed / Unseen first / Retry errors. These buttons cycle
  their finite options. Official theme identifiers remain recognizable across languages.
  Show themes is optional because themes can reveal the tactic. Empty filters show no matches.
- The initial opponent move is already played. Click or drag your pieces; the board faces
  the solving side. Correct moves advance along the verified answer; the prescribed opponent
  reply runs on the next UI tick. Always choose the actual promotion piece, including a knight.
- Hint first marks the source, then the destination, separately for each answer turn.
  Reveal opens read-only answer replay; Previous/Next or arrow keys navigate it. Details shows
  the full reference SAN. Retry starts this puzzle again; Next/Back records an unfinished attempt
  as Skipped. A different legal move is described as outside the reference, not objectively bad.
  For mateIn1, any verified immediate checkmate is accepted.
- Results distinguish First try (no hint/error), Assisted (hint or prior mismatch), Revealed,
  and Skipped. Attempts include retries; `#` counts distinct attempted IDs and `✓#` distinct
  solved IDs, so retries cannot inflate unique completion. The percentage is first-try attempts
  divided by all recorded attempts, including skipped/revealed, not a player rating.
- Progress: `%LOCALAPPDATA%\RUST CHESS\puzzle-progress-v1.json`; optional packs:
  `%LOCALAPPDATA%\RUST CHESS\puzzles-v1\user.json`. Both respect `RUST_CHESS_DATA_DIR`.
  Back up these two independent files while the app is closed. Corrupt/unsupported files are
  preserved and reported; progress writing is disabled rather than overwriting damaged history.
- Import CSV selects a small, uncompressed UTF-8 Lichess CSV (BOM/CRLF/quoted fields supported).
  Required columns: PuzzleId,FEN,Moves,Rating,RatingDeviation,Popularity,NbPlays,Themes.
  GameUrl,OpeningTags,DailyDate and other optional columns may be present. Limit16MiB/20000rows;
  each line2..32plies, even length, all legal; user collection total20000 and32MiB. Select a
  subset yourself before importing a larger official export; `.zst` is not supported.
  A single worker validates all input, then atomically merges by PuzzleId. Identical IDs are
  skipped without replacing existing puzzles or progress. Any invalid record aborts the whole
  import with its ID/record and zero archived; cancellation before commit leaves storage intact.
  Errors and added/duplicate counts appear in Details; source CSVs are never modified.
- Training owns its chess state, timer and storage. It cannot make real-game moves, run Pleco/UCI,
  or change Library, Opening statistics, Review Cache or autosave schema. No pack is bundled engine
  output newly calculated by this app; Lichess puzzle Rating is not your personal rating.
- [Pinned source, selection, hashes and Viewer revision limitation](data/chess-puzzles/README.md).
  This version has CLI/offscreen validation; native CSV dialogs and player UI need human playtesting.
