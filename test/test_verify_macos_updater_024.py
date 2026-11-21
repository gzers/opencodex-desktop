"""Actual ephemeral minisign crypto; synthetic non-executable sealed app.

Only codesign is mocked on non-macOS fixtures. No GitHub, installation or model
requests; payload mutations are re-signed to test content independently of crypto.
"""
import base64
import gzip
import importlib.util
import io
import json
from pathlib import Path
import plistlib
import stat
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch
from runtime_delivery_fixture024 import DeliveryFixture

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('verify_macos_updater_024', ROOT / '.github/scripts/verify_macos_updater_024.py')
verify = importlib.util.module_from_spec(spec); spec.loader.exec_module(verify)


class RawModeTarInfo(tarfile.TarInfo):
    def tobuf(self, *args, **kwargs):
        blocks = super().tobuf(*args, **kwargs)
        # The writer masks mode bits. Patch the actual member header after
        # any PAX blocks, then recalculate its checksum.
        header = bytearray(blocks[-tarfile.BLOCKSIZE:])
        header[100:108] = f'{self.mode:07o}\0'.encode('ascii')
        header[148:156] = b'        '
        header[148:156] = f'{sum(header):06o}\0 '.encode('ascii')
        return blocks[:-tarfile.BLOCKSIZE] + header


