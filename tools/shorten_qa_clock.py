"""Test-only: shorten an existing isolated save, with a preserved original."""
import argparse
import hashlib
import json
from pathlib import Path
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('save', type=Path)
    parser.add_argument('--seconds', type=int, default=6)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1] / 'target/v020-manual'
    path = args.save.resolve(strict=True)
    if not path.is_relative_to(root.resolve()) or path.name != 'autosave-v1.json':
        parser.error('Only target/v020-manual/**/autosave-v1.json is allowed')
    if not 1 <= args.seconds <= 60:
        parser.error('Test countdown must be 1..60 seconds')
    raw = path.read_bytes()
    envelope = json.loads(raw)
    if (envelope['schema_version'] != 1
            or hashlib.sha256(envelope['payload'].encode()).hexdigest() != envelope['sha256']):
        parser.error('Invalid input envelope/checksum; original preserved')
    snapshot = json.loads(envelope['payload'])
    if snapshot['schema_version'] != 1 or snapshot['result'] is not None:
        parser.error('Only an unfinished schema 1 test game is allowed')
    active = {'w': 'white', 'b': 'black'}[snapshot['current_fen'].split()[1]]
    snapshot['clocks'][active] = {'secs': args.seconds, 'nanos': 0}
    snapshot['clocks']['warned'] = [False, False]
    snapshot['board_preferences']['chess_clock_enabled'] = True
    payload = json.dumps(snapshot, ensure_ascii=False, separators=(',', ':'))
    envelope['payload'] = payload
    envelope['sha256'] = hashlib.sha256(payload.encode()).hexdigest()
    backup = path.with_name(f'before-short-clock-{uuid.uuid4().hex}.json')
    with backup.open('xb') as file:
        file.write(raw)
    temp = path.with_name(f'short-clock-{uuid.uuid4().hex}.tmp')
    with temp.open('x', encoding='utf-8', newline='\n') as file:
        json.dump(envelope, file, ensure_ascii=False, indent=2)
    temp.replace(path)
    print(f'Test-only {active} clock: {args.seconds}s; original: {backup}')


if __name__ == '__main__':
    main()
