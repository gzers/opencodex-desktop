"""Verify the observed alias identities, rollback, scoped moves, and refusals."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


binding = read("preparation-binding.json")
old_plan = read("first-attempt/profile-isolation-plan.json")
old_failure = read("first-attempt/profile-isolation-failure.json")
restored = read("first-attempt/partial-restoration-result.json")
plan, result = read("prepared/profile-isolation-plan.json"), read("prepared/profile-isolation-result.json")
guards, final = read("prepared/guard-validation.original.json"), read("prepared/final-preparation-readback.json")
assert len(old_plan["profiles"]) == len(restored["rootsAfterRestoration"]) == len(plan["aliases"]) == 4
assert len(old_failure["completedMoves"]) == len(restored["restoredMoves"]) == 2
assert restored["uniqueNativeDirectoryCount"] == len(plan["physicalProfiles"]) == 2
assert not old_failure["productStarted"] and not old_failure["dataDeleted"]
for item in restored["restoredMoves"]:
    assert item["before"]["fileId"] == item["after"]["fileId"]
    assert item["before"]["creationFileTime"] == item["after"]["creationFileTime"]
restored_ids = {row["path"]: (row["fileId"], row["creationFileTime"]) for row in restored["rootsAfterRestoration"]}
assert len(set(restored_ids.values())) == 2
assert {row["path"]: (row["identity"]["fileId"], row["identity"]["creationFileTime"])
        for row in plan["aliases"]} == restored_ids
assert len({group["fileId"] for group in plan["physicalProfiles"]}) == 2
assert {p for g in plan["physicalProfiles"] for p in g["logicalPaths"]} == set(restored_ids)
assert sum(len(g["logicalPaths"]) for g in plan["physicalProfiles"]) == 4
for move in result["profiles"]:
    assert move["before"]["fileId"] == move["after"]["fileId"]
    assert move["before"]["creationFileTime"] == move["after"]["creationFileTime"]
    assert move["allLogicalPathsAbsent"] and move["contentsMovedWithoutDeletion"]
assert result["physicalDirectoryMoves"] == 2 and result["logicalPathCount"] == 4
assert result["allFourOriginalDirectoriesAbsent"] and result["nativeDirectoryIdsPreserved"]
assert result["startupPointerUnchangedExact"] and result["nativeCount"] == 0
assert guards["emptyPlatformRootRejected"] and guards["emptyProbeRemovedWithoutRecursion"]
assert {row["method"] for row in guards["noHumanWindowRefusals"] if row["rejected"]} == {
    "prepare-recovery-v2.ps1", "launch-recovery-v2.ps1"}
assert final["ready"]["nativeCount"] == 0 and final["ready"]["fourPlatformRootsAbsent"]
assert final["ready"]["physicalDirectoryCount"] == 2 and final["ready"]["allSavedNativeDirectoryIdentitiesMatch"]
assert final["ready"]["preparationPlanSha256"] == digest(ROOT / "prepared/profile-isolation-plan.json")
assert final["ready"]["preparationResultSha256"] == digest(ROOT / "prepared/profile-isolation-result.json")
assert {row["fileId"] for row in final["savedNativeDirectories"]} == {g["fileId"] for g in plan["physicalProfiles"]}
token = final["actualObservationHarnessToken"]
assert token["status"] == "observed" and not token["elevated"] and token["elevationType"] == 3
assert final["productSource"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert final["diagnosticSource"] == "768e231811a2363b32f5fa67c8e74cbc31d751e1"
assert final["productTrackedChanges"] == final["diagnosticTrackedChanges"] == 0
assert final["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert final["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert final["pointerSha256"] == plan["startupPointerSha256"] == old_plan["startupPointerSha256"]
assert not final["productStarted"] and not final["installerExecuted"] and not final["foregroundWindowGranted"]
assert not binding["profileContentsArchived"] and not binding["profileContentsDeleted"]
assert binding["firstPartialRestorationExecutedAndNativeIdsVerified"] and not binding["fullProfileRestorationHelperExecuted"]
for item in binding["originalCopies"]:
    path = ROOT / item["path"]
    assert path.stat().st_size == item["bytes"] and digest(path) == item["sha256"]
for name, expected in binding["frozenControllerSha256"].items():
    assert digest(PROJECT / binding["frozenRecoveryControllerDirectory"] / name) == expected
inspection = read("source-inspection.json")
for identity, expected in inspection["sourceBlobSha256"].items():
    assert hashlib.sha256(subprocess.check_output(["git", "show", identity], cwd=PROJECT)).hexdigest() == expected
assert not inspection["rssRootCauseEstablished"] and not inspection["formalPairRepeated"]
raw = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN"
network = json.loads((raw / "network-continuation-checkpoint.json").read_text(encoding="utf-8-sig"))
assert not network["fullOriginalJobLogsVerified"] and not network["recordingDispatched"]
assert not network["originalLibraryOrPinnedNodeExecuted"] and not network["globalNetworkSettingsChanged"]
checkpoint = json.loads((raw / "checkpoint-manifest.json").read_text(encoding="utf-8"))
for item in checkpoint["files"]:
    path = raw / item["path"]
    assert path.stat().st_size == item["bytes"] and digest(path) == item["sha256"]
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        path = ROOT / item["path"]
        assert path.stat().st_size == item["bytes"] and digest(path) == item["sha256"]
print(json.dumps({
    "status": "verified-native-directory-preconditions-only", "runtimeOriginals": len(binding["originalCopies"]),
    "logicalProfilePaths": 4, "physicalProfileDirectories": 2, "nativeIdsPreserved": True,
    "emptyRootRejected": True, "noHumanWindowRefusals": 2, "nativeCountAtFinalReadback": 0,
    "originalPointerPreserved": True, "profileContentsDeleted": False, "productStarted": False,
    "fullRecoveryGatePassed": False, "etwCiLogsAdmitted": False, "etwRecordingDispatched": False,
    "rssRootCauseEstablished": False, "performanceBudgetChanged": False,
    "historicalFailuresCleared": False, "releaseAccepted": False,
}, indent=2))
