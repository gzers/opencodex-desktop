"""Build the complete offline I/runtime tree; never execute dependency lifecycle scripts.

Inputs: checked-in platform/production lock, the pinned official/runtime tgz, an
official Node archive, and explicit absolute build-host Node/npm CLI paths.
Output: one deterministic platform .tar.gz and its external descriptor. The
Desktop deployment context generates runtime/bin/ocx(.cmd) after extraction.
Failed work, npm logs and partial downloads remain under the requested output.
"""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform as host_platform
import shutil
import stat
import struct
import subprocess
import sys
import tarfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[2]
POLICY = ROOT/'.github/runtime-platforms-023.json'
MAX_ARCHIVE = 512 * 1024 * 1024
MAX_UNPACKED = 2 * 1024 * 1024 * 1024
MAX_ENTRIES = 100000
PACKAGE = 'node_modules/@bitkyc08/opencodex'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()


def relative_path(name):
    p = PurePosixPath(name)
    require(name and not p.is_absolute() and '\\' not in name and ':' not in name
            and '..' not in p.parts and str(p) == name and not name.startswith('/'),
            'Unsafe relative path: ' + name)
    return p


def validate_link(root, path, target):
    require(target and not PurePosixPath(target).is_absolute() and '\\' not in target
            and ':' not in target, 'Unsafe symlink target: ' + str(path))
    resolved = (path.parent/target).resolve(strict=True)
    require(resolved.is_relative_to(root.resolve()) and resolved.is_file(),
            'Symlink escapes runtime or has no regular target: ' + str(path))


def binary_kind(path, target):
    """Read the actual native header; reject JS stubs and the wrong CPU/OS."""
    with Path(path).open('rb') as source:
        header = source.read(4096)
        if target == 'darwin-arm64':
            require(header[:4] == b'\xcf\xfa\xed\xfe' and len(header) >= 8
                    and struct.unpack('<I', header[4:8])[0] == 0x100000c,
                    'Expected arm64 Mach-O native binary: ' + str(path))
            return 'mach-o-arm64'
        require(header[:2] == b'MZ' and len(header) >= 64, 'Expected native PE: ' + str(path))
        offset = struct.unpack('<I', header[60:64])[0]
        require(offset <= 1024 * 1024, 'Unreasonable PE header offset')
        source.seek(offset)
        pe = source.read(6)
        require(pe[:4] == b'PE\0\0' and pe[4:] == b'\x64\x86', 'Expected x64 PE: ' + str(path))
        return 'pe-x64'


def extract_node(archive, destination, spec):
    require(digest(archive) == spec['sha256'], 'Official Node archive SHA256 mismatch')
    destination.mkdir(parents=True)
    count = total = 0
    names = set()
    links = []

    def add(name, size, mode, data=None, link=None, directory=False):
        nonlocal count, total
        parts = relative_path(name).parts
        require(parts[0] == spec['name'].removesuffix('.tar.gz').removesuffix('.zip'),
                'Unexpected Node archive root')
        if len(parts) == 1:
            require(directory, 'Invalid Node root member')
            return
        relative = PurePosixPath(*parts[1:]).as_posix()
        require(relative not in names, 'Duplicate Node archive member')
        names.add(relative)
        count += 1
        total += size
        require(count <= MAX_ENTRIES and total <= MAX_UNPACKED, 'Node extraction limit exceeded')
        target = destination/relative
        target.parent.mkdir(parents=True, exist_ok=True)
        if directory:
            target.mkdir(exist_ok=True)
        elif link is not None:
            require(not PurePosixPath(link).is_absolute() and '\\' not in link and ':' not in link,
                    'Invalid Node symlink')
            links.append((target, link))
        else:
            require(size <= MAX_UNPACKED and data is not None, 'Invalid Node file')
            with target.open('xb') as output:
                shutil.copyfileobj(data, output, 1024 * 1024)
            require(target.stat().st_size == size, 'Node member truncated')
            target.chmod(0o755 if mode & 0o111 else 0o644)
    if spec['format'] == 'tar.gz':
        with tarfile.open(archive, 'r|gz') as source:
            for member in source:
                require(not member.islnk() and (member.isfile() or member.isdir() or member.issym()),
                        'Node hardlink/special member forbidden')
                add(member.name.rstrip('/'), member.size, member.mode,
                    source.extractfile(member) if member.isfile() else None,
                    member.linkname if member.issym() else None, member.isdir())
    else:
        with zipfile.ZipFile(archive) as source:
            for member in source.infolist():
                mode = member.external_attr >> 16
                require(not stat.S_ISLNK(mode), 'ZIP symlink forbidden')
                with source.open(member) as data:
                    add(member.filename.rstrip('/'), member.file_size, mode, data, directory=member.is_dir())
    # Materialize links only after regular extraction. No archive-created link
    # can become a parent directory of a later file write.
    for path, target in links:
        require(not path.exists(), 'Node symlink collides with an extracted path')
        path.symlink_to(target)
    for path, target in links:
        validate_link(destination, path, target)


