from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import re
from pathlib import Path
import sys
root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('api', root / 'github-read.py')
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location('admit', api.CODE / 'test/smoke/windows-etw-ci-admit.py')
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
token = api.credential()
run_id = int(sys.argv[1])
name = sys.argv[2]
if len(sys.argv) == 4:
    assert re.fullmatch("[0-9a-f]{40}", sys.argv[3])
    api.SOURCE = sys.argv[3]
terminal = root / f"run-{run_id}-terminal.original.json"
if terminal.exists():
    frozen = json.loads(terminal.read_text(encoding="utf-8-sig"))
    run, jobs = frozen["run"], frozen["jobs"]
else:
    _, run = api.request(f'actions/runs/{run_id}', token)
    _, jobs = api.request(f'actions/runs/{run_id}/jobs?per_page=100', token)
assert run['head_sha'] == api.SOURCE and run['run_attempt'] == 1
job, = [j for j in jobs['jobs'] if j['name'] == name]
assert job['status'] == 'completed'
target = root / f"run-{run_id}-job-{job['id']}.original.log"
if not target.exists():
    receipt = admit.download(api.BASE + f"actions/jobs/{job['id']}/logs", target, 16*1024*1024, token=token)
else:
    receipt = {'bytes':target.stat().st_size, 'sha256':hashlib.sha256(target.read_bytes()).hexdigest()}
metadata = root / f"run-{run_id}-job-{job['id']}.json"
if metadata.exists():
    prior = json.loads(metadata.read_text(encoding="utf-8-sig"))
    assert prior["source"] == api.SOURCE and prior["runId"] == run_id and prior["job"]["id"] == job["id"]
    assert prior["log"] == receipt
else:
    metadata.write_text(json.dumps({'atUtc':datetime.now(timezone.utc).isoformat(),'source':api.SOURCE,'runId':run_id,'job':job,'log':receipt}, indent=2)+'\n',encoding='utf-8')
text = target.read_text(encoding='utf-8-sig')
counts = [tuple(map(int, row)) for row in re.findall(
    r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
print(json.dumps({"run": run_id, "job": name, "jobId": job["id"], "conclusion": job["conclusion"],
                  "bytes": receipt["bytes"], "suites": len(counts),
                  "passed": sum(row[0] for row in counts), "failed": sum(row[1] for row in counts),
                  "ignored": sum(row[2] for row in counts)}))
