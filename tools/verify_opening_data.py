"""Offline verification of the sole pinned CC0 dataset (no downloads)."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMMIT = 'a6189a30dc273ccb21fc2536a9a2fefd5592a67a'
HASHES = {
    'a.tsv': '3282e4c9155289a29224f9a85fba0decb46efa35c2fa5e39662d2ac48fb0f793',
    'b.tsv': '1d5ed134ebbd87915ead5683416e37583bb3e5ae3582b4d8cb5cb4aa7ef4f623',
    'c.tsv': '45e843c7b7e79a0d04d5bda879c9bd23f5a10974488bd8615e4f4c5d0749986f',
    'd.tsv': '1cf4057a8aece94f27bb3f278ad410ee2844831a642364eeceaafa091dbcceda',
    'e.tsv': '48c6f6645b031a5937c8386839debab0e2e48837251687ba1afb6369677f8546',
    'COPYING.txt': 'a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499',
}


def verify(root=ROOT):
    data = root / 'data/chess-openings'
    proof = json.loads((data / 'provenance.json').read_text(encoding='utf-8'))
    if (proof['commit'], proof['license'], proof['attribution'], proof['source']) != (
            COMMIT, 'CC0-1.0', 'Lichess Contributors', 'https://github.com/lichess-org/chess-openings'):
        raise ValueError('Opening data source/license identity mismatch')
    if len(proof['files']) != 6 or {f['path'] for f in proof['files']} != set(HASHES):
        raise ValueError('Opening data file set mismatch')
    for f in proof['files']:
        raw = (data / f['path']).read_bytes()
        url = f"https://raw.githubusercontent.com/lichess-org/chess-openings/{COMMIT}/{f['path']}"
        if f['url'] != url or f['bytes'] != len(raw) or f['sha256'] != HASHES[f['path']] or hashlib.sha256(raw).hexdigest() != f['sha256']:
            raise ValueError(f"Opening data bytes/provenance mismatch: {f['path']}")
    if (root / 'LICENSES/chess-openings-CC0-1.0.txt').read_bytes() != (data / 'COPYING.txt').read_bytes():
        raise ValueError('Distributed CC0 text differs from original')
    if (root / 'LICENSES/chess-openings-provenance.json').read_bytes() != (data / 'provenance.json').read_bytes():
        raise ValueError('Distributed opening provenance differs')
    return {'commit': COMMIT, 'files': 6, 'license': 'CC0-1.0'}


if __name__ == '__main__':
    print(json.dumps(verify(), indent=2))
