"""Read-only official runtime delivery verification; never extract or execute code.

The checked-in policy is the trust input. A self-consistent descriptor alone
does not prove that the installed official package still has upstream bytes.
This is an assembly check, not native lifecycle or release acceptance.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import posixpath
import tarfile

ROOT = Path(__file__).resolve().parents[2]
POLICY = ROOT / '.github/runtime-platforms-024.json'
MAX_ARCHIVE = 512 * 1024 * 1024
MAX_UNPACKED = 2 * 1024 * 1024 * 1024
MAX_ENTRIES = 100000
PACKAGE = 'runtime/opencodex/node_modules/@bitkyc08/opencodex/'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha256(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def load_json(raw):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, 'duplicate JSON key: ' + key)
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=unique)


def safe_path(name):
    require(isinstance(name, str) and name and '\\' not in name and ':' not in name
            and not any(ord(c) < 32 for c in name), 'unsafe archive path')
    path = PurePosixPath(name)
    require(not path.is_absolute() and '..' not in path.parts and str(path) == name,
            'unsafe archive path: ' + name)
    return path


def regular(path, limit):
    require(not path.is_symlink() and path.is_file(), 'resource must be a regular file: ' + path.name)
    info = path.stat()
    require(info.st_nlink == 1 and 0 < info.st_size <= limit, 'invalid resource size/link count: ' + path.name)


def file_row(archive, member, capture=False):
    h, data = hashlib.sha256(), bytearray() if capture else None
    with archive.extractfile(member) as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            h.update(block)
            if data is not None:
                require(len(data) + len(block) <= 16 * 1024 * 1024, 'JSON entry exceeds read budget')
                data.extend(block)
    return {'kind': 'file', 'bytes': member.size, 'sha256': h.hexdigest()}, data


def finish_archive(archive):
    # tar iteration stops at its first end marker. Do not let a second archive
    # or an unbounded decompression tail hide after the verified membership.
    remaining = 0
    while block := archive.fileobj.read(10240):
        remaining += len(block)
        require(remaining <= 10240 and not any(block), 'unexpected archive trailer')


def upstream_rows(path):
    rows, total, count = {}, 0, 0
    with gzip.open(path, 'rb') as compressed, tarfile.open(fileobj=compressed, mode='r|') as archive:
        for member in archive:
            count += 1
            require(count <= MAX_ENTRIES, 'upstream entry count exceeds budget')
            name = member.name.rstrip('/') if member.isdir() else member.name
            safe_path(name)
            require(name == 'package' or name.startswith('package/'), 'upstream package root differs')
            if member.isdir():
                continue
            require(member.type in (tarfile.REGTYPE, tarfile.AREGTYPE) and not member.pax_headers,
                    'unsupported upstream member')
            relative = name.removeprefix('package/')
            require(relative not in rows, 'duplicate upstream member')
            total += member.size
            require(0 <= member.size and total <= MAX_UNPACKED, 'upstream size exceeds budget')
            rows[relative], _ = file_row(archive, member)
        finish_archive(archive)
    require(rows, 'empty upstream package')
    return rows


def verify(directory, platform, policy=None):
    policy = load_json(POLICY.read_bytes()) if policy is None else policy
    require(policy['bundleVersion'] == '0.2.24' and policy['runtime']['distribution'] == 'official-npm'
            and policy['runtime']['version'] == '2.99.0', 'unexpected official runtime policy')
    require(platform in policy['platforms'], 'unsupported platform')
    target, source = policy['platforms'][platform], policy['runtime']
    directory = Path(directory)
    require(not directory.is_symlink() and directory.is_dir(), 'bundle-local resource directory required')
    descriptor = directory / 'runtime-bundle.manifest.json'
    pin = directory / 'runtime-bundle.manifest.sha256'
    official = directory / 'official-runtime.tgz'
    regular(descriptor, 64 * 1024 * 1024); regular(pin, 128); regular(official, MAX_ARCHIVE)
    raw = descriptor.read_bytes()
    value = load_json(raw)
    descriptor_hash = hashlib.sha256(raw).hexdigest()
    require(pin.read_text().strip() == descriptor_hash, 'descriptor pin mismatch')
    for key, expected in {'schemaVersion': 1, 'platform': platform,
                          'runtimeVersion': source['version'], 'nodeVersion': policy['node']['version'],
                          'bunVersion': policy['bunVersion'], 'runtimeArchiveSha256': source['sha256'],
                          'archiveName': target['archiveName']}.items():
        require(value.get(key) == expected, 'descriptor policy mismatch: ' + key)
    name = value['archiveName']
    require(str(safe_path(name)) == name and '/' not in name, 'archive name must be a basename')
    bundle = directory / name
    regular(bundle, MAX_ARCHIVE)
    require(bundle.stat().st_size == value['archiveBytes'] and sha256(bundle) == value['archiveSha256'],
            'platform archive digest/size mismatch')
    require(sha256(official) == source['sha256'], 'official source digest mismatch')
    upstream = upstream_rows(official)
    entries = value['entries']
    require(isinstance(entries, list) and 0 < len(entries) <= MAX_ENTRIES, 'invalid descriptor entry count')
    names = [row['path'] for row in entries]
    require(names == sorted(set(names)), 'descriptor paths must be unique and sorted')
    by_name = dict(zip(names, entries))
    for name in names:
        path = safe_path(name)
        require(name.startswith('runtime/'), 'member outside runtime')
        require(not any(str(parent) in by_name for parent in path.parents), 'file/symlink used as directory')
        row = by_name[name]
        if row['kind'] == 'symlink':
            link = row['target']
            require(isinstance(link, str) and link and not link.startswith('/')
                    and '\\' not in link and ':' not in link, 'unsafe symlink target')
            visited, resolved = {name}, posixpath.normpath(posixpath.join(str(path.parent), link))
            while resolved in by_name and by_name[resolved]['kind'] == 'symlink':
                require(resolved not in visited, 'cyclic symlink')
                visited.add(resolved)
                resolved = posixpath.normpath(posixpath.join(posixpath.dirname(resolved), by_name[resolved]['target']))
            require(resolved.startswith('runtime/') and resolved in by_name
                    and by_name[resolved]['kind'] == 'file', 'symlink must resolve to bundle-local regular file')
        else:
            require(row['kind'] == 'file' and type(row['bytes']) is int and row['bytes'] >= 0
                    and row['mode'] in (0o644, 0o755), 'invalid file descriptor')
    wanted = {'runtime/opencodex/package.json', 'runtime/opencodex/package-lock.json'}
    wanted.update('runtime/opencodex/' + name + '/package.json'
                  for name in policy['packageLock']['packages'] if name)
    total, count, installed, documents = 0, 0, {}, {}
    with gzip.open(bundle, 'rb') as compressed, tarfile.open(fileobj=compressed, mode='r|') as archive:
        for member in archive:
            require(count < len(entries), 'extra platform archive member')
            row = entries[count]; count += 1
            require(member.name == row['path'] and not member.pax_headers, 'archive member order/path differs')
            require(member.uid == member.gid == member.mtime == 0, 'noncanonical archive metadata')
            if row['kind'] == 'symlink':
                require(member.issym() and member.linkname == row['target'] and member.mode == 0o777,
                        'archive symlink differs')
                continue
            total += member.size
            require(member.type == tarfile.REGTYPE and member.size == row['bytes'] and member.mode == row['mode']
                    and total <= MAX_UNPACKED, 'archive file type/size/mode differs')
            observed, data = file_row(archive, member, member.name in wanted)
            require(observed['sha256'] == row['sha256'], 'archive file digest differs: ' + member.name)
            if member.name.startswith(PACKAGE):
                installed[member.name.removeprefix(PACKAGE)] = observed
            if data is not None:
                documents[member.name] = load_json(data)
        finish_archive(archive)
    require(count == len(entries), 'missing platform archive member')
    require(installed == upstream, 'installed official package differs from upstream files')
    source_row = by_name.get('runtime/opencodex/sources/' + source['archive'], {})
    require(source_row.get('sha256') == source['sha256'] and source_row.get('kind') == 'file',
            'embedded recovery source differs')
    require(documents.get('runtime/opencodex/package-lock.json') == policy['packageLock']
            and documents.get('runtime/opencodex/package.json') == policy['projectPackage'], 'production lock/project differs')
    for name in (target['nodeExecutable'], target['npmCli'], target['bunExecutable'],
                 PACKAGE + 'bin/ocx.mjs'):
        require(name in by_name and by_name[name]['kind'] == 'file', 'missing runtime entry: ' + name)
    packages = 0
    for name, row in policy['packageLock']['packages'].items():
        if not name:
            continue
        os_ok = 'os' not in row or target['os'] in row['os'] or row['os'] == ['!aix']
        cpu_ok = 'cpu' not in row or target['cpu'] in row['cpu']
        if os_ok and cpu_ok:
            actual = documents.get('runtime/opencodex/' + name + '/package.json', {})
            require(actual.get('version') == row['version'], 'locked production package missing/drifted: ' + name)
            packages += 1
    return {'schemaVersion': 1, 'status': 'verified-offline', 'platform': platform,
            'descriptorSha256': descriptor_hash, 'officialRuntimeVersion': source['version'],
            'officialRuntimeSha256': source['sha256'], 'platformArchiveSha256': value['archiveSha256'],
            'platformArchiveBytes': value['archiveBytes'], 'entries': count, 'unpackedBytes': total,
            'upstreamFiles': len(upstream), 'lockedPackages': packages,
            'codeExecuted': False, 'releaseAcceptance': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--platform', choices=['darwin-arm64', 'win32-x64'], required=True)
    args = parser.parse_args()
    print(json.dumps(verify(args.directory, args.platform), indent=2))
