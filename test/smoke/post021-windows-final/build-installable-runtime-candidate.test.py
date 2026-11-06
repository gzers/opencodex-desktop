import base64
import hashlib
import importlib.util
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('candidate_builder', Path(__file__).with_name('build-installable-runtime-candidate.py'))
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)

class CandidateDeliveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='post021-candidate-test-')
        self.root = Path(self.temp.name)
        self.source = self.root / 'source'
        self.files = {
            'package.json': builder.json_bytes({'name': '@bitkyc08/opencodex', 'version': '2.97.0', 'bin': {'ocx': 'bin/ocx.mjs'}}),
            'bin/ocx.mjs': b'original launcher',
            'gui/dist/index.html': b'original GUI',
            builder.RUNTIME_PATHS[0]: b'old desktop',
            builder.RUNTIME_PATHS[1]: b'old appserver',
        }
        self.files[builder.COMPATIBILITY_MANIFEST] = builder.json_bytes({'files': [{'path': name, 'sha256': builder.sha(data)} for name, data in self.files.items()]})
        rows = []
        for index, name in enumerate(builder.RUNTIME_PATHS):
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            data = ('reviewed native source ' + str(index)).encode()
            path.write_bytes(data)
            rows.append({'path': name, 'beforeSha256': builder.sha(self.files[name]) if name in self.files else None, 'afterSha256': builder.sha(data)})
        self.review = {'files': rows, 'targetDev': '0' * 40, 'patchSha256': '1' * 64}
        self.review_path = self.root / 'review.json'
        self.review_path.write_bytes(builder.json_bytes(self.review))
        self.archive = self.root / 'original.tgz'
        self.write_archive()

    def tearDown(self):
        self.temp.cleanup()

    def write_archive(self, malicious=False):
        with tarfile.open(self.archive, 'w:gz') as tar:
            for name, data in self.files.items():
                member = tarfile.TarInfo('package/' + name)
                member.size = len(data)
                tar.addfile(member, io.BytesIO(data))
            if malicious:
                member = tarfile.TarInfo('package/src/link')
                member.type, member.linkname = tarfile.SYMTYPE, '../../outside'
                tar.addfile(member)
        data = self.archive.read_bytes()
        self.registry = {'name': '@bitkyc08/opencodex', 'version': '2.97.0', 'gitHead': '2' * 40, 'dist': {'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(data).digest()).decode(), 'shasum': hashlib.sha1(data).hexdigest()}}
        self.registry_path = self.root / 'registry.json'
        self.registry_path.write_bytes(builder.json_bytes(self.registry))

    def build(self, output='output'):
        return builder.build(self.registry_path, self.archive, self.source, self.review_path, self.root / output)

    def test_test_artifact_has_distinct_version_updated_identity_and_unchanged_gui(self):
        first, second = self.build(), self.build('second')
        self.assertEqual(first['sha256'], second['sha256'])
        self.assertEqual(first['kind'], 'local-installable-candidate-not-upstream-published')
        self.assertNotEqual(first['version'], self.registry['version'])
        with tarfile.open(self.root / 'output' / first['archive']) as tar:
            read = lambda name: tar.extractfile('package/' + name).read()
            self.assertEqual(read('gui/dist/index.html'), self.files['gui/dist/index.html'])
            package = json.loads(read('package.json'))
            self.assertEqual(package['post021Candidate']['baseTarballSha256'], builder.sha(self.archive.read_bytes()))
            records = {row['path']: row['sha256'] for row in json.loads(read(builder.COMPATIBILITY_MANIFEST))['files']}
            for name in ('package.json', *builder.RUNTIME_PATHS):
                self.assertEqual(records[name], builder.sha(read(name)))

    def test_release_integrity_and_reviewed_source_are_required_before_output(self):
        self.archive.write_bytes(self.archive.read_bytes() + b'tampered')
        with self.assertRaises(AssertionError):
            self.build()
        self.assertFalse((self.root / 'output').exists())
        self.write_archive()
        (self.source / builder.RUNTIME_PATHS[0]).write_bytes(b'unreviewed')
        with self.assertRaisesRegex(AssertionError, 'Candidate source drift'):
            self.build()
        self.assertFalse((self.root / 'output').exists())

    def test_archive_link_rejected_and_existing_output_never_overwritten(self):
        self.write_archive(malicious=True)
        with self.assertRaisesRegex(AssertionError, 'Archive links'):
            self.build()
        self.write_archive()
        target = self.root / 'output'
        target.mkdir()
        marker = target / 'keep'
        marker.write_bytes(b'unchanged')
        with self.assertRaisesRegex(AssertionError, 'fresh output'):
            self.build()
        self.assertEqual(marker.read_bytes(), b'unchanged')

    def test_reviewed_baseline_fix_gets_a_distinct_revision_and_exact_manifest(self):
        name = 'src/config/live-reconcile.ts'
        before, after = b'original provider adoption', b'reviewed provider discovery adoption'
        self.files[name] = before
        target = self.source / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(after)
        self.review.update(candidateRevision=2, extraRuntimePaths=[name])
        self.review['files'].append({'path': name, 'beforeSha256': builder.sha(before), 'afterSha256': builder.sha(after)})
        self.review_path.write_bytes(builder.json_bytes(self.review))
        self.write_archive()
        result = self.build()
        self.assertEqual(result['version'], '2.97.0-post021.ownerquery.2')
        self.assertEqual(result['runtimeFiles'][name], builder.sha(after))
        with tarfile.open(self.root / 'output' / result['archive']) as tar:
            self.assertEqual(tar.extractfile('package/' + name).read(), after)
            self.assertEqual(tar.extractfile('package/gui/dist/index.html').read(), self.files['gui/dist/index.html'])

    def test_unreviewed_additions_or_invalid_revision_are_rejected(self):
        for extra, revision in [(['src/unrelated.ts'], 2), ([], 0), ([], True), ([], '2')]:
            self.review.update(extraRuntimePaths=extra, candidateRevision=revision)
            self.review_path.write_bytes(builder.json_bytes(self.review))
            with self.assertRaises(AssertionError):
                self.build()
            self.assertFalse((self.root / 'output').exists())

if __name__ == '__main__':
    unittest.main()
