"""Exact integrated CI/artifact identity; no local installation or full acceptance."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
source = "4852820b89f7f9df7d54bc623e94f3f55d884023"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
for run_id in [38242543274, 38242585130]:
    run = read(f"run-{run_id}-final.json")
    assert run["id"] == run_id and run["head_sha"] == source and run["head_branch"] == "main"
    assert run["status"] == "completed" and run["conclusion"] == "success"
    jobs = read(f"jobs-{run_id}-final.json")["jobs"]
    assert all(job["status"] == "completed" and job["conclusion"] == "success" for job in jobs)
    if run_id == 38242543274:
        assert {job["name"] for job in jobs} == {
            "windows-smoke-evidence", "backend-windows", "release-tools", "frontend", "backend", "build",
        }
    else:
        assert {job["name"] for job in jobs} == {"package"}
assert read("candidate-final.json")["id"] == 38242585130
artifacts = {item["id"]: item for item in read("candidate-artifacts.json")["artifacts"]}
downloads = read("download-verification.json")
assert {item["id"] for item in downloads} == {11744123025, 11744028167}
for item in downloads:
    artifact = artifacts[item["id"]]
    assert not artifact["expired"] and artifact["workflow_run"]["head_sha"] == source
    assert artifact["workflow_run"]["id"] == 38242585130
    assert artifact["digest"] == item["expectedDigest"] == "sha256:" + item["sha256"]
    assert item["matched"] and item["sourceCommit"] == source
installer = read("ci-installer-verification.json")
assert installer["sourceCommit"] == source and installer["result"] == "pass"
assert not installer["codeSigned"] and not installer["releaseAccepted"]
identity = read("candidate-identity-verification.json")
assert identity["sourceCommit"] == source and identity["productVersion"] == "0.1.10"
assert identity["matchesCiInstalledExe"] and identity["matchesCiMsi"] and identity["matchesCiNsis"]
assert identity["twoArtifactReceiptsIdentical"] and identity["lockMatched"]
assert identity["localCargoLockSha256"] == identity["ciCandidateCargoLockSha256"]
assert hashlib.sha256((root / "ci-candidate/Cargo.lock").read_bytes()).hexdigest() == identity["localCargoLockSha256"]
msi = read("msi-identity.json")
assert msi["databaseMode"] == "read-only" and not msi["installerExecuted"]
assert msi["packageSha256"] == identity["msiSha256"]
assert msi["properties"]["ProductVersion"] == "0.1.10"
assert identity["sharesDailyUpgradeCode"] and msi["properties"]["UpgradeCode"] == "{30FC0B19-5C02-5849-A234-D548E7AFC0DE}"
exe = next(item for item in msi["extractedExecutables"] if item["name"] == "opencodex-desktop.exe")
assert exe["sha256"] == identity["executableSha256"] == "7526561ccfd6f79b5948fc27e16d232258449f13583b93ab9907a2a8c207f875"
for item in installer["installers"]:
    kind = item["kind"]
    assert kind in {"msi", "nsis"} and item["result"] == "pass" and item["firstScreen"] == "pass"
    assert item["installExitCode"] == item["uninstallExitCode"] == 0
    assert item["uninstallRemovedExecutable"]
    startup = read(f"ci-candidate/{kind}/startup.json")
    webview = read(f"ci-candidate/{kind}/webview.json")
    assert startup["result"] == webview["result"] == "pass"
    assert startup["expectedCommit"] == source and startup["sourceCommitMatched"]
    assert startup["sha256"] == item["installedExecutableSha256"] == item["expectedInstalledExecutableSha256"]
    assert startup["applicationTokenVerified"] and not startup["applicationModeMatched"]
    assert startup["mode"] == "any" and startup["applicationToken"]["elevated"]
    assert startup["applicationToken"]["elevationType"] == 1
    assert webview["mainUiMounted"] and webview["about"]["version"] == "0.1.10"
local = read("local-current-state.json")
assert local["mainSource"] == source and local["mainCodeWorktreeClean"]
assert all(item["identical"] and item["codeBlob"] == item["docsBlob"] for item in local["sharedFiles"])
assert local["retainedApplicationPid"] == 38924 and local["retainedApplicationCount"] == 1
assert local["retainedApplicationSource"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert local["originalApplicationAndMonitorLiveHandlesVerified"] and local["scheduleStillMatchesBaseline"]
assert local["currentUpdatePreferences"] == {"channel": "stable", "automaticCheck": True}
assert local["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not any(local[key] for key in [
    "newCandidateApplicationLaunched", "localInstallerExecuted", "scriptsExecuted",
    "scriptsSecondConfirmationReceived", "realDataMigrated", "localInstallationAccepted",
    "pairedPerformanceAccepted", "realCyclesAccepted", "fullNativeGateAccepted",
    "formalTaskReceipt", "releaseAccepted",
])
assert not local["task182Present"]
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".cab"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "exact main CI and Windows candidate provenance, read-only local extraction and unchanged retained old cycle lifecycle",
    "sourceCommit": source, "mainCi": "success, all six jobs", "candidateCi": "success",
    "localNewCandidateNativeAccepted": False, "localInstallerAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "mainCi": "success", "candidateCi": "success", "files": len(entries)}))
