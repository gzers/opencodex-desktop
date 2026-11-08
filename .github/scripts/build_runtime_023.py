"""Offline, bounded rebuild of Desktop 0.2.23's runtime from the adopted 0.2.22 tgz.

Usage: python3 .github/scripts/build_runtime_023.py FRESH_OUTPUT --archive OLD_TGZ
       [--original ORIGINAL_NPM_TGZ]
No install, download, service, or upstream checkout is used. Two independent patch
applications and canonical repacks must agree before any output is published.
"""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[2]
REVIEW = ROOT / 'test/smoke/runtime023-name-review.json'
PATCH = ROOT / 'test/smoke/runtime023-name-projection.patch'
VERSION = '2.97.0-desktop023.names.2'
BASE_VERSION = '2.97.0-post021.ownerquery.2'
BASE_SHA256 = 'bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957'
ORIGINAL_SHA256 = '3859a64198a05b0ebdbac0593b0cc6b675ec14f32b69ee100cda7a173404e552'
SOURCE_PATHS = ('bin/ocx.mjs', 'src/codex/catalog/derive-entry.ts', 'src/codex/catalog/gather-capture.ts', 'src/codex/catalog/model-hints.ts', 'src/codex/catalog/parsing.ts', 'src/config/diagnostics.ts', 'src/config/live-reconcile.ts', 'src/config/proxy-env.ts', 'src/config/schema/config-schema.ts', 'src/types/provider.ts')

COMPATIBILITY = 'src/generated/compatibility-version.json'
TEMP_ROOT = Path('/private/tmp') if Path('/private/tmp').is_dir() else Path(tempfile.gettempdir())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()


def read_archive(data):
    files, modes = {}, {}
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
        total = 0
        for member in archive:
            path = PurePosixPath(member.name)
            require(not path.is_absolute() and '..' not in path.parts
                    and path.parts and path.parts[0] == 'package', 'Unsafe archive path')
            require(not member.issym() and not member.islnk(), 'Archive links forbidden')
            if member.isdir():
                continue
            require(member.isfile() and member.size <= 128 * 1024 * 1024, 'Invalid archive member')
            total += member.size
            require(len(files) < 20000 and total <= 512 * 1024 * 1024, 'Archive budget exceeded')
            name = str(path.relative_to('package'))
            require(name not in files, 'Duplicate archive path')
            files[name], modes[name] = archive.extractfile(member).read(), member.mode
    return files, modes


def canonical_archive(files, modes):
    output = io.BytesIO()
    with gzip.GzipFile(filename='', mode='wb', fileobj=output, mtime=0, compresslevel=9) as compressed:
        with tarfile.open(fileobj=compressed, mode='w', format=tarfile.PAX_FORMAT) as archive:
            for name in sorted(files):
                member = tarfile.TarInfo('package/' + name)
                member.size, member.mode, member.mtime = len(files[name]), modes[name], 0
                member.uid = member.gid = 0
                member.uname = member.gname = ''
                archive.addfile(member, io.BytesIO(files[name]))
    return output.getvalue()


