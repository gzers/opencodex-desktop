"""Offline independent replay of archived original counters; no native input."""
import hashlib
import json
import math
import statistics
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(name):
    return hashlib.sha256((ROOT / name).read_bytes()).hexdigest()


def stamp(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


contract, comparison = read("contract.original.json"), read("comparison.json")
human, copied = read("human-window.json"), read("runtime-copy-receipt.json")
assert human["explicitHumanStart"] and not human["automaticContinuation"]
assert human["userMessage"] == "开始前台验证"
start, end = stamp(human["authorizedAtUtc"]), stamp(human["expiresAtUtc"])
assert (end - start).total_seconds() == 1800
assert human["scope"] == contract["humanWindowScope"]
assert not copied["managerProfileOrPrivateSddlCopied"]
for item in copied["files"]:
    assert (ROOT / item["path"]).stat().st_size == item["bytes"]
    assert sha(item["path"]) == item["sha256"]

actual = {}
for version, prefix in (("baseline019", "baseline"), ("candidate0110", "candidate")):
    launch_name = f"idle-{version}-launch.json"
    ready_name = f"{version}-sky-settled-ready.json"
    launch, ready = read(launch_name), read(ready_name)
    identity, terminal = read(f"{version}/identity.json"), read(f"{version}/terminal.json")
    stop = read(f"{version}-exit.json")
    assert launch["source"] == identity["source"] == contract[prefix + "Source"]
    assert launch["exeSha256"] == identity["exeSha256"] == contract[prefix + "ExeSha256"]
    assert launch["collectorSource"] == contract["collectorSource"]
    assert launch["humanWindowReceiptSha256"] == sha("human-window.json")
    assert launch["pid"] == identity["pid"] == stop["pid"]
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    assert launch["applicationToken"]["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    for token in (launch["applicationToken"], identity["actualToken"]):
        assert not token["elevated"] and token["elevationType"] == 3
    assert (stamp(ready["observedAtUtc"]) - stamp(launch["creationTimeUtc"])).total_seconds() >= 60
    assert "尚未接入 OpenCodex" in ready["accessibility"]["tree"]
    assert identity["launchInputSha256"] == sha(launch_name)
    assert identity["readinessInputSha256"] == sha(ready_name)
    assert identity["methodSha256"] == contract["methodWorkingFileSha256"]["test/smoke/windows-idle-observe.ps1"]
    assert identity["moduleSha256"] == contract["methodWorkingFileSha256"]["test/smoke/windows-idle-family.ps1"]
    assert (identity["clientWidth"], identity["clientHeight"]) == (2360, 1520)
    assert terminal["measurementCompleted"] and not terminal["observationInvalid"]
    assert terminal["readOrFrameFailures"] == 0
    assert read(f"{version}/progress.json") == terminal
    assert stop["exitObserved"] and stop["exitCode"] == 0 and stop["retainedHandle"]
    assert not stop["forcedTermination"] and stop["kernelTimes"]["exited"]
    assert stop["kernelTimes"]["sourceMethodSha256"] == contract["kernelTimesMethodSha256"]
    rows = [json.loads(line) for line in (ROOT / version / "samples.original.jsonl").read_text().splitlines()]
    assert len(rows) == terminal["samples"]
    span = rows[-1]["elapsedSeconds"] - rows[0]["elapsedSeconds"]
    assert span == terminal["counterSpanSeconds"] >= 600
    known, last_cpu = set(), -1
    member_rss = {}
    for row in rows:
        assert row["foreground"] and row["visible"] and not row["minimized"] and row["nativeCount"] == 1
        assert start <= stamp(row["atUtc"]) <= end
        members = {(m["pid"], m["creationUtc"]) for m in row["members"]}
        assert known <= members and len(members) == len(row["members"])
        known = members
        assert row["cpuSeconds"] >= last_cpu
        last_cpu = row["cpuSeconds"]
        assert row["cpuSeconds"] == sum(m["cpuSeconds"] for m in row["members"])
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
        for member in row["members"]:
            key = (member["pid"], member["creationUtc"], member["name"])
            member_rss.setdefault(key, []).append(member["workingSetBytes"])
    cpu = 100 * (rows[-1]["cpuSeconds"] - rows[0]["cpuSeconds"]) / span
    rss = statistics.mean(row["workingSetBytes"] for row in rows) / 1048576
    expected = comparison["versions"][version]
    assert sha(f"{version}/samples.original.jsonl") == expected["rawSha256"]
    assert math.isclose(cpu, expected["cpuOneLogicalProcessorPercent"], abs_tol=1e-10)
    assert math.isclose(rss, expected["meanRssMiB"], abs_tol=1e-10)
    root_rss = statistics.mean(
        sum(m["workingSetBytes"] for m in row["members"] if m["pid"] == launch["pid"])
        for row in rows) / 1048576
    actual[version] = {
        "samples": len(rows), "counterSpanSeconds": span, "cpuPercentOneLogicalCore": cpu,
        "meanFamilyRssMiB": rss, "meanNativeRootRssMiB": root_rss,
        "meanDescendantRssMiB": rss - root_rss,
        "members": [{"pid": key[0], "creationUtc": key[1], "name": key[2],
                     "meanRssMiB": statistics.mean(values) / 1048576}
                    for key, values in member_rss.items()],
    }

baseline_stop, candidate_launch = read("baseline019-exit.json"), read("idle-candidate0110-launch.json")
candidate_stop = read("candidate0110-exit.json")
assert int(baseline_stop["kernelTimes"]["exitFileTime"]) <= int(candidate_stop["kernelTimes"]["creationFileTime"])
cpu_delta = actual["candidate0110"]["cpuPercentOneLogicalCore"] - actual["baseline019"]["cpuPercentOneLogicalCore"]
rss_delta = actual["candidate0110"]["meanFamilyRssMiB"] - actual["baseline019"]["meanFamilyRssMiB"]
assert comparison["budgets"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
assert comparison["cpuBudgetPassed"] == (cpu_delta <= 0.5)
assert comparison["meanRssBudgetPassed"] == (rss_delta <= 10)
assert comparison["validPairForRecordedSubset"] and not comparison["priorSamplesUsed"]
assert not comparison["fullPerformanceGatePassed"] and not comparison["historicalFailuresCleared"]
assert not comparison["releaseAccepted"]
preserved, released = read("final-preservation-readback.json"), read("foreground-release.json")
assert preserved["nativeCount"] == released["nativeCount"] == 0
assert released["foregroundReleased"] and not released["recoveryAttempted"]
assert not preserved["recoveryPrerequisitesChanged"]
assert preserved["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert preserved["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert preserved["candidatePreferencesSha256"] == "2d612148a55152e05047d1f13c4587a3b75f2740b1a51cc2c0f64f30a767495e"
assert not preserved["productStatus"] and not preserved["diagnosticStatus"]
fixtures = read("midrun-fixture-and-preservation-readback.json")["fixtures"]
assert fixtures[0]["preferences"] == fixtures[1]["preferences"]
dpi = read("observer-dpi-readback.json")
assert (dpi["width"], dpi["height"], dpi["readerAwareness"]) == (2360, 1520, 2)
assert not dpi["uiInputInjected"]
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        assert sha(item["path"]) == item["sha256"]
        assert (ROOT / item["path"]).stat().st_size == item["bytes"]
print(json.dumps({
    "verification": "passed", "completedPairAdmitted": True,
    "cpuIncreasePercentagePoints": cpu_delta, "meanRssIncreaseMiB": rss_delta,
    "cpuBudgetPassed": cpu_delta <= 0.5, "meanRssBudgetPassed": rss_delta <= 10,
    "versions": actual, "causalAttributionEstablished": False,
    "nativeRecoveryExecuted": False, "fullPerformanceGatePassed": False,
    "releaseAccepted": False,
}, indent=2))
