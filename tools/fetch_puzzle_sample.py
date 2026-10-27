"""Explicit opt-in reproduction; only 32 x 100 official rows, never a database download.
The viewer does not prove revision in a response header: require the pinned repository
HEAD before/after AND every captured response SHA-256. Future viewer changes fail closed.
Output goes to target/; this tool never overwrites the shipped sample or user data.
"""
import argparse
import hashlib
import json
from pathlib import Path
import urllib.request
from verify_puzzle_data import ROOT, COMMIT, DATA_SHA


def get(url):
    with urllib.request.urlopen(url, timeout=60) as response:
        raw = response.read(2_000_001)
    if len(raw) > 2_000_000:
        raise ValueError('Response exceeds bounded slice size')
    return raw


def reproduce():
    api = 'https://huggingface.co/api/datasets/Lichess/chess-puzzles'
    if json.loads(get(api))['sha'] != COMMIT:
        raise ValueError('Official mirror HEAD changed; viewer cannot reliably reproduce pinned rows')
    proof = json.loads((ROOT / 'data/chess-puzzles/provenance.json').read_bytes())
    rows = []
    for batch in proof['batches']:
        raw = get(batch['url'])
        if hashlib.sha256(raw).hexdigest() != batch['sha256']:
            raise ValueError(f"Viewer response changed at offset {batch['offset']}; stop rather than mix snapshots")
        response = json.loads(raw)
        if len(response['rows']) != 100:
            raise ValueError('Incomplete official slice')
        rows.extend(response['rows'])
    if json.loads(get(api))['sha'] != COMMIT:
        raise ValueError('Official mirror changed while fetching')
    rows = [x for x in rows if not x['truncated_cells'] and x['row']['Popularity'] >= 80
            and x['row']['RatingDeviation'] <= 100 and 2 <= len(x['row']['Moves'].split()) <= 16
            and 400 <= x['row']['Rating'] <= 3000]
    order = lambda x: (-x['row']['Popularity'], x['row']['RatingDeviation'], x['row_idx'])
    selected = {}
    for theme in proof['theme_counts']:
        for row in sorted((x for x in rows if theme in x['row']['Themes']), key=order)[:16]:
            selected[row['row']['PuzzleId']] = row
    bins = [sorted((x for x in rows if lo <= x['row']['Rating'] < hi), key=order)
            for lo, hi in [(400,1000),(1000,1400),(1400,1800),(1800,2200),(2200,3001)]]
    while len(selected) < 512:
        changed = False
        for group in bins:
            while group and group[0]['row']['PuzzleId'] in selected:
                group.pop(0)
            if group and len(selected) < 512:
                row = group.pop(0)
                selected[row['row']['PuzzleId']] = row
                changed = True
        if not changed:
            raise ValueError('Insufficient official sample')
    output = []
    for row in sorted(selected.values(), key=lambda x: x['row_idx']):
        r = row['row']
        output.append(dict(id=r['PuzzleId'], fen=r['FEN'], moves=r['Moves'].split(),
                           rating=r['Rating'], deviation=r['RatingDeviation'], popularity=r['Popularity'],
                           themes=r['Themes'], url='https://lichess.org/' + r['GameId'], row=row['row_idx']))
    raw = (json.dumps(output, ensure_ascii=False, indent=2) + '\n').encode()
    if hashlib.sha256(raw).hexdigest() != DATA_SHA:
        raise ValueError('Reproduced sample differs')
    path = ROOT / 'target/v027-qa/reproduced-puzzles.json'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw)
    print(path, DATA_SHA)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--download', action='store_true', help='Explicitly permit bounded network requests')
    if not parser.parse_args().download:
        parser.error('Use --download to opt in; offline gates never run this tool')
    reproduce()
