#!/usr/bin/env python3
"""Wrap an already sealed .app in a folder-shaped, verified macOS DMG.

No install, app launch, signing, HOME writes or service operations occur here.
Always write a NEW candidate; the release workflow owns any final asset rename.
"""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shlex
import stat
import subprocess
import sys
import tempfile
import time


FOLDER = 'OpenCodeX Desktop'
APP = FOLDER + '.app'
VERSION = '0.2.23'
INSTALL = '''OpenCodeX Desktop — macOS 安装 / Installation

1. 在 Finder 中打开“前往”→“前往文件夹”，输入 ~/Applications。
   如果该目录尚不存在，请先在您的个人文件夹中创建 Applications 文件夹。
2. 将本磁盘映像中的整个“OpenCodeX Desktop”文件夹复制到 ~/Applications。
   安装位置应为 ~/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app。
   请勿只复制 .app，也不要直接从磁盘映像中运行。
3. 推出磁盘映像，再从复制后的文件夹打开 OpenCodeX Desktop.app。

请选择当前用户可写的安装目录，无需管理员权限。请保留整个安装文件夹；
首次打开时，应用会在该文件夹内准备随应用提供的运行程序。
程序与用户配置分开保存；复制程序文件不代表用户配置已迁移或安装已完成。

In Finder, choose Go → Go to Folder and enter ~/Applications. Create this
Applications folder inside your own home folder if it does not exist.
Copy the ENTIRE “OpenCodeX Desktop” folder from this disk image into it.
The app should be at ~/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app.
Do not copy only the .app, and do not run it from the disk image.
Eject the image, then open the copied app. Choose a location writable by your
user; administrator privileges are not required. Keep the whole folder together.
On first launch the app prepares its included runtime inside that folder.
Program files and user configuration are stored separately. Copying program
files does not mean that configuration was migrated or installation completed.
'''


def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def xattrs(path):
    if hasattr(os, 'listxattr'):
        return {name: hashlib.sha256(os.getxattr(path, name, follow_symlinks=False)).hexdigest()
                for name in sorted(os.listxattr(path, follow_symlinks=False))}
    # CPython exposes os.listxattr only on some Unix platforms. Darwin's native
    # no-follow API keeps this script stdlib-only on the release runner.
    lib = ctypes.CDLL(None, use_errno=True)
    lib.listxattr.argtypes = [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
    lib.listxattr.restype = ctypes.c_ssize_t
    lib.getxattr.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_void_p,
                            ctypes.c_size_t, ctypes.c_uint32, ctypes.c_int]
    lib.getxattr.restype = ctypes.c_ssize_t
    raw = os.fsencode(path)
    size = lib.listxattr(raw, None, 0, 1)  # XATTR_NOFOLLOW
    if size < 0 or size > 1024 * 1024:
        raise OSError(ctypes.get_errno(), 'list app extended attributes')
    names = ctypes.create_string_buffer(size)
    if lib.listxattr(raw, names, size, 1) != size:
        raise OSError(ctypes.get_errno(), 'app extended attributes changed')
    result = {}
    for name in sorted(n for n in names.raw.split(b'\0') if n):
        length = lib.getxattr(raw, name, None, 0, 0, 1)
        if length < 0 or length > 16 * 1024 * 1024:
            raise OSError(ctypes.get_errno(), 'size app extended attribute')
        data = ctypes.create_string_buffer(length)
        if lib.getxattr(raw, name, data, length, 0, 1) != length:
            raise OSError(ctypes.get_errno(), 'read app extended attribute')
        result[os.fsdecode(name)] = hashlib.sha256(data.raw).hexdigest()
    return result


