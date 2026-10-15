import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
names = [
    "OCX-0110-20261015-WIN-IDLE-02-019",
    "OCX-0110-20261015-WIN-IDLE-02-019-RESUMED",
    "OCX-0110-20261015-WIN-IDLE-02-019-FINAL",
    "OCX-0110-20261015-WIN-EVENT-RETRY-02",
]
results = []
for name in names:
    root = ROOT.parent / name
    manifest = json.loads((root / "manifest.json").read_text())
    files = {p.relative_to(root).as_posix() for p in root.rglob("*")
             if p.is_file() and p.relative_to(root).as_posix() not in {"manifest.json", "verification.json"}}
    assert files == {item["path"] for item in manifest["files"]}
    for item in manifest["files"]:
        data = (root / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
    results.append({"directory": name, "files": len(files)})

def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))

first = ROOT.parent / names[0]
assert read(first / "controlled-stop.original.json")["collectorForceStopped"]
assert not read(first / "controlled-stop.original.json")["productForceStopped"]
assert not read(first / "interrupted-summary.json")["fullPerformanceGatePassed"]
prior = ROOT.parent / names[1]
assert not read(prior / "terminal.json")["measurementCompleted"]
comparison = read(ROOT / "comparison.json")
assert comparison["validPairForRecordedSubset"]
assert comparison["cpuBudgetPassed"] and not comparison["meanRssBudgetPassed"]
assert comparison["meanWorkingSetIncreaseMiB"] > 10
assert all(item["observedSpanSeconds"] >= 600 for item in comparison["versions"].values())
assert not comparison["fullPerformanceGatePassed"] and not comparison["macHistoricalFailureCleared"]
event = ROOT.parent / names[3]
before = read(event / "before-preferences.original.json")
failed = read(event / "failed-preferences.original.json")
after = read(event / "retry-preferences.original.json")
assert before == failed and before["appearance"]["theme"] == "light"
assert after["appearance"]["theme"] == "dark"
result = read(event / "result.json")
assert result["identityBefore"] == result["identityAfter"]
assert result["originalFailureResolved"] and result["unrelatedFailuresRetained"] and result["normalExit"]
assert read(event / "attributes-restored.json")["attributesRestored"]
assert not result["fullEventGatePassed"]
print(json.dumps({"status": "verified", "archives": results, "rssBudgetFailedRetained": True,
                  "fullPerformanceGatePassed": False, "fullVersionGatePassed": False}))