def download_node(spec, target):
    require(spec['url'].startswith('https://nodejs.org/dist/'), 'Unexpected Node source')
    partial = target.with_suffix(target.suffix+'.partial')
    with urllib.request.urlopen(spec['url'], timeout=60) as response, partial.open('xb') as output:
        total = 0
        while True:
            block = response.read(1024 * 1024)
            if not block:
                break
            total += len(block)
            require(total <= MAX_ARCHIVE, 'Node download exceeds compressed limit')
            output.write(block)
    require(digest(partial) == spec['sha256'], 'Downloaded Node hash mismatch')
    partial.rename(target)


def scan_tree(root):
    entries, total = [], 0
    runtime = root/'runtime'
    for path in sorted(runtime.rglob('*')):
        name = path.relative_to(root).as_posix()
        relative_path(name)
        if path.is_symlink():
            target = os.readlink(path)
            validate_link(runtime, path, target)
            entries.append({'path': name, 'kind': 'symlink', 'target': target})
        elif path.is_file():
            info = path.stat()
            require(info.st_nlink == 1, 'Hardlinked bundle file forbidden: ' + name)
            mode = 0o755 if info.st_mode & 0o111 else 0o644
            path.chmod(mode)
            total += info.st_size
            entries.append({'path': name, 'kind': 'file', 'sha256': digest(path),
                            'bytes': info.st_size, 'mode': mode})
        else:
            require(path.is_dir(), 'Special bundle file forbidden')
        require(len(entries) <= MAX_ENTRIES and total <= MAX_UNPACKED, 'Bundle extraction budget exceeded')
    # Path's native component ordering differs from byte/string ordering for
    # siblings such as uv/ and uv.h. The descriptor and tar use full path order.
    entries.sort(key=lambda row: row['path'])
    return entries, total


def pack(root, entries, target):
    with target.open('xb') as raw, gzip.GzipFile(filename='', mode='wb', fileobj=raw, mtime=0,
                                                compresslevel=9) as compressed:
        # USTAR: no PAX extension entries for a consumer to interpret. Fail if a
        # future dependency path/target cannot be represented, rather than truncate.
        with tarfile.open(fileobj=compressed, mode='w|', format=tarfile.USTAR_FORMAT) as archive:
            for row in entries:
                member = tarfile.TarInfo(row['path'])
                member.uid = member.gid = member.mtime = 0
                member.uname = member.gname = ''
                if row['kind'] == 'symlink':
                    member.type, member.linkname, member.mode = tarfile.SYMTYPE, row['target'], 0o777
                    archive.addfile(member)
                else:
                    member.size, member.mode = row['bytes'], row['mode']
                    with (root/row['path']).open('rb') as source:
                        archive.addfile(member, source)
    require(target.stat().st_size <= MAX_ARCHIVE, 'Compressed bundle limit exceeded')


def command(args, cwd, env, log):
    with log.open('wb') as output:
        result = subprocess.run([str(arg) for arg in args], cwd=cwd, env=env,
                                stdout=output, stderr=subprocess.STDOUT, timeout=900)
    require(result.returncode == 0, 'Command failed; preserved log: ' + str(log))


def lock_packages(policy, prefix, target):
    """Every target-applicable lock record must exist, including optional natives."""
    selected = []
    for name, row in policy['packageLock']['packages'].items():
        if not name:
            continue
        os_ok = 'os' not in row or target['os'] in row['os'] or row['os'] == ['!aix']
        cpu_ok = 'cpu' not in row or target['cpu'] in row['cpu']
        if not (os_ok and cpu_ok):
            continue
        actual = prefix/name/'package.json'
        require(actual.is_file(), 'Required locked production dependency missing: ' + name)
        package = json.loads(actual.read_bytes())
        require(package['version'] == row['version'], 'Installed version drift: ' + name)
        require(row.get('integrity', '').startswith('sha512-'), 'Missing strong lock integrity: ' + name)
        selected.append({'path': name, 'version': row['version'], 'resolved': row['resolved'],
                         'integrity': row['integrity'], 'scripts': package.get('scripts', {})})
    return selected


