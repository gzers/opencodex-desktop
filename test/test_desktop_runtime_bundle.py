"""Bundle boundary tests and optional verification of actual built platform artifacts.

python3 test/test_desktop_runtime_bundle.py --bundle-dir ABSOLUTE_BUILD_OUTPUT
Repeat --bundle-dir for macOS and Windows. Actual foreign execution is never claimed.
"""
import argparse
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import struct
import sys
import tarfile
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('bundle_builder', ROOT/'.github/scripts/build_desktop_runtime_bundle.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)
BUNDLES = []


class BundleBoundaries(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='ocxd023-runtime-name-bundle-test-',
                                                 dir='/private/tmp' if Path('/private/tmp').is_dir() else None)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def test_policy_has_real_complete_integrity_lock_and_fixed_platforms(self):
        policy = json.loads(builder.POLICY.read_bytes())
        self.assertEqual(set(policy['platforms']), {'darwin-arm64', 'win32-x64'})
        lock = policy['packageLock']
        self.assertEqual(lock['lockfileVersion'], 3)
        self.assertGreater(len(lock['packages']), 100)
        for name, row in lock['packages'].items():
            if not name:
                continue
            self.assertTrue(row['integrity'].startswith('sha512-'), name)
            self.assertFalse(row.get('dev'), name)
            self.assertNotIn('/Users/', row['resolved'])
            self.assertNotIn('/private/', row['resolved'])
            if name == 'node_modules/@bitkyc08/opencodex':
                self.assertTrue(row['resolved'].startswith('file:sources/'))
            else:
                self.assertTrue(row['resolved'].startswith('https://registry.npmjs.org/'))
        for platform, target in policy['platforms'].items():
            self.assertTrue(target['nodeArchive']['url'].startswith('https://nodejs.org/dist/v26.8.1/'))
            self.assertRegex(target['nodeArchive']['sha256'], '^[a-f0-9]{64}$')
            self.assertEqual(lock['packages']['node_modules/'+target['bunPackage']]['version'], '1.4.4')
            self.assertEqual(lock['packages']['node_modules/'+target['keyringPackage']]['version'], '1.3.0')
        self.assertEqual(policy['platforms']['darwin-arm64']['nodeExecutable'], 'runtime/node/bin/node')
        self.assertEqual(policy['platforms']['win32-x64']['nodeExecutable'], 'runtime/node/node.exe')

    def test_binary_validation_rejects_shim_and_wrong_cpu(self):
        path = self.root/'binary'
        path.write_bytes(b'#!/usr/bin/env node\nconsole.log("bun")')
        for platform in ('darwin-arm64', 'win32-x64'):
            with self.assertRaises(ValueError):
                builder.binary_kind(path, platform)
        path.write_bytes(b'\xcf\xfa\xed\xfe'+struct.pack('<I', 0x1000007)+b'\0'*100)
        with self.assertRaises(ValueError):
            builder.binary_kind(path, 'darwin-arm64')
        path.write_bytes(b'\xcf\xfa\xed\xfe'+struct.pack('<I', 0x100000c)+b'\0'*100)
        self.assertEqual(builder.binary_kind(path, 'darwin-arm64'), 'mach-o-arm64')

    def test_links_must_resolve_inside_runtime_and_exist(self):
        runtime = self.root/'runtime'
        runtime.mkdir()
        (runtime/'actual').write_text('bytes')
        (runtime/'ok').symlink_to('actual')
        builder.validate_link(runtime, runtime/'ok', 'actual')
        for target in ('../outside', '/etc/passwd', 'missing', 'C:\\escape'):
            path = runtime/'bad'
            with self.assertRaises((ValueError, FileNotFoundError)):
                builder.validate_link(runtime, path, target)

    def test_relative_paths_reject_escape_and_windows_ambiguity(self):
        for name in ('../bad', '/bad', 'runtime/../bad', 'runtime\\bad', 'C:/bad', './runtime/a'):
            with self.assertRaises(ValueError):
                builder.relative_path(name)

    def test_tree_rejects_hardlinks_and_repack_is_identical(self):
        runtime = self.root/'runtime'
        runtime.mkdir()
        (runtime/'a').write_bytes(b'hello')
        os.link(runtime/'a', runtime/'hard')
        with self.assertRaisesRegex(ValueError, 'Hardlinked'):
            builder.scan_tree(self.root)
        (runtime/'hard').unlink()
        (runtime/'link').symlink_to('a')
        entries, total = builder.scan_tree(self.root)
        self.assertEqual(total, 5)
        builder.pack(self.root, entries, self.root/'a.tgz')
        builder.pack(self.root, entries, self.root/'b.tgz')
        self.assertEqual(builder.digest(self.root/'a.tgz'), builder.digest(self.root/'b.tgz'))

    def test_missing_optional_production_native_fails_closed(self):
        policy = {'packageLock': {'packages': {'node_modules/native': {
            'version': '1.0.0', 'os': ['win32'], 'cpu': ['x64'], 'optional': True,
            'resolved': 'https://registry.npmjs.org/native/-/native-1.0.0.tgz', 'integrity': 'sha512-locked'}}}}
        with self.assertRaisesRegex(ValueError, 'Required locked production dependency missing'):
            builder.lock_packages(policy, self.root, {'os': 'win32', 'cpu': 'x64'})

    def test_descriptor_uses_full_path_order_for_directory_and_dot_siblings(self):
        runtime = self.root/'runtime'
        (runtime/'uv').mkdir(parents=True)
        (runtime/'uv/a.h').write_text('a')
        (runtime/'uv.h').write_text('b')
        entries, _ = builder.scan_tree(self.root)
        self.assertEqual([row['path'] for row in entries], ['runtime/uv.h', 'runtime/uv/a.h'])

    def test_node_extractor_rejects_hardlink_and_traversal(self):
        for name, hardlink in [('node-v1/../../escape', False), ('node-v1/link', True)]:
            archive = self.root/('hard.tgz' if hardlink else 'escape.tgz')
            with tarfile.open(archive, 'w:gz') as output:
                member = tarfile.TarInfo(name)
                if hardlink:
                    member.type, member.linkname = tarfile.LNKTYPE, 'outside'
                output.addfile(member)
            with self.assertRaises(ValueError):
                builder.extract_node(archive, self.root/('hard' if hardlink else 'escape'),
                                     {'sha256': builder.digest(archive), 'format': 'tar.gz', 'name': 'node-v1.tar.gz'})


class BuiltArtifacts(unittest.TestCase):
    def test_actual_archives_exactly_match_external_descriptor(self):
        if not BUNDLES:
            self.skipTest('Actual bundle verification requires --bundle-dir')
        for directory in BUNDLES:
            result = json.loads((directory/'result.json').read_bytes())
            descriptor_path = Path(result['manifest'])
            descriptor = json.loads(descriptor_path.read_bytes())
            archive = directory/descriptor['archiveName']
            self.assertEqual(builder.digest(archive), descriptor['archiveSha256'])
            self.assertEqual(archive.stat().st_size, descriptor['archiveBytes'])
            self.assertEqual(builder.digest(descriptor_path), result['manifestSha256'])
            self.assertNotIn(str(directory), descriptor_path.read_text())
            expected = {row['path']: row for row in descriptor['entries']}
            self.assertEqual(len(expected), len(descriptor['entries']))
            seen, total = [], 0
            with tarfile.open(archive, 'r|gz') as source:
                for member in source:
                    self.assertIn(member.name, expected)
                    self.assertNotIn(member.name, seen)
                    row = expected[member.name]
                    self.assertEqual(member.mtime, 0)
                    self.assertEqual(member.uid, 0)
                    self.assertEqual(member.gid, 0)
                    self.assertFalse(member.islnk())
                    self.assertTrue(member.isfile() or member.issym())
                    seen.append(member.name)
                    if member.isfile():
                        self.assertEqual(row['kind'], 'file')
                        self.assertEqual(member.size, row['bytes'])
                        self.assertEqual(member.mode, row['mode'])
                        total += member.size
                        h = hashlib.sha256()
                        file = source.extractfile(member)
                        for block in iter(lambda: file.read(1024*1024), b''):
                            h.update(block)
                        self.assertEqual(h.hexdigest(), row['sha256'], member.name)
                    else:
                        self.assertEqual(row['kind'], 'symlink')
                        self.assertEqual(member.linkname, row['target'])
                        builder.validate_link(directory/'runtime-bundle/runtime',
                                              directory/'runtime-bundle'/member.name, member.linkname)
            self.assertEqual(seen, sorted(expected))
            self.assertLessEqual(total, builder.MAX_UNPACKED)
            self.assertLessEqual(len(seen), builder.MAX_ENTRIES)
            checks = json.loads((directory/'checks.json').read_bytes())
            self.assertFalse(checks['lifecycleScriptsExecuted'])
            self.assertEqual(checks['reproducibleRepacks'], 2)
            if descriptor['platform'] == 'win32-x64' and sys.platform != 'win32':
                self.assertEqual(checks['execution'], 'not-run')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle-dir', action='append', type=Path, default=[])
    args, remaining = parser.parse_known_args()
    BUNDLES = args.bundle_dir
    unittest.main(argv=[sys.argv[0], *remaining], verbosity=2)
