"""Check real installer outcomes and seeded-file limits without native UI claims."""
import hashlib
import json
from pathlib import Path
import sys
from datetime import datetime

root = Path(__file__).resolve().parent
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
source = "3c51367f1be89abf66f69805d29b2a660a570a2c"
candidate = "43142bdbcc7a0b9f1325103febae3f44dce407ab"
terminal = read(root / "retention-terminal.json")
assert terminal["harnessSource"] == terminal["run"]["head_sha"] == source
assert terminal["run"]["status"] == "completed" and terminal["run"]["run_attempt"] == 1
for name in ("package", "query-etw"):
    assert next(job for job in terminal["jobs"]["jobs"] if job["name"] == name)["conclusion"] == "skipped"
results = []
for kind in ("msi", "nsis"):
    directory = root / kind
    download = read(directory / "download-verification.json")
    assert download["harnessSource"] == source and download["kind"] == kind
    for row in download["entries"]:
        data = (directory / "original" / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
    paths = list((directory / "original").rglob("admission.json"))
    assert len(paths) == 1
    admission = read(paths[0])
    assert admission["harnessSource"] == source and admission["candidateSource"] == candidate
    assert admission["kind"] == kind and not admission["sameHarnessAndCandidateSourceClaimed"]
    assert admission["baselineVersion"] == "0.1.9" and admission["candidateVersion"] == "0.1.10"
    assert not admission["installerExecutedDuringAdmission"] and not admission["applicationLaunched"]
    outcomes = list((directory / "original").rglob("retention-result.json"))
    assert len(outcomes) == 1, "Read admission/setup failure originals; do not assume installation happened."
    outcome = read(outcomes[0])
    assert outcome["harnessSource"] == source and outcome["candidateSource"] == candidate and outcome["kind"] == kind
    assert not outcome["productLaunched"] and not outcome["explicitDataPurgeTested"]
    assert outcome["defaultDataFixtureSeeded"] and outcome["customRootIsSeededSiblingNotApplicationBinding"]
    assert not outcome["hostMachineAcceptanceClaimed"] and not outcome["fullVersionGatePassed"]
    successful = outcome["result"] == "pass"
    preserved_phases = []
    installer_steps = []
    if successful:
        assert not outcome["installerStateMayRemain"]
        assert outcome["baselineImage"]["version"] == "0.1.9"
        assert outcome["candidateImage"]["version"] == outcome["reinstalledImage"]["version"] == "0.1.10"
        assert outcome["candidateImage"]["sha256"] == outcome["reinstalledImage"]["sha256"] == admission["candidateInstalledExeSha256"]
        if kind == "msi":
            assert outcome["baselineImage"]["sha256"] == admission["baselineMsiExeSha256"]
        for step in outcome["steps"]:
            if "exitCode" in step:
                assert step["exitCode"] == 0
                installer_steps.append(step["name"])
            else:
                assert step["allBytesUnchanged"] and len(step["files"]) == 6
                if preserved_phases:
                    assert step["files"] == preserved_phases[0]["files"]
                preserved_phases.append(step)
        assert installer_steps == ["install-official-019", "upgrade-to-431", "default-uninstall",
                                   "reinstall-431", "final-default-uninstall"]
        assert [step["name"] for step in preserved_phases] == [
            "before-upgrade", "after-upgrade", "after-default-uninstall", "after-reinstall", "after-final-uninstall"]
        times = [datetime.fromisoformat(step["atUtc"].replace("Z", "+00:00")) for step in outcome["steps"]]
        assert times == sorted(times)
        upgrade_code_proven = False
        if kind == "msi":
            log_path = next((directory / "original").rglob("upgrade-to-431.log"))
            log = log_path.read_text(encoding="utf-16")
            assert "WIX_UPGRADE_DETECTED" in log and "{7C063778-E6A6-4D0E-9770-4C4349101866}" in log
            assert "RemoveExistingProducts. Return value 1." in log
            assert "Installation completed successfully." in log
            upgrade_code_proven = True
    results.append({
        "kind": kind, "result": outcome["result"], "baselineVersion": outcome.get("baselineImage", {}).get("version"),
        "candidateVersion": outcome.get("candidateImage", {}).get("version"),
        "candidateInstalledSha256": outcome.get("candidateImage", {}).get("sha256"),
        "installerSteps": installer_steps, "seededFilesVerifiedPerPhase": 6 if successful else None,
        "byteEqualityVerificationPhases": len(preserved_phases),
        "errorType": outcome.get("errorType"), "error": outcome.get("error"),
        "applicationLaunched": False, "customRootResolutionAccepted": False,
        "physicalHostOrdinaryPermissionAccepted": False, "explicitPurgeAccepted": False,
        "msiLegacyProductDetectedAndRemovalLogged": upgrade_code_proven if successful else False,
    })
report = {
    "harnessSource": source, "fixedCandidateSource": candidate, "runId": terminal["run"]["id"],
    "runConclusion": terminal["run"]["conclusion"], "installerResults": results,
    "bothInstallerUpgradeAndDefaultUninstallFixtureRetentionPassed": all(row["result"] == "pass" for row in results),
    "historicalQueryFailureCleared": False, "nativeRootAndPermissionsFullMatrixPassed": False,
    "pairedPerformancePassed": False, "fullVersionGatePassed": False, "releaseRequested": False,
}
target = root / "retention-analysis.json"
if target.exists():
    assert read(target) == report
else:
    target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