def build(args):
    policy_path = Path(getattr(args, 'policy', POLICY)).resolve()
    policy = json.loads(policy_path.read_bytes())
    target = policy['platforms'][args.platform]
    output = args.output.resolve()
    require(not output.exists(), 'A fresh output directory is required')
    require(args.build_node.is_absolute() and args.build_node.is_file(), 'Absolute build-host Node required')
    require(args.npm_cli.is_absolute() and args.npm_cli.is_file(), 'Absolute build-host npm-cli.js required')
    require(digest(args.runtime_archive) == policy['runtime']['sha256'], 'Pinned runtime archive hash mismatch')
    require(args.cache.is_absolute(), 'Absolute dedicated npm cache required')
    output.mkdir(parents=True)
    work = output/'work'
    tree = output/'runtime-bundle'
    prefix = tree/'runtime/opencodex'
    (prefix/'sources').mkdir(parents=True)
    work.mkdir()
    for name in ('home', 'tmp'):
        (work/name).mkdir()
    for name in ('user.npmrc', 'global.npmrc'):
        (work/name).write_text('')
    env = {'HOME': str(work/'home'), 'USERPROFILE': str(work/'home'),
           'PATH': str(args.build_node.parent), 'TMPDIR': str(work/'tmp'), 'TEMP': str(work/'tmp'),
           'OCX_REAL_HOME': str(Path.home()),
           'npm_config_update_notifier': 'false', 'npm_config_progress': 'false'}
    if os.name == 'nt':
        env['SystemRoot'] = os.environ['SystemRoot']
    if getattr(args, 'proxy', None):
        env['npm_config_proxy'] = args.proxy
        env['npm_config_https_proxy'] = args.proxy
    node_archive = args.node_archive
    if node_archive is None:
        node_archive = output/target['nodeArchive']['name']
        download_node(target['nodeArchive'], node_archive)
    require(Path(node_archive).stat().st_size <= MAX_ARCHIVE, 'Node input limit exceeded')
    extract_node(node_archive, tree/'runtime/node', target['nodeArchive'])
    shutil.copyfile(args.runtime_archive, prefix/'sources'/policy['runtime']['archive'])
    (prefix/'package.json').write_bytes(json_bytes(policy['projectPackage']))
    lock_bytes = json_bytes(policy['packageLock'])
    (prefix/'package-lock.json').write_bytes(lock_bytes)
    command([args.build_node, args.npm_cli, '--version'], work, env, output/'npm-version.log')
    require((output/'npm-version.log').read_text().strip() == policy['node']['npmVersion'], 'npm resolver version mismatch')
    ci = [args.build_node, args.npm_cli, 'ci', '--ignore-scripts', '--omit=dev', '--include=optional',
          '--bin-links=false', '--no-audit', '--no-fund', '--prefix', prefix, '--cache', args.cache,
          '--userconfig', work/'user.npmrc', '--globalconfig', work/'global.npmrc',
          '--registry', 'https://registry.npmjs.org', '--os='+target['os'], '--cpu='+target['cpu']]
    if args.offline:
        ci.append('--offline')
    command(ci, prefix, env, output/'npm-ci.log')
    require((prefix/'package-lock.json').read_bytes() == lock_bytes, 'npm ci changed frozen lock')
    production = lock_packages(policy, prefix, target)
    # Never run bun/install.js. Its reviewed operation is replaced with an exact
    # copy from the locked selected @oven package, after native-header validation.
    native_bun = prefix/'node_modules'/target['bunPackage']/target['bunSource']
    node = tree/target['nodeExecutable']
    keyring = prefix/'node_modules'/target['keyringPackage']/target['keyringBinary']
    wreq = prefix/'node_modules/wreq-js/rust'/target['wreqBinary']
    natives = {}
    for label, path in [('node', node), ('bun', native_bun), ('keyring', keyring), ('wreq', wreq)]:
        natives[label] = {'path': path.relative_to(tree).as_posix(), 'kind': binary_kind(path, args.platform),
                          'sha256': digest(path), 'bytes': path.stat().st_size}
    require(native_bun.stat().st_size > 10 * 1024 * 1024, 'Bun native binary is incomplete')
    bun = tree/target['bunExecutable']
    bun.parent.mkdir()
    shutil.copyfile(native_bun, bun)
    bun.chmod(0o755)
    launcher_bun = prefix/'node_modules/bun/bin/bun.exe'
    shim_before = digest(launcher_bun)
    shutil.copyfile(native_bun, launcher_bun)
    launcher_bun.chmod(0o755)
    binary_kind(launcher_bun, args.platform)
    # bunx is another JS placeholder shipped by npm; it is not a runtime entry.
    # Its exact upstream bytes remain in the complete dependency tree.
    require((tree/target['npmCli']).is_file(), 'Official npm CLI missing')
    require(json.loads((prefix/PACKAGE/'package.json').read_bytes())['version'] == policy['runtime']['version'],
            'Installed runtime identity drift')
    checks = {'platform': args.platform, 'nativeFiles': natives, 'lifecycleScriptsExecuted': False,
              'npmCiIgnoreScripts': True, 'bunPlaceholderBeforeSha256': shim_before,
              'bunMaterializedSha256': digest(launcher_bun), 'productionDependencies': production,
              'execution': 'not-run', 'runtimeSourceSha256': policy['runtime']['sha256']}
    entries, unpacked = scan_tree(tree)
    archive_name = target['archiveName']
    archive = output/archive_name
    pack(tree, entries, archive)
    second = output/(archive_name+'.rebuild')
    pack(tree, entries, second)
    require(digest(archive) == digest(second), 'Canonical rebuild hash mismatch')
    second.unlink()
    descriptor = {'schemaVersion': 1, 'platform': args.platform,
                  'runtimeVersion': policy['runtime']['version'], 'nodeVersion': policy['node']['version'],
                  'bunVersion': policy['bunVersion'], 'runtimeArchiveSha256': policy['runtime']['sha256'],
                  'archiveName': archive_name, 'archiveSha256': digest(archive),
                  'archiveBytes': archive.stat().st_size, 'entries': entries}
    descriptor_path = output/(archive_name.removesuffix('.tar.gz')+'.manifest.json')
    descriptor_path.write_bytes(json_bytes(descriptor))
    require(str(output).encode() not in descriptor_path.read_bytes(), 'Build path leaked into descriptor')
    checks.update(unpackedBytes=unpacked, entries=len(entries), reproducibleRepacks=2,
                  archiveSha256=descriptor['archiveSha256'], descriptorSha256=digest(descriptor_path))
    (output/'checks.json').write_bytes(json_bytes(checks))
    command([args.build_node, ROOT/'test/smoke/runtime023-bundle-check.mjs', '--policy', policy_path, '--root', tree,
             '--descriptor', descriptor_path], work, env, output/'bundle-structure.log')
    checks['structure'] = 'passed-complete-tree-and-relative-symlinks'
    (output/'checks.json').write_bytes(json_bytes(checks))
    if args.execute:
        host = ('darwin' if sys.platform == 'darwin' else 'win32' if sys.platform == 'win32' else sys.platform)
        cpu = 'arm64' if host_platform.machine().lower() in ('arm64', 'aarch64') else 'x64'
        require(args.platform == host+'-'+cpu, 'Cannot execute a foreign platform bundle')
        command([node, ROOT/'test/smoke/runtime023-bundle-check.mjs', '--policy', policy_path, '--root', tree,
                 '--descriptor', descriptor_path, '--execute', '--evidence', output/'execution'],
                work, env, output/'bundle-execution.log')
        checks['execution'] = 'passed-isolated-readonly-cli'
        (output/'checks.json').write_bytes(json_bytes(checks))
    result = {key: descriptor[key] for key in ('platform', 'archiveName', 'archiveSha256', 'archiveBytes')}
    result.update(archive=str(archive), manifest=str(descriptor_path),
                  manifestSha256=digest(descriptor_path), checks=str(output/'checks.json'),
                  execution=checks['execution'])
    (output/'result.json').write_bytes(json_bytes(result))
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--policy', type=Path, default=POLICY, help='Explicit checked-in platform/production lock; defaults to the historical0.2.23 policy')
    parser.add_argument('--platform', choices=('darwin-arm64', 'win32-x64'), required=True)
    parser.add_argument('--runtime-archive', type=Path, required=True)
    parser.add_argument('--node-archive', type=Path)
    parser.add_argument('--build-node', type=Path, required=True)
    parser.add_argument('--npm-cli', type=Path, required=True)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--proxy', help='Explicit build-only npm proxy; never writes npm configuration')
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--execute', action='store_true')
    args = parser.parse_args()
    output_existed = args.output.exists()
    try:
        print(json.dumps(build(args), indent=2))
    except Exception as error:
        if not output_existed and args.output.is_dir():
            (args.output/'failure.json').write_bytes(json_bytes({'error': str(error), 'workPreserved': True}))
        raise
