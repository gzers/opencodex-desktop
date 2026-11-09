"""Failure-preserving core admission, merged verdict and evidence tests."""
import contextlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import collect
import core

HEAD = 'a' * 40


def hypothesis():
    return dict(schemaVersion=1, id='H-core-01', source=HEAD, buildRun='1234',
                claim='Own-H records prevent one user observing the peer listener',
                changeUnderTest='Isolated own-H status source and lifecycle producer',
                refutation='Any mismatched PID or port, failed original start or failed official restore refutes this',
                rerunReason='Runtime identity changed; repeat only dependent integration checks',
                predictions=[dict(step='13', status='pass', observation='Both original readiness and port assertion sets pass')],
                unchangedPassingEvidence=[dict(gate='installer cells', source='prior source', evidence='immutable prior artifact', whyUnaffected='No installer changes')])


class CoreTests(unittest.TestCase):
    def put(self, root, name, value):
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value))

    def ready(self, root):
        for index in (1, 2):
            self.put(root, f'evidence/two-user/user{index}/sweep-runtime-readiness.json',
                     dict(status='pass', accepted=False, diagnosticOnly=True))
            self.put(root, f'evidence/two-user/user{index}/runtime-1/runtime-step.json',
                     dict(mode='start', result='pass-runtime-step', action={'result': 'started'},
                          startBudget={'budgetMilliseconds': 30000, 'elapsedMilliseconds': 12000},
                          status={'runtime': 'running', 'health': 'healthy', 'pid': str(index), 'port': 12000+index},
                          record={'pid': index, 'port': 12000+index}, attestation='verified-production-challenge'))

    def test_merged_verdict_requires_both_original_sets_and_keeps_all_errors(self):
        for readiness, ports in ((False, False), (True, False), (False, True), (True, True)):
            with self.subTest(readiness=readiness, ports=ports), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                if readiness:
                    self.ready(root)
                self.put(root, 'evidence/two-user/two-user-native.json',
                         dict(runtimeProxyPortIsolation='verified-runtime-ports' if ports else 'not-verified',
                              runtimeProtocol={'servicesStopped': True}))
                collect.collect(root, selected='13', core=True)
                path = root / 'sweep/13-runtime-readiness-and-ports.json'
                row = json.loads(path.read_text())
                self.assertEqual(row['status'], 'pass' if readiness and ports else 'fail')
                self.assertEqual(row['evidence']['readiness']['status'], 'pass' if readiness else 'fail')
                self.assertEqual(row['evidence']['ports']['status'], 'pass' if ports else 'fail')
                before = path.read_bytes()
                with self.assertRaises(FileExistsError):
                    collect.collect(root, selected='13', core=True)
                self.assertEqual(before, path.read_bytes())
                self.assertFalse(row['accepted'])
                self.assertFalse((root / 'sweep/14-runtime-ports.json').exists())

    def test_core_cleanup_still_requires_both_profiles_acl_and_no_unknown_process(self):
        for broken in (None, 'retained', 'same-sid', 'acl'):
            with self.subTest(broken=broken), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                rows = [dict(result='account-and-profile-removed', sid='SID1'), dict(result='account-and-profile-removed', sid='SID2')]
                if broken == 'retained':
                    rows.append(dict(result='retained-unproven'))
                if broken == 'same-sid':
                    rows[1]['sid'] = 'SID1'
                self.put(root, 'evidence/two-user/two-user-native.json', dict(
                    cleanup=rows, desktopAclRestored=broken != 'acl', evidenceAclRestored=True, sealedCandidateUnchanged=True))
                collect.collect(root, selected='18', core=True)
                row = json.loads((root / 'sweep/18-account-and-process-cleanup.json').read_text())
                self.assertEqual(row['status'], 'pass' if broken is None else 'fail')
                self.assertFalse(row['accepted'])

    def test_no_hypothesis_or_stale_source_and_empty_predictions_are_rejected(self):
        self.assertEqual(core.validate_hypothesis(hypothesis(), HEAD, '1234')['id'], 'H-core-01')
        for change in ({'claim': ''}, {'source': 'b'*40}, {'buildRun': '999'}, {'predictions': []},
                       {'predictions': [dict(step='14', status='pass', observation='Cannot split readiness from ports')]}):
            with self.subTest(change=change), self.assertRaises(AssertionError):
                core.validate_hypothesis(hypothesis() | change, HEAD, '1234')
        with self.assertRaises(AssertionError):
            core.validate_hypothesis({}, HEAD, '1234')

    def test_program_byte_changes_extra_files_and_traversal_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ('opencodex-desktop.exe', 'ocxd.exe', 'sweep-state.exe', 'sweep-filesystem.exe'):
                (root/name).write_bytes(b'test program')
            policy = core.read(core.REPO / '.github/runtime-platforms-023.json')
            self.put(root, 'runtime-bundle/runtime-bundle.manifest.json', dict(
                runtimeVersion=policy['runtime']['version'], runtimeArchiveSha256=policy['runtime']['sha256']))
            manifest = dict(source=HEAD, run='1234', accepted=False, diagnosticOnly=True,
                            policySha256=core.digest(core.REPO / '.github/runtime-platforms-023.json'), files=core.inventory(root))
            self.put(root, 'core-build.json', manifest)
            with patch.object(core, 'source', return_value=HEAD):
                core.verify(root, HEAD, '1234')
                (root/'extra.bin').write_bytes(b'unknown')
                with self.assertRaises(AssertionError):
                    core.verify(root, HEAD, '1234')
                (root/'extra.bin').unlink()
                (root/'ocxd.exe').write_bytes(b'tampered')
                with self.assertRaises(AssertionError):
                    core.verify(root, HEAD, '1234')
                manifest['files'][0]['path'] = '../outside'
                self.put(root, 'core-build.json', manifest)
                with self.assertRaises(AssertionError):
                    core.verify(root, HEAD, '1234')

    def test_summary_compares_hypothesis_and_never_accepts_or_overwrites(self):
        for admission in ('pass', 'fail'):
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.put(root, 'sweep/hypothesis.json', hypothesis() | {'startedAt': 0})
                self.put(root, 'sweep/01-hypothesis-and-inputs.json', dict(step='01', name='hypothesis-and-inputs',
                         status=admission, firstError='', accepted=False, diagnosticOnly=True))
                with contextlib.redirect_stdout(io.StringIO()):
                    core.finish(root)
                result = core.read(root/'sweep/summary.json')
                self.assertEqual(len(result['steps']), 8)
                self.assertFalse(result['accepted'])
                self.assertEqual(result['hypothesis']['verdict'], 'refuted' if admission == 'pass' else 'inconclusive')
                self.assertEqual(result['hypothesis']['comparisons'][0]['observed'], 'fail')
                self.assertFalse(result['cadence']['withinTarget'])
                with self.assertRaises(FileExistsError), contextlib.redirect_stdout(io.StringIO()):
                    core.finish(root)


if __name__ == '__main__':
    unittest.main(verbosity=2)
