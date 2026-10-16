"""Keep the failed whole push separate from the successful whole PR."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
assert {p.name for p in ROOT.iterdir() if p.is_file()
        and p.name not in {"manifest.json", "verify.py", "verification.json"}} == {
    e["path"] for e in manifest["files"]}
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
run = read("ci-38824163566-terminal.original.json")
assert run["id"] == 38824163566 and run["status"] == "completed"
assert run["conclusion"] == "failure" and run["run_attempt"] == 1
assert run["head_sha"] == "d94c215d869c0560717d79dc02881ce8858afe88"
jobs = read("ci-38824163566-jobs-failed.original.json")["jobs"]
failed, = [j for j in jobs if j["conclusion"] not in {"success", "skipped"}]
assert failed["name"] == "backend" and failed["id"] == 116727574022
assert all(s["status"] == "pending" for s in failed["steps"] if s["name"].startswith("Run cargo"))
annotations = read("ci-38824163566-mac-annotations.original.json")
assert any("hosted runner lost communication" in a["message"] for a in annotations)
classification = read("push-failure-classification.json")
assert not classification["failureRelabelledSuccess"] and not classification["workflowRerunRequested"]
assert not classification["actualRootCauseOfRunnerLossEstablished"]
assert classification["logReadError"].startswith("HTTP 404")
assert classification["independentPrSuccessRun"] == 38824151573
summary = {"status": "verified-companion-push-hosted-runner-failure",
           "source": run["head_sha"], "runId": run["id"], "files": len(manifest["files"]),
           "macCargoChecksExecuted": False, "jobOriginalLogAvailable": False,
           "wholePushClaimedPassed": False, "workflowRerunRequested": False,
           "fullVersionGatePassed": False, "releaseAccepted": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
