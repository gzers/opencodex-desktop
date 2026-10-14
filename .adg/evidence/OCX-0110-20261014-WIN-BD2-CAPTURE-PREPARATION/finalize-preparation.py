"""Bind preparatory checks and protect the distinction from an executed trace."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
docs = root.parents[2]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout
assert git("rev-parse", "HEAD").decode().strip() == source
changed = git("diff", "--name-only", base, source).decode().splitlines()
assert sorted(changed) == sorted([
    "test/smoke/windows-owned-etw.cs", "test/smoke/windows-owned-etw.ps1",
    "test/smoke/windows-owned-etw.test.ps1",
])
methods = []
for path in changed:
    raw = git("show", source + ":" + path)
    methods.append({"path": path, "gitBlob": git("rev-parse", source + ":" + path).decode().strip(),
                    "normalizedGitBytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
compiled = read("collector-preparation-and-guards.log")
assert compiled["status"] == "passed" and compiled["pureScopeChecks"] == 38
assert compiled["digestGuard"] == compiled["outputScopeGuard"] == compiled["ordinaryPrivilegeGuard"] == "passed"
assert not compiled["etwSessionStarted"] and not compiled["testOrNodeStarted"] and not compiled["recordingRootCreated"]
failed = read("failed-test-binary-admission.json")
package = read("traceevent-package-provenance.json")
for item in package["decoderFiles"]:
    raw = (Path(package["runtime"]) / item["file"]).read_bytes()
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"]
review = read("prior-byte-and-live-coverage-review.json")
assert review["nodeLiveSamples"] == review["nodeThreadRecords"] == review["nodeWaitChainRecords"] == 0
census = read("preparation-census.json")
assert census["localDesktopProcesses"] == [] and census["ownedOfficialNodeProcesses"] == []
assert not census["preparedRecordingRootExists"] and not census["etwRecordingApproved"]
pr = read("draft-pr-created.json")
assert pr["number"] == 5 and pr["draft"] and pr["state"] == "open" and not pr["merged"] and not pr["auto_merge"]
assert pr["head"]["sha"] == source and pr["base"]["sha"] == base
record = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "goalStatusActuallyReadBack": "active", "goalCreatedAtPreserved": 1791774103,
    "goalBudgetModifiedByAgent": False, "goalMarkedComplete": False,
    "priorGoalTurnClassifiedAs": "progress: Windows success-comparison evidence 019dd6c was produced and pushed",
    "collectorSource": source, "collectorBase": base, "changedFiles": methods,
    "productOrOriginalFixtureSourceChanged": False,
    "pureScopeChecks": 38, "ordinaryPrivilegeAndDigestAndScopeGuardsPassed": True,
    "failedPrRun": failed["runId"], "failedPrActualCheckout": failed["actualPrCheckout"],
    "failedPrTreeEqualsBd2": failed["entireTrackedTreeEqualsHead"],
    "failedPrTestExeSha256": failed["testExecutable"]["sha256"],
    "successPushTestExeSha256": failed["successfulPushExecutableSha256"],
    "testExeBytesDifferAcrossCiRuns": not failed["failedAndSuccessfulTestExecutablesByteIdentical"],
    "testExeDifferenceAttributedAsRootCause": False,
    "newTestOrNodeExecutions": 0, "newEtwSessionCount": 0,
    "oldLiveNodeCoverageCorrectionConfirmed": True,
    "preparedMicrosoftDecoderSha512Verified": package["officialCatalogSha512Matches"],
    "decoderInstalledGlobally": False, "automaticCiStatusProved": read("helper-ci-observation.json")["status"] == "observed",
    "newCandidateOrCiManuallyDispatched": False, "draftPr": 5, "draftPrMerged": False,
    "adminRecordingApproved": False, "adminRecordingStarted": False,
    "formalMacNativeHandoffIssued": False, "localNativeCount": 0,
    "originalFailureReproduced": False, "originalFailureClosed": False,
    "fullVersionGatePromoted": False, "releaseAccepted": False,
}
(root / "preparation-final-result.json").write_text(
    json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"preparation": "compiled-and-guards-passed", "collectorSource": source,
                  "originalFailedBinaryAdmitted": True, "newRecordingStarted": False,
                  "goalActive": True, "adminApprovalPending": True}))
