import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
value = read("official-baseline-safety-and-budget.json")
assert value["baselineCommit"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert value["baselineExecutableSha256"] == "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"
assert not value["baselineLaunchAttempted"] and not value["configuredIsolationSufficientlyProved"]
assert not value["performancePassed"] and not value["originalFailedPerformanceCleared"]
assert any("dirs::home_dir()" in line["text"] for line in value["platformWitness"])
assert any("platform::home_dir()" in line["text"] for line in value["startupWitness"])
assert value["originalPerformanceContract"] == {
    "coldStartsMinimum": 20, "hotStartsMinimum": 20,
    "p95IncrementMaxMilliseconds": 100, "p95IncrementMaxPercent": 5,
    "idleMinutes": 10, "idleCpuIncrementMaxPercentagePoints": 0.5,
    "rssIncrementMaxMiB": 10, "mainThreadTaskMaxMilliseconds": 50,
    "cacheRoutes": 100, "cacheRestarts": 10, "unexpiredCacheAutoNetworkOrQuerySubprocessMax": 0,
}
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, item):
    (root / name).write_bytes((json.dumps(item, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "official baseline identity recheck, source witness and preserved original budget only",
    "baselineNativeRun": False, "pairedPerformanceAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "performance": "not measured", "files": len(entries)}))
