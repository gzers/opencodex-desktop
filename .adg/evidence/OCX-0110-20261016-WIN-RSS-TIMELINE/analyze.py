"""Descriptive analysis of existing admitted samples, never a replacement gate."""
import hashlib
import json
import math
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PAIR = ROOT.parent / "OCX-0110-20261016-WIN-823-IDLE-PAIR"
MIB = 1048576


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def slope(minutes, values):
    return statistics.covariance(minutes, values) / statistics.variance(minutes)


original = read(PAIR / "comparison.json")
assert original["validPairForRecordedSubset"] and not original["priorSamplesUsed"]
assert original["budgets"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
assert not original["meanRssBudgetPassed"] and not original["fullPerformanceGatePassed"]
versions = {}
inputs = {}
for version in ["baseline019", "candidate0110"]:
    path = PAIR / version / "samples.original.jsonl"
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    expected = original["versions"][version]
    assert digest == expected["rawSha256"]
    rows = [json.loads(line) for line in data.splitlines()]
    assert len(rows) == expected["samples"]
    inputs[f"{version}/samples.original.jsonl"] = {"sha256": digest, "bytes": len(data)}
    first, last = rows[0]["elapsedSeconds"], rows[-1]["elapsedSeconds"]
    assert last - first == expected["counterSpanSeconds"] >= 600
    root_pid = read(PAIR / f"idle-{version}-launch.json")["pid"]
    keys = {(m["pid"], m["creationUtc"], m["name"]) for m in rows[0]["members"]}
    assert len(keys) == 7 and all({(m["pid"], m["creationUtc"], m["name"]) for m in r["members"]} == keys for r in rows)
    assert all(r["foreground"] and r["visible"] and not r["minimized"] and r["nativeCount"] == 1 for r in rows)
    assert all(not m["exited"] for r in rows for m in r["members"])
    assert all(r["workingSetBytes"] == sum(m["workingSetBytes"] for m in r["members"]) for r in rows)
    minutes = [(r["elapsedSeconds"] - first) / 60 for r in rows]
    family = [r["workingSetBytes"] / MIB for r in rows]
    members = []
    for pid, created, name in sorted(keys):
        values = [next(m["workingSetBytes"] for m in r["members"]
                       if m["pid"] == pid and m["creationUtc"] == created) / MIB for r in rows]
        members.append({"pid": pid, "creationUtc": created, "name": name,
            "actualRole": "native-root" if pid == root_pid else None,
            "meanRssMiB": statistics.mean(values),
            "ordinaryLeastSquaresSlopeMiBPerMinute": slope(minutes, values),
            "firstSampleRssMiB": values[0], "lastSampleRssMiB": values[-1]})
    largest = max((m for m in members if m["name"] == "msedgewebview2.exe"), key=lambda m: m["meanRssMiB"])
    assert all(max((m for m in r["members"] if m["name"] == "msedgewebview2.exe"),
                   key=lambda m: m["workingSetBytes"])["pid"] == largest["pid"] for r in rows)
    bins = []
    for index in range(10):
        selected = [r for r in rows if index * 60 <= r["elapsedSeconds"] - first < (index + 1) * 60]
        bins.append({"relativeSecondsStartInclusive": index * 60, "relativeSecondsEndExclusive": (index + 1) * 60,
            "samples": len(selected), "meanFamilyRssMiB": statistics.mean(r["workingSetBytes"] / MIB for r in selected),
            "meanLargestBySizeWebViewRssMiB": statistics.mean(
                next(m["workingSetBytes"] for m in r["members"] if m["pid"] == largest["pid"]) / MIB
                for r in selected)})
    overflow = [r for r in rows if r["elapsedSeconds"] - first >= 600]
    assert sum(b["samples"] for b in bins) + len(overflow) == len(rows)
    mean = statistics.mean(family)
    assert math.isclose(mean, expected["meanRssMiB"], abs_tol=1e-10)
    assert math.isclose(sum(m["meanRssMiB"] for m in members), mean, abs_tol=1e-10)
    family_slope = slope(minutes, family)
    assert math.isclose(sum(m["ordinaryLeastSquaresSlopeMiBPerMinute"] for m in members),
                        family_slope, abs_tol=1e-10)
    trapezoid = sum((a["workingSetBytes"] + b["workingSetBytes"]) / 2 *
                    (b["elapsedSeconds"] - a["elapsedSeconds"]) for a, b in zip(rows, rows[1:]))
    versions[version] = {
        "source": expected["source"], "exeSha256": expected["exeSha256"],
        "samples": len(rows), "counterSpanSeconds": last - first,
        "originalSampleMeanRssMiB": mean, "ordinaryLeastSquaresSlopeMiBPerMinute": family_slope,
        "firstSampleRssMiB": family[0], "lastSampleRssMiB": family[-1],
        "largestBySizeWebView": largest, "otherFamilyMeanRssMiB": mean - largest["meanRssMiB"],
        "timeWeightedTrapezoidMeanMiBDescriptiveOnly": trapezoid / (last - first) / MIB,
        "minuteBins": bins, "samplesAfterRelative600SecondsIncludedInWholeMean": len(overflow),
        "members": members,
    }
baseline, candidate = versions["baseline019"], versions["candidate0110"]
delta = candidate["originalSampleMeanRssMiB"] - baseline["originalSampleMeanRssMiB"]
largest_delta = candidate["largestBySizeWebView"]["meanRssMiB"] - baseline["largestBySizeWebView"]["meanRssMiB"]
other_delta = candidate["otherFamilyMeanRssMiB"] - baseline["otherFamilyMeanRssMiB"]
assert math.isclose(delta, largest_delta + other_delta, abs_tol=1e-10)
assert math.isclose(delta, original["meanRssIncreaseMiB"], abs_tol=1e-10)
result = {
    "status": "descriptive-existing-sample-analysis", "inputArchive": PAIR.name,
    "inputSampleFiles": inputs, "versions": versions,
    "originalMeanRssIncreaseMiB": delta, "originalBudgetMiB": 10, "originalBudgetPassed": False,
    "largestBySizeWebViewMeanRssDifferenceMiB": largest_delta,
    "largestBySizeDifferenceFractionOfTotal": largest_delta / delta,
    "otherFamilyMeanRssDifferenceMiB": other_delta,
    "firstRelativeMinuteGapMiB": candidate["minuteBins"][0]["meanFamilyRssMiB"] - baseline["minuteBins"][0]["meanFamilyRssMiB"],
    "tenthRelativeMinuteGapMiB": candidate["minuteBins"][9]["meanFamilyRssMiB"] - baseline["minuteBins"][9]["meanFamilyRssMiB"],
    "familySlopeDifferenceMiBPerMinute": candidate["ordinaryLeastSquaresSlopeMiBPerMinute"] - baseline["ordinaryLeastSquaresSlopeMiBPerMinute"],
    "timeWeightedDifferenceDescriptiveOnlyMiB": candidate["timeWeightedTrapezoidMeanMiBDescriptiveOnly"] - baseline["timeWeightedTrapezoidMeanMiBDescriptiveOnly"],
    "crossVersionRolesMatched": False, "gpuOrRendererRoleInferredFromRssSize": False,
    "sizeAccountingIsCausalAttribution": False, "memoryLeakConfirmedOrExcluded": False,
    "fullPerformanceGatePassed": False, "acceptanceMetricChanged": False,
    "newNativeSamplesTaken": False, "productLaunched": False, "historicalFailuresCleared": False,
    "nextDiagnosticTarget": "Bind actual live WebView roles and generation before allocation/resource inspection; do not reuse historical PIDs or repeat unchanged formal idle pair.",
}
path = ROOT / "analysis.json"
if path.exists():
    assert read(path) == result
else:
    path.write_bytes((json.dumps(result, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
print(json.dumps({k: v for k, v in result.items() if k not in {"versions", "inputSampleFiles"}}, indent=2))