def tree_manifest(root):
    """Record signed bytes, modes, links and extended attributes; never follow links.

    Symlinks must remain within the sealed app. Directory mtimes are intentionally
    excluded: copying/mounting changes them without changing the sealed product.
    """
    root = root.resolve(strict=True)
    rows = []

    def visit(path):
        info = path.lstat()
        relative = path.relative_to(root).as_posix()
        row = {'path': relative, 'mode': stat.S_IMODE(info.st_mode)}
        if stat.S_ISLNK(info.st_mode):
            target = os.readlink(path)
            if Path(target).is_absolute() or not path.resolve().is_relative_to(root):
                raise ValueError(f'app symlink escapes sealed bundle: {relative}')
            row.update(kind='symlink', target=target)
        elif stat.S_ISDIR(info.st_mode):
            row['kind'] = 'directory'
        elif stat.S_ISREG(info.st_mode):
            row.update(kind='file', bytes=info.st_size, sha256=sha256(path))
        else:
            raise ValueError(f'unsupported app entry: {relative}')
        # Resource forks can affect signatures. Quarantine is preserved too;
        # this script never strips it to make signature verification pass.
        row['xattrs'] = xattrs(path)
        rows.append(row)
        if row['kind'] == 'directory':
            for child in sorted(path.iterdir(), key=lambda p: p.name):
                visit(child)

    visit(root)
    return rows


def tree_digest(rows):
    return hashlib.sha256(json.dumps(rows, ensure_ascii=False, sort_keys=True,
                                     separators=(',', ':')).encode()).hexdigest()


def version_info(app, expected):
    with (app / 'Contents/Info.plist').open('rb') as stream:
        info = plistlib.load(stream)
    if info.get('CFBundleShortVersionString') != expected:
        raise ValueError(f'CFBundleShortVersionString must equal {expected}')
    if not info.get('CFBundleIdentifier') or not info.get('CFBundleExecutable'):
        raise ValueError('app bundle identity/executable missing')
    executable = info['CFBundleExecutable']
    if Path(executable).name != executable or not (app / 'Contents/MacOS' / executable).is_file():
        raise ValueError('app executable is not a bundle-local file')
    return {key: info.get(key) for key in (
        'CFBundleShortVersionString', 'CFBundleVersion', 'CFBundleIdentifier',
        'CFBundleExecutable')}


class Commands:
    def __init__(self, report):
        self.report = report

    def run(self, args, timeout=300):
        args = [str(arg) for arg in args]
        record = {'argv': args, 'command': shlex.join(args), 'timeoutSeconds': timeout}
        self.report['commands'].append(record)
        start = time.monotonic()
        try:
            result = subprocess.run(args, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, timeout=timeout, check=False)
            record.update(returncode=result.returncode,
                          stdout=result.stdout.decode('utf-8', errors='replace'),
                          stderr=result.stderr.decode('utf-8', errors='replace'))
        except (OSError, subprocess.TimeoutExpired) as exc:
            record['error'] = str(exc)
            raise
        finally:
            record['elapsedSeconds'] = round(time.monotonic() - start, 3)
        if result.returncode:
            raise RuntimeError(f'command failed ({result.returncode}): {shlex.join(args)}')
        return result.stdout


def destination(value):
    # Reject both lexical and resolved .app ancestors. In particular, an output
    # symlink must never turn a fresh artifact request into an app modification.
    raw = Path(value).expanduser().absolute()
    resolved = raw.resolve(strict=False)
    if any(p.suffix.lower() == '.app' for p in (*raw.parents, raw, *resolved.parents, resolved)):
        raise ValueError('artifact/evidence destination must not be inside any .app')
    if os.path.lexists(raw) or os.path.lexists(resolved):
        raise FileExistsError(f'new output required; refusing existing destination: {raw}')
    if not raw.parent.is_dir():
        raise ValueError(f'destination parent must already exist: {raw.parent}')
    return resolved


