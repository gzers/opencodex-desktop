"""Collector regressions only: no Windows process or product is launched."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('windows023-user1-evidence.py')


class User1EvidenceTests(unittest.TestCase):
    def fixture(self, root, source):
        batch = root / 'batch'
        (batch / 'program').mkdir(parents=True)
        (batch / 'user1/desktop-logs').mkdir(parents=True)
        (batch / 'program/core-build.json').write_text(json.dumps({'source': source, 'run': '999'}))
        (batch / 'controller-finished.json').write_text(json.dumps({'source': source}))
        (batch / 'user1/user1-result.json').write_text(json.dumps({
            'source': source, 'trace': [
                {'sequence': 12, 'stage': 'invoke-begin'},
                {'sequence': 1, 'stage': 'action-return'},
            ],
        }))
        value = {'lifecycleState': 'starting_failed', 'result': None}
        (batch / 'user1/action-001.raw.json').write_text(json.dumps(value))
        (batch / 'user1/action-001.json').write_text(json.dumps({'action': 'start', 'response': {'value': value}}))
        (batch / 'user1/desktop-logs/app.log').write_text(
            '[manager] start: failed exit_code=1; startup_output=stderr[bytes=12,eof=true,kind=reference]'
        )
        return batch

    def test_new_product_identity_and_output_trace_are_not_previous_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            batch = self.fixture(root, 'a' * 40)
            destination = root / 'archived'
            result = subprocess.run([sys.executable, str(SCRIPT), str(batch), str(destination)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((destination / 'four-part-report.json').read_text())
            env = json.loads((destination / 'environment.json').read_text())
            events = json.loads((destination / 'event-order.json').read_text())['events']
            self.assertEqual(report['source'], 'a' * 40)
            self.assertEqual(report['start']['rawReceiptCount'], 1)
            self.assertEqual(report['start']['successfulReceiptCount'], 0)
            self.assertEqual(len(report['startupOutputTrace']), 1)
            self.assertEqual(env['artifactRun'], '999')
            self.assertIsNone(env['artifactId'])
            self.assertIsNone(env['artifactSha256'])
            self.assertEqual([e['eventOrdinal'] for e in events], [1, 2])
            self.assertEqual([e['rawTraceRow']['sequence'] for e in events], [12, 1])
            retry = subprocess.run([sys.executable, str(SCRIPT), str(batch), str(destination)], capture_output=True, text=True)
            self.assertNotEqual(retry.returncode, 0)

    def test_inconsistent_actual_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            batch = self.fixture(root, 'a' * 40)
            (batch / 'controller-finished.json').write_text(json.dumps({'source': 'b' * 40}))
            result = subprocess.run([sys.executable, str(SCRIPT), str(batch), str(root / 'archived')], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('Actual product source differs', result.stderr)


if __name__ == '__main__':
    unittest.main()
