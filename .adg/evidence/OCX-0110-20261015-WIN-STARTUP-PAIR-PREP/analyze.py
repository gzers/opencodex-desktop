import json
import math
import statistics
from pathlib import Path
ROOT = Path(__file__).resolve().parent
rows = []
for path in sorted(ROOT.glob("[0-9][0-9]-*.json")):
    row = json.loads(path.read_text(encoding="utf-8-sig"))
    assert row["actualSettingsRouteObserved"] and row["foregroundAtInteraction"]
    assert row["actualDirectChildWebViewProfileMatched"] and row["webviewProfileExistsAfterStart"]
    assert not row["applicationUser"]["elevated"] and row["applicationUser"]["elevationType"] == 3
    assert row["exitObserved"] and row["exitCode"] == 0 and not row["forcedTermination"]
    assert 0 < row["buttonObservedMs"] <= row["invokedAtMs"] <= row["interactiveMs"] < 30000
    rows.append(row)
assert len(rows) == 80
comparison = {}
for kind in ("cache-cold", "process-warm"):
    versions = {}
    for version in ("baseline019", "candidate0110"):
        subset = [row for row in rows if row["kind"] == kind and row["version"] == version]
        assert len(subset) == 20 and {row["round"] for row in subset} == set(range(1, 21))
        values = sorted(row["interactiveMs"] for row in subset)
        versions[version] = {
            "count": len(values), "p95Ms": values[math.ceil(0.95 * len(values)) - 1],
            "medianMs": statistics.median(values), "minimumMs": min(values), "maximumMs": max(values),
        }
    delta = versions["candidate0110"]["p95Ms"] - versions["baseline019"]["p95Ms"]
    fraction = delta / versions["baseline019"]["p95Ms"]
    comparison[kind] = {"versions": versions, "p95IncreaseMs": delta, "p95IncreaseFraction": fraction,
                        "withinOriginal100MsAnd5PercentBudget": delta <= 100 and fraction <= 0.05}
result = {"samples": len(rows), "comparisons": comparison, "osColdStartupClaimed": False,
          "sourceMethod": "native Settings interaction, disk-profile cold/process warm, unchanged manager data",
          "fullPerformanceGatePassed": False, "memoryBudgetFailureCleared": False}
(ROOT / "comparison.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
