"""Bind a signed updater archive to the complete sealed app and official runtime.

Never extract an app, install, execute archive content, publish or rewrite the
sealed input. This assembly check does not grant whole-release acceptance.
"""
import argparse
import base64
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import posixpath
import stat
import subprocess
import tarfile
import tempfile


def module(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + '.py'))
    value = importlib.util.module_from_spec(spec); spec.loader.exec_module(value)
    return value


delivery = module('verify_runtime_delivery_024')
common = module('macos_package_common')
APP = 'OpenCodeX Desktop.app'
MAX_ENTRIES = 100000
MAX_OUTER_BYTES = 2 * 1024 * 1024 * 1024
MAX_COMPRESSED = 1024 * 1024 * 1024
MAX_EXTENDED_HEADER = 16384
UNIX_TYPES = {
    tarfile.REGTYPE: stat.S_IFREG,
    tarfile.AREGTYPE: stat.S_IFREG,
    tarfile.DIRTYPE: stat.S_IFDIR,
    tarfile.SYMTYPE: stat.S_IFLNK,
}


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def snapshot(path, limit):
    value = path.lstat()
    require(stat.S_ISREG(value.st_mode) and value.st_nlink == 1, 'single-link regular input required')
    require(0 < value.st_size <= limit, 'updater input size exceeds budget')
    return (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns,
            value.st_mode, common.sha256(path))


class BoundedTarInfo(tarfile.TarInfo):
    @classmethod
    def frombuf(cls, buf, encoding, errors):
        value = super().frombuf(buf, encoding, errors)
        # tarfile processes extended headers before yielding a member. Bound
        # them HERE, before an untrusted size can allocate/read a large buffer.
        if value.type in (tarfile.GNUTYPE_LONGNAME, tarfile.GNUTYPE_LONGLINK, tarfile.XHDTYPE):
            require(0 <= value.size <= MAX_EXTENDED_HEADER, 'extended updater header exceeds budget')
        else:
            require(value.type in (tarfile.REGTYPE, tarfile.AREGTYPE, tarfile.DIRTYPE, tarfile.SYMTYPE),
                    'special/hardlink/global updater header')
            require(0 <= value.size <= MAX_OUTER_BYTES, 'updater member size exceeds budget')
        return value

    def _proc_member(self, archive):
        archive.ocxd_header_count = getattr(archive, 'ocxd_header_count', 0) + 1
        require(archive.ocxd_header_count <= MAX_ENTRIES * 3, 'raw updater header count exceeds budget')
        return super()._proc_member(archive)


class BoundedStream:
    def __init__(self, source):
        self.source, self.read_bytes = source, 0

    def read(self, size):
        value = self.source.read(size)
        self.read_bytes += len(value)
        require(self.read_bytes <= MAX_OUTER_BYTES + MAX_ENTRIES * 2048 + 10240,
                'decompressed updater stream exceeds budget')
        return value


def archive_rows(path):
    """Complete streamed outer membership; no extract()/extractall()."""
    rows, total, count = {}, 0, 0
    with gzip.open(path, 'rb') as compressed, tarfile.open(
            fileobj=BoundedStream(compressed), mode='r|', tarinfo=BoundedTarInfo) as archive:
        for member in archive:
            count += 1
            require(count <= MAX_ENTRIES, 'updater member count exceeds budget')
            name = member.name.rstrip('/') if member.isdir() else member.name
            delivery.safe_path(name)
            require(name == APP or name.startswith(APP + '/'), 'member outside single app')
            require(name not in rows, 'duplicate updater member')
            # Only standard path/time metadata. Names have already been
            # resolved by tarfile; sparse files, xattr overlays and unknown
            # metadata cannot quietly change the extracted app's semantics.
            require(set(member.pax_headers) <= {'path', 'linkpath', 'mtime', 'atime', 'ctime'},
                    'unsupported updater member metadata')
            # Rust tar's complete headers can include the Unix file type in st_mode.
            require(member.type in UNIX_TYPES, 'unsupported updater member type')
            mode = member.mode & 0o777
            require(member.mode in (mode, UNIX_TYPES[member.type] | mode),
                    'special updater permission bits or mismatched file type')
            row = {'mode': mode}
            if member.isdir():
                require(member.size == 0, 'nonempty directory member')
                row['kind'] = 'directory'
            elif member.issym():
                link = member.linkname
                require(isinstance(link, str) and link and not link.startswith('/')
                        and '\\' not in link and ':' not in link
                        and not any(ord(c) < 32 for c in link), 'unsafe app symlink')
                resolved = posixpath.normpath(posixpath.join(posixpath.dirname(name), link))
                require(resolved == APP or resolved.startswith(APP + '/'), 'app symlink escapes')
                require(member.size == 0, 'nonempty symlink member')
                row.update(kind='symlink', target=link)
            else:
                require(member.type in (tarfile.REGTYPE, tarfile.AREGTYPE), 'special/hardlink updater member')
                total += member.size
                require(0 <= member.size <= MAX_OUTER_BYTES and total <= MAX_OUTER_BYTES,
                        'updater unpacked budget exceeded')
                observed, _ = delivery.file_row(archive, member)
                row.update(observed)
            rows[name] = row
        delivery.finish_archive(archive)
    require(rows.get(APP, {}).get('kind') == 'directory', 'single app directory root missing')
    # No file or symlink may also act as an archive member's parent.
    for name in rows:
        for parent in PurePosixPath(name).parents:
            if str(parent) == '.':
                break
            require(rows.get(str(parent), {}).get('kind') == 'directory', 'missing/non-directory updater parent')
    return rows, total


