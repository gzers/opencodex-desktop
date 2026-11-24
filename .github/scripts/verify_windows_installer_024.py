"""Read signed installer payloads without running/installing or extracting files.

MSI content IDs follow the pinned WiX template; this does NOT verify MSI table
destination/costing or native installation behavior. Those remain separate gates.
"""
import argparse
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import threading

spec = importlib.util.spec_from_file_location('delivery024', Path(__file__).with_name('verify_runtime_delivery_024.py'))
delivery = importlib.util.module_from_spec(spec); spec.loader.exec_module(delivery)
MAX_INSTALLER = 1024 * 1024 * 1024
MAX_MEMBER = 512 * 1024 * 1024
MAX_LISTING = 8 * 1024 * 1024
MAX_MEMBERS = 10000
CARGO_BINARIES = ('opencodex-desktop.exe', 'ocx.exe', 'ocxd.exe')
BUNDLE_TOKEN = b'__TAURI_BUNDLE_TYPE_VAR_UNK'
BUNDLE_MARKERS = {'msi': b'__TAURI_BUNDLE_TYPE_VAR_MSI', 'nsis': b'__TAURI_BUNDLE_TYPE_VAR_NSS'}
MSI_IDS = {
    # ocx.exe/ocxd.exe are Cargo [[bin]] targets, so the bundler installs them
    # from its own Bin_* components; only that component may own each file.
    'opencodex-desktop.exe': 'Path', 'ocx.exe': 'Bin_ocx',
    'ocxd.exe': 'Bin_ocxd', 'ocx-native-entry.json': 'OCX024NativeEntrySourceFile',
    'runtime-bundle/runtime-bundle-0.2.24-win32-x64.tar.gz': 'OCX023RuntimeArchive',
    'runtime-bundle/official-runtime.tgz': 'OCX023RuntimeSource',
    'runtime-bundle/runtime-bundle.manifest.json': 'OCX023RuntimeManifest',
    'runtime-bundle/runtime-bundle.manifest.sha256': 'OCX023RuntimePin',
}


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def file_metadata(info):
    return (info.st_dev, info.st_ino, info.st_mode, info.st_size, info.st_mtime_ns,
            info.st_ctime_ns, info.st_nlink, getattr(info, 'st_file_attributes', 0))


def require_regular(path, info, limit):
    attrs = getattr(info, 'st_file_attributes', 0)
    require(stat.S_ISREG(info.st_mode) and 0 < info.st_size <= limit
            and not attrs & stat.FILE_ATTRIBUTE_REPARSE_POINT,
            f'bounded non-reparse regular input required: {path.name} '
            f'(mode={info.st_mode:#o}, size={info.st_size}, links={info.st_nlink}, attributes={attrs:#x})')


def file_image(path, limit, *, cargo_output=False, bundle_kind=None, diagnostics=False):
    original = path.lstat()
    require_regular(path, original, limit)
    with path.open('rb') as source:
        handle = os.fstat(source.fileno())
        require_regular(path, handle, limit)
        if diagnostics:
            print(f'Build input {path.name}: path links={original.st_nlink}, '
                  f'handle links={handle.st_nlink}, bytes={handle.st_size}', file=sys.stderr, flush=True)
        # Only the verifier's three Cargo outputs can be hard-linked. Their
        # open handles and paths must still be bound to the same file.
        allow_links = cargo_output and path.name in CARGO_BINARIES
        require(handle.st_nlink == 1 or (allow_links and handle.st_nlink > 1),
                f'bounded single-link regular input required: {path.name} (links={handle.st_nlink})')
        require(os.path.samestat(original, handle) and original.st_size == handle.st_size,
                f'path and opened file differ: {path.name}')
        if bundle_kind is None:
            checksum = hashlib.sha256()
            remaining = handle.st_size
            while remaining:
                block = source.read(min(remaining, 1024 * 1024))
                require(block, f'truncated input during read: {path.name}')
                checksum.update(block)
                remaining -= len(block)
            require(not source.read(1), f'input grew during read: {path.name}')
            digest = checksum.hexdigest()
        else:
            # Tauri Bundler 2.9.4 patches the first marker before bundling,
            # then restores the Cargo output. Every other byte must match.
            image = source.read(handle.st_size + 1)
            require(len(image) == handle.st_size and BUNDLE_TOKEN in image,
                    f'expected Tauri bundle marker in Cargo output: {path.name}')
            digest = hashlib.sha256(image.replace(BUNDLE_TOKEN, BUNDLE_MARKERS[bundle_kind], 1)).hexdigest()
        current = path.lstat()
        require(file_metadata(os.fstat(source.fileno())) == file_metadata(handle)
                and file_metadata(current) == file_metadata(original), f'input changed during read: {path.name}')
    return (*file_metadata(handle), digest)


