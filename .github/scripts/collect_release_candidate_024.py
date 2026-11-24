"""Collect exact candidate bytes and distinguish updater and platform signatures."""
import argparse
import base64
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from verify_runtime_delivery_024 import sha256

ROOT = Path(__file__).resolve().parents[2]
TAURI = ROOT / 'apps/desktop/tauri'


def only(directory, pattern):
    (match,) = sorted(directory.glob(pattern))
    if match.is_symlink() or not match.is_file():
        raise ValueError(f'Expected one regular file for {pattern} in {directory}')
    return match


def platform_signature(package):
    command = [
        'pwsh', '-NoProfile', '-NonInteractive', '-Command',
        '$ErrorActionPreference = "Stop"; '
        '(Get-AuthenticodeSignature -LiteralPath $env:OCXD_CANDIDATE_PACKAGE).Status.ToString()',
    ]
    environment = dict(os.environ, OCXD_CANDIDATE_PACKAGE=str(package))
    status = subprocess.check_output(command, env=environment, text=True, timeout=60).strip()
    if status not in {'Valid', 'NotSigned'}:
        raise ValueError(f'Unverified Authenticode status for {package.name}: {status}')
    return status


def macos_signatures(app, image):
    subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True, timeout=60)
    details = subprocess.run(['codesign', '-dvv', str(app)], check=True, capture_output=True,
                             text=True, timeout=30).stderr.splitlines()
    identity = next((line.removeprefix('Authority=') for line in details if line.startswith('Authority=')), None)
    developer_id = bool(identity and identity.startswith('Developer ID Application:'))
    stapled = {}
    if developer_id:
        for path in (app, image):
            result = subprocess.run(['xcrun', 'stapler', 'validate', str(path)], capture_output=True,
                                    timeout=60)
            stapled[path.suffix] = result.returncode == 0
    return {'codeSignatureValid': True, 'developerId': developer_id, 'authority': identity,
            'adHoc': 'Signature=adhoc' in details, 'stapledTicketVerified': stapled}


def collect(source, output):
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if source != head:
        raise ValueError('Candidate source differs from checked out HEAD')
    config = json.loads((TAURI / 'tauri.conf.json').read_bytes())
    if config['version'] != '0.2.24':
        raise ValueError('Candidate must use Desktop 0.2.24')

    if sys.platform == 'darwin':
        bundle = TAURI / 'target/aarch64-apple-darwin/release/bundle'
        image = only(ROOT / 'test/out/macos024-package', '*.dmg')
        packages = [('dmg', image, False), ('updater', only(bundle / 'macos', '*.app.tar.gz'), True)]
        platform = 'darwin-aarch64'
        os_signature = macos_signatures(bundle / 'macos/OpenCodeX Desktop.app', image)
    elif sys.platform == 'win32':
        bundle = TAURI / 'target/release/bundle'
        packages = [('msi', only(bundle / 'msi', '*.msi'), True),
                    ('nsis', only(bundle / 'nsis', '*-setup.exe'), True)]
        platform = 'windows-x86_64'
        os_signature = {role: platform_signature(package) for role, package, _ in packages}
    else:
        raise ValueError('Unsupported candidate host')

    output.mkdir(parents=True)
    report = {'schemaVersion': 1, 'version': config['version'], 'sourceSha': head,
              'platform': platform, 'osSignatures': os_signature, 'accepted': False, 'files': []}
    with tempfile.TemporaryDirectory(prefix='ocxd024-verify-signature-') as directory:
        public_key = Path(directory) / 'updater.pub'
        public_key.write_bytes(base64.b64decode(config['plugins']['updater']['pubkey'], validate=True))
        for role, package, signed in packages:
            delivered = Path(shutil.copyfile(package, output / package.name))
            record = {'role': role, 'name': delivered.name, 'bytes': delivered.stat().st_size,
                      'sha256': sha256(delivered)}
            if signed:
                signature = package.with_name(package.name + '.sig')
                detached = Path(shutil.copyfile(signature, output / signature.name))
                decoded = Path(directory) / 'candidate.minisig'
                decoded.write_bytes(base64.b64decode(detached.read_bytes().strip(), validate=True))
                subprocess.run(['minisign', '-V', '-m', str(delivered), '-p', str(public_key),
                                '-x', str(decoded)], check=True, timeout=60)
                record['updaterSignature'] = {'name': detached.name, 'sha256': sha256(detached),
                                              'verified': True}
            report['files'].append(record)
    (output / 'candidate.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    collect(args.source, args.output)
