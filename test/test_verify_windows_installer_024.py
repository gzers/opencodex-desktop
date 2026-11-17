"""Offline installer-content contracts. Crypto and bounded reader are real;
installer/container responses are explicit fixtures, never a Windows install.
"""
import base64
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from runtime_delivery_fixture024 import DeliveryFixture

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('win_installer024', ROOT / '.github/scripts/verify_windows_installer_024.py')
verify = importlib.util.module_from_spec(spec); spec.loader.exec_module(verify)


class Listing024(unittest.TestCase):
    def test_real_style_msi_and_nsis_members_are_explicit(self):
        msi = b'7zip\n--\nType = Compound\nPath = app.cab\n--\nType = Cab\n\n----------\nPath = Path\nSize = 42\nAttributes = A\n'
        nsis = b'7zip\nType = Nsis\n\n----------\nPath = opencodex-desktop.exe\nSize = 42\n\nPath = uninstall.exe\nSize = \n'
        self.assertEqual(verify.member_index(msi, 'msi')['Path']['bytes'], 42)
        self.assertEqual(verify.member_index(nsis, 'nsis')['uninstall.exe']['bytes'], '')
        with self.assertRaises(ValueError): verify.member_index(msi, 'nsis')
        with self.assertRaises(ValueError): verify.member_index(nsis, 'msi')

    def test_ambiguous_names_fields_sizes_links_and_framing_are_rejected(self):
        valid = 'Type = Nsis\n\n----------\nPath = file\nSize = 1\n'
        bad = [valid + '\nPath = FILE\nSize = 1\n', valid.replace('file', '../escape'),
               valid.replace('file', 'x//y'), valid.replace('file', 'C:\\x'),
               valid.replace('Size = 1', 'Size = -1'), valid.replace('Size = 1', 'Size = '),
               valid.replace('Size = 1', 'Size = 1\nSize = 1'), valid + 'Folder = +\n',
               valid + 'Symbolic Link = other\n', valid + 'Encrypted = +\n',
               valid.replace('Size = 1', 'Size = 999999999999'), valid + 'not-a-record\n',
               valid.replace('file', 'bad\x00name'), valid + '\n----------\n']
        for data in bad:
            with self.subTest(data=data):
                with self.assertRaises(ValueError): verify.member_index(data.encode(), 'nsis')

    def test_actual_bounded_reader_handles_both_pipes_and_refuses_overflow_and_timeout(self):
        out = verify.bounded_tool([sys.executable, '-c', 'import sys;sys.stderr.write("note");sys.stdout.buffer.write(b"abc")'], 3, True)
        self.assertEqual(out['output'], b'abc'); self.assertEqual(out['sha256'], hashlib.sha256(b'abc').hexdigest())
        for code in ('import sys;sys.stdout.buffer.write(b"x"*65536)', 'import sys;sys.stderr.buffer.write(b"x"*(2*1024*1024))'):
            with self.assertRaisesRegex(ValueError, 'budget'): verify.bounded_tool([sys.executable, '-c', code], 3)
        with self.assertRaises(subprocess.TimeoutExpired):
            verify.bounded_tool([sys.executable, '-c', 'import time;time.sleep(10)'], 3, timeout=.05)
        with self.assertRaisesRegex(ValueError, '7zip read failed'):
            verify.bounded_tool([sys.executable, '-c', 'raise SystemExit(2)'], 3)


