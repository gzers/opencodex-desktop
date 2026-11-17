"""Read-only archive counterexamples, with synthetic non-executable packages."""
import importlib.util
import gzip
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from runtime_delivery_fixture024 import DeliveryFixture, digest, encoded

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('verify_runtime_delivery_024', ROOT / '.github/scripts/verify_runtime_delivery_024.py')
verify = importlib.util.module_from_spec(spec); spec.loader.exec_module(verify)


class RuntimeDelivery024(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ocxd024-delivery-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.fixture = DeliveryFixture(self.root / 'resources')

    def check(self):
        f = self.fixture
        return verify.verify(f.root, f.platform, f.policy)

    def test_both_platforms_are_verified_without_extraction_execution_or_mutation(self):
        for platform in ('darwin-arm64', 'win32-x64'):
            self.fixture = DeliveryFixture(self.root / platform, platform)
            before = {p.name: p.read_bytes() for p in self.fixture.root.iterdir()}
            result = self.check()
            self.assertEqual(result['status'], 'verified-offline')
            self.assertEqual(result['upstreamFiles'], 2)
            self.assertEqual(result['lockedPackages'], 1)
            self.assertFalse(result['codeExecuted']); self.assertFalse(result['releaseAcceptance'])
            self.assertEqual(before, {p.name: p.read_bytes() for p in self.fixture.root.iterdir()})

    def test_resealed_mutated_missing_or_extra_official_file_is_refused(self):
        name = 'runtime/opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs'
        for mode in ('mutated', 'missing', 'extra'):
            with self.subTest(mode=mode):
                self.fixture = DeliveryFixture(self.root / mode)
                if mode == 'mutated': self.fixture.files[name] = b'changed official source'
                elif mode == 'missing': del self.fixture.files[name]
                else: self.fixture.files[name + '.extra'] = b'injected'
                self.fixture.write()
                with self.assertRaisesRegex(ValueError, 'differs from upstream'): self.check()

    def test_resealed_lock_recovery_source_and_dependency_drift_are_refused(self):
        for name in ('runtime/opencodex/package-lock.json', 'runtime/opencodex/package.json',
                     'runtime/opencodex/sources/official.tgz'):
            with self.subTest(name=name):
                self.fixture = DeliveryFixture(self.root / digest(name.encode()))
                self.fixture.files[name] = b'{}'; self.fixture.write()
                with self.assertRaises(ValueError): self.check()
        self.fixture = DeliveryFixture(self.root / 'missing-native')
        self.fixture.policy['packageLock']['packages']['node_modules/native'] = {'version': '1.0.0'}
        self.fixture.files['runtime/opencodex/package-lock.json'] = encoded(self.fixture.policy['packageLock'])
        self.fixture.write()
        with self.assertRaisesRegex(ValueError, 'locked production package'): self.check()

    def test_truncated_extra_duplicate_and_special_members_are_refused(self):
        for kind in ('extra', 'duplicate', 'hardlink', 'device'):
            with self.subTest(kind=kind):
                self.fixture = DeliveryFixture(self.root / kind)
                member = tarfile.TarInfo(sorted(self.fixture.files)[0] if kind == 'duplicate' else 'runtime/z')
                if kind == 'hardlink': member.type = tarfile.LNKTYPE; member.linkname = sorted(self.fixture.files)[0]
                elif kind == 'device': member.type = tarfile.CHRTYPE
                self.fixture.write(extra_members=[member])
                with self.assertRaises(ValueError): self.check()
        self.fixture = DeliveryFixture(self.root / 'truncated')
        self.fixture.descriptor['entries'].append({'path': 'runtime/z', 'kind': 'file', 'bytes': 0, 'mode': 0o644, 'sha256': digest(b'')})
        self.fixture.pin()
        with self.assertRaisesRegex(ValueError, 'missing platform'): self.check()

    def test_descriptor_path_and_link_attacks_are_refused(self):
        for name in ('../escape', 'runtime/../escape', 'runtime//x', 'runtime/a\\b', '/runtime/x', 'runtime/a:b'):
            with self.subTest(name=name):
                self.fixture = DeliveryFixture(self.root / digest(name.encode()))
                self.fixture.files[name] = b'x'; self.fixture.write()
                with self.assertRaises(ValueError): self.check()

        for target in ('../../../escape', '/runtime/node/bin/node', 'missing', 'link'):
            with self.subTest(target=target):
                self.fixture = DeliveryFixture(self.root / digest(target.encode()))
                self.fixture.descriptor['entries'].append({'path': 'runtime/link', 'kind': 'symlink', 'target': target})
                self.fixture.descriptor['entries'].sort(key=lambda row: row['path']); self.fixture.pin()
                with self.assertRaises(ValueError): self.check()

    def test_hidden_second_archive_after_end_marker_is_refused(self):
        path = self.fixture.root / 'complete.tar.gz'
        raw = path.read_bytes() + gzip.compress(b'not part of the verified archive')
        path.write_bytes(raw)
        self.fixture.descriptor.update(archiveBytes=len(raw), archiveSha256=digest(raw)); self.fixture.pin()
        with self.assertRaisesRegex(ValueError, 'archive trailer'): self.check()

    def test_budget_malformed_duplicate_json_wrong_policy_and_links_are_refused(self):
        self.fixture.descriptor['entries'][0]['bytes'] = verify.MAX_UNPACKED + 1; self.fixture.pin()
        with self.assertRaises(ValueError): self.check()
        for name in ('runtime-bundle.manifest.json', 'complete.tar.gz', 'official-runtime.tgz'):
            self.fixture = DeliveryFixture(self.root / name)
            path = self.fixture.root / name
            os.link(path, self.root / (name + '.link'))
            with self.assertRaises(ValueError): self.check()
        self.fixture = DeliveryFixture(self.root / 'duplicate-json')
        raw = (self.fixture.root / 'runtime-bundle.manifest.json').read_bytes().replace(b'{', b'{"schemaVersion":1,', 1)
        (self.fixture.root / 'runtime-bundle.manifest.json').write_bytes(raw)
        (self.fixture.root / 'runtime-bundle.manifest.sha256').write_text(digest(raw))
        with self.assertRaisesRegex(ValueError, 'duplicate JSON'): self.check()
        self.fixture = DeliveryFixture(self.root / 'policy')
        self.fixture.descriptor['nodeVersion'] = '0.0.0'; self.fixture.pin()
        with self.assertRaisesRegex(ValueError, 'policy mismatch'): self.check()


if __name__ == '__main__':
    unittest.main()
