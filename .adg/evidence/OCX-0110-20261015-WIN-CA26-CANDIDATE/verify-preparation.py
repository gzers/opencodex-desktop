"""Verify only the frozen candidate preparation checkpoint, not the live build outcome."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
preflight = read("candidate-preflight.json")
payload = read("candidate-dispatch-payload.json")
attempt = read("candidate-dispatch-attempt.json")
request = read("candidate-dispatch-result.json")
binding = read("candidate-run-binding.json")
checkpoint = read("candidate-preparation-checkpoint.original.json")
result = read("candidate-preparation-result.json")
assert preflight["source"] == attempt["source"] == binding["source"] == checkpoint["source"] == sha
assert preflight["admissionRun"] == 38660621496 and preflight["priorRunIds"] == [38662786466]
assert payload["ref"] == "feature/0.1.10-windows-query-etw"
assert payload["inputs"]["purpose"] == "candidate"
assert attempt["singleCandidateRequest"] and not attempt["etwRerunRequested"]
assert not attempt["automaticRequestRetryAllowed"] and not attempt["releaseRequested"]
assert request["httpStatus"] == 204 and request["remoteRunMustBeReadBack"]
assert binding["runId"] == checkpoint["run"]["id"] == result["candidateRun"] == 38663871103
assert checkpoint["run"]["head_sha"] == sha and checkpoint["run"]["run_attempt"] == 1
assert checkpoint["run"]["status"] == result["remoteStatus"] == "in_progress"
assert checkpoint["run"]["conclusion"] is result["remoteConclusion"] is None
assert next(row for row in checkpoint["jobs"]["jobs"] if row["name"] == "package")["status"] == "in_progress"
assert next(row for row in checkpoint["jobs"]["jobs"] if row["name"] == "query-etw")["conclusion"] == "skipped"
assert result["nativeCount"] == 0 and result["nativeProcesses"] == []
for key in ("localApplicationStarted", "localInstallerExecuted", "localEtwStarted", "newDiagnosticRequested",
            "candidateAccepted", "fullVersionGatePassed"):
    assert not result[key]
assert result["candidateDispatchCount"] == 1 and result["dailyExeUnchanged"] and result["protectedSandboxUnchanged"]
assert result["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert result["missingFailureEvidenceStillOpen"] and result["automaticGoalContinuationIsNotForegroundAuthorization"]
subprocess.run([sys.executable, str(root / "audit-replay-context.py")], check=True, capture_output=True)
audit = read("replay-context-audit.json")
assert audit["testExecutableBytesSame"] and not audit["sameRuntimeEnvironmentProven"]
assert audit["testAndNodeLocationsChanged"] and not audit["perCaseEtwPidToQueryLabelMappingProven"]
assert not audit["targetFailureReproduced"] and not audit["causalClaimMade"]
names = [
    ".gitattributes", "candidate-api.py", "candidate-preflight.json", "candidate-dispatch-payload.json",
    "candidate-dispatch-attempt.json", "candidate-dispatch-result.json", "candidate-run-binding.json",
    "candidate-preparation-checkpoint.original.json", "candidate-preparation-result.json",
    "audit-replay-context.py", "replay-context-audit.json", "verify-preparation.py",
]
manifest = [
    {"path": name, "bytes": (root / name).stat().st_size,
     "sha256": hashlib.sha256((root / name).read_bytes()).hexdigest()}
    for name in names
]
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(manifest), "source": sha,
    "candidateRun": 38663871103, "candidateBuildInProgressAtCheckpoint": True,
    "candidateAccepted": False, "diagnosticRerunRequested": False,
    "historicalReplayContextEquivalenceNotProven": True,
    "perCaseFailureEtwCorrelationStillMissing": True, "localNativeInstances": 0,
    "fullVersionGatePassed": False,
}
if "--build" in sys.argv:
    assert not (root / "preparation-manifest.json").exists() and not (root / "preparation-verification.json").exists()
    (root / "preparation-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (root / "preparation-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read("preparation-manifest.json") == manifest
    assert read("preparation-verification.json") == report
print(json.dumps(report))
