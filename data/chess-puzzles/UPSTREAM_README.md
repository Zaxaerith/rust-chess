---
license: cc0-1.0
size_categories:
- 1M<n<10M
pretty_name: Lichess puzzles
dataset_info:
  features:
  - name: PuzzleId
    dtype: large_string
  - name: GameId
    dtype: string
  - name: FEN
    dtype: large_string
  - name: Moves
    dtype: large_string
  - name: Rating
    dtype: uint16
  - name: RatingDeviation
    dtype: uint16
  - name: Popularity
    dtype: int8
  - name: NbPlays
    dtype: uint32
  - name: Themes
    list: string
  - name: OpeningTags
    list: string
  splits:
  - name: train
    num_bytes: 1256607736
    num_examples: 6157341
  download_size: 884570255
  dataset_size: 1256607736
configs:
- config_name: default
  data_files:
  - split: train
    path: data/train-*
tags:
- chess
- lichess
- puzzles
---
# Dataset Card for Lichess Puzzles

<!-- Provide a quick summary of the dataset. -->
## Dataset Description

**6,157,341 puzzles**, rated and tagged. See them in action on [Lichess](https://lichess.org/training/themes). 

This dataset is updated monthly, and was last updated on October 7th, 2026.

### Dataset Creation
Generating the initial dataset chess puzzles took more than **50 years of CPU time**. We went through **300,000,000 analyzed games** from the Lichess database, and re-analyzed interesting positions with Stockfish 12/13/14/15 NNUE at 40 meganodes. The resulting puzzles were then [automatically tagged](https://github.com/ornicar/lichess-puzzler/tree/master/tagger). To determine the rating, each attempt to solve is considered as a Glicko-2 rated game between the player and the puzzle. Finally, player votes refine the tags and define popularity.


### Dataset Usage

Using the `datasets` library:

```python
from datasets import load_dataset
dset = load_dataset("Lichess/chess-puzzles", split="train")
```

## Dataset Details

### Dataset Sample

One row of the dataset looks like this:

```python
{'PuzzleId': '000hf',
 'GameId': '71ygsFeE/black#38',
 'FEN': 'r1bqk2r/pp1nbNp1/2p1p2p/8/2BP4/1PN3P1/P3QP1P/3R1RK1 b kq - 0 19',
 'Moves': 'e8f7 e2e6 f7f8 e6f7',
 'Rating': 1575,
 'RatingDeviation': 75,
 'Popularity': 92,
 'NbPlays': 674,
 'Themes': ['mate', 'mateIn2', 'middlegame', 'short'],
 'OpeningTags': ['Horwitz_Defense', 'Horwitz_Defense_Other_variations']}
```
### Dataset Fields

Every row of the dataset contains the following fields:

- **`PuzzleId`**: `string`, the puzzle's unique identifier. The puzzle would be live at `https://lichess.org/training/{PuzzleID}`.
- **`GameId`**: `string`, the unique identifier of the specific game and move the puzzle was extracted from. The game would be accessible at `https://lichess.org/{GameId}`
- **`FEN`**: `string`, the FEN string of the position before the opponent makes their move.
- **`Moves`**: `string`, the solution to the puzzle. All player moves of the solution are "only moves", i.e. playing any other move would considerably worsen the player position. An exception is made for mates in one: there can be several. Any move that checkmates should win the puzzle.
- **`Rating`**: `int`, the Glicko-2 rating of the puzzle.
- **`RatingDeviation`**: `int`, the Glicko-2 rating deviation of the puzzle.
- **`Popularity`**: `int`, a number between `100` (best) and `-100` (worst), calculated as `100 * (upvotes - downvotes)/(upvotes + downvotes)`. Votes are weighted by various factors such as whether the puzzle was solved successfully or the solver's puzzle rating in comparison to the puzzle's.
- **`NbPlays`**: `int`, the number of times a puzzle was played.
- **`Themes`**: `list`, a list of puzzle themes.
- **`OpeningTags`**: `list`, a list of openings. This is only set for puzzles starting before move 20.

## Additional Information
- For a list of all possible puzzle themes and their description: [puzzleTheme.xml](https://github.com/ornicar/lila/blob/master/translation/source/puzzleTheme.xml)
- To better understand puzzle themes, check out this study: https://lichess.org/study/viiWlKjv
- GitHub Repo: https://github.com/lichess-org/database
- Official Website: https://database.lichess.org/#puzzles
- For a list of all possible openings: https://github.com/lichess-org/chess-openings
- For the blog post introducing puzzles from 2020: [blog/new-puzzles-are-here](https://lichess.org/@/lichess/blog/new-puzzles-are-here/X-S6gRUA)
- For the blog post analyzing Lichess puzzle usage from 2021: [blog/some-puzzling-analysis](https://lichess.org/@/Lichess/blog/some-puzzling-analysis/YAMFfhUA)