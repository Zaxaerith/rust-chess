"""Regression checks for the pinned Pleco license override; no network required."""
import json
from pathlib import Path
import shutil
import tempfile
import unittest

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


if __name__ == '__main__':
    unittest.main()
