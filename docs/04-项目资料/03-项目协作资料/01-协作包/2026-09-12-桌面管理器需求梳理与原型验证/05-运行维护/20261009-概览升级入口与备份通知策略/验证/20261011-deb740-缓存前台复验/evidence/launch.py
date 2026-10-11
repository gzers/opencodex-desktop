import json, subprocess, sys, time
from pathlib import Path
base = Path('/private/tmp/ocx-0110-deb740-foreground')
app = base / 'test-install/OpenCodeX Desktop.app'
exe = app / 'Contents/MacOS/opencodex-desktop'
phase = sys.argv[1]
assert phase.replace('-', '').isalnum()
def pids():
    rows = subprocess.check_output(['ps', '-axo', 'pid=,command='], text=True).splitlines()
    return [int(v[0]) for row in rows if len(v := row.strip().split(None, 1)) == 2 and v[1] == str(exe)]
assert not pids(), 'Test already running'
log = base / f'probe-{phase}.jsonl'
assert not log.exists(), 'Never overwrite evidence'
env = {'OPENCODEX_SANDBOX': '1', 'OPENCODEX_SANDBOX_ROOT': str(base / 'test-data'),
       'OPENCODEX_SANDBOX_BOUNDARY': str(base), 'DYLD_INSERT_LIBRARIES': str(base / 'update-probe.dylib'),
       'OPENCODEX_UPDATE_PROBE_LOG': str(log)}
argv = ['open', '-n', '-a', str(app), '--stdout', str(base / f'probe-{phase}-console.log'),
        '--stderr', str(base / f'probe-{phase}-stderr.log')]
for key, value in env.items():
    argv.extend(['--env', f'{key}={value}'])
started = time.time()
result = subprocess.run(argv, capture_output=True, text=True)
with (base / f'probe-{phase}-open.json').open('x') as out:
    json.dump({'argv': argv, 'started_at': started, 'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr}, out, indent=2)
assert result.returncode == 0, result.stderr
deadline = time.monotonic() + 10
while time.monotonic() < deadline:
    found = pids()
    rows = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
    if len(found) == 1 and any(r['event'] == 'probe_loaded' and r['pid'] == found[0] for r in rows):
        break
    time.sleep(0.2)
else:
    raise AssertionError('Candidate or injection not ready')
value = {'pid': found[0], 'phase': phase, 'started_at': started, 'observed_at': time.time(),
         'source_sha': 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3', 'log': str(log), 'injection_loaded': True,
         'method': 'standard macOS bundle launch via open CLI; explicit sandbox and probe environment; no background flag'}
with (base / f'probe-{phase}-launch.json').open('x') as out:
    json.dump(value, out, indent=2); out.write(chr(10))
print(json.dumps(value))
