"""Verify official runtime bytes in the release's user-facing packages."""
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile
from urllib.parse import unquote

import package_macos_024 as macos_package
import verify_macos_updater_024 as macos_updater
import verify_runtime_delivery_024 as delivery
import verify_windows_installer_024 as windows


OFFICIAL_SHA256 = 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'


def verify_policy(policy):
    source = policy['runtime']
    if (policy['bundleVersion'], source['distribution'], source['version'], source['sha256'], source['url']) != (
        '0.2.24', 'official-npm', '2.99.0', OFFICIAL_SHA256,
        'https://registry.npmjs.org/@bitkyc08/opencodex/-/opencodex-2.99.0.tgz',
    ):
        raise ValueError('release must use the pinned official OpenCodex 2.99.0 npm archive')


def canonical_platforms(entries):
    mac, win = 'darwin-aarch64', 'windows-x86_64'
    required = {mac: '.app.tar.gz', win + '-msi': '.msi', win + '-nsis': '-setup.exe'}
    if set(entries) not in (set(required) | {win}, set(required) | {win, mac + '-app'}):
        raise ValueError('unexpected 0.2.24 updater platform set')
    if entries[win] not in (entries[win + '-msi'], entries[win + '-nsis']):
        raise ValueError('Windows updater alias differs from both signed installer entries')
    if mac + '-app' in entries and entries[mac + '-app'] != entries[mac]:
        raise ValueError('macOS updater alias differs from the signed app entry')
    for platform, suffix in required.items():
        if not unquote(entries[platform]['url']).endswith(suffix):
            raise ValueError(f'incorrect updater artifact type for {platform}')
    return {platform: entries[platform] for platform in required}


def verify_macos(image, archive, signature, config, policy):
    # A private mount directory guarantees cleanup targets only this verification.
    scratch = tempfile.TemporaryDirectory(prefix='ocxd024-release-mount-')
    mount = Path(scratch.name) / 'volume'
    mount.mkdir()
    image_hash = delivery.sha256(image)
    try:
        raw = subprocess.check_output(['/usr/bin/hdiutil', 'attach', str(image), '-readonly', '-nobrowse',
                                       '-noautoopen', '-noautofsck', '-mountpoint', str(mount), '-plist'], timeout=120)
        entities = plistlib.loads(raw)['system-entities']
        mounted = [Path(row['mount-point']).resolve() for row in entities if 'mount-point' in row]
        if mounted != [mount.resolve()] or not os.statvfs(mount).f_flag & os.ST_RDONLY:
            raise ValueError('expected exactly one owned read-only DMG volume')
        if {p.name for p in mount.iterdir()} - macos_package.METADATA != {
            macos_package.APP, 'Applications', 'Install.txt',
        }:
            raise ValueError('DMG must contain a single .app, Applications link and instructions')
        link = mount / 'Applications'
        instructions = mount / 'Install.txt'
        if not link.is_symlink() or os.readlink(link) != '/Applications':
            raise ValueError('invalid Applications link in disk image')
        if (instructions.is_symlink() or not instructions.is_file()
                or instructions.stat().st_size != len(macos_package.INSTALL.encode())
                or instructions.read_text(encoding='utf-8') != macos_package.INSTALL):
            raise ValueError('invalid installation instructions in disk image')
        report = macos_updater.verify(mount / macos_package.APP, archive, signature, config, policy)
        if delivery.sha256(image) != image_hash:
            raise ValueError('disk image changed during verification')
        print('Verified signed macOS updater and matching disk image with official runtime', flush=True)
        return report
    finally:
        try:
            if os.path.ismount(mount):
                subprocess.run(['/usr/bin/hdiutil', 'detach', str(mount)], check=True,
                               capture_output=True, timeout=120)
        finally:
            if os.path.ismount(mount):
                scratch._finalizer.detach()
            else:
                scratch.cleanup()


def verify_windows(archive, kind, policy):
    sevenzip = shutil.which('7zz') or shutil.which('7z')
    if sevenzip is None:
        raise ValueError('7-Zip is required for Windows installer verification')
    before = windows.file_image(archive, windows.MAX_INSTALLER)
    listing = windows.bounded_tool([sevenzip, 'l', '-slt', '-sccUTF-8', '--', str(archive)],
                                  windows.MAX_LISTING, capture=True)
    members = windows.member_index(listing['output'], kind)
    with tempfile.TemporaryDirectory(prefix='ocxd024-release-resources-') as directory:
        resources = Path(directory)
        for logical_path, msi_id in windows.MSI_IDS.items():
            if not logical_path.startswith('runtime-bundle/'):
                continue
            member = msi_id if kind == 'msi' else logical_path
            size = members.get(member, {}).get('bytes')
            if type(size) is not int or size <= 0:
                raise ValueError(f'missing embedded runtime resource: {logical_path}')
            with (resources / Path(logical_path).name).open('xb') as output:
                observed = windows.bounded_tool([sevenzip, 'x', '-so', '-spd', '--', str(archive), member],
                                               size, sink=output)
            if observed['bytes'] != size:
                raise ValueError(f'truncated embedded runtime resource: {logical_path}')
        report = delivery.verify(resources, 'win32-x64', policy)
    if windows.file_image(archive, windows.MAX_INSTALLER) != before:
        raise ValueError('Windows installer changed during verification')
    print(f'Verified signed Windows {kind} official runtime', flush=True)
    return report