class Installer024(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ocxd024-win-distribution-unit-')
        self.addCleanup(self.temp.cleanup); self.root = Path(self.temp.name)
        self.public, self.secret = self.root / 'fixture.pub', self.root / 'fixture.key'
        subprocess.run(['minisign', '-G', '-W', '-p', str(self.public), '-s', str(self.secret)], check=True, capture_output=True)
        self.fixture = DeliveryFixture(self.root / 'resources', 'win32-x64')
        archive_name = 'runtime-bundle-0.2.24-win32-x64.tar.gz'
        self.fixture.target['archiveName'] = archive_name
        self.fixture.descriptor['archiveName'] = archive_name
        (self.fixture.root / 'complete.tar.gz').rename(self.fixture.root / archive_name); self.fixture.pin()
        self.built = self.root / 'built'; self.built.mkdir()
        for name in ('opencodex-desktop.exe', 'ocx.exe', 'ocxd.exe'):
            (self.built / name).write_bytes(b'never execute fixture ' + name.encode())
        self.entry = self.root / 'ocx-native-entry.json'; self.entry.write_text('{"fixture":true}')
        self.archive = self.root / 'fixture.msi'; self.archive.write_bytes(b'signed fixture; never execute')
        self.sig = self.root / 'fixture.sig'; self.sign()
        self.config = self.root / 'tagged.json'; self.config.write_text(json.dumps({'version': '0.2.24', 'identifier': 'com.gzers.opencodex.desktop',
            'plugins': {'updater': {'pubkey': base64.b64encode(self.public.read_bytes()).decode()}}}))
        self.kind, self.corrupt, self.missing, self.late = 'msi', None, None, False
        self.reads = []

    def sign(self):
        path = self.root / 'fixture.minisig'
        subprocess.run(['minisign', '-S', '-m', str(self.archive), '-s', str(self.secret), '-x', str(path)], check=True, capture_output=True)
        self.sig.write_bytes(base64.b64encode(path.read_bytes()))

    def expected(self):
        files = {name: self.built / name for name in ('opencodex-desktop.exe', 'ocx.exe', 'ocxd.exe')}
        files['ocx-native-entry.json'] = self.entry
        files.update({'runtime-bundle/' + p.name: p for p in self.fixture.root.iterdir() if p.is_file()})
        return {(verify.MSI_IDS[k] if self.kind == 'msi' else k): v.read_bytes() for k, v in files.items()}

    def sevenzip(self, argv, limit, capture=False, timeout=120):
        files = self.expected(); self.reads.append(argv)
        if argv[1] == 'l':
            head = 'Type = Compound\nPath = app.cab\nType = Cab' if self.kind == 'msi' else 'Type = Nsis'
            raw = (head + '\n\n----------\n' + '\n\n'.join('Path = ' + name + '\nSize = ' + str(len(data)) for name, data in files.items() if name != self.missing) + '\n').encode()
            return {'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'output': raw}
        self.assertEqual(argv[1:5], ['x', '-so', '-spd', '--'])
        data = files[argv[-1]]
        if argv[-1] == self.corrupt: data = b'x' * len(data)
        if self.late: (self.built / 'ocxd.exe').write_bytes(b'late build change')
        return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'output': b''}

    def check(self):
        with patch.object(verify, 'bounded_tool', side_effect=self.sevenzip):
            return verify.verify(self.archive, self.sig, self.config, self.fixture.root, self.built,
                                 self.entry, Path(sys.executable).resolve(), self.kind, self.fixture.policy)

    def test_both_container_mappings_bind_every_resource_binary_and_entry(self):
        for kind, suffix in (('msi', '.msi'), ('nsis', '.exe')):
            self.kind = kind
            self.archive = self.archive.rename(self.root / ('fixture' + suffix))
            result = self.check()
            self.assertEqual(len(result['members']), 8); self.assertTrue(result['updaterSignatureVerified'])
            self.assertFalse(result['installerExecuted']); self.assertTrue(result['noFilesExtracted'])
            self.assertFalse(result['installationDestinationVerified']); self.assertFalse(result['releaseAcceptance'])

    def test_missing_resigned_payload_and_digest_drift_refuse(self):
        for member in verify.MSI_IDS.values():
            self.missing = member
            with self.assertRaisesRegex(ValueError, 'missing/wrong-size'): self.check()
        self.missing = None
        for member in verify.MSI_IDS.values():
            self.corrupt = member
            with self.assertRaisesRegex(ValueError, 'differs from selected build'): self.check()

    def test_invalid_signature_or_changed_source_refuse(self):
        self.archive.write_bytes(b'tampered after signing')
        with self.assertRaises(subprocess.CalledProcessError): self.check()
        self.sign(); self.late = True
        with self.assertRaisesRegex(ValueError, 'changed during|missing/wrong-size|differs from selected build'): self.check()

    def test_wrong_config_or_untrusted_resource_is_not_an_installer_pass(self):
        value = json.loads(self.config.read_bytes()); value['version'] = '0.2.23'; self.config.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, 'config/policy'): self.check()
        value['version'] = '0.2.24'; self.config.write_text(json.dumps(value))
        (self.fixture.root / 'official-runtime.tgz').write_bytes(b'changed upstream')
        with self.assertRaisesRegex(ValueError, 'source digest'): self.check()


if __name__ == '__main__':
    unittest.main()