def bounded_tool(argv, limit, capture=False, timeout=120, sink=None):
    """Drain both pipes, bounded bytes/time, exact owned child; no shell/disk extraction."""
    process = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    output, stderr, failures = bytearray(), bytearray(), []
    digest, total = hashlib.sha256(), [0]

    def read(stream, error=False):
        try:
            for block in iter(lambda: stream.read(64 * 1024), b''):
                if error:
                    require(len(stderr) + len(block) <= 1024 * 1024, '7zip diagnostics exceed budget')
                    stderr.extend(block)
                else:
                    total[0] += len(block)
                    require(total[0] <= limit, '7zip streamed output exceeds declared budget')
                    digest.update(block)
                    if capture: output.extend(block)
                    if sink is not None: sink.write(block)
        except BaseException as exc:
            failures.append(exc)
            process.kill()

    readers = [threading.Thread(target=read, args=(process.stdout,)),
               threading.Thread(target=read, args=(process.stderr, True))]
    for reader in readers: reader.start()
    try:
        try: raw = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill(); process.wait(); raise
    finally:
        for reader in readers: reader.join()
        process.stdout.close(); process.stderr.close()
    if failures: raise failures[0]
    require(raw == 0, '7zip read failed: ' + stderr.decode('utf-8', errors='replace')[:512])
    return {'bytes': total[0], 'sha256': digest.hexdigest(), 'output': bytes(output)}


def records(raw):
    """Parse only the bounded technical listing, rejecting ambiguous keys/names."""
    text = raw.decode('utf-8', errors='strict').replace('\r\n', '\n')
    require('\x00' not in text and '\r' not in text, 'invalid 7zip listing framing')
    parts = text.split('\n----------\n')
    require(len(parts) == 2, 'one technical payload boundary required')
    header, body = parts
    result = []
    for block in body.strip('\n').split('\n\n'):
        if not block.strip(): continue
        row = {}
        for line in block.splitlines():
            require(' = ' in line, 'unrecognized 7zip member record')
            key, value = line.split(' = ', 1)
            require(key and key.strip() == key and key not in row, 'duplicate/invalid 7zip member field')
            row[key] = value
        require('Path' in row and len(result) < MAX_MEMBERS, 'missing path or excess installer members')
        result.append(row)
    require(result, 'empty installer payload')
    return header, result


def member_index(raw, kind):
    header, rows = records(raw)
    types = re.findall(r'^Type = (.+)$', header, re.M)
    if kind == 'msi':
        require(types == ['Compound', 'Cab'] and re.findall(r'^Path = app[.]cab$', header, re.M),
                'one pinned embedded MSI app.cab required')
    else:
        require(kind == 'nsis' and types == ['Nsis'], 'actual NSIS container required')
    result, folded, total = {}, set(), 0
    for row in rows:
        name = row['Path'].replace('\\', '/')
        require(name and not name.startswith(('/', '-')) and ':' not in name
                and all(part not in ('', '.', '..') for part in name.split('/'))
                and not any(ord(c) < 32 for c in name), 'unsafe installer member name')
        require(name.casefold() not in folded, 'duplicate/case-alias installer member')
        folded.add(name.casefold())
        require(not any(row.get(key) for key in ('Symbolic Link', 'Hard Link', 'Alternate Stream'))
                and row.get('Folder', '-') != '+' and row.get('Encrypted', '-') != '+',
                'linked/directory/encrypted installer member')
        size = row.get('Size', '')
        if size == '':
            require(kind == 'nsis' and name == 'uninstall.exe', 'unknown installer member size')
        else:
            require(size.isascii() and size.isdecimal() and len(size) <= 12, 'invalid installer member size')
            size = int(size); total += size
            require(size <= MAX_MEMBER and total <= 2 * 1024 * 1024 * 1024, 'installer content size exceeds budget')
        result[name] = {'bytes': size}
    return result


