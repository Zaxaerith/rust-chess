"""Collect the exact Windows lockfile dependency license texts for distribution.
Run after cargo build. Never downloads a license or silently omits a package.
"""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'LICENSES' / 'dependencies'

PLECO_SHA256 = '42679ebf22772f1a2901273259e9396a14c7866afd0a0b4564091cefee5a9962'
PLECO_BLOB = '6adc426716168a2d3066000613a9d61f023a6eeb'
PLECO_VCS = '7122ae552507abd3853c5ad6ed37cc36637a0831'
PLECO_CRATE = '28a8c8ab569c544644c468a63f4fe4b33c0706b1472bebb517fabb75ec0f688e'
PLECO_HISTORY = 'd8db0d4e94c68334ec51c36dd99001efeda8640c'


def verified_pleco_sources(package, folder, root=ROOT):
    """Fail closed: this historical original is verified only for this release."""
    locked = tomllib.loads((root / 'Cargo.lock').read_text(encoding='utf-8'))
    entry = next(p for p in locked['package'] if p['name'] == 'pleco')
    if (package['version'] != '0.5.0' or package['license'] != 'MIT'
            or entry['version'] != '0.5.0' or entry.get('checksum') != PLECO_CRATE
            or entry.get('source') != 'registry+https://github.com/rust-lang/crates.io-index'):
        raise RuntimeError('Pleco historical license applies only to the verified locked MIT 0.5.0 crate')
    vcs = json.loads((folder / '.cargo_vcs_info.json').read_text(encoding='utf-8'))
    if vcs['git']['sha1'] != PLECO_VCS:
        raise RuntimeError('Pleco published-source commit mismatch')
    original = root / 'LICENSES/upstream/pleco-0.5.0-LICENSE-MIT'
    provenance = root / 'LICENSES/upstream/pleco-0.5.0-provenance.json'
    data = original.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(data)).encode('ascii') + b'\0' + data).hexdigest()
    proof = json.loads(provenance.read_text(encoding='utf-8'))
    expected_provenance = {
        'source_commit': PLECO_HISTORY,
        'source_url': f'https://github.com/pleco-rs/Pleco/blob/{PLECO_HISTORY}/pleco/LICENSE',
        'published_source_url': f'https://github.com/pleco-rs/Pleco/blob/{PLECO_VCS}/pleco/LICENSE',
        'manifest_source_url': f'https://github.com/pleco-rs/Pleco/blob/{PLECO_VCS}/pleco/Cargo.toml',
        'file': 'pleco-0.5.0-LICENSE-MIT',
        'copyright': 'Copyright (c) 2017 Stephen Fleischman',
    }
    if (hashlib.sha256(data).hexdigest() != PLECO_SHA256 or blob != PLECO_BLOB
            or proof['file_sha256'] != PLECO_SHA256 or proof['git_blob_sha1'] != PLECO_BLOB
            or proof['crate_vcs_commit'] != PLECO_VCS or proof['crate_checksum_sha256'] != PLECO_CRATE
            or proof['package'] != 'pleco' or proof['version'] != '0.5.0' or proof['license'] != 'MIT'
            or any(proof.get(key) != value for key, value in expected_provenance.items())):
        raise RuntimeError('Pleco upstream original/provenance verification failed')
    return [original, provenance, folder / 'README.md'], proof

def main():
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
    DEST.mkdir(parents=True, exist_ok=True)
    rows = []
    manifest = []
    for identity in sorted(selected, key=lambda i: (packages[i]['name'], packages[i]['version'])):
        package = packages[identity]
        if package['source'] is None:
            continue
        name, version = package['name'], package['version']
        folder = Path(package['manifest_path']).parent
        sources = sorted(p for p in folder.iterdir() if p.is_file() and
                         p.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'COPYRIGHT', 'AUTHORS')))
        note = ''
        provenance = None
        if not sources and name == 'failure_derive':
            shared = next(p for p in packages.values() if p['name'] == 'failure' and p['version'] == version)
            shared_folder = Path(shared['manifest_path']).parent
            sources = sorted(shared_folder.glob('LICENSE-*'))
            note = ' Shared failure workspace license texts.'
        if not sources and name == 'mucow' and version == '0.1.0':
            # The published crate and repository have no standalone license.
            # Preserve its exact dual-license grant and choose the Apache option.
            shared = next(p for p in packages.values() if p['name'] == 'failure')
            sources = [folder / 'Cargo.toml', Path(shared['manifest_path']).parent / 'LICENSE-APACHE']
            note = (' Upstream manifest offers MIT OR Apache-2.0 but has no license file. '
                    'Apache-2.0 option selected; canonical Apache text supplied from failure, '
                    'not represented as a mucow-authored file. Exact author/license declaration is in Cargo.toml.')
        if name == 'pleco':
            sources, provenance = verified_pleco_sources(package, folder)
            note = (' Verified historical MIT original (2017 Stephen Fleischman), byte-identical '
                    'at the published 0.5.0 source commit. Full pinned URLs, commits and SHA-256 '
                    'in [provenance](pleco-0.5.0/pleco-0.5.0-provenance.json). '
                    'Current repository-root GPL is not substituted for the library MIT license.')
        if not sources:
            raise RuntimeError(f'Missing license texts: {name} {version}; manual review required')
        if folder / 'Cargo.toml' not in sources:
            sources.append(folder / 'Cargo.toml')
        target = DEST / f'{name}-{version}'
        target.mkdir(exist_ok=True)
        files = []
        for source in sources:
            dest = target / ('LICENSE-MIT' if source.name == 'pleco-0.5.0-LICENSE-MIT' else source.name)
            shutil.copyfile(source, dest)
            files.append({'path': str(dest.relative_to(ROOT)).replace('\\', '/'),
                          'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()})
        rows.append(f"| {name} | {version} | {package['license']} | [texts]({name}-{version}/) |{note}")
        manifest.append({'name': name, 'version': version, 'license': package['license'],
                         'repository': package.get('repository'), 'authors': package.get('authors', []), 'files': files, 'note': note.strip()})
        if provenance is not None:
            manifest[-1]['provenance'] = provenance
    (DEST / 'INDEX.md').write_text('# Locked Windows dependency license texts\n\n'
        'Includes runtime and build dependencies. SPDX alternatives are retained verbatim; '
        'all available upstream license/notice files are bundled, including Unicode license conjunctions.\n\n'
        '| Package | Version | License expression | Files | Review / source |\n| --- | --- | --- | --- | --- |\n' + '\n'.join(rows) + '\n', encoding='utf-8', newline='\n')
    (DEST / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
    print(f'Collected license and notice files for {len(manifest)} locked Windows dependencies.')

if __name__ == '__main__':
    main()
