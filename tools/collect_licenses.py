"""Collect the exact Windows lockfile dependency license texts for distribution.
Run after cargo build. Never downloads a license or silently omits a package.
"""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'LICENSES' / 'dependencies'

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
        sources = [p for p in folder.iterdir() if p.is_file() and
                   p.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'COPYRIGHT', 'AUTHORS'))]
        note = ''
        if not sources and name == 'failure_derive':
            shared = next(p for p in packages.values() if p['name'] == 'failure' and p['version'] == version)
            shared_folder = Path(shared['manifest_path']).parent
            sources = list(shared_folder.glob('LICENSE-*'))
            note = ' Shared failure workspace license texts.'
        if not sources and name == 'mucow' and version == '0.1.0':
            # The published crate and repository have no standalone license.
            # Preserve its exact dual-license grant and choose the Apache option.
            shared = next(p for p in packages.values() if p['name'] == 'failure')
            sources = [folder / 'Cargo.toml', Path(shared['manifest_path']).parent / 'LICENSE-APACHE']
            note = (' Upstream manifest offers MIT OR Apache-2.0 but has no license file. '
                    'Apache-2.0 option selected; canonical Apache text supplied from failure, '
                    'not represented as a mucow-authored file. Exact author/license declaration is in Cargo.toml.')
        if not sources and name == 'pleco' and version == '0.5.0':
            sources = [folder / 'Cargo.toml', folder / 'README.md']
            note = (' Published crate explicitly declares MIT and README references LICENSE-MIT, '
                    'but that file is excluded from the crate. Exact author/license declaration '
                    'is preserved. Upstream standalone text could not be retrieved in this run; '
                    'MANUAL LICENSE REVIEW REQUIRED before public distribution.')
        if not sources:
            raise RuntimeError(f'Missing license texts: {name} {version}; manual review required')
        if folder / 'Cargo.toml' not in sources:
            sources.append(folder / 'Cargo.toml')
        target = DEST / f'{name}-{version}'
        target.mkdir(exist_ok=True)
        files = []
        for source in sources:
            dest = target / source.name
            shutil.copyfile(source, dest)
            files.append({'path': str(dest.relative_to(ROOT)).replace('\\', '/'),
                          'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()})
        rows.append(f"| {name} | {version} | {package['license']} | [texts]({name}-{version}/) |{note}")
        manifest.append({'name': name, 'version': version, 'license': package['license'],
                         'repository': package.get('repository'), 'authors': package.get('authors', []), 'files': files, 'note': note.strip()})
    (DEST / 'INDEX.md').write_text('# Locked Windows dependency license texts\n\n'
        'Includes runtime and build dependencies. SPDX alternatives are retained verbatim; '
        'all available upstream license/notice files are bundled, including Unicode license conjunctions.\n\n'
        '| Package | Version | License expression | Files |\n| --- | --- | --- | --- |\n' + '\n'.join(rows) + '\n', encoding='utf-8')
    (DEST / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Collected license and notice files for {len(manifest)} locked Windows dependencies.')

if __name__ == '__main__':
    main()
