"""Use an archived native report and in-memory mutations to check the Host preflight gate."""
import copy
import importlib.util
import json
import sys
from pathlib import Path, PureWindowsPath


verifier, archive = map(Path, sys.argv[1:])
spec = importlib.util.spec_from_file_location('host_verifier', verifier)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)

original = gate.decode(gate.read(archive, 'Host-fixture-preflight.json'))
creation = gate.decode(gate.read(archive, 'Host-creation.json'))
profile = PureWindowsPath(original['profile'])
image = PureWindowsPath(creation['image'])

gate.verify_preflight(original, profile, image)
print(json.dumps({'case': 'archived-native-preflight', 'accepted': True}))


def expect_refusal(case, evidence, expected, candidate_image=image):
    try:
        gate.verify_preflight(evidence, profile, candidate_image)
    except ValueError as error:
        if expected not in str(error):
            raise
        print(json.dumps({'case': case, 'accepted': False, 'error': str(error)}))
    else:
        raise AssertionError(f'{case}: evidence was accepted')


for case, field, value in (
    ('missing-target', 'targets', original['targets'][:-1]),
    ('duplicate-target', 'targets', original['targets'][:-1] + original['targets'][:1]),
    ('different-fixture-target', 'targets', original['targets'][:-1] + [
        dict(original['targets'][-1], path=str(profile / 'other/preparation'))]),
    ('unknown-absence', 'targets', [dict(original['targets'][0], absent=None)] + original['targets'][1:]),
    ('native-absence-error', 'targets', [dict(original['targets'][0], raw=5)] + original['targets'][1:]),
    ('absence-error-detail', 'targets', [dict(original['targets'][0], error='unknown')] + original['targets'][1:]),
    ('old-scene-exclusion-missing', 'excludedRoots', original['excludedRoots'][:-1]),
    ('different-exclusion', 'excludedRoots', original['excludedRoots'][:-1] + [str(profile / 'other')]),
    ('duplicate-exclusion', 'excludedRoots', original['excludedRoots'][:-1] + original['excludedRoots'][:1]),
    ('filtered-token', 'elevationType', 3),
    ('boolean-token-type', 'elevationType', True),
):
    expected = {'targets': 'target absence', 'excludedRoots': 'exclusion roots',
                'elevationType': 'ordinary token'}[field]
    expect_refusal(case, dict(original, **{field: value}), expected)

for case, input_base in (
    ('excludes-fixture', profile / '.codex'),
    ('exclusion-under-fixture', profile / 'AppData'),
):
    evidence = copy.deepcopy(original)
    for target in evidence['targets']:
        leaf = PureWindowsPath(target['path']).relative_to(image.parent.parent)
        target['path'] = str(input_base / leaf)
    expect_refusal(case, evidence, 'overlap the fixture', input_base / 'I/ocxd.exe')
expect_refusal('parent-traversal-fixture', original, 'fixture layout', profile / '../I/ocxd.exe')

try:
    gate.verify(archive, original['sid'], original['session'], profile, image, original['imageSha256'])
except ValueError as error:
    if not str(error).startswith('Host refused:'):
        raise
    print(json.dumps({'case': 'archived-host-failure', 'accepted': False, 'error': str(error)}))
else:
    raise AssertionError('Archived unsuccessful Host was accepted')