def patched_files(original, review):
    require(tuple(row['path'] for row in review['files']) == SOURCE_PATHS, 'Unapproved source paths')
    require(review['version'] == VERSION and review['baseSha256'] == BASE_SHA256, 'Review identity drift')
    require(sha(PATCH.read_bytes()) == review['patchSha256'], 'Reviewed patch changed')
    files = dict(original)
    with tempfile.TemporaryDirectory(prefix='ocxd023-runtime-name-patch-', dir=TEMP_ROOT) as directory:
        source = Path(directory)
        for row in review['files']:
            name = row['path']
            require(sha(files[name]) == row['beforeSha256'], 'Source preimage drift: ' + name)
            target = source/name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(files[name])
        command = ['git', '-c', 'core.autocrlf=false', 'apply', '--no-index', '--unidiff-zero', '--whitespace=error']
        subprocess.run([*command, '--check', str(PATCH)], cwd=source, check=True, capture_output=True)
        subprocess.run([*command, str(PATCH)], cwd=source, check=True, capture_output=True)
        actual = sorted(p.relative_to(source).as_posix() for p in source.rglob('*') if p.is_file())
        require(actual == list(SOURCE_PATHS), 'Patch changed unapproved paths')
        for row in review['files']:
            data = (source/row['path']).read_bytes()
            require(sha(data) == row['afterSha256'], 'Patch postimage drift: ' + row['path'])
            files[row['path']] = data
    package = json.loads(files['package.json'])
    require(package['name'] == '@bitkyc08/opencodex' and package['version'] == BASE_VERSION,
            'Unexpected base package identity')
    require(package['post021Candidate']['baseTarballSha256'] == ORIGINAL_SHA256, 'Base lineage drift')
    package['version'] = VERSION
    package['desktop023Runtime'] = {
        'kind': 'project-maintained-offline-runtime',
        'baseVersion': BASE_VERSION, 'baseTarballSha256': BASE_SHA256,
        'originalPublishedVersion': '2.97.0', 'originalTarballSha256': ORIGINAL_SHA256,
        'patchSha256': review['patchSha256'],
        'runtimeFiles': {row['path']: row['afterSha256'] for row in review['files']},
    }
    files['package.json'] = json_bytes(package)
    manifest = json.loads(files[COMPATIBILITY])
    records = {row['path']: row for row in manifest['files']}
    require(len(records) == len(manifest['files']), 'Duplicate compatibility records')
    for name in ('package.json', *SOURCE_PATHS):
        require(name in records or name == 'bin/ocx.mjs', 'Missing compatibility record: ' + name)
        records[name] = {**records.get(name, {'path': name}), 'sha256': sha(files[name])}
    manifest['files'] = [records[key] for key in sorted(records)]
    files[COMPATIBILITY] = json_bytes(manifest)
    allowed = {'package.json', COMPATIBILITY, *SOURCE_PATHS}
    require(set(files) == set(original), 'Archive membership changed')
    require({name for name in files if files[name] != original[name]} == allowed,
            'Unexpected changed-file set')
    return files


def build(output, archive_path, original_path=None):
    output = Path(output)
    require(not output.exists(), 'A fresh output directory is required')
    data = Path(archive_path).read_bytes()
    require(sha(data) == BASE_SHA256, 'Pinned maintained archive changed')
    if original_path is not None:
        require(sha(Path(original_path).read_bytes()) == ORIGINAL_SHA256, 'Pinned original archive changed')
    original, modes = read_archive(data)
    review = json.loads(REVIEW.read_bytes())
    results = [canonical_archive(patched_files(original, review), modes) for _ in range(2)]
    require(results[0] == results[1], 'Independent rebuilds differ')
    files, _ = read_archive(results[0])
    archive_name = 'opencodex-' + VERSION + '.tgz'
    report = {
        'schemaVersion': 1, 'kind': 'project-maintained-offline-runtime',
        'maintainer': 'gzers/opencodex-desktop', 'upstreamPublished': False,
        'installation': 'desktop-integrated-runtime',
        'version': VERSION, 'archive': archive_name,
        'sha256': sha(results[0]), 'bytes': len(results[0]),
        'base': {'version': BASE_VERSION, 'sha256': BASE_SHA256, 'bytes': len(data)},
        'original': {'version': '2.97.0', 'sha256': ORIGINAL_SHA256,
                     'verifiedInput': original_path is not None},
        'patches': [{'file': PATCH.name, 'sha256': review['patchSha256']}],
        'runtimeFiles': {name: sha(files[name]) for name in SOURCE_PATHS},
        'changedFiles': sorted(name for name in files if files[name] != original[name]),
        'files': [{'path': name, 'sha256': sha(files[name]), 'bytes': len(files[name])} for name in sorted(files)],
        'reproducibility': {'builds': 2, 'sha256': [sha(result) for result in results],
                            'equalBytes': True, 'gzipMtime': 0, 'tarMtime': 0,
                            'tarOwner': '0:0', 'order': 'sorted-path', 'format': 'pax'},
    }
    output.mkdir(parents=True)
    (output/archive_name).write_bytes(results[0])
    (output/'maintained-runtime.json').write_bytes(json_bytes(report))
    (output/'maintained-runtime.sha256').write_text(report['sha256'] + '  ' + archive_name + '\n')
    (output/'runtime023-rebuild-summary.json').write_bytes(json_bytes({
        key: report[key] for key in ('version', 'archive', 'sha256', 'bytes', 'changedFiles', 'reproducibility')}))
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--archive', required=True, type=Path)
    parser.add_argument('--original', type=Path)
    args = parser.parse_args()
    result = build(args.output, args.archive, args.original)
    print(json.dumps({key: result[key] for key in ('version', 'archive', 'sha256', 'bytes', 'changedFiles', 'reproducibility')}, indent=2))
