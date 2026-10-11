from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess, time, zipfile

base = Path('/private/tmp/ocx-0110-deb740-foreground')
dest = base / 'windows-download'
dest.mkdir(exist_ok=False)
source_sha = 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3'
repo = 'gzers/opencodex-desktop'
run_id = 38149645814
sha = lambda b: hashlib.sha256(b).hexdigest()
def api(path):
    return subprocess.check_output(['gh', 'api', path])
run_raw = api(f'repos/{repo}/actions/runs/{run_id}')
run = json.loads(run_raw)
assert run['head_sha'] == source_sha and run['status'] == 'completed' and run['conclusion'] == 'success'
(base / 'windows-run-final.json').write_bytes(run_raw)
jobs_raw = api(f'repos/{repo}/actions/runs/{run_id}/jobs')
jobs = json.loads(jobs_raw)
assert len(jobs['jobs']) == 1 and jobs['jobs'][0]['conclusion'] == 'success'
(base / 'windows-jobs-final.json').write_bytes(jobs_raw)
artifact_raw = api(f'repos/{repo}/actions/runs/{run_id}/artifacts')
(base / 'windows-artifacts-final.json').write_bytes(artifact_raw)
artifacts = json.loads(artifact_raw)['artifacts']
assert {a['id'] for a in artifacts} == {11706329022, 11706169265}
results = []
for a in artifacts:
    assert a['workflow_run']['head_sha'] == source_sha and not a['expired']
    archive = dest / f"{a['id']}.zip"
    started = time.time()
    with archive.open('xb') as out:
        proc = subprocess.run(['gh', 'api', f"repos/{repo}/actions/artifacts/{a['id']}/zip"], stdout=out, stderr=subprocess.PIPE)
    assert proc.returncode == 0, proc.stderr.decode(errors='replace')
    digest = sha(archive.read_bytes())
    assert a['digest'] == 'sha256:' + digest and archive.stat().st_size == a['size_in_bytes']
    members = []
    unpacked = dest / str(a['id'])
    with zipfile.ZipFile(archive) as z:
        assert z.testzip() is None
        for info in z.infolist():
            name = Path(info.filename)
            assert not name.is_absolute() and '..' not in name.parts
            if info.is_dir(): continue
            data = z.read(info)
            target = unpacked / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            members.append({'name': info.filename, 'bytes': len(data), 'sha256': sha(data)})
    results.append({'artifact': a, 'zip_sha256': digest, 'zip_bytes': archive.stat().st_size, 'zip_crc_passed': True, 'download_started_at': started, 'members': members})
report = {'source_sha': source_sha, 'run_id': run_id, 'verified_at_utc': datetime.now(timezone.utc).isoformat(), 'run_conclusion': 'success', 'artifacts': results, 'installer_executed_on_this_host': False, 'release_accepted': False}
(base / 'windows-download-verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+chr(10))
print(json.dumps({'verified': True, 'artifacts': [{'id': x['artifact']['id'], 'members':len(x['members'])} for x in results]}))
