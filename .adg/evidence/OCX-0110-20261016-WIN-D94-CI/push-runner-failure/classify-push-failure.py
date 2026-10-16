"""Record the hosted-runner failure from GitHub's own terminal annotation."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
jobs = json.loads((ROOT / "ci-38824163566-jobs-failed.original.json").read_text(encoding="utf-8"))
job, = [j for j in jobs["jobs"] if j["name"] == "backend"]
annotations = json.loads((ROOT / "ci-38824163566-mac-annotations.original.json").read_text(encoding="utf-8"))
assert job["conclusion"] == "failure"
assert any("hosted runner lost communication" in a["message"] for a in annotations)
assert not any(s["status"] == "completed" and s["name"].startswith("Run cargo")
               for s in job["steps"])
result = {
    "source": "d94c215d869c0560717d79dc02881ce8858afe88",
    "runId": 38824163566, "jobId": job["id"], "conclusion": "failure",
    "githubFailureAnnotation": next(a["message"] for a in annotations
                                    if "hosted runner lost communication" in a["message"]),
    "lastObservedInProgressStep": "Run dtolnay/rust-toolchain@stable",
    "macFmtClippyTestsExecuted": False,
    "originalJobLogAvailable": False,
    "logReadError": "HTTP 404: The specified blob does not exist",
    "actualRootCauseOfRunnerLossEstablished": False,
    "failureRelabelledSuccess": False, "workflowRerunRequested": False,
    "independentPrSuccessRun": 38824151573,
    "candidateDispatchedOnceBeforeCompanionPushTerminal": 38827408691,
    "candidateConstructionAdmission": "whole successful PR with identical complete source tree",
    "fullVersionGatePassed": False, "releaseAccepted": False,
}
(ROOT / "push-failure-classification.json").write_text(
    json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
