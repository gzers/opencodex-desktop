"""One source-bound candidate request after the existing full CI; no replay."""
import importlib.util
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[3]
SOURCE = "99a62e4ad35e9404cb08922571ad8b77f89214dd"
BRANCH = "feature/0.1.10-empty-root-initialization"
spec = importlib.util.spec_from_file_location("api", Path(
    "D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/"
    "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py"))
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def save(name, value):
    with (ROOT / name).open("x", encoding="utf-8") as out:
        json.dump(value, out, indent=2)
        out.write("\n")


admission = read("ci-complete-admission.json")
assert admission["source"] == SOURCE and admission["runId"] == 39006330384
assert admission["wholeToolCiAdmitted"] and admission["status"] == "completed" and admission["conclusion"] == "success"
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO).decode().strip() == SOURCE
assert subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=REPO) == b""
token = api.credential()
_, ref = api.request("git/ref/heads/" + BRANCH, token)
assert ref["object"]["sha"] == SOURCE
_, state = api.request("actions/runs/39006330384", token)
assert state["head_sha"] == SOURCE and state["run_attempt"] == 1
assert state["status"] == "completed" and state["conclusion"] == "success"
if (ROOT / "candidate-handle.json").exists():
    handle = read("candidate-handle.json")
    assert handle["source"] == SOURCE
    print(json.dumps({"existingCandidateRunId": handle["runId"], "newDispatchRequested": False}))
    sys.exit(0)
query = "actions/workflows/windows-candidate.yml/runs?event=workflow_dispatch&head_sha=" + SOURCE + "&per_page=30"
_, before = api.request(query, token)
existing = [r for r in before["workflow_runs"] if r["head_sha"] == SOURCE and r["event"] == "workflow_dispatch"]
if not (ROOT / "candidate-dispatch-attempt.json").exists():
    assert not existing, "Candidate already exists; inspect rather than dispatch another"
    payload = {"ref": BRANCH, "inputs": {"purpose": "candidate", "admission_run": "39006330384",
                                        "diagnostic_path": "isolated-replay"}}
    save("candidate-dispatch-before.original.json", before)
    save("candidate-dispatch-request.json", payload)
    save("candidate-dispatch-attempt.json", {"atUtc": datetime.now(timezone.utc).isoformat(),
                                           "source": SOURCE, "automaticRetryAllowed": False})
    status, response = api.request("actions/workflows/windows-candidate.yml/dispatches", token, payload=payload)
    assert status == 204 and response is None
    save("candidate-dispatch-response.json", {"status": status, "acceptedAtUtc": datetime.now(timezone.utc).isoformat()})
# A delayed discovery or interrupted dispatch never authorizes another POST.
_, after = api.request(query, token)
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
save("candidate-discovery-" + stamp + ".original.json", after)
matches = [r for r in after["workflow_runs"] if r["head_sha"] == SOURCE and r["event"] == "workflow_dispatch"]
assert len(matches) <= 1
if matches:
    run = matches[0]
    assert run["run_attempt"] == 1 and run["head_branch"] == BRANCH
    save("candidate-handle.json", {"source": SOURCE, "runId": run["id"], "attempt": 1,
                                 "purpose": "candidate", "admissionCiRunId": 39006330384})
    print(json.dumps({"candidateRunId": run["id"], "status": run["status"],
                      "hostProductStarted": False, "releaseRequested": False}))
else:
    print(json.dumps({"dispatchAttemptPreserved": True, "discoveryPending": True, "dispatchWillNotBeRetried": True}))
