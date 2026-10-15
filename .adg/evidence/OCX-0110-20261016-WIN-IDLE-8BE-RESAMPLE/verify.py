"""Verify the frozen raw evidence and exactly recompute the recorded comparison."""
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
    assert len(data) == entry["bytes"]
    assert hashlib.sha256(data).hexdigest() == entry["sha256"]
before = (ROOT / "comparison.json").read_bytes()
analysis = subprocess.run([sys.executable, str(ROOT / "analyze.py")],
                          capture_output=True, text=True, check=True)
assert before == (ROOT / "comparison.json").read_bytes()
result = read("comparison.json")
assert not result["interruptedCandidateSamplesUsed"]
assert not result["pairedVersionsObservedWithoutInterveningInterruption"]
assert result["readinessObserverDiffersFromBaseline"]
assert not result["fullPerformanceGatePassed"]
assert not result["macHistoricalFailureCleared"]
assert result["cpuBudgetPassed"] == (result["cpuIncreasePercentagePoints"] <= 0.5)
assert result["meanRssBudgetPassed"] == (result["meanWorkingSetIncreaseMiB"] <= 10)
final = read("at-rest-protection.json")
assert final["nativeCount"] == 0 and final["source"] == result["versions"]["candidate0110"]["source"]
assert final["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert final["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert all(final["sharedFilesIdentical"].values())
summary = {
    "status": "verified-completed-candidate-and-explicitly-reused-baseline",
    "files": len(files), "source": final["source"],
    "validPairForRecordedSubset": result["validPairForRecordedSubset"],
    "cpuIncreasePercentagePoints": result["cpuIncreasePercentagePoints"],
    "meanWorkingSetIncreaseMiB": result["meanWorkingSetIncreaseMiB"],
    "cpuBudgetPassed": result["cpuBudgetPassed"],
    "meanRssBudgetPassed": result["meanRssBudgetPassed"],
    "normalProductExit": True, "interruptedSamplesUsed": False,
    "fullPerformanceGatePassed": False, "releaseAccepted": False,
}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
