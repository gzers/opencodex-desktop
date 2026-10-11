from pathlib import Path
from datetime import datetime, timezone
import gzip
import hashlib
import json
import subprocess

repo = Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop')
code = Path('/Users/ezio/.codex/worktrees/ocx-0110-main')
base = Path('/private/tmp/ocx-0110-deb740-foreground')
sha = 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3'
collab = Path('docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略')
record = collab / '验证/20261011-deb740-缓存前台复验'
dest = repo / record / 'evidence'
digest = lambda data: hashlib.sha256(data).hexdigest()
before = json.loads(Path('/private/tmp/ocx-0110-clock-audit/docs-dirty-before.json').read_text())
assert subprocess.check_output(['git', 'branch', '--show-current'], cwd=repo).decode().strip() == 'docs/governance-main'
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=code).decode().strip() == sha
assert not subprocess.check_output(['git', 'diff', '--cached', '--name-only'], cwd=repo)
for name, old in before.items():
    p = repo / name
    assert {'exists': p.exists(), 'sha256': digest(p.read_bytes()) if p.exists() else None} == old, name
assert not (repo / record).exists()

summary = json.loads((base / 'foreground-summary.json').read_text())
cache = json.loads((base / 'cache-before.json').read_text())
assert summary['source_sha'] == sha and summary['restarts_verified'] == 10
pids = []
minimum_duration = float('inf')
for phase_index in range(1, 11):
    phase = f'restart-valid-{phase_index:02d}'
    launch = json.loads((base / f'probe-{phase}-launch.json').read_text())
    verification = json.loads((base / f'probe-{phase}-verification.json').read_text())
    raw = [json.loads(line) for line in (base / f'probe-{phase}.jsonl').read_text().splitlines()]
    assert launch['source_sha'] == sha and launch['injection_loaded']
    assert len(raw) == 1 and raw[0]['event'] == 'probe_loaded' and raw[0]['pid'] == launch['pid']
    assert verification['pid'] == launch['pid'] and verification['boundary_attempts'] == 0
    assert verification['files'] == cache['files'] and verification['cache_and_schedule_unchanged']
    monitor = json.loads((base / verification['foreground_file']).read_text())
    assert monitor['pid'] == launch['pid'] and monitor['all_samples_frontmost_visible']
    assert len(monitor['samples']) == 47 and monitor['duration_monotonic_s'] >= 46
    assert all(x['target_frontmost'] and x['target_window_on_screen'] for x in monitor['samples'])
    assert monitor['started_at'] >= launch['started_at']
    assert all(x['next_due'] > monitor['ended_at'] for x in cache['schedule']['entries'].values())
    ax = (base / summary['restarts'][phase_index-1]['mounted_ui_evidence']).read_text()
    assert 'tauri://localhost#overview' in ax and 'OpenCodeX' in ax
    pids.append(launch['pid'])
    minimum_duration = min(minimum_duration, monitor['duration_monotonic_s'])
assert len(set(pids)) == 10
wrong = json.loads((base / 'probe-restart-valid-02-foreground.json').read_text())
assert wrong['pid'] == 20962 and wrong['pid'] != pids[1] and 'error' in wrong
route = json.loads((base / 'routes-100-verification.json').read_text())
monitor = json.loads((base / 'routes-100-foreground.json').read_text())
actions = [json.loads(x) for x in (base / 'routes-100-actions.jsonl').read_text().splitlines()]
assert len(actions) == 200 and route['pid'] == monitor['pid'] == pids[2]
assert route['source_sha'] == sha and route['boundary_attempts'] == 0 and route['files'] == cache['files']
assert route['settings_entries'] == route['overview_entries'] == 100
for index, action in enumerate(actions, 1):
    page = 'settings' if index % 2 else 'overview'
    assert action['seq'] == index and action['page'] == page
    assert f'tauri://localhost#{page}' in action['ax']
assert monitor['started_at'] <= actions[0]['started_at'] < actions[-1]['ended_at'] <= monitor['ended_at']
assert len(monitor['samples']) == 2493
assert all(x['target_frontmost'] and x['target_window_on_screen'] for x in monitor['samples'])
ordinary = json.loads((base / 'ordinary-final-verification.json').read_text())
assert ordinary['source_sha'] == sha and ordinary['pid'] == 23060
assert all(ordinary[x] for x in ['only_one_test_instance', 'injection_environment_absent', 'lock_held', 'daily_process_unchanged', 'cache_and_schedule_unchanged'])
assert ordinary['files'] == cache['files']
win = json.loads((base / 'windows-installer-independent-verification.json').read_text())
assert win['source_sha'] == sha and win['result'] == 'pass' and win['run_id'] == 38149645814
assert win['release_accepted'] is False

selected = ['cache-before.json', 'test-identity.json', 'old-quit.json', 'probe-final-quit.json',
    'ordinary-console.log', 'ordinary-stderr.log', 'ordinary-final-ax.txt', 'ordinary-final.jpg',
    'ordinary-launch.json', 'ordinary-open.json', 'ordinary-final-verification.json',
    'foreground-summary.json', 'foreground-monitor-ready.swift', 'foreground-monitor-routes.swift', 'launch.py', 'verify.py',
    'probe-restart-valid-02-foreground-corrected.json',
    'restart-valid-01-ax.txt', 'restart-valid-01.jpg', 'restart-valid-02-ax.txt',
    'restart-valid-04-ax.txt', 'restart-valid-05-ax.txt', 'restart-valid-06-ax.txt',
    'restart-valid-07-ax.txt', 'restart-valid-07-final-ax.txt', 'restart-valid-08-ax.txt',
    'restart-valid-08-final-ax.txt', 'restart-valid-09-ax.txt', 'restart-valid-10-ax.txt', 'restart-valid-10-final.jpg',
    'routes-100-actions.jsonl', 'routes-100-stop.json', 'routes-100-foreground.json', 'routes-100-verification.json',
    'routes-100-final-ax.txt', 'routes-100-final.jpg',
    'ci-status-20261011T145300Z.json', 'ci-success-20261011T145824Z.json', 'remote-main-20261011T145824Z.txt',
    'windows-candidate-dedup-20261011T145824Z.json', 'windows-candidate-dispatch.json',
    'windows-run-final.json', 'windows-jobs-final.json', 'windows-artifacts-final.json',
    'windows-download-verification.json', 'windows-receipt-readable.json', 'windows-installer-independent-verification.json']
