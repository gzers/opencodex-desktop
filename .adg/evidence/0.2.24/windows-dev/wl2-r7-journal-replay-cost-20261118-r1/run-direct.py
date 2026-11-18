import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import time

root = pathlib.Path(sys.argv[1]).resolve()
compile_dir = root / sys.argv[2]
source = [json.loads(line) for line in (compile_dir / 'stdout.log').read_bytes().splitlines()]
binaries = [entry['executable'] for entry in source if entry.get('reason') == 'compiler-artifact' and entry.get('profile', {}).get('test') and entry.get('target', {}).get('name') == 'opencodex_desktop_lib' and entry.get('executable')]
assert len(binaries) == 1, binaries
exe = pathlib.Path(binaries[0])
assert exe.is_file(), exe

def file_hash(path):
    with path.open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()

def write_json(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as handle:
        json.dump(value, handle, ensure_ascii=False, indent=2)
        handle.write('\n')

artifact = json.loads((root / 'test-binary' / 'artifact.json').read_bytes())
exe = pathlib.Path(artifact['retainedPath'])
original = file_hash(exe)
assert original == artifact['sha256']
filters = ['wl2_r7_journal_replay_cost_tracks_native_prefix_and_pure_data_separately', 'wl2_r6_batch_typed_immutable_event_replay_', 'wl2_r6_v3_native_journal_']
roots = (pathlib.Path(tempfile.gettempdir()).resolve(), pathlib.Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target').resolve())
def existing_roots():
    return {path for folder in roots for path in folder.iterdir() if path.is_dir()}
results = []
cleanup = []
for test_filter in filters:
    folder = root / ('direct-' + test_filter)
    folder.mkdir()
    arguments = [str(exe), test_filter, '--test-threads=1', '--nocapture']
    before = existing_roots()
    started = time.monotonic()
    with (folder / 'stdout.log').open('xb') as out, (folder / 'stderr.log').open('xb') as err:
        raw = subprocess.run(arguments, stdout=out, stderr=err).returncode
    elapsed = round((time.monotonic() - started) * 1000)
    output = (folder / 'stdout.log').read_text(encoding='utf-8', errors='replace')
    match = re.search(r'test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored;', output)
    count = None if not match else dict(zip(('passed', 'failed', 'ignored'), (int(group) for group in match.groups())))
    report = {'arguments': arguments, 'sha256': original, 'elapsedMs': elapsed, 'raw': raw, 'counts': count, 'newRoots': [str(path) for path in sorted(existing_roots() - before)]}
    write_json(folder / 'result.json', report)
    results.append(report)
    for path in set(re.findall(r'(?:fixture-start|cleanup-absent|r4-retained-synthetic-fixture)=([^\r\n]+)', output)):
        path = pathlib.Path(path)
        if path.is_absolute():
            cleanup.append({'suite': test_filter, 'path': str(path), 'absent': not os.path.lexists(path), 'reportedByFixture': True, 'existedBeforeSuite': path in before, 'suiteRaw': raw})
    print(f'{test_filter} raw={raw} counts={count} elapsedMs={elapsed}', flush=True)
assert file_hash(exe) == original
write_json(root / 'direct-results.json', results)
write_json(root / 'fixture-state.json', sorted(cleanup, key=lambda entry: (entry['suite'], entry['path'])))
sys.exit(any(entry['raw'] != 0 or not entry['counts'] or not entry['counts']['passed'] for entry in results))


