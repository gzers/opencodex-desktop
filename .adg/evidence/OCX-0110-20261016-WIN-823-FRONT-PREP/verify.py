"""Verify prepared inputs and refusal/collector checks; no native acceptance."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
preview = sys.argv[1:] == ["--preview"]
assert not sys.argv[1:] or preview


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
if not preview:
    manifest = read("manifest.json")
    assert set(files) == {row["path"] for row in manifest["files"]}
    for row in manifest["files"]:
        assert files[row["path"]].stat().st_size == row["bytes"]
        assert sha(files[row["path"]]) == row["sha256"]
contract = read("contract.json")
assert contract["candidateSource"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert contract["candidateExeSha256"] == "2614489ff3a7c62110fb74fd288432b37c3abeaa5cdaa3119551d66c97640e53"
assert contract["baselineSource"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert contract["baselineExeSha256"] == "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"
assert contract["order"] == ["baseline019", "candidate0110"] and contract["recoveryRunsAfterIdlePair"]
assert contract["settleSecondsPerVersion"] == 60 and contract["minimumCompletedCounterSpanSeconds"] == 600
assert contract["zeroReadOrFrameFailuresRequired"]
assert (contract["clientWidth"], contract["clientHeight"]) == (2360, 1520)
assert contract["budgetsUnchanged"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
assert contract["freshHumanWindowRequired"] and not contract["foregroundWindowGranted"]
assert sha(Path(contract["candidateExecutable"])) == contract["candidateExeSha256"]
assert sha(Path(contract["recoveryExecutable"])) == contract["candidateExeSha256"]
assert sha(Path(contract["baselineExecutable"])) == contract["baselineExeSha256"]
assert sha(Path(contract["candidateVerificationPath"])) == contract["candidateVerificationSha256"]
assert sha(Path(contract["candidateManifestPath"])) == contract["candidateManifestSha256"]
candidate = json.loads(Path(contract["candidateVerificationPath"]).read_text(encoding="utf-8"))
assert candidate["source"] == contract["candidateSource"] and candidate["status"] == "verified-exact-823-candidate"
assert not candidate["hostProductStarted"] and not candidate["fullVersionGatePassed"]
assert sha(ROOT / "retained-process-times.ps1") == contract["kernelTimesMethodSha256"]
method = read("controlled-times-result.original.json")
assert method["status"] == "passed" and method["scope"] == "two controlled PowerShell processes only"
assert not method["live"]["exited"] and method["live"]["exitFileTime"] is None
assert method["exited"]["exited"] and method["exitTimeMatchesFramework"]
assert method["wrongGenerationRejected"] and method["liveExitClaimRejected"]
assert int(method["next"]["creationFileTime"]) >= int(method["exited"]["exitFileTime"])
assert not method["forcedFixtureTermination"] and not method["productStarted"] and not method["nativeRecoveryGatePassed"]
for name in ("live", "exited", "next"):
    assert method[name]["sourceMethodSha256"] == contract["kernelTimesMethodSha256"]
    assert isinstance(method[name]["creationFileTime"], str) and method[name]["retainedHandle"]
checks = read("preparation-checks.json")
assert checks["status"] == "passed" and len(checks["parsedFiles"]) == 11
assert len(checks["rejectedWithoutHuman"]) == 4 and all(row["rejected"] for row in checks["rejectedWithoutHuman"])
for row in checks["parsedFiles"]:
    assert row["errors"] == 0 and sha(ROOT / row["path"]) == row["sha256"]
assert checks["pointerUnchanged"] and checks["aclUnchanged"]
assert not checks["actionRecordsWritten"] and not checks["productStarted"]
assert not checks["freshHumanWindowGranted"] and not checks["profileMoved"]
analysis = read("analyzer-checks.json")
assert analysis["status"] == "passed" and analysis["pythonSyntaxPassed"]
assert analysis["actualControlledFileTimeRoundTrips"] == 4
assert analysis["missingHumanAndMeasurementInputsRejected"] and not analysis["comparisonCreated"]
assert sha(ROOT / "analyze-pair.py") == analysis["analysisMethodSha256"]
origins = read("controller-methods.json")
for row in origins["methods"]:
    assert sha(ROOT / row["method"]) == row["finalPreparedSha256"]
template = read("human-window.template.json")
assert template["explicitHumanStart"] is None and template["authorizedAtUtc"] is None
assert template["expiresAtUtc"] is None and template["automaticContinuation"] is None
assert template["scope"] == contract["humanWindowScope"]
assert not (ROOT / "human-window.json").exists() and not (ROOT / "comparison.json").exists()
preparation = read("preparation-readback.json")
assert preparation["executableCopiedOnly"] and not preparation["productStarted"]
assert not preparation["hkcuModified"] and not preparation["aclModified"]
assert not preparation["humanReceiptCreated"]
protect = read("final-protection.json")
assert protect["nativeCount"] == 0 and protect["fixtureHashesUnchanged"]
assert protect["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protect["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not protect["nativeProductStarted"] and not protect["foregroundGranted"]
summary = {
    "status": "preview-prepared-controls-verified" if preview else "verified-prepared-823-foreground-controls",
    "files": len(files), "source": contract["candidateSource"], "exeSha256": contract["candidateExeSha256"],
    "parsedScripts": 11, "missingHumanRefusals": 4, "kernelTimeControlledProcesses": 2,
    "actualFileTimeRoundTrips": 4, "nativeProductStarted": False,
    "foregroundWindowGranted": False, "fullRecoveryControllerExecuted": False,
    "pairedPerformanceExecuted": False, "nativeRecoveryGatePassed": False,
    "fullPerformanceGatePassed": False, "historicalFailuresCleared": False, "releaseAccepted": False,
}
if not preview:
    (ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
