import hashlib
import json
from pathlib import Path
import re
import subprocess
import time

base = Path('/private/tmp/ocx-0110-c1e3-candidate-hqjf10k6')
app = base / 'test-install/OpenCodeX Desktop.app'
exe = app / 'Contents/MacOS/opencodex-desktop'
daily = '/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop'
source = 'c1e3c09d8ed33238543ce5943e439991b8099088'
def app_processes():
    rows = subprocess.check_output(['ps', '-axo', 'pid=,command='], text=True).splitlines()
    return [(int(parts[0]), parts[1]) for row in rows
            if len(parts := row.strip().split(None, 1)) == 2
            and parts[1].endswith('/Contents/MacOS/opencodex-desktop')]
assert all(path == daily for _, path in app_processes()), 'An isolated test is already running'
assert hashlib.sha256(exe.read_bytes()).hexdigest() == 'b32c9a8d07955bcd5c0b3edaec34ba7ef738c9444bf0cfd7ac5af204077508f2'
assert source.encode() in exe.read_bytes()
target = base / 'noninjected-opportunity01-launch.json'
assert not target.exists(), 'Never overwrite evidence'
environment = {'OPENCODEX_SANDBOX': '1', 'OPENCODEX_SANDBOX_ROOT': str(base / 'test-data'),
               'OPENCODEX_SANDBOX_BOUNDARY': str(base),
               'DYLD_INSERT_LIBRARIES': '', 'OPENCODEX_UPDATE_PROBE_LOG': ''}
argv = ['open', '-n', '-a', str(app), '--stdout', str(base / 'noninjected-opportunity01-console.log'),
        '--stderr', str(base / 'noninjected-opportunity01-stderr.log')]
for key, value in environment.items():
    argv.extend(['--env', f'{key}={value}'])
started = time.time()
result = subprocess.run(argv, capture_output=True, text=True)
assert result.returncode == 0, result.stderr
deadline = time.monotonic() + 10
while time.monotonic() < deadline:
    processes = app_processes()
    found = [pid for pid, path in processes if path == str(exe)]
    lock_path = base / 'test-data/manager-state/app.lock'
    lock = json.loads(lock_path.read_text())
    if len(found) == 1 and lock.get('pid') == found[0]:
        break
    time.sleep(0.2)
else:
    raise AssertionError('Test instance/lock not ready')
assert len([path for _, path in processes if path != daily]) == 1
raw = subprocess.check_output(['ps', 'eww', '-p', str(found[0]), '-o', 'command='], text=True)
def env_value(key):
    match = re.search(r'(?:^| )' + key + r'=([^ ]*)', raw)
    return match.group(1).strip() if match else None
assert all(env_value(key) == value for key, value in environment.items() if value)
assert not env_value('DYLD_INSERT_LIBRARIES')
assert not env_value('OPENCODEX_UPDATE_PROBE_LOG')
value = {'source_sha': source, 'argv': argv, 'started_at': started, 'observed_at': time.time(),
         'pid': found[0], 'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr,
         'sole_isolated_instance': True, 'sandbox_environment_matches': True,
         'injection_environment_empty_or_absent': True, 'lock_metadata_matches': True}
with target.open('x') as handle:
    json.dump(value, handle, indent=2); handle.write(chr(10))
print(json.dumps(value))
