"""Check complete same-source CI and preserve the actual event-order limitation."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


for row in read("manifest.json")["files"]:
    data = (ROOT / row["path"]).read_bytes()
    assert sha(data) == row["sha256"] and len(data) == row["bytes"]
admission = read("admission.json")
state = read("terminal.original.json")
assert admission["source"] == state["run"]["head_sha"] == "ce670ce76bb1c29f70989ee54838d5b31e97059c"
assert admission["runId"] == state["run"]["id"] == 38995008914
assert admission["status"] == state["run"]["status"] == "completed"
assert admission["conclusion"] == state["run"]["conclusion"] == "success"
assert admission["attempt"] == state["run"]["run_attempt"] == 1
assert state["run"]["event"] == "push"
assert state["jobs"]["total_count"] == len(state["jobs"]["jobs"]) == 6
required = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert {j["name"] for j in state["jobs"]["jobs"] if j["conclusion"] == "success"} == required
assert [j["name"] for j in state["jobs"]["jobs"] if j["conclusion"] == "skipped"] == ["build"]
counts = {}
for row in admission["requiredJobLogs"]:
    data = (ROOT / row["path"]).read_bytes()
    assert sha(data) == row["sha256"] and len(data) == row["bytes"]
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == row["actualCheckout"] == admission["source"]
    results = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    if results:
        assert all(int(r[1]) == 0 for r in results)
        counts[row["job"]] = [len(results), sum(int(r[0]) for r in results), sum(int(r[2]) for r in results)]
    if row["job"] == "frontend":
        assert re.search(r"Test Files\s+100 passed \(100\)", text)
        assert re.search(r"\bTests\s+540 passed \(540\)", text)
assert counts == {"backend": [35, 924, 6], "backend-windows": [35, 651, 7]}
events = read("real-context-event-order.json")
assert events["detached"]["targetEvents"] == 1 and events["detached"]["documentCaptured"] == 0
assert events["attached"]["documentCaptured"] == 1
assert events["childExit"]["code"] == 0 and events["childExit"]["signal"] is None
assert not events["originalNormalDetachedDestroyObservedToTriggerDocumentFallback"]
assert events["guardRegressionUsesSynchronousSimulatedEvent"]
assert not events["rssBenefitMeasured"] and not events["nativeProductStarted"]
handle = read("candidate-handle.json")
assert handle["source"] == admission["source"] and handle["runId"] == 38996189227
assert handle["attempt"] == 1 and handle["purpose"] == "candidate"
request = read("candidate-dispatch-request.json")
assert request["ref"] == "feature/0.1.10-windows-webgl-lifetime"
assert request["inputs"]["purpose"] == "candidate" and request["inputs"]["admission_run"] == "38995008914"
assert read("candidate-dispatch-response.json")["status"] == 204
assert sha((ROOT.parent / "manifest.json").read_bytes()) == read("checkpoint-binding.json")["originalManifestSha256"]
print(json.dumps({"source": admission["source"], "wholeCiVerified": True, "frontendTests": 540,
                  "rust": counts, "candidateRunId": handle["runId"],
                  "nativeTeardownFallbackProven": False, "rssBenefitMeasured": False,
                  "rssFailureCleared": False, "candidateExeAdmitted": False}))
