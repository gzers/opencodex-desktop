"""Verify preparation, without mistaking it for failure reproduction or ETW."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
head = "bd2f6eb10d58115235e35d575507bb69f44666da"
collector = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
failed_sha = "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
failed = read("failed-test-binary-admission.json")
assert failed["associatedHead"] == head and failed["actualPrCheckout"] == "31e7748d9e82c79bef97fc1b73233f08bc1f1b3d"
assert failed["actualTree"] == failed["headTree"] == "753be1611f9e96e6077a4a164937b5bd6f42f0cc"
assert failed["entireTrackedTreeEqualsHead"] and failed["runId"] == 38537166560
assert failed["runConclusion"] == "failure" and failed["apiDigestAndZipCrcPassed"]
assert failed["runnerTestExecutableReceiptMatches"] and failed["testExecutable"]["sha256"] == failed_sha
assert not failed["failedAndSuccessfulTestExecutablesByteIdentical"]
assert failed["successfulPushExecutableSha256"] == "2709775efbce5a08a02827c61a74abb1adfb3170653898b8c614eebf9c367400"
assert failed["nodeVersion"] == "v24.21.0"
for key in ["nodePrewarmed", "ciRerunOrNewCandidateDispatched",
            "historicalFailureClosed", "schedulingEtwOrFailureRawTraceObtained"]:
    assert failed[key] is False
review = read("prior-byte-and-live-coverage-review.json")
assert review["pathKeyedBytesAndDigestsPassed"] and review["files"] == 35 and review["pathOrderIgnored"]
assert review["macCoverageReviewConfirmed"] and not review["originalManifestOrEvidenceModified"]
assert review["nodeLiveSamples"] == review["nodeThreadRecords"] == review["nodeWaitChainRecords"] == 0
assert review["allThreadAndWaitChainRecordsAreTestRoot"] and not review["oldCiFailureCleared"]
package = read("traceevent-package-provenance.json")
assert package["version"] == "3.1.21" and package["officialCatalogSha512Matches"] and package["zipCrcPassed"]
assert package["repositorySignaturePresent"] and not package["repositorySignatureCryptographicallyVerified"]
assert not package["installedGlobally"] and not package["packageScriptsExecuted"]
assert not package["etwSessionStarted"] and not package["testExeOrNodeStarted"]
assert len(package["decoderFiles"]) == 3 and len(package["sourceFiles"]) == 2
compiled = read("collector-preparation-and-guards.log")
assert compiled["status"] == "passed" and compiled["pureScopeChecks"] == 38
assert compiled["digestGuard"] == compiled["outputScopeGuard"] == compiled["ordinaryPrivilegeGuard"] == "passed"
assert not compiled["etwSessionStarted"] and not compiled["testOrNodeStarted"]
assert not compiled["recordingRootCreated"] and not compiled["diagnosticRecordExecutionTested"]
assert (root / "collector-prepare-initial.log").stat().st_size == 0
assert any(item.get("originalErrorInToolOutputButNotSavedStderr") for item in
           read("preparation-errors-and-limits.json")["observedPreparationErrors"])
plan = read("record-request-review-only.json")
assert plan["collectorSource"] == collector and plan["testSource"] == head
assert plan["testBinarySha256"] == failed_sha and plan["nodeVersion"] == "24.21.0"
assert plan["originalLibraryRuns"] == 1 and plan["testDeadlineSeconds"] == 120
assert [plan[key] for key in ["originalQuerySeconds", "originalCleanupSeconds",
                              "originalObserverSeconds", "originalWatchdogSeconds"]] == [2, 4, 5, 8]
assert plan["defaultConcurrency"] and plan["originalCapture"] and plan["requiresElevatedController"]
assert plan["kernelEventsTransientlyDecodedInMemory"] and plan["onlyOwnedFamilyRecordsPersisted"]
for key in ["prewarm", "desktopStarted", "selfElevationPerformed", "wholeMachineEtlWritten",
            "counterpartySchedulingIdsPersisted", "commandLinesOrSidsPersisted",
            "networkOrRegistryProviderEnabled", "unrelatedSystemSessionRestarted",
            "defenseOrGlobalPathChanged", "evidenceUploadOrExternalMessageSent",
            "failureOrRootCausePreclaimed"]:
    assert plan[key] is False
pr = read("draft-pr-created.json")
assert pr["number"] == 5 and pr["head"]["sha"] == collector and pr["base"]["sha"] == head
assert pr["draft"] and pr["state"] == "open" and not pr["merged"] and not pr["auto_merge"]
assert pr["body"] == (root / "pr-body.md").read_text(encoding="utf-8")
result = read("preparation-final-result.json")
assert result["goalStatusActuallyReadBack"] == "active" and result["goalCreatedAtPreserved"] == 1791774103
assert result["collectorSource"] == collector and len(result["changedFiles"]) == 3
assert not result["productOrOriginalFixtureSourceChanged"] and result["pureScopeChecks"] == 38
assert result["newTestOrNodeExecutions"] == result["newEtwSessionCount"] == 0
for key in ["goalBudgetModifiedByAgent", "goalMarkedComplete", "testExeDifferenceAttributedAsRootCause",
            "automaticCiStatusProved", "newCandidateOrCiManuallyDispatched",
            "draftPrMerged", "adminRecordingApproved", "adminRecordingStarted",
            "formalMacNativeHandoffIssued", "originalFailureReproduced", "originalFailureClosed",
            "fullVersionGatePromoted", "releaseAccepted"]:
    assert result[key] is False
assert result["localNativeCount"] == 0
census = read("preparation-census.json")
assert census["localDesktopProcesses"] == census["ownedOfficialNodeProcesses"] == []
assert not census["preparedRecordingRootExists"] and not census["etwRecordingApproved"]
authority = read("source-and-permission-basis.json")
assert authority["furtherPrivacyApprovalRequiredBeforeRecord"] and not authority["approvedInCurrentTurn"]
assert not authority["recordApprovedInvoked"] and not authority["realTimeRecordAndDrainExecuted"]
assert authority["preparedOnlyDoesNotProveEventCoverage"] and authority["fullVersionGoalActiveNotCompleted"]
files = sorted((path for path in root.rglob("*") if path.is_file() and path.name not in
                {"manifest.json", "archive-verification.json"}), key=lambda path: path.relative_to(root).as_posix())
manifest = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
             "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    expected = read("manifest.json")
    assert {item["path"]: item for item in expected} == {item["path"]: item for item in manifest}
report = {"archiveIntegrity": "passed", "files": len(files), "failedPrTestExeAdmitted": True,
          "collectorSource": collector, "pureScopeChecks": 38,
          "newTestRuns": 0, "etwRecorded": False, "adminApprovalPending": True,
          "goalStatus": "active", "nativeInstanceCount": 0,
          "originalFailureClosed": False, "fullVersionGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
