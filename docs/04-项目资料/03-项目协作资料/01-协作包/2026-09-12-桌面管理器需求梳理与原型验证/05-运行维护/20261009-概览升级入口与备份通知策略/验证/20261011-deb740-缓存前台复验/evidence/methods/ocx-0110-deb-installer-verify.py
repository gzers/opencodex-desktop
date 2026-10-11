from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess, zipfile

base = Path('/private/tmp/ocx-0110-deb740-foreground')
code = Path('/Users/ezio/.codex/worktrees/ocx-0110-main')
source_sha = 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3'
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=code).decode().strip() == source_sha
run = read(base / 'windows-run-final.json')
assert (run['id'], run['head_sha'], run['conclusion']) == (38149645814, source_sha, 'success')
download = read(base / 'windows-download-verification.json')
zips = []
for a in download['artifacts']:
    api = a['artifact']
    p = base / 'windows-download' / (str(api['id']) + '.zip')
    actual = digest(p)
    assert p.stat().st_size == api['size_in_bytes'] == a['zip_bytes']
    assert actual == a['zip_sha256'] == api['digest'].split(':', 1)[1]
    with zipfile.ZipFile(p) as z:
        assert z.testzip() is None
        members = z.namelist()
    zips.append({'artifact_id': api['id'], 'bytes': p.stat().st_size, 'sha256': actual, 'api_digest_matches': True, 'crc_passed': True, 'members': len(members), 'expires_at': api['expires_at']})
candidate = base / 'windows-download/11706329022'
evidence = base / 'windows-download/11706169265'
r1 = candidate / 'test/out/windows-candidate/installer-verification.json'
r2 = evidence / 'installer-verification.json'
assert r1.read_bytes() == r2.read_bytes()
receipt = read(r1)
assert receipt == read(base / 'windows-receipt-readable.json')
assert receipt['sourceCommit'] == source_sha and receipt['version'] == '0.1.10'
assert receipt['result'] == 'pass' and not receipt['codeSigned'] and not receipt['releaseAccepted']
installers = []
for item in receipt['installers']:
    kind = item['kind']
    p = candidate / 'apps/desktop/tauri/target/release/bundle' / kind / item['name']
    actual = digest(p)
    assert p.stat().st_size == item['bytes'] and actual == item['sha256']
    assert item['result'] == item['firstScreen'] == 'pass'
    assert item['installExitCode'] == item['uninstallExitCode'] == 0 and item['uninstallRemovedExecutable']
    assert item['installedExecutableSha256'] == item['expectedInstalledExecutableSha256']
    assert item['productVersion'] == '0.1.10' and item['bundleMarker'] == {'msi': 'MSI', 'nsis': 'NSS'}[kind]
    start = read(evidence / (kind + '-startup/startup.json'))
    web = read(evidence / (kind + '-startup/webview.json'))
    diag = read(evidence / (kind + '-startup/startup-diagnostics.json'))
    cdp = read(evidence / (kind + '-startup/cdp-diagnostics.json'))
    assert start['result'] == 'pass' and start['expectedCommit'] == source_sha and start['sourceCommitMatched']
    assert start['sha256'] == item['installedExecutableSha256'] and start['observedSeconds'] == 30
    assert start['responding'] and start['mainWindowPresent']
    assert diag['rootPid'] == start['pid'] and diag['cdpPort'] == start['cdpPort']
    assert web['result'] == 'pass' and web['mainUiMounted'] and web['sandboxDataRootInitialized']
    assert (web['about']['version'], web['about']['platform'], web['discovery']['gate'], web['cliEnabled']) == ('0.1.10', 'Windows', 'missing_ocx', False)
    assert cdp['phase'] == 'complete' and cdp['result'] == 'pass' and cdp['target']['url'] == 'http://tauri.localhost/#overview'
    installers.append({'kind': kind, 'path': str(p), 'bytes': p.stat().st_size, 'sha256': actual, 'receipt_matches': True, 'runner_installed_exe_sha256': start['sha256'], 'runner_pid': start['pid'], 'runner_observed_seconds': start['observedSeconds'], 'source_identity_and_webview_match': True, 'webview_version': cdp['lastVersion']['Browser'], 'installed_exe_not_independently_extracted': True})
harness = (code / 'test/smoke/windows-startup.ps1').read_text()
assert 'WindowsIdentity]::GetCurrent()' in harness
result = {'verified_at_utc': datetime.now(timezone.utc).isoformat(), 'source_sha': source_sha, 'run_id': run['id'], 'result': 'pass', 'zips': zips, 'receipt_bytes': r1.stat().st_size, 'receipt_sha256': digest(r1), 'receipts_byte_identical': True, 'installers': installers, 'installer_executed_on_this_host': False, 'permission_fields_scope': 'PowerShell harness current identity, not target application token', 'limits': ['runner installed EXEs are not archived members; only runner hashes', 'sandbox data root / DPR1 / forced teardown do not accept default root, physical DPI or normal exit', 'CDP screenshots do not accept Mica/native appearance', 'code signing and real update/install acceptance remain open'], 'code_signed': False, 'release_accepted': False}
(base / 'windows-installer-independent-verification.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + chr(10))
print(json.dumps({'result': result['result'], 'installers': [{k: v for k, v in x.items() if k in ('kind', 'bytes', 'sha256', 'receipt_matches')} for x in installers], 'receipt_sha256': result['receipt_sha256'], 'release_accepted': False}, ensure_ascii=False))