def app_rows(app):
    complete = common.tree_manifest(app)
    rows = {}
    for item in complete:
        relative = item['path']
        name = APP if relative == '.' else APP + '/' + relative
        rows[name] = {key: value for key, value in item.items() if key not in ('path', 'xattrs')}
    return rows, complete


def verify(app, archive, signature, config, policy=None):
    app, archive, signature, config = map(Path, (app, archive, signature, config))
    require(app.name == APP and not app.is_symlink() and app.is_dir(), 'explicit sealed single app required')
    app = app.resolve(strict=True)
    for path in (archive, signature, config):
        require(not path.is_symlink() and not path.resolve().is_relative_to(app), 'input inside sealed app or symlink')
    limits = (MAX_COMPRESSED, 16384, 1024 * 1024)
    before = [snapshot(path, limit) for path, limit in zip((archive, signature, config), limits)]
    policy = delivery.load_json(delivery.POLICY.read_bytes()) if policy is None else policy
    settings = delivery.load_json(config.read_bytes())
    require(settings.get('version') == policy['bundleVersion'], 'tagged config version differs from policy')
    info = common.version_info(app, policy['bundleVersion'])
    require(info['CFBundleIdentifier'] == settings.get('identifier') == 'com.gzers.opencodex.desktop'
            and info['CFBundleExecutable'] == 'opencodex-desktop', 'sealed app identity differs')
    require({p.name for p in (app / 'Contents/MacOS').iterdir()} == {'opencodex-desktop', 'ocxd', 'ocx'},
            'sealed executable membership differs')
    public = base64.b64decode(settings['plugins']['updater']['pubkey'], validate=True)
    encoded_signature = signature.read_bytes().strip()
    require(encoded_signature, 'missing updater signature')
    detached = base64.b64decode(encoded_signature, validate=True)
    with tempfile.TemporaryDirectory(prefix='ocxd024-updater-crypto-') as temporary:
        root = Path(temporary); key = root / 'tagged.pub'; sig = root / 'artifact.minisig'
        key.write_bytes(public); sig.write_bytes(detached)
        subprocess.run(['minisign', '-V', '-m', str(archive), '-p', str(key), '-x', str(sig)],
                       check=True, capture_output=True, timeout=60)
    subprocess.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', str(app)],
                   check=True, capture_output=True, timeout=60)
    expected, original = app_rows(app)
    observed, unpacked = archive_rows(archive)
    require(observed == expected, 'updater complete app membership/bytes/modes/links differ from sealed app')
    runtime = delivery.verify(app / 'Contents/Resources/runtime-bundle', 'darwin-arm64', policy)
    require(common.tree_manifest(app) == original, 'sealed source app changed during verification')
    require([snapshot(path, limit) for path, limit in zip((archive, signature, config), limits)] == before,
            'updater archive/signature/tagged config changed during verification')
    return {'schemaVersion': 1, 'status': 'verified-signed-macos-updater',
            'appVersion': info['CFBundleShortVersionString'], 'archiveSha256': before[0][-1],
            'archiveBytes': before[0][2], 'archiveMembers': len(observed), 'unpackedBytes': unpacked,
            'sealedAppTreeSha256': common.tree_digest(original), 'officialRuntime': runtime,
            'updaterSignatureVerified': True, 'sealedAppSignatureVerified': True,
            'allMembersMatchSealedApp': True, 'sourceUnchanged': True, 'temporaryKeyFilesRemoved': True,
            'archiveExtracted': False, 'applicationLaunched': False, 'releaseAcceptance': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('app', 'archive', 'signature', 'config'):
        parser.add_argument('--' + name, type=Path, required=True)
    arguments = parser.parse_args()
    print(json.dumps(verify(arguments.app, arguments.archive, arguments.signature, arguments.config), indent=2))
