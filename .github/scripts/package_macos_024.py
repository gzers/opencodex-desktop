#!/usr/bin/env python3
"""Package a sealed single .app with unchanged pinned official runtime resources.

Creates a NEW DMG, verifies its read-only mounted payload and leaves source
bytes/signatures untouched. Never installs, launches, changes HOME or publishes.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import sys
import tempfile

_spec = importlib.util.spec_from_file_location('macos_package_common', Path(__file__).with_name('macos_package_common.py'))
common = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(common)
ROOT = Path(__file__).resolve().parents[2]
APP = 'OpenCodeX Desktop.app'
VERSION = '0.2.24'
INSTALL = '''OpenCodeX Desktop — macOS 安装 / Installation

将 OpenCodeX Desktop.app 拖入 Applications 后推出磁盘映像，再打开应用。
也可将应用复制到当前用户的 ~/Applications；无需再套一层安装文件夹。
首次打开会从应用内置离线资源准备当前用户专属的运行程序，无需另装系统 Node.js。
应用、运行程序与用户配置分别保存；复制应用不会自动迁移旧配置。

Drag OpenCodeX Desktop.app to Applications, eject the disk image, then open it.
You may instead copy the app to ~/Applications. No enclosing installation folder
is required. The app prepares its included official runtime in the current user's
program directory; system Node.js is not required. Configuration stays separate
and is not automatically migrated when copying the app.
'''
METADATA = {'.fseventsd', '.Spotlight-V100', '.Trashes', '.HFS+ Private Directory Data\r',
            '.HFS+ Private Directory Data', '.metadata_never_index', '.TemporaryItems'}


def policy():
    value = json.loads((ROOT / '.github/runtime-platforms-024.json').read_bytes())
    runtime = value['runtime']
    if (value['bundleVersion'] != VERSION or runtime['distribution'] != 'official-npm'
            or runtime['version'] != '2.99.0'
            or runtime['url'] != 'https://registry.npmjs.org/@bitkyc08/opencodex/-/' + runtime['archive']):
        raise ValueError('expected unchanged official2.99 tagged distribution policy')
    return value


def resources(app):
    """Check only bundle-local read-only resources. No extraction or code execution."""
    root = app / 'Contents/Resources/runtime-bundle'
    if root.is_symlink() or not root.is_dir():
        raise ValueError('single .app must carry its own runtime-bundle directory')
    descriptor = root / 'runtime-bundle.manifest.json'
    if descriptor.is_symlink() or descriptor.stat().st_size > 64 * 1024 * 1024:
        raise ValueError('invalid bundled runtime descriptor')
    raw = descriptor.read_bytes()
    value = json.loads(raw)
    source = policy()['runtime']
    if (value['schemaVersion'] != 1 or value['platform'] != 'darwin-arm64'
            or value['runtimeVersion'] != source['version']
            or value['runtimeArchiveSha256'] != source['sha256']):
        raise ValueError('app runtime descriptor differs from the official policy')
    name = value['archiveName']
    if (not isinstance(name, str) or Path(name).name != name or '\\' in name
            or ':' in name or name in {'', '.', '..'}):
        raise ValueError('invalid runtime archive resource name')
    archive = root / name
    official = root / 'official-runtime.tgz'
    for entry in (archive, official, root / 'runtime-bundle.manifest.sha256'):
        if entry.is_symlink() or not entry.is_file() or entry.stat().st_nlink != 1:
            raise ValueError('runtime resource must be a single-link bundle-local file')
    pin = hashlib.sha256(raw).hexdigest()
    if (root / 'runtime-bundle.manifest.sha256').read_text().strip() != pin:
        raise ValueError('resource descriptor pin differs from its actual bytes')
    if common.sha256(official) != source['sha256']:
        raise ValueError('original official runtime archive differs from tagged bytes')
    if archive.stat().st_size != value['archiveBytes'] or common.sha256(archive) != value['archiveSha256']:
        raise ValueError('complete platform bundle differs from its descriptor')
    return {'descriptorSha256': pin, 'officialRuntimeVersion': source['version'],
            'officialRuntimeSha256': source['sha256'], 'platformArchiveSha256': value['archiveSha256'],
            'platformArchiveBytes': value['archiveBytes']}


def verify_payload(mount, expected, original, command, report):
    if not os.statvfs(mount).f_flag & os.ST_RDONLY:
        raise ValueError('verification volume is not read-only')
    names = {p.name for p in mount.iterdir()}
    if names - METADATA != {APP, 'Applications', 'Install.txt'}:
        raise ValueError('DMG must contain exactly a single .app, Applications link and instructions')
    link = mount / 'Applications'
    if not link.is_symlink() or os.readlink(link) != '/Applications':
        raise ValueError('Applications must be the exact system destination link')
    app = mount / APP
    if app.is_symlink() or not app.is_dir() or (mount / 'Install.txt').read_text() != INSTALL:
        raise ValueError('mounted app or instructions do not match single-app layout')
    report['mountedInfo'] = common.version_info(app, expected)
    command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', app])
    mounted = common.tree_manifest(app)
    if mounted != original:
        raise ValueError('mounted signed app differs from immutable source')
    if resources(app) != report['resources']:
        raise ValueError('mounted official runtime resources differ from sealed source')
    report.update(mountedAppTreeSha256=common.tree_digest(mounted), mountedTopLevel=sorted(names),
                  readOnlyVerified=True, appNeverLaunched=True, runtimeAbsentBesideApp=not (mount / 'runtime').exists())


def package(app, output, metadata, expected_version=VERSION, input_kind='candidate'):
    app = Path(app).expanduser().absolute()
    if app.is_symlink() or app.name != APP or not app.is_dir():
        raise ValueError('input must be the explicit sealed single .app directory')
    app = app.resolve(strict=True)
    output, metadata = common.destination(output), common.destination(metadata)
    if (output == metadata or output.suffix.lower() != '.dmg' or expected_version != VERSION
            or input_kind not in {'candidate', 'release-input'}):
        raise ValueError('distinct fresh .dmg/metadata, version0.2.24 and explicit input kind required')
    if output.is_relative_to(app.parent) or metadata.is_relative_to(app.parent):
        raise ValueError('artifacts must be outside the source app directory')
    report = {'schemaVersion': 1, 'status': 'started', 'inputKind': input_kind,
              'sourceApp': str(app), 'output': str(output), 'expectedVersion': expected_version,
              'layout': 'single-app', 'defaultInstalledApp': '/Applications/' + APP,
              'userInstalledApp': '~/Applications/' + APP,
              'runtimeRoot': '~/Library/Application Support/OpenCodeX Desktop/runtime',
              'commands': [], 'cleanup': {}, 'installationPerformed': False, 'releaseAcceptance': False}
    with metadata.open('x', encoding='utf-8') as evidence:
        command = common.Commands(report)
        scratch, mounted, published, original = None, False, False, None
        try:
            if sys.platform != 'darwin':
                raise RuntimeError('real packaging requires macOS hdiutil/codesign')
            report['sourceInfo'] = common.version_info(app, expected_version)
            original = common.tree_manifest(app)
            report['sourceAppTreeSha256'] = common.tree_digest(original)
            command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', app])
            report['resources'] = resources(app)
            scratch = tempfile.TemporaryDirectory(prefix='.ocxd024-macos-package-', dir=output.parent)
            work = Path(scratch.name)
            payload, mount = work / 'payload', work / 'mounted'
            payload.mkdir(); mount.mkdir()
            staged = payload / APP
            command.run(['/usr/bin/ditto', '--rsrc', '--extattr', '--acl', app, staged])
            if common.tree_manifest(staged) != original:
                raise ValueError('staged signed app differs from source')
            command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', staged])
            (payload / 'Applications').symlink_to('/Applications', target_is_directory=True)
            (payload / 'Install.txt').write_text(INSTALL, encoding='utf-8')
            candidate = work / 'verified-candidate.dmg'
            command.run(['/usr/bin/hdiutil', 'create', '-srcfolder', payload, '-volname', 'OpenCodeX Desktop ' + expected_version,
                         '-fs', 'HFS+', '-format', 'UDZO', candidate], timeout=600)
            command.run(['/usr/bin/hdiutil', 'verify', candidate])
            mounted = True
            raw = command.run(['/usr/bin/hdiutil', 'attach', candidate, '-readonly', '-nobrowse', '-noautoopen',
                               '-noautofsck', '-mountpoint', mount, '-plist'], timeout=120)
            if not any(Path(e.get('mount-point', '/nonexistent')).resolve() == mount.resolve()
                       for e in plistlib.loads(raw).get('system-entities', [])):
                raise ValueError('mountpoint not confirmed by hdiutil')
            verify_payload(mount, expected_version, original, command, report)
            command.run(['/usr/bin/hdiutil', 'detach', mount], timeout=120)
            mounted = False
            report['cleanup']['ownMountDetached'] = True
            if common.tree_manifest(app) != original:
                raise ValueError('source app changed during packaging')
            command.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', '--verbose=2', app])
            report.update(sourceUnchanged=True, artifactSha256=common.sha256(candidate), artifactBytes=candidate.stat().st_size)
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
                for force in (False, True):
                    if not os.path.ismount(mount):
                        break
                    try:
                        command.run(['/usr/bin/hdiutil', 'detach', *(['-force'] if force else []), mount], timeout=120)
                        report['cleanup']['ownMountDetached'] = True
                    except Exception as exc:
                        cleanup_error = str(exc)
                if not os.path.ismount(mount):
                    cleanup_error = None
            if scratch is not None:
                if not os.path.ismount(Path(scratch.name) / 'mounted'):
                    try:
                        scratch.cleanup(); report['cleanup']['scratchRemoved'] = True
                    except OSError as exc:
                        cleanup_error = str(exc)
                else:
                    scratch._finalizer.detach()
                    report['cleanup']['retainedScratch'] = scratch.name
                    cleanup_error = cleanup_error or 'own volume remains mounted'
            if original is not None:
                try:
                    report['sourceUnchanged'] = common.tree_manifest(app) == original
                except Exception as exc:
                    report['sourceRecheckError'] = str(exc)
            if cleanup_error:
                report.update(status='cleanup-failed', cleanupError=cleanup_error)
            report['cleanup']['outputPublished'] = published
            evidence.seek(0); evidence.truncate()
            json.dump(report, evidence, ensure_ascii=False, indent=2); evidence.write('\n')
            evidence.flush(); os.fsync(evidence.fileno())
            if cleanup_error and sys.exc_info()[0] is None:
                raise RuntimeError('own mount/scratch cleanup failed; inspect metadata')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--metadata', type=Path, required=True)
    parser.add_argument('--expected-version', choices=[VERSION], default=VERSION)
    parser.add_argument('--input-kind', choices=['candidate', 'release-input'], default='candidate')
    args = parser.parse_args()
    report = package(args.app, args.output, args.metadata, args.expected_version, args.input_kind)
    print(json.dumps({k: report[k] for k in ('status', 'output', 'artifactSha256', 'artifactBytes')}))


if __name__ == '__main__':
    main()
