"""Fail before artifact/network work when tag and all actual build versions drift."""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]


def verify(root, tag):
    if tag != 'v0.2.24':
        raise ValueError('this release assembly is exclusively for v0.2.24')
    config = json.loads((root / 'apps/desktop/tauri/tauri.conf.json').read_bytes())
    ui = json.loads((root / 'apps/desktop/ui/package.json').read_bytes())
    cargo = (root / 'apps/desktop/tauri/Cargo.toml').read_text()
    package = re.search(r'(?ms)^\[package\]\s*$(.*?)(?=^\[|\Z)', cargo)
    matches = re.findall(r'(?m)^version\s*=\s*"([^"\r\n]+)"\s*$', package.group(1) if package else '')
    if len(matches) != 1:
        raise ValueError('Cargo package version is not explicit and unique')
    policy = json.loads((root / '.github/runtime-platforms-024.json').read_bytes())
    values = {'tauri': config['version'], 'ui': ui['version'], 'rust': matches[0],
              'runtimeBundle': policy['bundleVersion']}
    if any(value != tag[1:] for value in values.values()):
        raise ValueError('actual release source versions do not match the tag; no candidate admitted')
    if (policy['runtime']['version'] != '2.99.0'
            or policy['runtime']['distribution'] != 'official-npm'):
        raise ValueError('release requires the unchanged official2.99 distribution')
    return {'schemaVersion': 1, 'tag': tag, 'versions': values,
            'runtimeDistribution': 'official-npm', 'artifactBuilt': False, 'releaseAcceptance': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag', required=True)
    args = parser.parse_args()
    print(json.dumps(verify(ROOT, args.tag), indent=2))
