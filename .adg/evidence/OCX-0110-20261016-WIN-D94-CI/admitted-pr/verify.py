"""Verify one whole successful PR run; keep the companion push status separate."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
assert {p.name for p in ROOT.iterdir() if p.is_file()
        and p.name not in {"manifest.json", "verification.json"}} == {
    e["path"] for e in manifest["files"]} | {"verify.py"}
ci, admission = read("ci-log-verification.json"), read("ci-admission.json")
assert ci["source"] == admission["source"] == SOURCE
assert ci["wholeCiAdmitted"] and ci["actualCheckoutTreesEqual"]
assert admission["wholeCiAdmitted"] and not admission["partialJobsCombinedForAdmission"]
assert len(ci["runs"]) == 1 and ci["runs"][0]["id"] == 38824151573
assert not ci["allObservedRunsCompleted"] and not admission["allObservedRunsCompleted"]
job_state = read("ci-38824151573-jobs.original.json")
required = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert all(j["status"] == "completed" and j["conclusion"] == "success"
           for j in job_state["jobs"] if j["name"] in required)
assert required <= {j["name"] for j in job_state["jobs"]} <= required | {"build"}
assert all(j["conclusion"] == "skipped" for j in job_state["jobs"] if j["name"] == "build")
for job in ci["runs"][0]["jobs"]:
    data = (ROOT / job["log"]).read_bytes()
    assert len(data) == job["bytes"] and hashlib.sha256(data).hexdigest() == job["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == job["actualCheckout"] == "f2c5f1bd8ed827897255e2aaf174c23a3de7fbf9"
    assert job["tree"] == ci["tree"] == read("head-commit.original.json")["tree"]["sha"]
    assert job["counts"]["failed"] == 0
summary = {"status": "verified-one-whole-exact-source-pr-ci", "source": SOURCE,
           "runId": 38824151573, "tree": ci["tree"], "originalLogs": 3,
           "frontendPassed": 535, "macPassed": 923, "windowsPassed": 648,
           "companionPushClaimedPassed": False, "nativeMemoryGatePassed": False,
           "historicalQueryFailureCleared": False, "releaseAccepted": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
