"""Packager contract tests. Mock commands prove orchestration, NOT a real DMG.

Actual codesign/hdiutil evidence is produced separately by the packaging CLI.
"""
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / '.github/scripts/package_macos_023.py'
spec = importlib.util.spec_from_file_location('package_macos_023', SCRIPT)
pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pack)


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ocxd023-macos-package-test-')
        self.root = Path(self.temp.name)
        self.source = self.root / 'sealed source 空间' / pack.APP
        (self.source / 'Contents/MacOS').mkdir(parents=True)
        (self.source / 'Contents/Resources').mkdir()
        self.plist = self.source / 'Contents/Info.plist'
        self.plist.write_bytes(plistlib.dumps({
            'CFBundleShortVersionString': '0.2.23', 'CFBundleVersion': '23',
            'CFBundleIdentifier': 'test.fixture.only', 'CFBundleExecutable': 'fixture'}))
        executable = self.source / 'Contents/MacOS/fixture'
        executable.write_bytes(b'never executed test bytes')
        executable.chmod(0o755)
        (self.source / 'Contents/Resources/名称 with spaces').write_bytes(b'preserve sealed resource')
        self.target = self.root / 'artifact staging 空间'
        self.target.mkdir()
        self.output = self.target / 'OpenCodeX Desktop_0.2.23_aarch64.dmg'
        self.metadata = self.target / 'evidence.json'
        self.mounted = False
        self.calls = []
        self.fail = None
        self.corrupt = False
        self.readonly = True

    def tearDown(self):
        self.temp.cleanup()

    def command(self, argv, **kwargs):
        self.calls.append(argv)
        self.assertIsNone(kwargs.get('shell'))
        self.assertEqual(kwargs['stdin'], subprocess.DEVNULL)
        if self.fail and self.fail(argv):
            return subprocess.CompletedProcess(argv, 1, b'', b'mocked command failure')
        if argv[0] == '/usr/bin/ditto':
            shutil.copytree(argv[-2], argv[-1], symlinks=True)
        elif argv[:2] == ['/usr/bin/hdiutil', 'create']:
            Path(argv[-1]).write_bytes(b'mock DMG does not prove actual image validity')
        elif argv[:2] == ['/usr/bin/hdiutil', 'attach']:
            mount = Path(argv[argv.index('-mountpoint') + 1])
            payload = mount.parent / 'payload'
            shutil.copytree(payload, mount, dirs_exist_ok=True, symlinks=True)
            if self.corrupt:
                (mount / pack.FOLDER / pack.APP / 'Contents/Resources/名称 with spaces').write_bytes(b'mutated in transit')
            self.mounted = True
            return subprocess.CompletedProcess(argv, 0, plistlib.dumps({
                'system-entities': [{'dev-entry': '/dev/fixture-only', 'mount-point': str(mount)}]}), b'')
        elif argv[:2] == ['/usr/bin/hdiutil', 'detach']:
            self.mounted = False
        return subprocess.CompletedProcess(argv, 0, b'', b'')

    def run_package(self):
        with patch.object(pack.sys, 'platform', 'darwin'), \
             patch.object(pack.subprocess, 'run', side_effect=self.command), \
             patch.object(pack.os.path, 'ismount', side_effect=lambda _: self.mounted), \
             patch.object(pack.os, 'statvfs', return_value=SimpleNamespace(f_flag=os.ST_RDONLY if self.readonly else 0)):
            return pack.package(self.source, self.output, self.metadata)

    def test_folder_payload_sealed_bytes_readonly_and_new_asset_basename(self):
        before = pack.tree_manifest(self.source)
        report = self.run_package()
        self.assertEqual(pack.tree_manifest(self.source), before)
        self.assertEqual(report['status'], 'verified-candidate')
        self.assertEqual(report['inputKind'], 'candidate')
        self.assertFalse(report['releaseAcceptance'])
        self.assertFalse(report['installationPerformed'])
        self.assertTrue(report['readOnlyVerified'])
        self.assertTrue(report['runtimeAbsentBesideApp'])
        self.assertEqual(report['sourceAppTreeSha256'], report['mountedAppTreeSha256'])
        self.assertEqual(report['artifactSha256'], pack.sha256(self.output))
        self.assertEqual(report['mountedTopLevel'], ['Install.txt', pack.FOLDER])
        self.assertTrue(report['cleanup']['ownMountDetached'])
        self.assertTrue(report['cleanup']['scratchRemoved'])
        self.assertEqual(json.loads(self.metadata.read_text())['artifactSha256'], report['artifactSha256'])
        self.assertEqual({p.name for p in self.target.iterdir()}, {self.output.name, self.metadata.name})
        attach = next(argv for argv in self.calls if argv[:2] == ['/usr/bin/hdiutil', 'attach'])
        for flag in ['-readonly', '-nobrowse', '-noautoopen', '-noautofsck', '-mountpoint']:
            self.assertIn(flag, attach)
        self.assertFalse(any('--sign' in argv for argv in self.calls))
        self.assertFalse(any(argv[0] in ['/usr/bin/open', '/bin/launchctl'] for argv in self.calls))
        self.assertIn('~/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app', pack.INSTALL)

    def test_refuses_existing_output_metadata_and_app_destinations(self):
        self.output.write_bytes(b'original Tauri output must survive')
        with self.assertRaises(FileExistsError):
            self.run_package()
        self.assertEqual(self.output.read_bytes(), b'original Tauri output must survive')
        self.assertEqual(self.calls, [])
        self.output.unlink()
        self.metadata.write_bytes(b'other task evidence')
        with self.assertRaises(FileExistsError):
            self.run_package()
        self.assertEqual(self.metadata.read_bytes(), b'other task evidence')
        for path in [self.source / 'candidate.dmg', self.source / 'Contents/log.json']:
            with self.assertRaisesRegex(ValueError, 'inside any .app'):
                pack.destination(path)
        alias = self.target / 'app-link'
        alias.symlink_to(self.source, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'inside any .app'):
            pack.destination(alias / 'candidate.dmg')

    def test_version_and_signature_fail_before_image_and_do_not_modify_source(self):
        value = plistlib.loads(self.plist.read_bytes())
        value['CFBundleShortVersionString'] = '0.2.22'
        self.plist.write_bytes(plistlib.dumps(value))
        with self.assertRaisesRegex(ValueError, '0.2.23'):
            self.run_package()
        self.assertFalse(self.output.exists())
        self.assertEqual(self.calls, [])
        self.assertEqual(json.loads(self.metadata.read_text())['status'], 'failed')
        self.metadata.unlink()
        value['CFBundleShortVersionString'] = '0.2.23'
        self.plist.write_bytes(plistlib.dumps(value))
        self.fail = lambda argv: argv[0] == '/usr/bin/codesign'
        before = pack.tree_manifest(self.source)
        with self.assertRaises(RuntimeError):
            self.run_package()
        self.assertEqual(pack.tree_manifest(self.source), before)
        self.assertFalse(self.output.exists())
        self.assertEqual(len(self.calls), 1)

    def test_mounted_corruption_and_writable_mount_never_publish_and_always_detach(self):
        for corrupt, readonly in [(True, True), (False, False)]:
            self.corrupt, self.readonly = corrupt, readonly
            with self.assertRaises(ValueError):
                self.run_package()
            self.assertFalse(self.output.exists())
            report = json.loads(self.metadata.read_text())
            self.assertTrue(report['cleanup']['ownMountDetached'])
            self.assertTrue(report['cleanup']['scratchRemoved'])
            self.assertTrue(report['sourceUnchanged'])
            self.assertFalse(self.mounted)
            self.metadata.unlink()

    def test_finally_retries_only_own_mount_after_detach_failure(self):
        self.fail = lambda argv: argv[:2] == ['/usr/bin/hdiutil', 'detach'] and '-force' not in argv
        with self.assertRaises(RuntimeError):
            self.run_package()
        self.assertFalse(self.output.exists())
        detaches = [argv for argv in self.calls if argv[:2] == ['/usr/bin/hdiutil', 'detach']]
        self.assertTrue(any('-force' in argv for argv in detaches))
        self.assertEqual(len({argv[-1] for argv in detaches}), 1)
        self.assertTrue(json.loads(self.metadata.read_text())['cleanup']['ownMountDetached'])

    def test_output_race_never_replaces_foreign_artifact(self):
        original_link = os.link
        def race(source, target):
            Path(target).write_bytes(b'concurrent publisher')
            return original_link(source, target)
        with patch.object(pack.os, 'link', side_effect=race):
            with self.assertRaises(FileExistsError):
                self.run_package()
        self.assertEqual(self.output.read_bytes(), b'concurrent publisher')
        self.assertFalse(json.loads(self.metadata.read_text())['cleanup']['outputPublished'])

    def test_attach_partial_success_is_detached_even_when_plist_is_invalid(self):
        command = self.command
        def bad_attach(argv, **kwargs):
            result = command(argv, **kwargs)
            if argv[:2] == ['/usr/bin/hdiutil', 'attach']:
                return subprocess.CompletedProcess(argv, 0, b'bad plist after successful mount', b'')
            return result
        with patch.object(self, 'command', side_effect=bad_attach):
            with self.assertRaises(plistlib.InvalidFileException):
                self.run_package()
        self.assertFalse(self.mounted)
        self.assertFalse(self.output.exists())
        report = json.loads(self.metadata.read_text())
        self.assertTrue(report['cleanup']['ownMountDetached'])
        self.assertTrue(report['cleanup']['scratchRemoved'])

    def test_symlink_escape_refused_before_external_commands(self):
        (self.source / 'Contents/Resources/escape').symlink_to('/etc/passwd')
        with self.assertRaisesRegex(ValueError, 'escapes sealed bundle'):
            self.run_package()
        self.assertFalse(self.output.exists())
        self.assertEqual(self.calls, [])


if __name__ == '__main__':
    unittest.main()
