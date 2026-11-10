"""Build/stage unchanged official2.99 with pinned complete platform dependencies.

No maintained source generation, installation, service startup or user
configuration writes. Outputs are explicit fresh build paths.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
POLICY = ROOT / '.github/runtime-platforms-024.json'


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def download_official(destination, source, proxy=None):
    if destination.exists():
        raise ValueError('official input path must be fresh')
    if not source['url'].startswith('https://registry.npmjs.org/@bitkyc08/opencodex/-/'):
        raise ValueError('official package source is not the pinned registry')
    opener = urllib.request.build_opener(urllib.request.ProxyHandler(
        {'http': proxy, 'https': proxy} if proxy else {}))
    with opener.open(source['url'], timeout=60) as response, destination.open('xb') as target:
        size = 0
        while block := response.read(1024 * 1024):
            size += len(block)
            if size > 64 * 1024 * 1024:
                raise ValueError('official input exceeds source limit')
            target.write(block)
    if digest(destination) != source['sha256']:
        raise ValueError('official source SHA256 mismatch; partial retained')


def stage(platform, node, npm, output, archive=None, node_archive=None, proxy=None, resource_directory=None):
    output = output.resolve()
    if output.exists():
        raise ValueError('fresh output required')
    policy = json.loads(POLICY.read_bytes())
    if policy['runtime']['distribution'] != 'official-npm':
        raise ValueError('0.2.24 uses the unchanged official package')
    destination = resource_directory or ROOT / 'apps/desktop/tauri/resources/runtime-bundle'
    destination = destination.resolve()
    if destination.exists():
        raise ValueError('fresh resource directory required')
    output.mkdir(parents=True)
    archive = Path(archive).resolve() if archive else output / policy['runtime']['archive']
    if not archive.exists():
        download_official(archive, policy['runtime'], proxy)
    if digest(archive) != policy['runtime']['sha256']:
        raise ValueError('official source input digest mismatch')
    bundle = output / 'platform-bundle'
    argv = ['python3', str(ROOT / '.github/scripts/build_desktop_runtime_bundle.py'),
            '--policy', str(POLICY), '--platform', platform, '--runtime-archive', str(archive),
            '--build-node', str(node), '--npm-cli', str(npm), '--cache', str(output / 'npm-cache'),
            '--output', str(bundle)]
    if node_archive:
        argv += ['--node-archive', str(node_archive)]
    if proxy:
        argv += ['--proxy', proxy]
    subprocess.run(argv, check=True)
    result = json.loads((bundle / 'result.json').read_bytes())
    destination.mkdir(parents=True, exist_ok=False)
    shutil.copy2(result['archive'], destination / result['archiveName'])
    shutil.copy2(archive, destination / 'official-runtime.tgz')
    manifest = Path(result['manifest'])
    shutil.copy2(manifest, destination / 'runtime-bundle.manifest.json')
    pin = digest(manifest)
    (destination / 'runtime-bundle.manifest.sha256').write_text(pin + '\n')
    override = {'bundle': {'resources': {
        'resources/LICENSE': 'resources/LICENSE',
        'resources/THIRD-PARTY-NOTICES.txt': 'resources/THIRD-PARTY-NOTICES.txt',
        'resources/runtime-bundle/': 'runtime-bundle/'}}}
    config = output / 'tauri.runtime-bundle.conf.json'
    config.write_text(json.dumps(override, indent=2) + '\n')
    return {'resourceDirectory': str(destination), 'descriptorSha256': pin,
            'officialSourceSha256': digest(archive), 'tauriConfig': str(config)}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=['darwin-arm64', 'win32-x64'], required=True)
    parser.add_argument('--build-node', type=Path, required=True)
    parser.add_argument('--npm-cli', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archive', type=Path)
    parser.add_argument('--node-archive', type=Path)
    parser.add_argument('--proxy')
    args = parser.parse_args()
    print(json.dumps(stage(args.platform, args.build_node.resolve(), args.npm_cli.resolve(),
                           args.output.resolve(), args.archive, args.node_archive, args.proxy), indent=2))
