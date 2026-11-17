"""Small synthetic delivery, never a runnable official package."""
import hashlib
import io
import json
from pathlib import Path
import tarfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value).encode()


def tar_bytes(files):
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode='w:gz', format=tarfile.USTAR_FORMAT) as archive:
        for name, data in sorted(files.items()):
            member = tarfile.TarInfo(name); member.size = len(data); member.mode = 0o644
            archive.addfile(member, io.BytesIO(data))
    return output.getvalue()


class DeliveryFixture:
    def __init__(self, root, platform='darwin-arm64'):
        self.root = Path(root); self.root.mkdir(parents=True, exist_ok=True)
        self.upstream = {'package/package.json': encoded({'name': '@bitkyc08/opencodex', 'version': '2.99.0'}),
                         'package/bin/ocx.mjs': b'never execute this fixture'}
        self.source = tar_bytes(self.upstream)
        self.target = {'os': 'win32' if platform == 'win32-x64' else 'darwin',
                       'cpu': 'x64' if platform == 'win32-x64' else 'arm64',
                       'archiveName': 'complete.tar.gz', 'nodeExecutable': 'runtime/node/bin/node',
                       'npmCli': 'runtime/node/lib/node_modules/npm/bin/npm-cli.js',
                       'bunExecutable': 'runtime/bun/bun'}
        self.policy = {'bundleVersion': '0.2.24',
                       'runtime': {'distribution': 'official-npm', 'version': '2.99.0',
                                   'archive': 'official.tgz', 'sha256': digest(self.source)},
                       'node': {'version': '26.8.1'}, 'bunVersion': '1.4.4',
                       'platforms': {platform: self.target}, 'projectPackage': {'private': True},
                       'packageLock': {'packages': {'node_modules/@bitkyc08/opencodex': {'version': '2.99.0'}}}}
        self.platform = platform
        self.files = {'runtime/opencodex/node_modules/@bitkyc08/opencodex/' + name.removeprefix('package/'): data
                      for name, data in self.upstream.items()}
        self.files.update({'runtime/opencodex/sources/official.tgz': self.source,
                           'runtime/opencodex/package.json': encoded(self.policy['projectPackage']),
                           'runtime/opencodex/package-lock.json': encoded(self.policy['packageLock'])})
        for name in ('nodeExecutable', 'npmCli', 'bunExecutable'):
            self.files[self.target[name]] = b'fixture; never execute'
        self.write()

    def write(self, extra_members=(), mutate_rows=None):
        data = tar_bytes(self.files) if not extra_members else self.with_extra(extra_members)
        (self.root / 'complete.tar.gz').write_bytes(data)
        (self.root / 'official-runtime.tgz').write_bytes(self.source)
        self.descriptor = {'schemaVersion': 1, 'platform': self.platform, 'runtimeVersion': '2.99.0',
                           'nodeVersion': '26.8.1', 'bunVersion': '1.4.4',
                           'runtimeArchiveSha256': self.policy['runtime']['sha256'],
                           'archiveName': 'complete.tar.gz', 'archiveSha256': digest(data), 'archiveBytes': len(data),
                           'entries': [{'path': name, 'kind': 'file', 'bytes': len(value), 'mode': 0o644,
                                        'sha256': digest(value)} for name, value in sorted(self.files.items())]}
        if mutate_rows:
            mutate_rows(self.descriptor['entries'])
        self.pin()

    def with_extra(self, extra):
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode='w:gz', format=tarfile.USTAR_FORMAT) as archive:
            for name, data in sorted(self.files.items()):
                member = tarfile.TarInfo(name); member.size = len(data); member.mode = 0o644
                archive.addfile(member, io.BytesIO(data))
            for member in extra:
                archive.addfile(member, io.BytesIO(b'x' * member.size) if member.isfile() else None)
        return output.getvalue()

    def pin(self):
        raw = encoded(self.descriptor)
        (self.root / 'runtime-bundle.manifest.json').write_bytes(raw)
        (self.root / 'runtime-bundle.manifest.sha256').write_text(digest(raw) + '\n')
