"""Offline, byte-strict verification of the bounded official Lichess puzzle sample."""
import hashlib
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
COMMIT = '461725966ea46898ec6481372a6326de3281bfbd'
DATA_SHA = 'c372fd943e5ad910deee03a25b73b2a58a0ef2f2c6b2a20a41ba1a1d7c27c1f5'
CARD_SHA = '080193c64118ba9e01ebe04dc96797d595ec57994fd8bb04629e123a498b8eed'
CC0_SHA = 'a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499'
THEMES = ['mateIn1','mateIn2','mateIn3','fork','pin','skewer','discoveredAttack','backRankMate','hangingPiece','sacrifice','promotion','endgame']

def verify(root=ROOT):
    data = root / 'data/chess-puzzles'
    proof = json.loads((data / 'provenance.json').read_bytes())
    if (proof['source'], proof['official'], proof['commit'], proof['license'], proof['attribution'], proof['count'], proof['sha256']) != (
            'https://huggingface.co/datasets/Lichess/chess-puzzles', 'https://database.lichess.org/#puzzles', COMMIT, 'CC0-1.0', 'Lichess Contributors', 512, DATA_SHA):
        raise ValueError('Puzzle provenance identity mismatch')
    for name, sha in [('puzzles.json', DATA_SHA), ('UPSTREAM_README.md', CARD_SHA), ('COPYING.txt', CC0_SHA)]:
        if hashlib.sha256((data / name).read_bytes()).hexdigest() != sha:
            raise ValueError(f'Puzzle input bytes differ: {name}')
    if (proof['license_declaration']['sha256'] != CARD_SHA or proof['cc0_text']['sha256'] != CC0_SHA
            or proof['license_declaration']['url'] != f'https://huggingface.co/datasets/Lichess/chess-puzzles/raw/{COMMIT}/README.md'):
        raise ValueError('Puzzle license evidence mismatch')
    puzzles = json.loads((data / 'puzzles.json').read_bytes())
    if len(puzzles) != 512 or len({p['id'] for p in puzzles}) != 512 or len({p['row'] for p in puzzles}) != 512:
        raise ValueError('Puzzle count/identities mismatch')
    batches = proof['batches']
    if len(batches) != 32 or [b['offset'] for b in batches] != [i * 190000 for i in range(32)]:
        raise ValueError('Puzzle offset selection changed')
    for b in batches:
        if b['length'] != 100 or len(b['sha256']) != 64 or f'revision={COMMIT}' not in b['url']:
            raise ValueError('Unpinned/unbounded puzzle fetch record')
    for p in puzzles:
        if not any(b['offset'] <= p['row'] < b['offset'] + 100 for b in batches):
            raise ValueError('Puzzle source row outside sampled slices')
        if p['popularity'] < 80 or p['deviation'] > 100 or not 2 <= len(p['moves']) <= 16 or not 400 <= p['rating'] <= 3000:
            raise ValueError('Puzzle selection limits differ')
    for t in THEMES:
        count = sum(t in p['themes'] for p in puzzles)
        if count < 16 or proof['theme_counts'][t] != count:
            raise ValueError(f'Missing required theme coverage: {t}')
    if (root / 'LICENSES/chess-puzzles-CC0-1.0.txt').read_bytes() != (data / 'COPYING.txt').read_bytes():
        raise ValueError('Puzzle CC0 distribution mismatch')
    if (root / 'LICENSES/chess-puzzles-provenance.json').read_bytes() != (data / 'provenance.json').read_bytes():
        raise ValueError('Puzzle provenance distribution mismatch')
    return {'puzzles': 512, 'commit': COMMIT, 'sha256': DATA_SHA, 'license': 'CC0-1.0'}

if __name__ == '__main__':
    print(json.dumps(verify(), indent=2))
