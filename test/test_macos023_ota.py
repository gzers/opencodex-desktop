"""No app/network tests for 023 signed-OTA admission and legacy driver isolation."""
import argparse
import importlib.util
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parent / 'smoke/ocx023-macos-ota.py'
SPEC = importlib.util.spec_from_file_location('ota023', SCRIPT)
ota = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ota)


class OtaAdmissionTests(unittest.TestCase):
    def args(self, **changes):
        values = dict(mode='prepare', docs_root='/tmp/docs', out='/tmp/owned-evidence',
                      source_sha=None, parent_beta_published=False,
                      publication_receipt=None, target_archive=None, target_signature=None)
        values.update(changes)
        return argparse.Namespace(**values)

    def test_prepare_rejects_every_execution_input(self):
        ota.validate_inputs(self.args())
        for key, value in [('parent_beta_published', True), ('source_sha', 'a' * 40),
                           ('publication_receipt', '/tmp/receipt'), ('target_archive', '/tmp/app'),
                           ('target_signature', '/tmp/sig')]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                ota.validate_inputs(self.args(**{key: value}))

    def test_missing_execution_evidence_is_rejected_before_run_creation(self):
        with self.assertRaises(ValueError): ota.validate_inputs(self.args(mode='execute'))
        with self.assertRaises(ValueError):
            ota.validate_inputs(self.args(mode='execute', parent_beta_published=True, source_sha='a' * 40,
                                          publication_receipt='/nonexistent/receipt',
                                          target_archive='/nonexistent/app', target_signature='/nonexistent/sig'))

    def test_execute_needs_valid_full_source_and_all_original_absolute_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ['receipt', 'archive', 'signature']: (root / name).write_bytes(b'fixture')
            values = dict(mode='execute', parent_beta_published=True, source_sha='a' * 40,
                          publication_receipt=str(root / 'receipt'), target_archive=str(root / 'archive'),
                          target_signature=str(root / 'signature'))
            ota.validate_inputs(self.args(**values))
            for bad in ['a' * 39, 'A' * 40, 'HEAD', 'main']:
                with self.subTest(source=bad), self.assertRaises(ValueError):
                    ota.validate_inputs(self.args(**{**values, 'source_sha': bad}))
            with self.assertRaises(ValueError):
                ota.validate_inputs(self.args(**{**values, 'target_archive': 'relative'}))

    def test_explicit_absolute_evidence_and_docs_roots(self):
        for key in ['out', 'docs_root']:
            with self.subTest(key=key), self.assertRaises(ValueError):
                ota.validate_inputs(self.args(**{key: 'relative'}))

    def test_loading_new_driver_does_not_change_legacy_defaults(self):
        self.assertEqual((ota.base.FROM_VERSION, ota.base.TO_VERSION), ('0.2.20', '0.2.21'))
        self.assertEqual(ota.base.OtaRun.restarted_log(type('Run', (), {'root': Path('/tmp/root')})()),
                         Path('/tmp/root/logs/app.log'))
        self.assertEqual(ota.Ota023.restarted_log(type('Run', (), {'root': Path('/tmp/root')})()),
                         Path('/tmp/root/data/desktop/logs/app.log'))


if __name__ == '__main__': unittest.main()
