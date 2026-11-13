"""Stage the actual pinned Windows024 resources in a fresh, test-only directory.

Uses the shared stage implementation unchanged. Does not start official services,
install anything or use the ordinary user's HOME/npm configuration.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import traceback

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]


def load(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work', type=Path, required=True)
    args = parser.parse_args()
    work = args.work.resolve()
    if not work.is_relative_to(Path(__file__).resolve().parent / 'target'):
        raise ValueError('proof output must be inside this standalone target')
    work.mkdir(exist_ok=False, parents=True)
    report = {'scope': 'actual Windows resource staging only', 'passed': False,
              'officialExecuted': False, 'installed': False}
    try:
        policy = json.loads((ROOT / '.github/runtime-platforms-024.json').read_bytes())
        bundle = load('bundle024', '.github/scripts/build_desktop_runtime_bundle.py')
        stage = load('stage024', '.github/scripts/stage_runtime_bundle_024.py')
        node_spec = policy['platforms']['win32-x64']['nodeArchive']
        archive = work / node_spec['name']
        bundle.download_node(node_spec, archive)
        bundle.extract_node(archive, work / 'build-node', node_spec)
        official = work / policy['runtime']['archive']
        stage.download_official(official, policy['runtime'])
        result = stage.stage('win32-x64', work / 'build-node/node.exe',
                             work / 'build-node/node_modules/npm/bin/npm-cli.js',
                             work / 'stage', official, archive,
                             resource_directory=work / 'tauri/resources/runtime-bundle')
        report.update(passed=True, stage=result, policySha256=hashlib.sha256(
            (ROOT / '.github/runtime-platforms-024.json').read_bytes()).hexdigest())
    except Exception as error:
        report['firstError'] = {'type': type(error).__name__, 'message': str(error)}
        traceback.print_exc()
    finally:
        (work / 'stage-result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
