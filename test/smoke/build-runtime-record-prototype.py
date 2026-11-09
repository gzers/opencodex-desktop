"""Deterministic, offline owner-fix archive prototype. Never updates release pins."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('runtime_builder', ROOT / '.github/scripts/build_runtime_023.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)
VERSION = '2.97.0-desktop023.lifecycle.1-record-prototype'
BASE = 'fcd2740ecc32ae1e11c1c79d84c801b2c5d5447ae89e070ab060548d61f75ee6'
PATCH = ROOT / 'test/smoke/runtime023-record-owner.patch'
REVIEW = ROOT / 'test/smoke/runtime023-record-owner-review.json'


def build(archive, output, materialize=False):
    output = Path(output)
    assert not output.exists(), 'Fresh output; never overwrite original evidence'
    raw = Path(archive).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == BASE
    original, modes = builder.read_archive(raw)
    review = json.loads(REVIEW.read_text())
    assert review['baseArchiveSha256'] == BASE and review['candidateOnly'] is True
    assert hashlib.sha256(PATCH.read_bytes()).hexdigest() == review['patchSha256']
    assert len(review['files']) == 2
    results = []
    for _ in range(2):
        files = dict(original)
        candidate_modes = dict(modes)
        with tempfile.TemporaryDirectory(prefix='ocxd-record-prototype-', dir=builder.TEMP_ROOT) as directory:
            root = Path(directory)
            for row in review['files']:
                name = row['path']
                assert name in ['src/config/atomic-write.ts', 'src/lib/windows-runtime-private-record.ts']
                if row.get('before') == 'absent':
                    assert name not in files
                else:
                    assert hashlib.sha256(files[name]).hexdigest() == row['beforeSha256']
                    p = root / name
                    p.parent.mkdir(parents=True, exist_ok=True)
                    p.write_bytes(files[name])
            args = ['git', 'apply', '--no-index', '--whitespace=error']
            subprocess.run(args + ['--check', str(PATCH)], cwd=root, check=True, capture_output=True)
            subprocess.run(args + [str(PATCH)], cwd=root, check=True, capture_output=True)
            actual = sorted(str(p.relative_to(root)) for p in root.rglob('*') if p.is_file())
            assert actual == sorted(row['path'] for row in review['files'])
            for row in review['files']:
                data = (root / row['path']).read_bytes()
                assert hashlib.sha256(data).hexdigest() == row['afterSha256']
                files[row['path']] = data
                candidate_modes[row['path']] = modes.get(row['path'], 0o644)
        package = json.loads(files['package.json'])
        assert package['version'] == '2.97.0-desktop023.names.2'
        package['version'] = VERSION
        package['desktop023RecordPrototype'] = dict(
            candidateOnly=True, userAdopted='2026-11-09', baseVersion='2.97.0-desktop023.names.2',
            baseArchiveSha256=BASE, patchSha256=review['patchSha256'],
            scope='New runtime-port sync atomic temp creation only. Lifetime protocol not integrated. Not a release artifact.',
            files={row['path']: row['afterSha256'] for row in review['files']})
        files['package.json'] = builder.json_bytes(package)
        manifest = json.loads(files[builder.COMPATIBILITY])
        records = {row['path']: row for row in manifest['files']}
        assert len(records) == len(manifest['files'])
        for name in ['package.json', *(row['path'] for row in review['files'])]:
            records[name] = {**records.get(name, {'path': name}), 'sha256': builder.sha(files[name])}
        manifest['files'] = [records[key] for key in sorted(records)]
        files[builder.COMPATIBILITY] = builder.json_bytes(manifest)
        changed = {name for name in files if name not in original or files[name] != original[name]}
        assert changed == {'package.json', builder.COMPATIBILITY, *(row['path'] for row in review['files'])}
        assert set(files) - set(original) == {'src/lib/windows-runtime-private-record.ts'}
        results.append(builder.canonical_archive(files, candidate_modes))
    assert results[0] == results[1], 'Two independent canonical rebuilds must match'
    rebuilt, rebuilt_modes = builder.read_archive(results[0])
    compatibility = json.loads(rebuilt[builder.COMPATIBILITY])
    original_records = {row['path']: row for row in json.loads(original[builder.COMPATIBILITY])['files']}
    inherited_absent = []
    for row in compatibility['files']:
        if row['path'] in rebuilt:
            assert builder.sha(rebuilt[row['path']]) == row['sha256']
        else:
            assert row['path'] not in original and row == original_records[row['path']]
            inherited_absent.append(row['path'])
    output.mkdir()
    path = output / ('opencodex-' + VERSION + '.tgz')
    path.write_bytes(results[0])
    if materialize:
        package_root = output / 'package'
        package_root.mkdir()
        for name, data in rebuilt.items():
            target = package_root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            target.chmod(rebuilt_modes[name])
    summary = dict(accepted=False, diagnosticOnly=True, candidateOnly=True,
                   version=VERSION, baseArchiveSha256=BASE, patchSha256=review['patchSha256'],
                   archive=path.name, bytes=path.stat().st_size, sha256=builder.sha(results[0]),
                   twoIndependentBuildsIdentical=True, archiveMembers=len(rebuilt),
                   compatibilityPresentMembersVerified=len(compatibility['files']) - len(inherited_absent),
                   compatibilityInheritedAbsentMembersUnchanged=inherited_absent,
                   productionPinsChanged=False, lifetimeProtocolIntegrated=False,
                   releaseAccepted=False, materializedExactArchive=materialize)
    (output / 'verification.json').write_bytes(builder.json_bytes(summary))
    print(json.dumps(summary))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--materialize', action='store_true')
    args = parser.parse_args()
    build(args.archive, args.output, args.materialize)
