"""Verify a completed CI and prepared bindings, without claiming a native run."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DIAG = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop")
SOURCE = "f2271f361b9da562b62c71afcdbeb15e61d183e2"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
manifest = read("manifest.json")
assert set(files) == {f["path"] for f in manifest["files"]}
for row in manifest["files"]:
    data = files[row["path"]].read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"]
contract, ci = read("contract.json"), read("ci-collection.json")
assert ci["runId"] == contract["collectorCiRunId"] == 38834762492
assert ci["source"] == contract["collectorSource"] == SOURCE
assert ci["status"] == "completed" and ci["conclusion"] == "success" and ci["wholeCiSucceeded"]
required = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert {j["name"] for j in ci["jobs"]} == required | {"build"}
assert all(j["status"] == "completed" and j["conclusion"] == "success"
           for j in ci["jobs"] if j["name"] in required)
build, = [j for j in ci["jobs"] if j["name"] == "build"]
assert build["status"] == "completed" and build["conclusion"] == "skipped"
assert read("ci-status.json")["status"] == "completed"
assert not ci["ciRerunRequested"] and not ci["productStarted"] and not ci["etwReplayRequested"]
assert not ci["fullPerformanceGatePassed"]
texts = {}
for job in ci["archivedLogs"]:
    assert job["logArchived"] and job["actualCheckout"] == SOURCE
    data = (ROOT / (job["name"] + ".original.log")).read_bytes()
    receipt = read(job["name"] + "-download.json")
    assert len(data) == job["bytes"] == receipt["bytes"]
    assert sha(data) == job["sha256"] == receipt["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    texts[job["name"]] = text
assert re.findall(r"Test Files\s+(\d+) passed", texts["frontend"]) == ["99"]
assert re.findall(r"Tests\s+(\d+) passed", texts["frontend"]) == ["535"]
tests = {}
for name, expected_passed, expected_ignored in (("backend", 923, 6), ("backend-windows", 648, 7)):
    suites = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", texts[name])
    assert len(suites) == 35
    passed, failed, ignored = [sum(int(row[i]) for row in suites) for i in range(3)]
    assert (passed, failed, ignored) == (expected_passed, 0, expected_ignored)
    tests[name] = {"suites": 35, "passed": passed, "ignored": ignored}
binding = read("source-binding.json")
assert binding["appsTreeIdentical"] and binding["bothCodeWorktreesClean"]
assert binding["sharedContentIdentical"] and binding["expectedBuildSkipCheckedInWorkflow"]
for row in binding["sharedFiles"].values():
    assert row["gitObjectsIdentical"] and row["workingContentMatchesGitExceptCrLf"]
for path, row in binding["methods"].items():
    data = subprocess.run(["git", "-c", "core.longpaths=true", "-C", str(DIAG),
                           "show", SOURCE + ":" + path], check=True, capture_output=True).stdout
    assert sha(data) == row["gitBlobSha256"]
    assert sha((DIAG / path).read_bytes()) == row["workingFileSha256"] == contract["methodWorkingFileSha256"][path]
assert contract["candidateSource"] == "d94c215d869c0560717d79dc02881ce8858afe88"
assert contract["candidateExeSha256"] == "b2284ba66a20c7ef11b61fea328f7aa960cd6530011d6e7a09fc78e782c5b224"
assert contract["baselineSource"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert contract["baselineExeSha256"] == "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"
for prefix in ("candidate", "baseline"):
    assert sha(Path(contract[prefix + "Executable"]).read_bytes()) == contract[prefix + "ExeSha256"]
assert sha(Path(contract["sidMethodPath"]).read_bytes()) == contract["sidMethodSha256"]
assert contract["order"] == ["baseline019", "candidate0110"]
assert contract["settleSecondsPerVersion"] == 60 and contract["minimumCompletedCounterSpanSeconds"] == 600
assert contract["zeroReadOrFrameFailuresRequired"]
assert contract["budgetsUnchanged"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
assert contract["preparationOnly"] and not contract["foregroundWindowGranted"]
assert not contract["priorSamplesUsedInNewComparison"] and not contract["productBuildRequested"]
assert not contract["etwReplayRequested"] and not contract["fullPerformanceGatePassed"]
assert not contract["historicalFailuresCleared"]
checks = read("preparation-checks.json")
assert checks["status"] == "passed" and len(checks["parsedFiles"]) == 7
assert all(row["errors"] == 0 for row in checks["parsedFiles"])
assert checks["sidCompanionSelfRead"]["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
assert not checks["productTokenGateClaimed"] and not checks["productStarted"]
template = read("human-window.template.json")
assert template["explicitHumanStart"] is None and template["authorizedAtUtc"] is None
assert template["expiresAtUtc"] is None and template["automaticContinuation"] is None
assert not (ROOT / "human-window.json").exists()
assert not (ROOT / "comparison.json").exists()
assert not list(ROOT.glob("*samples.original.jsonl"))
protection = read("final-protection.json")
assert protection["nativeCount"] == 0 and not protection["productStarted"]
assert protection["fixturePreferencesUnchanged"] and protection["taskUnchanged"] and protection["dailyExeUnchanged"]
assert protection["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protection["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not protection["nativeLongMeasurementExecuted"] and not protection["etwReplayExecuted"]
reference = read("reused-smoke-reference.json")
original = ROOT.parent / "OCX-0110-20261016-WIN-IDLE-COLLECTOR-FIX"
for path, expected in reference["files"].items():
    assert sha((original / path).read_bytes()) == expected
smoke = json.loads((original / "smoke-verification.json").read_text(encoding="utf-8-sig"))
assert smoke["source"] == SOURCE and smoke["checks"] == 55
assert smoke["artifactId"] == 12018632501
assert smoke["artifactSha256"] == "8eaff37475237e59400aa37a60708c5a483b387aa588c767c86c9d0681fd0bdd"
summary = {
    "status": "verified-complete-ci-and-prepared-bindings", "files": len(files),
    "collectorSource": SOURCE, "ciRunId": 38834762492, "wholeCiSucceeded": True,
    "frontendFiles": 99, "frontendTests": 535, "rust": tests, "reusedSmokeChecks": 55,
    "candidateSource": contract["candidateSource"], "candidateExeSha256": contract["candidateExeSha256"],
    "preparedControllerExecutedEndToEnd": False, "humanWindowGranted": False,
    "nativeProductStarted": False, "longMeasurementExecuted": False,
    "fullPerformanceGatePassed": False, "historicalFailuresCleared": False, "releaseAccepted": False,
}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