def verify_payload(mount, expected, original, command, report):
    if not os.statvfs(mount).f_flag & os.ST_RDONLY:
        raise ValueError('verification volume was not mounted read-only')
    # HFS+ volume metadata is owned by the filesystem, never an extra app,
    # installer, Applications alias, runtime directory or user config payload.
    metadata = {'.fseventsd', '.Spotlight-V100', '.Trashes', '.HFS+ Private Directory Data\r',
                '.HFS+ Private Directory Data', '.metadata_never_index', '.TemporaryItems'}
    names = {entry.name for entry in mount.iterdir()}
    if names - metadata != {FOLDER, 'Install.txt'}:
        raise ValueError(f'unexpected mounted payload entries: {sorted(names)}')
    folder = mount / FOLDER
    if folder.is_symlink() or {p.name for p in folder.iterdir()} != {APP}:
        raise ValueError('installation folder must contain only the sealed .app')
    if (mount / 'Install.txt').read_text(encoding='utf-8') != INSTALL:
        raise ValueError('installation instructions differ from the intended layout')
    app = folder / APP
    if app.is_symlink() or not app.is_dir():
        raise ValueError('mounted .app must be a real bundle directory')
    report['mountedInfo'] = version_info(app, expected)
    command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', app])
    mounted = tree_manifest(app)
    if mounted != original:
        raise ValueError('mounted .app bytes/modes/links/xattrs differ from sealed source')
    report['mountedAppTreeSha256'] = tree_digest(mounted)
    report['mountedTopLevel'] = sorted(names)
    report['readOnlyVerified'] = True
    report['appNeverLaunched'] = True
    report['runtimeAbsentBesideApp'] = not (folder / 'runtime').exists()