class MacUpdater024(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.keys = tempfile.TemporaryDirectory(prefix='ocxd024-updater-testkeys-')
        cls.public, cls.secret = (Path(cls.keys.name) / name for name in ('fixture.pub', 'fixture.key'))
        subprocess.run(['minisign', '-G', '-W', '-p', str(cls.public), '-s', str(cls.secret)],
                       check=True, capture_output=True)

    @classmethod
    def tearDownClass(cls):
        cls.keys.cleanup()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ocxd024-updater-unit-')
        self.addCleanup(self.temp.cleanup); self.root = Path(self.temp.name)
        self.app = self.root / verify.APP
        self.macos = self.app / 'Contents/MacOS'; self.macos.mkdir(parents=True)
        for name in ('opencodex-desktop', 'ocxd', 'ocx'):
            path = self.macos / name; path.write_bytes(b'never run fixture ' + name.encode()); path.chmod(0o755)
        (self.app / 'Contents/Info.plist').write_bytes(plistlib.dumps({
            'CFBundleShortVersionString': '0.2.24', 'CFBundleIdentifier': 'com.gzers.opencodex.desktop',
            'CFBundleExecutable': 'opencodex-desktop'}))
        self.fixture = DeliveryFixture(self.app / 'Contents/Resources/runtime-bundle')
        self.config = self.root / 'tagged-config.json'
        self.config.write_text(json.dumps({'version': '0.2.24', 'identifier': 'com.gzers.opencodex.desktop',
            'plugins': {'updater': {'pubkey': base64.b64encode(self.public.read_bytes()).decode()}}}))
        self.archive = self.root / 'candidate.app.tar.gz'; self.signature = self.root / 'candidate.sig'
        self.real_run = subprocess.run; self.calls = []

    def run_tool(self, args, **kwargs):
        self.calls.append(args)
        if args[0] == '/usr/bin/codesign':
            self.assertEqual(args[1:4], ['--verify', '--deep', '--strict'])
            return subprocess.CompletedProcess(args, 0, b'', b'')
        self.assertEqual(args[0], 'minisign')
        return self.real_run(args, **kwargs)

    def pack(self, mutate=None, extra=()):
        rows, _ = verify.app_rows(self.app)
        with tarfile.open(self.archive, 'w:gz', format=tarfile.PAX_FORMAT) as archive:
            for name, row in rows.items():
                member = RawModeTarInfo(name); member.mode = row['mode']
                data = b''
                if row['kind'] == 'directory': member.type = tarfile.DIRTYPE
                elif row['kind'] == 'symlink': member.type = tarfile.SYMTYPE; member.linkname = row['target']
                else:
                    data = (self.app / name.removeprefix(verify.APP + '/')).read_bytes(); member.size = len(data)
                if mutate: member, data = mutate(member, data)
                if member is not None:
                    archive.addfile(member, io.BytesIO(data) if member.isfile() else None)
            for member, data in extra:
                archive.addfile(member, io.BytesIO(data) if member.isfile() else None)
        self.sign()

    def sign(self):
        path = self.root / 'native.minisig'
        self.real_run(['minisign', '-S', '-m', str(self.archive), '-s', str(self.secret), '-x', str(path)],
                      check=True, capture_output=True)
        self.signature.write_bytes(base64.b64encode(path.read_bytes()))

    def check(self):
        with patch.object(verify.subprocess, 'run', side_effect=self.run_tool):
            return verify.verify(self.app, self.archive, self.signature, self.config, self.fixture.policy)

    def test_real_crypto_complete_payload_and_runtime_pass_without_mutation_or_extraction(self):
        (self.app / 'Contents/Resources/internal-link').symlink_to('../MacOS/ocx')
        self.pack(); before = verify.common.tree_manifest(self.app)
        result = self.check()
        self.assertTrue(result['allMembersMatchSealedApp']); self.assertTrue(result['updaterSignatureVerified'])
        self.assertFalse(result['archiveExtracted']); self.assertFalse(result['applicationLaunched'])
        self.assertFalse(result['releaseAcceptance']); self.assertEqual(result['officialRuntime']['upstreamFiles'], 2)
        self.assertEqual(verify.common.tree_manifest(self.app), before)

    def test_resigned_changed_missing_extra_member_and_mode_are_rejected(self):
        target = verify.APP + '/Contents/MacOS/ocx'
        for mode in ('changed', 'missing', 'permissions', 'hardlink'):
            def mutate(member, data):
                if member.name == target:
                    if mode == 'missing': return None, data
                    if mode == 'changed': data = b'x' * len(data)
                    if mode == 'permissions': member.mode = 0o644
                    if mode == 'hardlink': member.type = tarfile.LNKTYPE; member.linkname = target
                return member, data
            with self.subTest(mode=mode):
                self.pack(mutate)
                with self.assertRaises(ValueError): self.check()
        extra = tarfile.TarInfo(verify.APP + '/Contents/unexpected'); extra.size = 1
        self.pack(extra=[(extra, b'x')])
        with self.assertRaisesRegex(ValueError, 'differ'): self.check()

    def test_tauri_mode_type_bits_must_match_the_tar_header(self):
        (self.app / 'Contents/Resources/internal-link').symlink_to('../MacOS/ocx')
        type_modes = {tarfile.REGTYPE: stat.S_IFREG, tarfile.DIRTYPE: stat.S_IFDIR,
                      tarfile.SYMTYPE: stat.S_IFLNK}

        def encode_mode(member, data):
            member.mode |= type_modes[member.type]
            return member, data

        self.pack(encode_mode)
        with tarfile.open(self.archive, 'r:gz') as archive:
            for member in archive:
                self.assertEqual(member.mode & ~0o777, type_modes[member.type])
        self.assertTrue(self.check()['allMembersMatchSealedApp'])

        for extra_bits in (stat.S_IFDIR, stat.S_IFREG | stat.S_ISUID,
                           stat.S_IFREG | stat.S_ISGID, stat.S_IFREG | stat.S_ISVTX, 0o200000):
            with self.subTest(extra_bits=oct(extra_bits)):
                def invalid_file_mode(member, data):
                    if member.name.endswith('/ocx'):
                        member.mode |= extra_bits
                    return member, data

                self.pack(invalid_file_mode)
                with tarfile.open(self.archive, 'r:gz') as archive:
                    self.assertEqual(archive.getmember(verify.APP + '/Contents/MacOS/ocx').mode,
                                     0o755 | extra_bits)
                with self.assertRaisesRegex(ValueError, 'special updater permission bits or mismatched file type'):
                    self.check()

    def test_duplicate_traversal_other_root_special_and_file_parent_are_rejected(self):
        for name, kind in ((verify.APP, tarfile.DIRTYPE), ('../escape', tarfile.REGTYPE),
                           ('Other.app/file', tarfile.REGTYPE), (verify.APP + '/Contents/socket', tarfile.FIFOTYPE),
                           (verify.APP + '/Contents/MacOS/ocx/child', tarfile.REGTYPE)):
            with self.subTest(name=name):
                member = tarfile.TarInfo(name); member.type = kind
                self.pack(extra=[(member, b'')])
                with self.assertRaises(ValueError): self.check()

    def test_wrong_signature_or_tagged_key_and_codesign_failure_refuse(self):
        self.pack(); self.archive.write_bytes(self.archive.read_bytes() + b'unsigned change')
        with self.assertRaises(subprocess.CalledProcessError): self.check()
        self.pack(); self.signature.write_text('not-base64')
        with self.assertRaises(ValueError): self.check()
        self.pack()
        original = self.config.read_bytes()
        other_public, other_secret = self.root / 'other.pub', self.root / 'other.key'
        self.real_run(['minisign', '-G', '-W', '-p', str(other_public), '-s', str(other_secret)],
                      check=True, capture_output=True)
        settings = json.loads(original)
        settings['plugins']['updater']['pubkey'] = base64.b64encode(other_public.read_bytes()).decode()
        self.config.write_text(json.dumps(settings))
        with self.assertRaises(subprocess.CalledProcessError): self.check()
        self.config.write_bytes(original)
        with patch.object(verify.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, ['minisign'])):
            with self.assertRaises(subprocess.CalledProcessError): verify.verify(self.app, self.archive, self.signature, self.config, self.fixture.policy)
        def signer(args, **kwargs):
            if args[0] == '/usr/bin/codesign': raise subprocess.CalledProcessError(1, args)
            return self.real_run(args, **kwargs)
        with patch.object(verify.subprocess, 'run', side_effect=signer):
            with self.assertRaises(subprocess.CalledProcessError): verify.verify(self.app, self.archive, self.signature, self.config, self.fixture.policy)

    def test_signed_outer_with_self_consistent_but_mutated_official_payload_refuses(self):
        path = 'runtime/opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs'
        self.fixture.files[path] = b'mutated official'; self.fixture.write(); self.pack()
        with self.assertRaisesRegex(ValueError, 'differs from upstream'): self.check()

    def test_signed_hidden_trailer_pax_overlay_and_special_permissions_refuse(self):
        self.pack(); self.archive.write_bytes(self.archive.read_bytes() + gzip.compress(b'hidden archive'))
        self.sign()
        with self.assertRaisesRegex(ValueError, 'trailer'): self.check()
        for mode in ('pax', 'setuid', 'escaping-link'):
            def mutate(member, data):
                if member.name.endswith('/ocx'):
                    if mode == 'pax': member.pax_headers = {'SCHILY.xattr.user.overlay': 'x'}
                    if mode == 'setuid': member.mode |= 0o4000
                    if mode == 'escaping-link': member.type = tarfile.SYMTYPE; member.size = 0; member.linkname = '/etc/passwd'
                return member, data
            self.pack(mutate)
            with self.assertRaises(ValueError): self.check()

    def test_version_membership_input_identity_and_late_source_change_refuse(self):
        self.pack(); original = self.config.read_bytes()
        value = json.loads(original); value['version'] = '0.2.25'; self.config.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, 'version'): self.check()
        self.config.write_bytes(original)
        alias = self.root / 'alias.tar.gz'; alias.symlink_to(self.archive)
        with self.assertRaises(ValueError): verify.verify(self.app, alias, self.signature, self.config, self.fixture.policy)
        real_delivery = verify.delivery.verify
        def change(*args):
            result = real_delivery(*args); (self.macos / 'ocx').write_bytes(b'changed during verification'); return result
        with patch.object(verify.delivery, 'verify', side_effect=change):
            with self.assertRaisesRegex(ValueError, 'changed during'): self.check()

    def test_budgets_and_unknown_structure_refuse_before_unbounded_work(self):
        self.pack()
        with patch.object(verify, 'MAX_COMPRESSED', 1):
            with self.assertRaisesRegex(ValueError, 'size exceeds'): self.check()
        with patch.object(verify, 'MAX_ENTRIES', 1):
            with self.assertRaisesRegex(ValueError, 'count exceeds'): self.check()
        with patch.object(verify, 'MAX_OUTER_BYTES', 1):
            with self.assertRaisesRegex(ValueError, 'budget'): self.check()
        (self.macos / 'unexpected').write_bytes(b'unknown executable')
        with self.assertRaisesRegex(ValueError, 'membership'): self.check()

    def test_extended_header_size_is_rejected_before_reading_its_declared_body(self):
        for kind in (tarfile.GNUTYPE_LONGNAME, tarfile.GNUTYPE_LONGLINK, tarfile.XHDTYPE):
            with self.subTest(kind=kind):
                header = tarfile.TarInfo('././@LongLink'); header.type = kind
                header.size = 16 * 1024 * 1024 * 1024
                self.archive.write_bytes(gzip.compress(header.tobuf(format=tarfile.GNU_FORMAT)))
                self.sign()
                with self.assertRaisesRegex(ValueError, 'extended updater header'): self.check()


if __name__ == '__main__':
    unittest.main()
