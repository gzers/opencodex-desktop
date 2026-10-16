"""Verify only the one build admission; this does not admit an artifact or EXE."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CI = ROOT.parent / "OCX-0110-20261016-WIN-RECOVERY-HANDOFF"
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = read("admission-manifest.json")
expected = {"dispatch.py", "dispatch-request.json", "dispatch-attempt.json",
            "dispatch-result.json", "candidate-handle.json",
            "candidate-dispatch-readback.original.json", "verify-admission.py"}
assert {row["path"] for row in manifest["files"]} == expected
for row in manifest["files"]:
    path = ROOT / row["path"]
    assert path.stat().st_size == row["bytes"] and sha(path) == row["sha256"]
verification = json.loads((CI / "verification.json").read_text(encoding="utf-8"))
assert verification["source"] == SOURCE and verification["wholeCiSucceeded"]
attempt, request, result, handle = [read(n) for n in (
    "dispatch-attempt.json", "dispatch-request.json", "dispatch-result.json", "candidate-handle.json")]
assert attempt["source"] == result["source"] == handle["source"] == SOURCE
assert attempt["ciRunId"] == 38838969102
assert attempt["ciVerificationSha256"] == sha(CI / "verification.json")
assert attempt["ciManifestSha256"] == sha(CI / "manifest.json")
assert not attempt["automaticMutationRetry"] and not attempt["releaseRequested"]
assert request == {"ref": "feature/0.1.10-windows-query-etw", "inputs": {"purpose": "candidate"}}
assert result["httpStatus"] == 204 and not result["automaticMutationRetryAllowed"]
assert handle["runId"] == 38841117656 and handle["runAttempt"] == 1 and handle["maximumDispatches"] == 1
assert handle["purpose"] == "candidate" and not handle["releaseRequested"] and not handle["nativeRecoveryGatePassed"]
run = read("candidate-dispatch-readback.original.json")
assert run["id"] == handle["runId"] and run["head_sha"] == SOURCE
assert run["event"] == "workflow_dispatch" and run["run_attempt"] == 1
summary = {
    "status": "verified-single-candidate-admission", "source": SOURCE, "runId": run["id"],
    "ciRunId": attempt["ciRunId"], "admissionFiles": len(expected), "maximumDispatches": 1,
    "artifactAdmitted": False, "executableAdmitted": False, "nativeProductStartedLocally": False,
    "nativeRecoveryGatePassed": False, "fullPerformanceGatePassed": False, "releaseAccepted": False,
}
(ROOT / "admission-verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
