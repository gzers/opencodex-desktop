"""Verify Windows workstream request and bounded read-only caption diagnosis; no gate closure."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
image = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
plan = read("windows-workstream-plan.json")
assert plan["originalVersionObjectivePreserved"] and plan["globalCompletionDependsOnOriginalFullEvidence"]
assert plan["currentSource"]["candidateSource"] == source and plan["currentSource"]["candidateExecutableSha256"] == image
assert plan["coordination"]["globalMaximumIsolatedNativeDesktopInstances"] == 1
assert not plan["coordination"]["windowsMayEditMacOwnedImplementationFilesNow"]
assert not plan["coordination"]["macExecutionObservedByThisThread"]
assert plan["currentAuthorizedMode"].startswith("background-only")
assert len(plan["nextWindowsActions"]) == 8 and not plan["releaseAuthorizedByThisRequest"]
objective = (root / "windows-objective.txt").read_text(encoding="utf-8")
for value in ["原合同", "原预算", "100/125/150", "30分钟", "0.1.9", "全局仅一个", "历史failed", "Mac依赖"]:
    assert value in objective
assert plan["appGoalObservation"]["observedStatus"] == "paused"
assert not plan["appGoalObservation"]["appGoalUiTextEdited"]
assert not plan["appGoalObservation"]["appGoalStatusResumedByTool"]
assert not plan["appGoalObservation"]["goalCompletedToCreateReplacement"]
capability = read("goal-edit-capability-readback.json")
assert capability["observedGoalStatus"] == "paused" and capability["humanContinuedWorkAuthority"]
assert capability["officialProxyReadOnlyAttempt"]["error"] == "Read-only proxy exited before response"
assert not capability["officialProxyReadOnlyAttempt"]["goalMutationIssued"]
assert not capability["officialProxyReadOnlyAttempt"]["daemonStartedRestartedOrStopped"]
assert not capability["goalMutationSucceeded"] and not capability["usageAccountingReset"]
caption = read("actual-readonly-caption.json")
assert caption["sourceCommit"] == source and caption["executableSha256"] == image
assert caption["applicationPid"] == 10904 and caption["hwnd"] == 6685560
assert caption["actualLiveHandleVerified"] and caption["actualImagePathVerified"] and caption["windowOwnerVerified"]
assert caption["titlebarStructBytes"] == 140 and caption["queryApi"] == ["WM_GETTITLEBARINFOEX", "WM_NCHITTEST"]
controls = caption["captionControls"]
assert [(item["index"], item["control"]) for item in controls] == [(2, "minimize"), (3, "maximize-or-restore"), (5, "close")]
assert [item["hitResult"] for item in controls] == [2, 9, 20]
assert [item["expectedHit"] for item in controls] == [8, 9, 20]
assert [item["expectedHitMatched"] for item in controls] == [False, True, True]
assert all(item["available"] and item["nonempty"] and not item["actualClickIssued"] for item in controls)
for item in controls:
    rect, center = item["screenBounds"], item["queriedScreenCenter"]
    assert center == {"x": (rect["left"] + rect["right"]) // 2, "y": (rect["top"] + rect["bottom"]) // 2}
assert caption["windowBefore"] == caption["windowAfter"] and caption["windowGeometryAndStateUnchangedDuringReadback"]
assert caption["windowBefore"]["dpi"] == 192 and caption["windowBefore"]["visible"]
assert not caption["windowBefore"]["maximized"] and not caption["windowBefore"]["minimized"]
assert not caption["windowBefore"]["foreground"] and caption["windowBefore"]["backdropAttribute"] == 1
assert caption["protectedDataHashesBefore"] == caption["protectedDataHashesAfter"]
for key in ["inputOrFocusIssued", "windowOrProcessMutated", "productCodeChanged",
            "captionRestoreClickAccepted", "pastClickFailureCauseIsolated",
            "fullNativeGateAccepted", "pairedPerformanceAccepted", "releaseAccepted"]:
    assert caption[key] is False
static = read("caption-source-review.json")
assert len(static["sourceWitnesses"]) == 3 and static["reviewIsStaticNotNativeInputProof"]
assert all(item["mainBlob"] == item["candidateBlob"] for item in static["sourceWitnesses"])
assert not static["releaseProductSourceChanged"] and not static["pastCaptionClickFailureCauseIsolated"]
life = read("lifecycle-after-readonly-diagnosis.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 10904 and life["desktopCount"] == 1
assert life["supervisorExecSession"] == 42581
assert {item["pid"] for item in life["actualProcessHandles"]} == {10904, 32492}
assert all(item["actualLiveHandle"] for item in life["actualProcessHandles"])
assert not life["automaticCheck"] and not life["applicationRestarted"] and not life["foregroundInputIssued"]
assert life["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
token = read("actual-live-application-token.json")
assert token["processId"] == 10904 and token["creationTimeUtc"] == "2026-10-13T14:57:31.5757733Z"
assert token["elevated"] is False and token["elevationType"] == 3
refs = read("remote-refs-before-workstream.json")["remoteRefs"]
assert any(line.startswith(source + "\t") and line.endswith("feature/0.1.10-update-center-focus") for line in refs)
assert any(line.startswith("eae85322ebca3b755c034db97cbebf2ea5bf71fc\t") and line.endswith("/main") for line in refs)
docs = read("primary-api-references.json")
assert docs["documentationRetrievedViaWeb"] and len(docs["methodDocumentation"]) == 3
assert docs["documentationDoesNotProvePriorClickCause"]
assert not docs["documentationCopiedIntoRepository"] and not docs["controlsOrSecuritySettingsMutated"]
method = (root / "read-caption.py").read_text(encoding="utf-8")
assert "send_query(0x033F" in method and "send_query(0x0084" in method
assert not any(value in method for value in ["ShowWindow", "SetForegroundWindow", "SendInput", "mouse_event",
                                           "keybd_event", "UIAutomation", "PostMessage", "SetWindowPos"])
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries),
    "scope": "user-requested Windows thread scope, unchanged global goal, actual read-only normal caption and live ordinary token; min-hit discrepancy retained",
    "requestedObjectiveTextPrepared": True, "windowsBackgroundWorkPerformed": True,
    "appGoalUiEdited": False, "appGoalStatusResumedByTool": False,
    "captionMaxAndCloseCentersHitExpectedCodes": True, "captionMinCenterHitExpectedCode": False,
    "captionRestoreActualClickAccepted": False, "pastCaptionClickFailureCauseIsolated": False,
    "foregroundInputIssued": False, "productCodeChanged": False,
    "fullNativeGateAccepted": False, "pairedPerformanceAccepted": False,
    "realFullCyclesAccepted": False, "formalTaskReceipt": False, "mainIntegrated": False, "releaseAccepted": False,
})
print(json.dumps({
    "archiveIntegrity": "passed", "files": len(entries), "windowsWorkstreamPrepared": True,
    "appGoalUiEdited": False, "readonlyCaptionObserved": True,
    "minHitDiscrepancyRetained": True, "nativeInputIssued": False, "fullNativeAccepted": False,
}))
