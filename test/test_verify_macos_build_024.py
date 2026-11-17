"""Offline build-binding boundaries; mocked signing does not prove native sealing."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('verify_macos_build_024', ROOT / '.github/scripts/verify_macos_build_024.py')
binding = importlib.util.module_from_spec(spec); spec.loader.exec_module(binding)


class MacBuildBinding(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='ocxd024-binding-test-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.project = self.root / 'project'; self.project.mkdir()
        self.app = self.root / 'OpenCodeX Desktop.app'
        self.macos = self.app / 'Contents/MacOS'; self.macos.mkdir(parents=True)
        self.artifacts = self.root / 'cargo.jsonl'
        self.rows = []
        self.source = 'a' * 40
        for name in sorted(binding.NAMES):
            compiled = self.root / name; compiled.write_bytes(name.encode() + b':signature:rustc')
            (self.macos / name).write_bytes(name.encode() + b':signature:sealed')
            src = self.project / ('src/main.rs' if name == 'opencodex-desktop' else 'src/bin/' + name + '.rs')
            self.rows.append({'reason': 'compiler-artifact', 'executable': str(compiled),
                              'target': {'kind': ['bin'], 'name': name, 'src_path': str(src)}})
        self.rows.append({'reason': 'build-finished', 'success': True})
        self.write()
        self.strip_paths = []

    def write(self):
        self.artifacts.write_text(''.join(json.dumps(row) + chr(10) for row in self.rows))

    def git(self, argv, **kwargs):
        return self.source + chr(10) if argv[1:3] == ['rev-parse', 'HEAD'] else b''

    def command(self, argv, **kwargs):
        self.assertEqual(argv[0], '/usr/bin/codesign')
        if argv[1] == '--remove-signature':
            path = Path(argv[-1]); self.strip_paths.append(path)
            self.assertFalse(path.is_relative_to(self.app)); self.assertNotEqual(path.parent, self.root)
            path.write_bytes(path.read_bytes().split(b':signature:')[0])
        elif argv[1:4] == ['--force', '--sign', '-']:
            path = Path(argv[-1])
            self.assertFalse(path.is_relative_to(self.app))
            self.assertIn('--timestamp=none', argv)
            path.write_bytes(path.read_bytes() + b':signature:canonical')
        else:
            self.assertEqual(argv[1:4], ['--verify', '--deep', '--strict'])
        return subprocess.CompletedProcess(argv, 0, b'', b'')

    def check(self):
        with patch.object(binding.subprocess, 'check_output', side_effect=self.git), patch.object(binding.subprocess, 'run', side_effect=self.command):
            return binding.verify(self.app, self.artifacts, self.project, self.source)

    def test_signing_only_differences_match_without_mutating_inputs_and_copies_are_removed(self):
        before = {p.name: p.read_bytes() for p in self.macos.iterdir()}
        result = self.check()
        self.assertEqual(result['status'], 'verified-build-binding')
        self.assertEqual(len(result['executables']), 3)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.macos.iterdir()})
        self.assertTrue(result['copiesRemoved']); self.assertFalse(result['applicationLaunched'])
        self.assertFalse(result['releaseAcceptance'])
        self.assertTrue(all(not p.exists() for p in self.strip_paths))

    def test_stale_helper_code_is_refused_even_with_a_valid_app_signature(self):
        (self.macos / 'ocxd').write_bytes(b'old helper:signature:sealed')
        with self.assertRaisesRegex(ValueError, 'differs from compiled'): self.check()
        self.assertTrue(all(not p.exists() for p in self.strip_paths))

    def test_missing_duplicate_failed_or_foreign_build_receipts_are_refused(self):
        original = list(self.rows)
        for rows in (original[1:], original[:-1], original + [original[0]],
                     original[:-1] + [{'reason': 'build-finished', 'success': False}]):
            self.rows = rows; self.write()
            with self.assertRaises(ValueError): self.check()
        self.rows = json.loads(json.dumps(original)); self.rows[0]['target']['src_path'] = str(self.root / 'foreign.rs'); self.write()
        with self.assertRaisesRegex(ValueError, 'foreign'): self.check()

    def test_unknown_members_and_linked_binaries_are_refused(self):
        extra = self.macos / 'untracked-helper'; extra.write_bytes(b'unknown')
        with self.assertRaisesRegex(ValueError, 'membership'): self.check()
        extra.unlink(); os.link(self.macos / 'ocx', self.root / 'second-link')
        with self.assertRaisesRegex(ValueError, 'single-link'): self.check()

    def test_source_change_and_signing_failure_do_not_get_accepted(self):
        with patch.object(binding.subprocess, 'check_output', return_value='b'*40):
            with self.assertRaisesRegex(ValueError, 'source changed'): binding.verify(self.app,self.artifacts,self.project,self.source)
        with patch.object(binding.subprocess, 'check_output', side_effect=self.git), patch.object(binding.subprocess, 'run', side_effect=subprocess.CalledProcessError(1,['codesign'])):
            with self.assertRaises(subprocess.CalledProcessError): binding.verify(self.app,self.artifacts,self.project,self.source)


if __name__ == '__main__':
    unittest.main()
