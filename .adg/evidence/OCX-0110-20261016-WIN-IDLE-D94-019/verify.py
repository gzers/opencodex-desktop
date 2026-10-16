"""Verify this pair's originals and conclusions independently from CI results."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(files) == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = files[entry["path"]].read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
before = (ROOT / "comparison.json").read_bytes()
subprocess.run([sys.executable, "-B", str(ROOT / "analyze.py")],
               capture_output=True, text=True, check=True)
assert before == (ROOT / "comparison.json").read_bytes()
result = read("comparison.json")
assert result["bothFreshProcessesInSameHumanWindow"] and result["bothSameReadinessObserver"]
assert not result["priorMeasurementsUsed"] and not result["historicalFailuresErased"]
valid_pair = all(v["validForRecordedForegroundIdleSubset"] for v in result["versions"].values())
assert result["validPairForRecordedSubset"] == valid_pair
assert result["cpuBudgetPassed"] == (valid_pair and result["cpuIncreasePercentagePoints"] <= 0.5)
assert result["meanRssBudgetPassed"] == (valid_pair and result["meanWorkingSetIncreaseMiB"] <= 10)
assert not result["fullPerformanceGatePassed"] and not result["macHistoricalFailureCleared"]
state = read("at-rest-protection.json")
assert state["nativeCount"] == 0 and state["productSource"] == "d94c215d869c0560717d79dc02881ce8858afe88"
assert state["codeWorktreeClean"] and all(state["sharedFilesIdentical"].values())
assert state["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert state["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
summary = {"status": "verified-same-window-d94-019-attempt-with-recorded-validity",
           "files": len(files), "source": state["productSource"],
           "validPairForRecordedSubset": result["validPairForRecordedSubset"],
           "cpuIncreasePercentagePoints": result["cpuIncreasePercentagePoints"],
           "meanWorkingSetIncreaseMiB": result["meanWorkingSetIncreaseMiB"],
           "cpuBudgetPassed": result["cpuBudgetPassed"],
           "meanRssBudgetPassed": result["meanRssBudgetPassed"],
           "priorSamplesUsed": False, "bothNormalExits": True,
           "fullPerformanceGatePassed": False, "historicalFailuresErased": False,
           "macHistoricalFailureCleared": False, "releaseAccepted": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
