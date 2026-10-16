"""Offline completed-counter arithmetic and actual-generation role accounting."""
import hashlib
import json
import re
import statistics
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
assert not (ROOT / "counter-role-analysis.json").exists()


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def instant(value):
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    digits = re.search(r"\.(\d+)(?:Z|[+-]\d\d:\d\d)$", value)
    fraction = digits.group(1) if digits else ""
    assert len(fraction) <= 7
    whole = parsed.replace(microsecond=0) - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return (whole.days * 86400 + whole.seconds) * 10_000_000 + int(fraction.ljust(7, "0"))


human = read("human-window.json")
start, end = instant(human["authorizedAtUtc"]), instant(human["expiresAtUtc"])
versions = {}
for version in ["baseline019", "candidate0110"]:
    launch = read("idle-" + version + "-launch.json")
    terminal = read(version + "/terminal.json")
    roles = read(version + "-settled-roles.original.json")
    assert terminal["measurementCompleted"] and not terminal["observationInvalid"]
    assert terminal["readOrFrameFailures"] == 0 and terminal["counterSpanSeconds"] >= 600
    assert roles["rootPid"] == launch["pid"] and roles["exeSha256"] == launch["exeSha256"]
    role_map = {(m["pid"], instant(m["creationUtc"])): m["role"] for m in roles["members"]}
    path = ROOT / version / "samples.original.jsonl"
    data = path.read_bytes()
    rows = [json.loads(line) for line in data.decode("utf-8-sig").splitlines()]
    assert len(rows) == terminal["samples"] == 597
    span = rows[-1]["elapsedSeconds"] - rows[0]["elapsedSeconds"]
    assert span == terminal["counterSpanSeconds"] >= 600
    role_samples = {}
    for row in rows:
        assert row["foreground"] and row["visible"] and not row["minimized"] and row["nativeCount"] == 1
        assert start <= instant(row["atUtc"]) <= end
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
        totals = {}
        for member in row["members"]:
            key = (member["pid"], instant(member["creationUtc"]))
            assert key in role_map, "Never infer an unobserved process role"
            role = role_map[key]
            totals[role] = totals.get(role, 0) + member["workingSetBytes"]
        for role in set(role_map.values()):
            role_samples.setdefault(role, []).append(totals.get(role, 0))
    mean = statistics.mean(r["workingSetBytes"] for r in rows) / 1048576
    per_role = {role: statistics.mean(values) / 1048576 for role, values in role_samples.items()}
    assert abs(sum(per_role.values()) - mean) < 1e-9
    versions[version] = {
        "source": launch["source"], "exeSha256": launch["exeSha256"], "samples": len(rows),
        "spanSeconds": span, "rawSha256": hashlib.sha256(data).hexdigest(),
        "cpuOneLogicalProcessorPercent": 100 * (rows[-1]["cpuSeconds"] - rows[0]["cpuSeconds"]) / span,
        "meanRssMiB": mean, "meanRssMiBByActualRole": per_role,
        "firstCounterAtUtc": rows[0]["atUtc"], "lastCounterAtUtc": rows[-1]["atUtc"],
        "processCreationUtc": launch["creationTimeUtc"],
        "actualGenerationRoleBindings": [
            {"pid": member["pid"], "creationUtc": member["creationUtc"], "role": member["role"]}
            for member in roles["members"]],
    }
baseline, candidate = versions["baseline019"], versions["candidate0110"]
cpu = candidate["cpuOneLogicalProcessorPercent"] - baseline["cpuOneLogicalProcessorPercent"]
rss = candidate["meanRssMiB"] - baseline["meanRssMiB"]
difference = {
    role: candidate["meanRssMiBByActualRole"].get(role, 0) - baseline["meanRssMiBByActualRole"].get(role, 0)
    for role in sorted(set(baseline["meanRssMiBByActualRole"]) | set(candidate["meanRssMiBByActualRole"]))
}
assert abs(sum(difference.values()) - rss) < 1e-9
result = {
    "status": "completed-counter-subset-and-actual-role-accounting-only",
    "versions": versions, "cpuIncreasePercentagePoints": cpu, "meanRssIncreaseMiB": rss,
    "roleRssDifferenceMiB": difference, "budgets": {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10},
    "cpuCounterSubsetBudgetPassed": cpu <= 0.5, "rssCounterSubsetBudgetPassed": rss <= 10,
    "metricFormulaUnchanged": True, "rolesJoinedByActualPidAndCreation": True,
    "sizeUsedToGuessRoles": False, "roleAccountingIsCausalAllocationAttribution": False,
    "gpuSharedUsageAddedToRss": False, "historicalPidsAssignedRoles": False,
    "candidateExitDeadline": read("candidate0110-exit.json"),
    "candidatePostspanRolesTaken": False, "foregroundCloseActionSent": False,
    "formalPairLifecycleAdmitted": False, "fullPerformanceGatePassed": False,
    "historicalFailuresCleared": False, "releaseAccepted": False,
}
(ROOT / "counter-role-analysis.json").write_bytes(
    (json.dumps(result, ensure_ascii=False, indent=2) + "\n").encode())
print(json.dumps({k: v for k, v in result.items() if k not in ["versions", "candidateExitDeadline"]},
                 ensure_ascii=False))
