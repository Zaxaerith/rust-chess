"""Verify a formal local/CI ZIP against the locked dependencies and build inputs."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def verify(path):
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    manifest = json.loads((ROOT / 'LICENSES/dependencies/manifest.json').read_text(encoding='utf-8'))
    metadata = json.loads(subprocess.check_output([
        'cargo', 'metadata', '--locked', '--offline', '--format-version', '1',
        '--filter-platform', 'x86_64-pc-windows-msvc'], cwd=ROOT))
    packages = {p['id']: p for p in metadata['packages']}
    nodes = {n['id']: n for n in metadata['resolve']['nodes']}
    selected = set()

    def visit(identity):
        if identity in selected:
            return
        selected.add(identity)
        for dependency in nodes[identity]['dependencies']:
            visit(dependency)

    visit(metadata['resolve']['root'])
    expected_packages = {(packages[i]['name'], packages[i]['version'])
                         for i in selected if packages[i]['source'] is not None}
    actual_packages = {(p['name'], p['version']) for p in manifest}
    require(expected_packages == actual_packages and len(manifest) == len(actual_packages),
            'Dependency manifest does not match the complete locked Windows dependency graph')
    licenses = {
        'LICENSES/chasolver-MIT.txt', 'LICENSES/chess-MIT.txt',
        'LICENSES/upstream/pleco-0.5.0-LICENSE-MIT',
        'LICENSES/upstream/pleco-0.5.0-provenance.json',
        'LICENSES/dependencies/INDEX.md', 'LICENSES/dependencies/manifest.json',
    }
    license_file_count = 0
    for package in manifest:
        require(package['license'] and package['files'], 'Missing dependency license declaration/text')
        require('MANUAL LICENSE REVIEW REQUIRED' not in package['note'].upper(),
                'Unresolved dependency license review')
        for file in package['files']:
            relative = file['path']
            source = (ROOT / relative).resolve()
            require(source.is_relative_to((ROOT / 'LICENSES/dependencies').resolve()),
                    'Dependency file escapes the license directory')
            require(hashlib.sha256(source.read_bytes()).hexdigest() == file['sha256'],
                    f'Dependency file checksum mismatch: {relative}')
            licenses.add(relative)
            license_file_count += 1
    actual_licenses = {p.relative_to(ROOT).as_posix() for p in (ROOT / 'LICENSES').rglob('*') if p.is_file()}
    require(licenses == actual_licenses, 'Unindexed or missing files in the license distribution tree')
    sources = {name: ROOT / name for name in licenses | {
        'README.md', 'README_ZH.md', 'CHANGELOG.md', 'LICENSE',
        'THIRD_PARTY_NOTICES.md', 'assets/ATTRIBUTION.md',
    }}
    sources['rust-chess.exe'] = ROOT / 'target/release/chess_rust.exe'
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        require(len(names) == len(set(names)), 'Duplicate ZIP entry')
        require(archive.testzip() is None, 'ZIP CRC/integrity failure')
        require(all(not name.startswith('/') and '\\' not in name
                    and '..' not in name.split('/') for name in names), 'Invalid ZIP path')
        actual = {name for name in names if not name.endswith('/')}
        require(actual == set(sources) | {'checksums.sha256'},
                f'ZIP whitelist mismatch: unexpected={actual - set(sources) - {"checksums.sha256"}}, '
                f'missing={set(sources) - actual}')
        checksums = {}
        for line in archive.read('checksums.sha256').decode('utf-8-sig').splitlines():
            digest, relative = line.split('  ', 1)
            require(relative not in checksums, 'Duplicate checksum entry')
            checksums[relative] = digest
        require(set(checksums) == set(sources), 'Incomplete or unexpected checksum entries')
        for relative, source in sources.items():
            data = archive.read(relative)
            require(data == source.read_bytes(), f'ZIP differs from validated source/build: {relative}')
            require(hashlib.sha256(data).hexdigest() == checksums[relative],
                    f'ZIP SHA-256 mismatch: {relative}')
    return {
        'version': version, 'zip': str(path.resolve()), 'zip_bytes': path.stat().st_size,
        'zip_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'exe_sha256': hashlib.sha256(sources['rust-chess.exe'].read_bytes()).hexdigest(),
        'verified_zip_files': len(sources) + 1, 'license_packages': len(manifest),
        'license_files': license_file_count, 'zip_integrity': 'passed',
        'whitelist_source_and_checksums': 'passed',
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('zip', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    result = verify(args.zip)
    rendered = json.dumps(result, ensure_ascii=False, indent=2) + '\n'
    if args.output:
        args.output.write_text(rendered, encoding='utf-8', newline='\n')
    print(rendered, end='')
