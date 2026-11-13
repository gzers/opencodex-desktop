"""Official input/staging boundary, without a network request or lifecycle script."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('stage024', ROOT / '.github/scripts/stage_runtime_bundle_024.py')
stage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(stage)


class OfficialStageBoundary(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ocxd024-stage-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.archive = self.root / 'official.tgz'
        self.archive.write_bytes(b'explicit synthetic official input')
        self.policy = self.root / 'policy.json'
        self.policy.write_text(json.dumps({'runtime': {'version': '2.99.0', 'archive': 'official.tgz',
            'distribution': 'official-npm', 'url': 'https://registry.npmjs.org/@bitkyc08/opencodex/-/opencodex-2.99.0.tgz',
            'sha256': hashlib.sha256(self.archive.read_bytes()).hexdigest()}}))
        self.output = self.root / 'output'
        self.resources = self.root / 'resources'

    def run_stage(self):
        return stage.stage('darwin-arm64', self.root / 'node', self.root / 'npm.js',
                           self.output, self.archive, resource_directory=self.resources)

    def test_official_policy_preserves_transitive_lock_and_both_platforms(self):
        before = json.loads((ROOT / '.github/runtime-platforms-023.json').read_bytes())
        after = json.loads((ROOT / '.github/runtime-platforms-024.json').read_bytes())
        self.assertEqual(after['runtime']['version'], '2.99.0')
        self.assertEqual(after['runtime']['distribution'], 'official-npm')
        self.assertEqual(after['runtime']['sha256'], 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c')
        self.assertEqual(set(after['platforms']), {'darwin-arm64', 'win32-x64'})
        for name, row in before['packageLock']['packages'].items():
            if name not in ('', 'node_modules/@bitkyc08/opencodex'):
                self.assertEqual(row, after['packageLock']['packages'][name], name)
        self.assertEqual(before['node'], after['node'])

    def test_wrong_archive_stops_before_build_or_resource_creation(self):
        self.archive.write_bytes(b'wrong bytes')
        with patch.object(stage, 'POLICY', self.policy), patch.object(stage.subprocess, 'run') as build:
            with self.assertRaisesRegex(ValueError, 'digest mismatch'):
                self.run_stage()
            build.assert_not_called()
        self.assertFalse(self.resources.exists())

    def test_unknown_resource_directory_is_not_overwritten_or_downloaded(self):
        self.resources.mkdir()
        sentinel = self.resources / 'sentinel'
        sentinel.write_bytes(b'keep')
        with patch.object(stage, 'POLICY', self.policy), patch.object(stage, 'download_official') as download:
            with self.assertRaisesRegex(ValueError, 'fresh resource'):
                self.run_stage()
            download.assert_not_called()
        self.assertEqual(sentinel.read_bytes(), b'keep')
        self.assertFalse(self.output.exists())

    def test_staging_copies_the_exact_official_source_and_isolated_config(self):
        commands = []
        def built(argv, check):
            commands.append(argv)
            bundle = self.output / 'platform-bundle'
            bundle.mkdir()
            archive = bundle / 'bundle.tar.gz'
            archive.write_bytes(b'synthetic built platform closure')
            manifest = bundle / 'manifest.json'
            manifest.write_bytes(b'{"schemaVersion":1}')
            (bundle / 'result.json').write_text(json.dumps({'archive': str(archive),
                'archiveName': archive.name, 'manifest': str(manifest)}))
        with patch.object(stage, 'POLICY', self.policy), patch.object(stage.subprocess, 'run', side_effect=built):
            result = self.run_stage()
        self.assertEqual((self.resources / 'official-runtime.tgz').read_bytes(), self.archive.read_bytes())
        self.assertFalse((self.resources / 'maintained-runtime.tgz').exists())
        self.assertEqual((self.resources / 'runtime-bundle.manifest.sha256').read_text().strip(), result['descriptorSha256'])
        self.assertTrue(Path(result['tauriConfig']).is_relative_to(self.output.resolve()))
        self.assertEqual(len(commands), 1)
        self.assertIn('--policy', commands[0])
        self.assertNotIn('--execute', commands[0])
        self.assertIn('build_desktop_runtime_bundle.py', commands[0][1])


if __name__ == '__main__':
    unittest.main()
