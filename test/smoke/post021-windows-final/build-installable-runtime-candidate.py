"""Build an explicitly local test tarball; never modifies or publishes a package.

Inputs: registry metadata, its original tgz, reviewed source checkout, Mac port
review JSON, and a fresh output directory. The three runtime files must match
the supplied review exactly. A separately reviewed provider-reload baseline fix
may be included in a new candidate revision. Other files retain original bytes.
"""
import base64
import gzip
import hashlib
import io
import json
import re
import sys
import tarfile
from pathlib import Path, PurePosixPath

RUNTIME_PATHS = (
    'src/codex/desktop-app/windows.ts',
    'src/codex/app-server-processes.ts',
    'src/codex/windows-process-identity.ts',
)
COMPATIBILITY_MANIFEST = 'src/generated/compatibility-version.json'

def sha(data):
    return hashlib.sha256(data).hexdigest()

def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')

def build(registry_path, archive_path, source_root, review_path, output_root):
    registry = json.loads(Path(registry_path).read_bytes())
    review = json.loads(Path(review_path).read_bytes())
    extra = review.get('extraRuntimePaths', [])
    assert extra in ([], ['src/config/live-reconcile.ts']), 'Unapproved extra runtime path'
    runtime_paths = (*RUNTIME_PATHS, *extra)
    revision = review.get('candidateRevision', 1)
    assert type(revision) is int and 1 <= revision <= 99, 'Invalid candidate revision'
    source_root, output_root = Path(source_root).resolve(), Path(output_root).resolve()
    assert not output_root.exists(), 'A fresh output directory is required'
    archive = Path(archive_path).read_bytes()
    assert registry['name'] == '@bitkyc08/opencodex'
    assert re.fullmatch(r'\d+\.\d+\.\d+', registry['version'])
    assert registry['dist']['integrity'] == 'sha512-' + base64.b64encode(hashlib.sha512(archive).digest()).decode()
    assert registry['dist']['shasum'] == hashlib.sha1(archive).hexdigest()
    files, modes = {}, {}
    with tarfile.open(fileobj=io.BytesIO(archive), mode='r:gz') as tar:
        for member in tar:
            path = PurePosixPath(member.name)
            assert not path.is_absolute() and '..' not in path.parts and path.parts[0] == 'package'
            assert not member.issym() and not member.islnk(), 'Archive links are forbidden'
            if member.isdir():
                continue
            assert member.isfile() and member.size <= 128 * 1024 * 1024
            name = str(path.relative_to('package'))
            assert name not in files, 'Duplicate archive path'
            files[name], modes[name] = tar.extractfile(member).read(), member.mode
    assert len(files) <= 20000 and sum(map(len, files.values())) <= 512 * 1024 * 1024
    package = json.loads(files['package.json'])
    assert package['name'] == registry['name'] and package['version'] == registry['version']
    original = dict(files)
    by_path = {row['path']: row for row in review['files']}
    for name in runtime_paths:
        expected = by_path[name]
        if expected['beforeSha256'] is None:
            assert name not in files, 'New helper already exists in the release'
        else:
            assert sha(files[name]) == expected['beforeSha256'], 'Release preimage drift: ' + name
        candidate_path = source_root / name
        assert candidate_path.is_file() and not candidate_path.is_symlink()
        data = candidate_path.read_bytes()
        assert sha(data) == expected['afterSha256'], 'Candidate source drift: ' + name
        files[name], modes[name] = data, 0o644
    version = registry['version'] + '-post021.ownerquery.' + str(revision)
    identity = {
        'kind': 'local-installable-candidate-not-upstream-published',
        'basePublishedVersion': registry['version'],
        'baseGitHead': registry['gitHead'],
        'baseTarballSha256': sha(archive),
        'portTargetDev': review['targetDev'],
        'reviewedPatchSha256': review['patchSha256'],
        'runtimeFiles': {name: sha(files[name]) for name in runtime_paths},
    }
    package.update(version=version, post021Candidate=identity)
    files['package.json'] = json_bytes(package)
    # The published compatibility identity must not continue claiming old source.
    # Preserve every unrelated manifest entry; replace only changed package/src
    # bytes and explicitly register the new embedded-query module.
    manifest = json.loads(files[COMPATIBILITY_MANIFEST])
    records = {row['path']: row for row in manifest['files']}
    assert len(records) == len(manifest['files']) and 'package.json' in records
    for name in ('package.json', *runtime_paths):
        records[name] = {'path': name, 'sha256': sha(files[name])}
    manifest['files'] = [records[key] for key in sorted(records, key=lambda x: x.encode())]
    files[COMPATIBILITY_MANIFEST] = json_bytes(manifest)
    allowed = {'package.json', COMPATIBILITY_MANIFEST, *runtime_paths}
    assert all(name in allowed or data == original[name] for name, data in files.items())
    output_root.mkdir(parents=True)
    target = output_root / ('opencodex-' + version + '.tgz')
    with target.open('xb') as raw, gzip.GzipFile(filename='', mode='wb', fileobj=raw, mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode='w', format=tarfile.PAX_FORMAT) as tar:
            for name, data in sorted(files.items()):
                member = tarfile.TarInfo('package/' + name)
                member.size, member.mode, member.mtime = len(data), modes.get(name, 0o644), 0
                tar.addfile(member, io.BytesIO(data))
    report = {
        **identity, 'version': version, 'archive': target.name,
        'sha256': sha(target.read_bytes()), 'bytes': target.stat().st_size,
        'changedFiles': sorted(name for name in files if original.get(name) != files[name]),
        'files': [{'path': name, 'sha256': sha(data), 'bytes': len(data)} for name, data in sorted(files.items())],
        'delivery': 'Isolated test import only. Production delivery requires an upstream release or explicit approval of a changed installation source.',
    }
    (output_root / 'candidate-manifest.json').write_bytes(json_bytes(report))
    return report

if __name__ == '__main__':
    assert len(sys.argv) == 6, 'registry.json original.tgz source-root review.json fresh-output'
    report = build(*sys.argv[1:])
    print(json.dumps({key: report[key] for key in ['kind', 'version', 'archive', 'sha256', 'bytes', 'changedFiles']}))
