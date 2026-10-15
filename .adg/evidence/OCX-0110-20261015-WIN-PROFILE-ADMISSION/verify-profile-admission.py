"""Verify the frozen helper/negative-control checkpoint, never accept a real Profile or full CI."""
import hashlib
import json
import sys
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
method_sha = "b75bbf4202c51fd49db63a7694deed97ffba9fd6e36d30097efb4e51c08011fb"
contract = read("profile-contract.original.json")
assert contract["result"] == "pass" and contract["checksPassed"] == 31
assert len(contract["checks"]) == 31 and all(row["result"] == "pass" for row in contract["checks"])
assert contract == read("profile-contract.stdout.original.json")
assert contract["methodSha256"] == method_sha
assert not contract["nativeProfileProbeExecuted"] and not contract["productStarted"]
assert not contract["syntheticAccountCreated"] and not contract["fullVersionGatePassed"]
tokens = read("process-token.original.json")
assert tokens["result"] == "pass" and len(tokens["checks"]) == 9
assert all(row["result"] == "pass" for row in tokens["checks"])
assert tokens["sourceCommit"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
native = read("native-current-account-rejection.original.json")
assert native == read("native-probe.stdout.original.json")
assert not (root / "native-probe.stderr.original.txt").read_text(encoding="utf-8-sig").strip()
assert native["sourceCommit"] == source and native["status"] == "rejected" and not native["admitted"]
assert native["methodSha256"] == method_sha and native["methodMatchesCurrentFile"]
assert native["dailySidMatched"] and native["expectedSidMatched"]
assert not native["elevated"] and native["elevationType"] == 3
assert native["localAccountConfirmed"] and not native["threadImpersonating"]
assert not native["existingDirectoriesVerified"] and native["desktopCount"] == 0
assert not native["proposedTestAccountExists"]
assert {"daily-account", "ordinary-nonadministrator-account-required",
        "profile-directories-unverified", "known-profile-outside-fixture",
        "known-roaming-outside-fixture", "known-local-outside-fixture"} == set(native["reasons"])
assert {row["folder"] for row in native["knownFolders"]} == {"profile", "roaming", "local"}
for row in native["knownFolders"]:
    assert len(row["pathSha256"]) == 64 and set(row) == {"folder", "pathSha256"}
for key in ("rawUserPathsOrSidsPublished", "accountOrProfileCreated", "knownFolderBindingsChanged",
            "environmentChanged", "permissionsChanged", "productStarted", "inputInjected",
            "installerIsolationProved", "profileAdmissionIsLaunchPermission", "fullVersionGatePassed"):
    assert native[key] is False
provenance = read("method-provenance.json")
assert provenance["source"] == source and provenance["codeWorktreeClean"]
assert provenance["appsAndOriginalQueriesAndCollectorUnchanged"]
assert sorted(provenance["changedPaths"]) == sorted([
    ".github/workflows/ci.yml", "test/smoke/windows-acceptance-profile.ps1",
    "test/smoke/windows-acceptance-profile.test.ps1"])
assert next(row["sha256"] for row in provenance["files"]
            if row["path"] == "test/smoke/windows-acceptance-profile.ps1") == method_sha
before, after = read("pr-before-update.original.json"), read("pr-after-update.original.json")
request = read("pr-update-request.json")
assert all(after[key] == value for key, value in request.items())
assert before["head"]["sha"] == after["head"]["sha"] == source
assert before["base"]["sha"] == after["base"]["sha"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
assert after["state"] == "open" and after["draft"] and not after["merged"] and after["auto_merge"] is None
assert read("pr-update-response.original.json")["body"] == request["body"]
assert not read("pr-update-attempt.json")["automaticRepeatAllowed"]
observations = sorted(root.glob("ci-observation-*.json"))
assert observations
latest = None
for path in observations:
    snapshot = read(path.name)
    assert snapshot["source"] == source and not snapshot["hostProductStarted"]
    assert not snapshot["diagnosticRerunRequested"]
    assert {row["run"]["id"] for row in snapshot["observations"]} == {38670771363, 38670774817}
    for row in snapshot["observations"]:
        assert row["run"]["head_sha"] == source and row["run"]["run_attempt"] == 1
        assert row["run"]["status"] == "in_progress" and row["run"]["conclusion"] is None
    latest = snapshot
for row in latest["observations"]:
    jobs = {job["name"]: job for job in row["jobs"]["jobs"]}
    for name in ("windows-smoke-evidence", "frontend", "release-tools", "backend"):
        assert jobs[name]["status"] == "completed" and jobs[name]["conclusion"] == "success"
    assert jobs["backend-windows"]["status"] == "in_progress"
    assert jobs["backend-windows"]["conclusion"] is None
post, zero = read("protection-postcheck.json"), read("native-postcheck.json")
assert post["protectedFileHashesMatched"] == 7
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not post["hostProductStarted"] and not post["accountOrProfileCreated"]
assert not post["permissionsChanged"] and not post["installerExecuted"] and not post["fullVersionGatePassed"]
assert zero["nativeCount"] == zero["newNativeExecutions"] == 0 and zero["nativeProcesses"] == []
assert not zero["installerExecuted"] and not zero["inputInjected"]

excluded = {"manifest.json", "archive-verification.json"}
paths = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path)}
           for path in paths]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists() and not (root / "archive-verification.json").exists()
    (root / "manifest.json").write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == entries
for row in entries:
    relative = PurePosixPath(row["path"])
    assert not relative.is_absolute() and ".." not in relative.parts
    assert not (root / row["path"]).is_symlink()
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "source": source,
    "scope": "Profile helper contracts and current daily-account rejection only",
    "methodSha256": method_sha, "offlineProfileChecks": 31, "existingTokenChecks": 9,
    "actualDailyAccountRejected": True, "actualTestProfilePositiveAdmissionVerified": False,
    "testAccountExists": False, "nativeApplicationsAtCheckpoint": 0, "newNativeProductExecutions": 0,
    "sourceProvenanceFiles": 3, "appsAndOriginalQueryAndCollectorUnchanged": True,
    "prMetadataReadBackVerified": True, "prDraftUnmerged": True,
    "observedCiRuns": [38670771363, 38670774817],
    "ciStatusAtCheckpoint": "both in progress; four jobs successful, Windows backend still running",
    "ciLogAndArtifactProvenanceVerified": False, "fullCiSuccessVerified": False,
    "officialBaselineLaunched": False, "pairedPerformanceVerified": False,
    "installerIsolationVerified": False, "realCycleVerified": False, "fullVersionGatePassed": False,
}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read("archive-verification.json")
print(json.dumps(report))
