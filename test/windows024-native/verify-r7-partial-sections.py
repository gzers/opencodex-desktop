"""Check actual native collection sections in the small Windows proof host.

No synthetic claim/Owner/lease capability is defined. These tests exercise the
real file collection functions only; product slot/claim wiring needs its own
product typecheck and native runtime evidence.
"""
import argparse
import hashlib
import json
from pathlib import Path


def between(text, start, end):
    start = text.index(start)
    return text[start:text.index(end, start)]


def verify(repo, generated):
    base = repo / 'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
    tree = (base / 'tree.rs').read_text(encoding='utf-8')
    anchored = (base / 'tree/anchored.rs').read_text(encoding='utf-8')
    phase = (base / 'phase.rs').read_text(encoding='utf-8')
    coordination = (base / 'coordination.rs').read_text(encoding='utf-8')
    sections = [
        between(phase, 'pub(super) fn invalid()', 'pub(super) fn digest(')
        .replace('pub(super) fn invalid()', 'fn invalid()'),
        between(tree, '#[derive(PartialEq, Eq)]\nstruct Entry',
                '/// The actual working directory handle'),
        between(tree, 'fn check(end:', 'fn open(path:')
        .replace('super::bounded::QueryError', 'QueryError'),
        between(tree, 'fn id(file:', 'mod streams;'),
        between(tree, 'fn names(path:', 'fn scan('),
        between(anchored, '#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum Kind',
                '/// All fields are actual native objects'),
        between(anchored, 'fn open_node(', 'impl AnchoredTree {'),
    ]
    actual = (generated / 'native_partial_collect.rs').read_text(encoding='utf-8')
    start = actual.index('fn invalid()')
    end = actual.index('\n#[cfg(test)]', start)
    assert actual[start:end] == ''.join(sections), 'native collection section drift'
    expected = between(coordination, 'pub(super) fn validate_stable_bytes(', 'fn no_sidecars(')
    current = (generated / 'coordination_validation.rs').read_text(encoding='utf-8')
    assert current[current.index('pub(super) fn validate_stable_bytes('):] == expected
    assert 'const MAX: u64 = 2 * 1024 * 1024;' in coordination
    contract = (repo / 'apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs').read_text(encoding='utf-8')
    for name in ('.opencodex-native-main.claim.sqlite', '.opencodex-native-main.owner.sqlite'):
        assert name in actual and name in contract
    return {
        'nativeFileFunctionsSameSource': True,
        'readSecurityAndADSFunctions': 'actual native backend/query/stream modules linked by path',
        'nativeGrantConstructed': False,
        'omitted': ['AnchoredClaim/native lease and production slot/Owner',
                    'whole journal/Store recovery', 'MoveOriginal/working/Closed'],
        'adaptations': ['module imports/type visibility', 'two constant names and fixed MAX checked against source',
                        'error envelope for coordination validation', 'QueryError import path'],
        'files': [{'path': name, 'bytes': (generated / name).stat().st_size,
                   'sha256': hashlib.sha256((generated / name).read_bytes()).hexdigest()}
                  for name in ('native_partial_collect.rs', 'coordination_validation.rs')],
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--generated', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    value = verify(args.repo, args.generated)
    with args.output.open('x', encoding='utf-8') as target:
        json.dump(value, target, ensure_ascii=False, indent=2)
        target.write('\n')
    print(json.dumps({'actualCollectionSameSource': True, 'files': len(value['files'])}))