def package(app, output, metadata, expected_version=VERSION, input_kind='candidate'):
    app = Path(app).expanduser().absolute()
    if app.is_symlink() or app.name != APP or not app.is_dir():
        raise ValueError(f'input must be the explicit sealed {APP} directory')
    app = app.resolve(strict=True)
    output, metadata = destination(output), destination(metadata)
    if output == metadata or output.suffix.lower() != '.dmg':
        raise ValueError('distinct new .dmg output and JSON metadata paths required')
    if expected_version != VERSION:
        raise ValueError(f'this bounded release packager expects {VERSION}')
    if input_kind not in {'candidate', 'release-input'}:
        raise ValueError('input kind must be candidate or release-input')
    if output.is_relative_to(app.parent) or metadata.is_relative_to(app.parent):
        raise ValueError('write artifact/evidence outside the source app directory')
    report = {'schemaVersion': 1, 'status': 'started', 'inputKind': input_kind,
              'sourceApp': str(app), 'output': str(output), 'metadata': str(metadata),
              'expectedVersion': expected_version, 'installationRoot': '~/Applications/' + FOLDER,
              'installedApp': '~/Applications/' + FOLDER + '/' + APP,
              'commands': [], 'cleanup': {}, 'installationPerformed': False,
              'releaseAcceptance': False}
    # Reserve metadata exclusively before external work. Failed runs also keep
    # their exact commands/cleanup history, without clobbering someone else's log.
    with metadata.open('x', encoding='utf-8') as evidence:
        command = Commands(report)
        published = False
        scratch = None
        mounted = False
        original = None
        try:
            if sys.platform != 'darwin':
                raise RuntimeError('real packaging requires macOS hdiutil/codesign')
            report['sourceInfo'] = version_info(app, expected_version)
            original = tree_manifest(app)
            report['sourceAppTreeSha256'] = tree_digest(original)
            command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', app])
            command.run(['/usr/bin/codesign', '-d', '--verbose=4', app])
            scratch = tempfile.TemporaryDirectory(prefix='.ocxd023-macos-package-', dir=output.parent)
            work = Path(scratch.name)
            payload, mount = work / 'payload', work / 'mounted'
            (payload / FOLDER).mkdir(parents=True)
            mount.mkdir()
            staged = payload / FOLDER / APP
            command.run(['/usr/bin/ditto', '--rsrc', '--extattr', '--acl', app, staged])
            if tree_manifest(staged) != original:
                raise ValueError('staged .app differs from immutable signed input')
            command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', staged])
            (payload / 'Install.txt').write_text(INSTALL, encoding='utf-8')
            candidate = work / 'verified-candidate.dmg'
            command.run(['/usr/bin/hdiutil', 'create', '-srcfolder', payload,
                         '-volname', FOLDER + ' ' + expected_version, '-fs', 'HFS+',
                         '-format', 'UDZO', candidate], timeout=600)
            command.run(['/usr/bin/hdiutil', 'verify', candidate], timeout=300)
            # Set before attach: on timeout or malformed plist an already-mounted
            # volume is still detached via this task's unique explicit mountpoint.
            mounted = True
            raw = command.run(['/usr/bin/hdiutil', 'attach', candidate, '-readonly',
                               '-nobrowse', '-noautoopen', '-noautofsck', '-mountpoint',
                               mount, '-plist'], timeout=120)
            entities = plistlib.loads(raw).get('system-entities', [])
            if not any(Path(e.get('mount-point', '/nonexistent')).resolve() == mount.resolve()
                       for e in entities):
                raise ValueError('hdiutil did not confirm this task mountpoint')
            verify_payload(mount, expected_version, original, command, report)
            command.run(['/usr/bin/hdiutil', 'detach', mount], timeout=120)
            mounted = False
            report['cleanup']['ownMountDetached'] = True
            if tree_manifest(app) != original:
                raise ValueError('source app changed during packaging')
            command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', app])
            report['sourceUnchanged'] = True
            report['artifactSha256'] = sha256(candidate)
            report['artifactBytes'] = candidate.stat().st_size
            # Same-filesystem hard-link publishes the verified candidate without
            # replacing any destination, even if another writer raced this task.
            os.link(candidate, output)
            published = True
            report['status'] = 'verified-candidate'
        except BaseException as exc:
            report.update(status='failed', error=f'{type(exc).__name__}: {exc}')
            raise
        finally:
            cleanup_error = None
            if mounted and scratch is not None:
                mount = Path(scratch.name) / 'mounted'
                # Never detach by a discovered global device or another volume.
                if os.path.ismount(mount):
                    for force in (False, True):
                        try:
                            command.run(['/usr/bin/hdiutil', 'detach', *(['-force'] if force else []), mount], timeout=120)
                            report['cleanup']['ownMountDetached'] = True
                            break
                        except Exception as exc:
                            cleanup_error = str(exc)
                    if not os.path.ismount(mount):
                        cleanup_error = None
                else:
                    report['cleanup']['ownMountDetached'] = True
            if scratch is not None:
                if not mounted or not os.path.ismount(Path(scratch.name) / 'mounted'):
                    try:
                        scratch.cleanup()
                        report['cleanup']['scratchRemoved'] = True
                    except OSError as exc:
                        cleanup_error = str(exc)
                else:
                    # Avoid deleting through a live mount or TempDir finalization.
                    scratch._finalizer.detach()
                    report['cleanup']['retainedScratch'] = scratch.name
            if original is not None:
                try:
                    report['sourceUnchanged'] = tree_manifest(app) == original
                except Exception as exc:
                    report['sourceRecheckError'] = str(exc)
            if cleanup_error:
                report.update(status='cleanup-failed', cleanupError=cleanup_error)
            report['cleanup']['outputPublished'] = published
            evidence.seek(0)
            evidence.truncate()
            json.dump(report, evidence, ensure_ascii=False, indent=2)
            evidence.write('\n')
            evidence.flush()
            os.fsync(evidence.fileno())
            if cleanup_error and sys.exc_info()[0] is None:
                raise RuntimeError('own mount/scratch cleanup failed; inspect metadata')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True,
                        help='NEW DMG candidate; may use the release asset basename in a staging directory')
    parser.add_argument('--metadata', type=Path, required=True, help='NEW JSON evidence path')
    parser.add_argument('--expected-version', default=VERSION, choices=[VERSION])
    parser.add_argument('--input-kind', choices=['candidate', 'release-input'], default='candidate')
    args = parser.parse_args()
    report = package(args.app, args.output, args.metadata, args.expected_version, args.input_kind)
    print(json.dumps({key: report[key] for key in ('status', 'inputKind', 'output', 'metadata',
                                                  'artifactSha256', 'artifactBytes')}, ensure_ascii=False))


if __name__ == '__main__':
    main()