def verify(archive, signature, config, resources, built, native_entry, sevenzip, kind, policy=None, evidence=None):
    archive, signature, config, resources, built, native_entry, sevenzip = map(
        Path, (archive, signature, config, resources, built, native_entry, sevenzip))
    require(kind in ('msi', 'nsis') and sevenzip.is_absolute() and sevenzip.is_file()
            and not sevenzip.is_symlink(), 'explicit installed 7zip and installer kind required')
    require(archive.suffix.lower() == ('.msi' if kind == 'msi' else '.exe'), 'installer extension differs')
    inputs = [(archive, MAX_INSTALLER), (signature, 16384), (config, 1024 * 1024)]
    tool_image = file_image(sevenzip, MAX_MEMBER)
    before = [file_image(p, limit) for p, limit in inputs]
    policy = delivery.load_json(delivery.POLICY.read_bytes()) if policy is None else policy
    settings = delivery.load_json(config.read_bytes())
    require(settings.get('version') == policy['bundleVersion'] == '0.2.24'
            and settings.get('identifier') == 'com.gzers.opencodex.desktop', 'tagged installer config/policy differs')
    with tempfile.TemporaryDirectory(prefix='ocxd024-windows-updater-crypto-') as temporary:
        root = Path(temporary); key, detached = root / 'tagged.pub', root / 'installer.minisig'
        key.write_bytes(base64.b64decode(settings['plugins']['updater']['pubkey'], validate=True))
        detached.write_bytes(base64.b64decode(signature.read_bytes().strip(), validate=True))
        subprocess.run(['minisign', '-V', '-m', str(archive), '-p', str(key), '-x', str(detached)],
                       check=True, capture_output=True, timeout=60)
    runtime = delivery.verify(resources, 'win32-x64', policy)
    expected = {name: built / name for name in CARGO_BINARIES}
    expected['ocx-native-entry.json'] = native_entry
    for name in ('runtime-bundle-0.2.24-win32-x64.tar.gz', 'official-runtime.tgz',
                 'runtime-bundle.manifest.json', 'runtime-bundle.manifest.sha256'):
        expected['runtime-bundle/' + name] = resources / name
    listing = bounded_tool([str(sevenzip), 'l', '-slt', '-sccUTF-8', '--', str(archive)], MAX_LISTING, True)
    if evidence is not None:
        (evidence / f'{kind}.slt').write_bytes(listing['output'])
    members = member_index(listing['output'], kind)

    def build_images(diagnostics=False):
        return {name: file_image(path, MAX_MEMBER, cargo_output=name in CARGO_BINARIES,
                                 bundle_kind=kind if name == 'opencodex-desktop.exe' else None,
                                 diagnostics=diagnostics and name in CARGO_BINARIES)
                for name, path in expected.items()}

    images = build_images(diagnostics=True)
    rows = []
    for name, image in images.items():
        member = MSI_IDS[name] if kind == 'msi' else name
        require(member in members and members[member]['bytes'] == image[3], 'missing/wrong-size embedded payload: ' + name)
        observed = bounded_tool([str(sevenzip), 'x', '-so', '-spd', '--', str(archive), member], image[3])
        require(observed['bytes'] == image[3] and observed['sha256'] == image[-1],
                'embedded installer payload differs from selected build: ' + name)
        rows.append({'logicalPath': name, 'archiveMember': member, 'bytes': observed['bytes'], 'sha256': observed['sha256']})
    require(build_images() == images
            and [file_image(p, limit) for p, limit in inputs] == before
            and file_image(sevenzip, MAX_MEMBER) == tool_image, 'installer/build/tool inputs changed during verification')
    return {'schemaVersion': 1, 'status': 'verified-signed-windows-embedded-payload', 'kind': kind,
            'archiveBytes': before[0][3], 'archiveSha256': before[0][-1], 'members': rows,
            'listingSha256': listing['sha256'], 'updaterSignatureVerified': True,
            'readerExecutableSha256': tool_image[-1],
            'officialRuntime': runtime, 'noFilesExtracted': True, 'installerExecuted': False,
            'installationDestinationVerified': False, 'nativeInstallUninstallVerified': False,
            'sourceUnchanged': True, 'releaseAcceptance': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('archive', 'signature', 'config', 'resources', 'built', 'native-entry', 'sevenzip'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--kind', choices=['msi', 'nsis'], required=True)
    parser.add_argument('--evidence', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.archive, args.signature, args.config, args.resources, args.built,
                            args.native_entry, args.sevenzip, args.kind, evidence=args.evidence), indent=2))
