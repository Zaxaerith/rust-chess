"""Regression checks for the pinned Pleco license override; no network required."""
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import hashlib
import subprocess

import collect_licenses as licenses


class HistoricalLicenseTests(unittest.TestCase):
    def setUp(self):
        parent = licenses.ROOT / 'target/license-tests'
        parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        shutil.copytree(licenses.ROOT / 'LICENSES/upstream', self.root / 'LICENSES/upstream')
        (self.root / 'Cargo.lock').write_text(
            '[[package]]\nname = "pleco"\nversion = "0.5.0"\n'
            'source = "registry+https://github.com/rust-lang/crates.io-index"\n'
            f'checksum = "{licenses.PLECO_CRATE}"\n', encoding='utf-8')
        self.folder = self.root / 'crate'
        self.folder.mkdir()
        (self.folder / '.cargo_vcs_info.json').write_text(
            json.dumps({'git': {'sha1': licenses.PLECO_VCS}}), encoding='utf-8')
        self.package = {'version': '0.5.0', 'license': 'MIT'}

    def verify(self):
        return licenses.verified_pleco_sources(self.package, self.folder, self.root)

    def test_verified_original(self):
        sources, proof = self.verify()
        self.assertIn(b'Copyright (c) 2017 Stephen Fleischman\n', sources[0].read_bytes())
        self.assertEqual(proof['git_blob_sha1'], licenses.PLECO_BLOB)

    def test_changed_license_bytes_fail(self):
        path = self.root / 'LICENSES/upstream/pleco-0.5.0-LICENSE-MIT'
        path.write_bytes(path.read_bytes().replace(b'\n', b'\r\n'))
        with self.assertRaises(RuntimeError):
            self.verify()

    def test_other_version_fails(self):
        self.package['version'] = '0.5.1'
        with self.assertRaises(RuntimeError):
            self.verify()

    def test_other_license_fails(self):
        self.package['license'] = 'GPL-3.0'
        with self.assertRaises(RuntimeError):
            self.verify()

    def test_other_crate_checksum_fails(self):
        path = self.root / 'Cargo.lock'
        path.write_text(path.read_text().replace(licenses.PLECO_CRATE, '0' * 64))
        with self.assertRaises(RuntimeError):
            self.verify()

    def test_other_vcs_commit_fails(self):
        (self.folder / '.cargo_vcs_info.json').write_text(json.dumps({'git': {'sha1': '0' * 40}}))
        with self.assertRaises(RuntimeError):
            self.verify()

    def test_missing_original_fails(self):
        (self.root / 'LICENSES/upstream/pleco-0.5.0-LICENSE-MIT').unlink()
        with self.assertRaises(FileNotFoundError):
            self.verify()

    def test_inconsistent_provenance_fails(self):
        path = self.root / 'LICENSES/upstream/pleco-0.5.0-provenance.json'
        proof = json.loads(path.read_text(encoding='utf-8'))
        proof['file_sha256'] = '0' * 64
        path.write_text(json.dumps(proof), encoding='utf-8')
        with self.assertRaises(RuntimeError):
            self.verify()

    def test_unpinned_source_url_fails(self):
        path = self.root / 'LICENSES/upstream/pleco-0.5.0-provenance.json'
        proof = json.loads(path.read_text(encoding='utf-8'))
        proof['source_url'] = 'https://github.com/pleco-rs/Pleco/blob/main/LICENSE'
        path.write_text(json.dumps(proof), encoding='utf-8')
        with self.assertRaises(RuntimeError):
            self.verify()


class LicenseCheckoutTests(unittest.TestCase):
    def test_every_current_manifest_file_matches_original_bytes(self):
        manifest = json.loads((licenses.DEST / 'manifest.json').read_text(encoding='utf-8'))
        for package in manifest:
            for file in package['files']:
                self.assertEqual(hashlib.sha256((licenses.ROOT / file['path']).read_bytes()).hexdigest(),
                                 file['sha256'], file['path'])

    def test_all_third_party_files_disable_git_text_conversion(self):
        files = [p.relative_to(licenses.ROOT).as_posix() for p in (licenses.ROOT / 'LICENSES').rglob('*') if p.is_file()]
        output = subprocess.check_output(['git', 'check-attr', 'text', '--', *files], cwd=licenses.ROOT, text=True)
        self.assertTrue(all(line.endswith(': text: unset') for line in output.splitlines()), output)

    def test_windows_autocrlf_checkout_preserves_ab_glyph_license(self):
        relative = 'LICENSES/dependencies/ab_glyph-0.2.32/LICENSE'
        data = subprocess.check_output(['git', '-c', 'core.autocrlf=true', 'cat-file',
            '--filters', f'--path={relative}', f'HEAD:{relative}'], cwd=licenses.ROOT)
        self.assertNotIn(b'\r\n', data)
        self.assertEqual(data, (licenses.ROOT / relative).read_bytes())


class OpeningDataTests(unittest.TestCase):
    def test_pinned_cc0_originals_and_distribution(self):
        import verify_opening_data
        self.assertEqual(verify_opening_data.verify()['files'], 6)

    def test_changed_cc0_data_or_provenance_fails(self):
        import verify_opening_data
        parent = licenses.ROOT / 'target/license-tests'
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent) as folder:
            root = Path(folder)
            shutil.copytree(licenses.ROOT / 'data/chess-openings', root / 'data/chess-openings')
            (root / 'LICENSES').mkdir()
            for name in ['chess-openings-CC0-1.0.txt', 'chess-openings-provenance.json']:
                shutil.copyfile(licenses.ROOT / 'LICENSES' / name, root / 'LICENSES' / name)
            path = root / 'data/chess-openings/a.tsv'
            original = path.read_bytes()
            path.write_bytes(original.replace(b'\n', b'\r\n'))
            with self.assertRaises(ValueError):
                verify_opening_data.verify(root)
            path.write_bytes(original)
            path = root / 'data/chess-openings/provenance.json'
            path.write_text(path.read_text().replace(verify_opening_data.COMMIT, 'main'), encoding='utf-8')
            with self.assertRaises(ValueError):
                verify_opening_data.verify(root)

    def test_cc0_data_disables_checkout_conversion(self):
        files = [p.relative_to(licenses.ROOT).as_posix() for p in (licenses.ROOT / 'data/chess-openings').iterdir()]
        output = subprocess.check_output(['git', 'check-attr', 'text', '--', *files], cwd=licenses.ROOT, text=True)
        self.assertTrue(all(line.endswith(': text: unset') for line in output.splitlines()), output)


if __name__ == '__main__':
    unittest.main()