for n in range(1, 11):
    stem = f'probe-restart-valid-{n:02d}'
    selected += [stem + suffix for suffix in ['-console.log', '-stderr.log', '-foreground.json', '-launch.json', '-open.json', '-verification.json', '.jsonl']]
assert len(selected) == len(set(selected))
sources = []
dest.mkdir(parents=True)

def archive(source, target):
    data = source.read_bytes()
    if source.suffix == '.json':
        json.loads(data)
    compressed = source.suffix in ['.log', '.txt', '.jsonl']
    target = target + '.gz' if compressed else target
    packed = gzip.compress(data, mtime=0) if compressed else data
    p = dest / target
    p.parent.mkdir(parents=True, exist_ok=True)
    assert not p.exists()
    p.write_bytes(packed)
    restored = gzip.decompress(packed) if compressed else packed
    assert restored == data
    sources.append({'source': str(source), 'source_bytes': len(data), 'source_sha256': digest(data),
        'archive': target, 'archive_bytes': len(packed), 'archive_sha256': digest(packed),
        'transformation': 'lossless gzip; mtime=0' if compressed else 'byte-identical',
        'live_log_snapshot': source.name in ['ordinary-console.log', 'ordinary-stderr.log']})

for name in selected:
    archive(base / name, name)
winbase = base / 'windows-download/11706169265'
archive(winbase / 'installer-verification.json', 'windows/installer-verification.json')
for kind in ['msi', 'nsis']:
    for name in ['startup.json', 'webview.json', 'startup-diagnostics.json', 'cdp-diagnostics.json', 'main-window.png', 'startup-stdout.log', 'startup-stderr.log']:
        archive(winbase / f'{kind}-startup' / name, f'windows/{kind}-startup/{name}')
for name in ['ocx-0110-deb-native-prepare.py', 'ocx-0110-deb-finalize.py', 'ocx-0110-deb-windows-fetch.py', 'ocx-0110-deb-verify.py', 'ocx-0110-deb-installer-verify.py', 'ocx-0110-deb-archive.py']:
    archive(Path('/private/tmp') / name, 'methods/' + name)

digest_only = []
paths = [base / 'foreground-monitor-ready', base / 'foreground-monitor-routes', base / 'update-probe.dylib',
    base / 'test-install/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop',
    base / 'windows-download/11706329022.zip', base / 'windows-download/11706169265.zip']
paths += [Path(x['path']) for x in win['installers']]
for p in paths:
    data = p.read_bytes()
    digest_only.append({'source': str(p), 'bytes': len(data), 'sha256': digest(data), 'retention': 'digest only; not copied to docs'})
provenance = {'source_sha': sha, 'source_root': str(base), 'archived_at_utc': datetime.now(timezone.utc).isoformat(),
    'selection': 'Explicit named foreground rounds and invalid wrong-PID monitor, scoped lifecycle records, AX/JPEG, Windows API/dispatch/download verification, receipt and seven startup files per installer, external method sources. No recursive data-root/profile/payload copy, installers or compiled probes.',
    'sources': sources, 'digest_only': digest_only, 'unrelated_tracked_paths_verified': len(before)}
(dest / 'source-provenance.json').write_text(json.dumps(provenance, ensure_ascii=False, indent=2) + chr(10))
files = sorted(p for p in dest.rglob('*') if p.is_file())
(dest / 'SHA256SUMS').write_text(''.join(f'{digest(p.read_bytes())}  {p.relative_to(dest).as_posix()}' + chr(10) for p in files))
for line in (dest / 'SHA256SUMS').read_text().splitlines():
    expected, name = line.split('  ', 1)
    assert digest((dest / name).read_bytes()) == expected
result = {'verified_at_utc': datetime.now(timezone.utc).isoformat(), 'source_sha': sha,
    'archive_files_verified': len(files), 'sources_verified': len(sources), 'digest_only_files': len(digest_only),
    'all_checksums_match': True, 'roundtrip_byte_match': True, 'unrelated_tracked_paths_unchanged': len(before),
    'restarts_verified': len(pids), 'restarts_minimum_monotonic_seconds': minimum_duration, 'settings_entries': 100, 'overview_entries': 100,
    'routes_foreground_samples_verified': len(monitor['samples']), 'all_scoped_foreground_samples_visible': True,
    'main_process_boundary_attempts': 0, 'cache_and_schedule_unchanged': True, 'wrong_pid_monitor_preserved': True,
    'only_one_ordinary_test_instance_at_final_snapshot': True, 'ordinary_pid_at_final_snapshot': ordinary['pid'],
    'windows_installers_independently_verified': 2, 'release_accepted': False}
(repo / record / 'archive-verification.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + chr(10))
print(json.dumps(result, ensure_ascii=False))
