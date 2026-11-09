"""Release delivery metadata must satisfy the publisher without changing package bytes."""
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('lifecycle_builder', ROOT / '.github/scripts/build_runtime_lifecycle_023.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


class DeliveryTests(unittest.TestCase):
    def setUp(self):
        self.policy = json.loads((ROOT / '.github/maintained-runtime.json').read_bytes())
        self.report = {key: self.policy[key] for key in ('version', 'archive', 'sha256', 'bytes', 'maintainer', 'installation', 'recordPathFix')}
        self.report.update(accepted=False, reproducibility={'equalBytes': True})

    def test_publisher_receives_every_tagged_policy_field_and_build_evidence(self):
        delivered = builder.delivery_manifest(self.report, self.policy)
        self.assertTrue(all(delivered.get(k) == v for k, v in self.policy.items()))
        self.assertFalse(delivered['accepted'])
        self.assertEqual(delivered['reproducibility'], {'equalBytes': True})
        self.assertNotIn('previousLifecycleRevision', self.report)

    def test_policy_cannot_replace_computed_identity_or_source_fix(self):
        for key in ('version', 'archive', 'sha256', 'bytes', 'recordPathFix', 'installation'):
            with self.subTest(key=key):
                changed = {**self.report, key: 'wrong'}
                with self.assertRaises(ValueError):
                    builder.delivery_manifest(changed, self.policy)

    def test_additional_conflicting_metadata_fails_closed(self):
        with self.assertRaises(ValueError):
            builder.delivery_manifest({**self.report, 'upstreamPublished': True}, self.policy)


if __name__ == '__main__':
    unittest.main()
