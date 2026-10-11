import fcntl, hashlib, json, os, subprocess, time
from pathlib import Path

base = Path('/private/tmp/ocx-0110-deb740-foreground')
source_sha = 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3'
sha = lambda b: hashlib.sha256(b).hexdigest()
def save(name, value):
    with (base / name).open('x') as f:
        json.dump(value, f, indent=2, ensure_ascii=False); f.write(chr(10))
def processes():
    return subprocess.check_output(['ps', '-axo', 'pid=,lstart=,command='], text=True).splitlines()
def test_pids():
    return [int(row.split()[0]) for row in processes() if str(exe) in row and row.rstrip().endswith(str(exe))]

rounds = []
for i in range(1, 11):
    phase = f'restart-valid-{i:02d}'
    v = json.loads((base / f'probe-{phase}-verification.json').read_text())
    assert v['boundary_attempts'] == 0 and v['cache_and_schedule_unchanged']
    assert v['all_foreground_samples_visible'] and v['foreground_duration_s'] >= 46
    ax_name = 'routes-100-final-ax.txt' if i == 3 else phase + ('-final-ax.txt' if i in (7, 8) else '-ax.txt')
    ax = (base / ax_name).read_text()
    assert 'tauri://localhost#overview' in ax and '尚未接入 OpenCodex' in ax
    rounds.append({**v, 'mounted_ui_evidence': ax_name})
routes = json.loads((base / 'routes-100-verification.json').read_text())
assert routes['source_sha'] == source_sha and routes['overview_entries'] == 100
assert routes['all_action_interval_covered'] and routes['boundary_attempts'] == 0
save('foreground-summary.json', {'source_sha': source_sha, 'verified_at': time.time(),
    'restarts_verified': 10, 'restarts': rounds, 'routes': routes,
    'preserved_invalid_monitor': 'probe-restart-valid-02-foreground.json',
    'invalid_reason': 'Wrong PID 20962 was supplied to the monitor; corrected monitor follows PID20777. Not an application failure.',
    'scope': 'Cache valid, future deadlines, preparation-state overview; main-process boundary attempts only.',
    'not_acceptance': ['CPU/startup performance', 'system-wide network traffic', 'ready-state overview updates', 'stable 24h / beta 6h cycles', 'real update install']})
app = base / 'test-install/OpenCodeX Desktop.app'
exe = app / 'Contents/MacOS/opencodex-desktop'
assert not test_pids(), 'Injected test remains running'
lock_path = base / 'test-data/manager-state/app.lock'
with lock_path.open('r+b') as f:
    fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
    fcntl.flock(f, fcntl.LOCK_UN)
daily = [r.strip() for r in processes() if r.strip().startswith('75764 ')][0]
assert 'Fri Oct  9 22:00:24 2026' in daily and daily.endswith('/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop')
save('probe-final-quit.json', {'observed_at': time.time(), 'method': 'CUA Cmd-Q; no AX after quit', 'test_process_absent': True, 'existing_app_lock_free': True, 'lock_file_not_deleted': True, 'daily_process': daily})
argv = ['open', '-n', '-a', str(app), '--stdout', str(base / 'ordinary-console.log'), '--stderr', str(base / 'ordinary-stderr.log')]
for key, value in {'OPENCODEX_SANDBOX':'1', 'OPENCODEX_SANDBOX_ROOT':str(base / 'test-data'), 'OPENCODEX_SANDBOX_BOUNDARY':str(base)}.items():
    argv += ['--env', f'{key}={value}']
env = {k: v for k, v in os.environ.items() if not k.startswith('DYLD_') and k != 'OPENCODEX_UPDATE_PROBE_LOG'}
started = time.time()
r = subprocess.run(argv, capture_output=True, text=True, env=env)
save('ordinary-open.json', {'argv': argv, 'started_at': started, 'exit_code': r.returncode, 'stdout': r.stdout, 'stderr': r.stderr})
assert r.returncode == 0, r.stderr
deadline = time.monotonic() + 10
while time.monotonic() < deadline:
    found = test_pids()
    if len(found) == 1: break
    time.sleep(0.1)
else: raise AssertionError('Ordinary test not ready')
pid = found[0]
psenv = subprocess.check_output(['ps', 'eww', '-p', str(pid), '-o', 'command='], text=True)
assert 'DYLD_INSERT_LIBRARIES=' not in psenv and 'OPENCODEX_UPDATE_PROBE_LOG=' not in psenv
assert f'OPENCODEX_SANDBOX_ROOT={base / "test-data"}' in psenv
print(json.dumps({'pid': pid, 'injection_env_absent': True, 'source_sha': source_sha}))
save('ordinary-launch.json', {'pid': pid, 'started_at': started, 'observed_at': time.time(), 'source_sha': source_sha, 'test_exe_sha256': sha(exe.read_bytes()), 'data_root': str(base / 'test-data'), 'probe_environment_absent': True, 'only_one_test_instance': True, 'daily_process': daily})
