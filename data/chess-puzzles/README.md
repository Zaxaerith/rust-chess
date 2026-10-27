# Embedded Lichess puzzle sample

Source: https://huggingface.co/datasets/Lichess/chess-puzzles (official Lichess organization).
Official description/dedication: https://database.lichess.org/#puzzles.
Repository snapshot: `461725966ea46898ec6481372a6326de3281bfbd` (2026-10-07 update).
License: CC0-1.0, Lichess Contributors. No affiliation or endorsement implied.

`puzzles.json` contains 512 real puzzles; no generated puzzles. Each `row` is the original
Viewer train row index, and `id` is the original PuzzleId. The mirror uses a `GameId`
with side/ply suffix, not CSV `GameUrl`: `url = https://lichess.org/ + GameId`.
Themes are copied from its string array. RatingDeviation becomes `deviation`.
Moves are split on whitespace, without changing their order. The first move belongs
to the opponent; the interactive player starts after that move.

Bounded selection: 32 `/rows` requests, offsets `i * 190000`, i=0..31, each length100.
A failed request was retried; successful responses were reused. No Parquet, full CSV,
compressed database, engine or executable was downloaded. Responses were <2MiB each.
The selected rows have Popularity>=80, RatingDeviation<=100, 2..16 answer plies and
Rating400..3000. For the 12 required themes (in provenance order), take up to16 rows
sorted by popularity descending, deviation ascending, source row ascending. Then fill
round-robin from rating bins [400,1000), [1000,1400), [1400,1800), [1800,2200), [2200,3001),
using the same sort and unique PuzzleId. Sort final rows by source index. Every selected
FEN and entire answer line is validated by Rust/shakmaty; all512 pass.

The Viewer accepts a revision query, but **does not return a revision response header**.
Therefore we record the actual API repository HEAD before/after extraction, each response
SHA-256, original IDs/rows and final immutable bytes. We do not claim that an undocumented
query parameter alone proves the served revision. `tools/fetch_puzzle_sample.py --download`
can reproduce to ignored target/ only while HEAD and every response hash still match;
it stops on any mismatch. There is no network request at runtime or in normal CI gates.
`tools/verify_puzzle_data.py` independently pins the final hash, card, CC0 text and coverage.

The original pinned dataset card is UPSTREAM_README.md. COPYING.txt is the unmodified
formal CC0 1.0 text already distributed by Lichess chess-openings; the puzzle dedication
is independently declared by the pinned puzzle card and official database page.
Provenance and CC0 are also distributed under LICENSES/. The application remains GPL.

Theme counts (overlapping): mateIn1 77, mateIn2 78, mateIn3 26, fork59, pin37, skewer16,
discoveredAttack22, backRankMate17, hangingPiece19, sacrifice51, promotion18, endgame264.
Rating bins above:127/131/97/83/74. This is a small, deliberately balanced sample, not a
representative global rating distribution. Ratings belong to puzzles, not the player.
