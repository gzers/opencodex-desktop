"""Independently recompute raw counter arithmetic and real-generation roles."""
import hashlib
import json
import re
import statistics
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def ticks(value):
    match = re.search(r"\.(\d+)(?:Z|[+-]\d\d:\d\d)$", value)
    digits = match.group(1) if match else ""
    assert len(digits) <= 7
    value = datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    whole = value.replace(microsecond=0) - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return (whole.days * 86400 + whole.seconds) * 10_000_000 + int(digits.ljust(7, "0"))


scope = read("scope.json")
analysis = read("original/counter-role-analysis.json")
human = read("original/human-window.json")
assert human["explicitHumanStart"] and not human["automaticContinuation"] and human["userMessage"] == "开始前台验证"
assert human["scope"] == "one-single-instance-foreground-idle-pair"
start, end = ticks(human["authorizedAtUtc"]), ticks(human["expiresAtUtc"])
assert end - start == 1800 * 10_000_000
for copy in read("original-copies.json"):
    data = (ROOT / copy["path"]).read_bytes()
    assert len(data) == copy["bytes"] and sha(data) == copy["sha256"]
    assert data == Path(copy["runtimePath"]).read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig"))
calculated = {}
for version, source, image in [
    ("baseline019", "de6862d40389d527527b9db8ef52e186a58b625f",
     "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"),
    ("candidate0110", "b4d487e4e04cd4c80fd5570663e1ed21dd483b19",
     "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db"),
]:
    launch = read(f"original/idle-{version}-launch.json")
    identity = read(f"original/{version}/identity.json")
    terminal = read(f"original/{version}/terminal.json")
    initial = read(f"original/{version}-sky-initial.json")
    ready = read(f"original/{version}-sky-settled-ready.json")
    roles = read(f"original/{version}-settled-roles.original.json")
    attempt = read(f"original/{version}-settled-roles-attempt.json")
    assert launch["source"] == identity["source"] == source
    assert launch["exeSha256"] == identity["exeSha256"] == roles["exeSha256"] == image
    assert launch["pid"] == identity["pid"] == roles["rootPid"] == attempt["pid"]
    assert launch["creationTimeUtc"] == identity["creationTimeUtc"]
    token = launch["applicationToken"]
    assert not token["elevated"] and token["elevationType"] == 3
    assert token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    assert ticks(ready["observedAtUtc"]) - ticks(initial["observedAtUtc"]) >= 60 * 10_000_000
    assert "尚未接入 OpenCodex" in ready["accessibility"]["tree"]
    assert (identity["clientWidth"], identity["clientHeight"]) == (2360, 1520)
    assert identity["launchInputSha256"] == sha((ROOT / f"original/idle-{version}-launch.json").read_bytes())
    assert identity["readinessInputSha256"] == sha((ROOT / f"original/{version}-sky-settled-ready.json").read_bytes())
    assert terminal["measurementCompleted"] and not terminal["observationInvalid"]
    assert terminal["readOrFrameFailures"] == 0
    assert read(f"original/{version}-collector-exit.json")["observerExitCode"] == 0
    data = (ROOT / f"original/{version}/samples.original.jsonl").read_bytes()
    rows = [json.loads(line) for line in data.decode("utf-8-sig").splitlines()]
    assert len(rows) == terminal["samples"] == 597
    span = rows[-1]["elapsedSeconds"] - rows[0]["elapsedSeconds"]
    assert span == terminal["counterSpanSeconds"] >= 600
    role_map = {(m["pid"], ticks(m["creationUtc"])): m["role"] for m in roles["members"]}
    assert len(role_map) == 7 and list(role_map.values()).count("gpu") == 1
    role_values = {role: [] for role in role_map.values()}
    known = None
    for row in rows:
        assert row["foreground"] and row["visible"] and not row["minimized"] and row["nativeCount"] == 1
        assert start <= ticks(row["atUtc"]) <= end
        keys = {(m["pid"], ticks(m["creationUtc"])) for m in row["members"]}
        assert keys == set(role_map)
        if known is not None:
            assert known == keys
        known = keys
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
        assert row["cpuSeconds"] == sum(m["cpuSeconds"] for m in row["members"])
        totals = {}
        for member in row["members"]:
            role = role_map[(member["pid"], ticks(member["creationUtc"]))]
            totals[role] = totals.get(role, 0) + member["workingSetBytes"]
        for role in role_values:
            role_values[role].append(totals.get(role, 0))
    assert all(b["elapsedSeconds"] > a["elapsedSeconds"] and b["cpuSeconds"] >= a["cpuSeconds"]
               for a, b in zip(rows, rows[1:]))
    mean = sum(row["workingSetBytes"] for row in rows) / len(rows) / 1048576
    cpu = 100 * (rows[-1]["cpuSeconds"] - rows[0]["cpuSeconds"]) / span
    by_role = {role: sum(values) / len(values) / 1048576 for role, values in role_values.items()}
    original = analysis["versions"][version]
    assert mean == original["meanRssMiB"] and cpu == original["cpuOneLogicalProcessorPercent"]
    assert sha(data) == original["rawSha256"]
    assert all(abs(by_role[r] - original["meanRssMiBByActualRole"][r]) < 1e-10 for r in by_role)
    calculated[version] = {"rss": mean, "cpu": cpu, "roles": by_role}
baseline, candidate = calculated["baseline019"], calculated["candidate0110"]
rss = candidate["rss"] - baseline["rss"]
cpu = candidate["cpu"] - baseline["cpu"]
assert rss == analysis["meanRssIncreaseMiB"] == scope["meanRssIncreaseMiB"]
assert cpu == analysis["cpuIncreasePercentagePoints"]
assert rss > 10 and cpu <= 0.5 and not analysis["rssCounterSubsetBudgetPassed"]
assert analysis["cpuCounterSubsetBudgetPassed"]
for role in candidate["roles"]:
    assert abs(candidate["roles"][role] - baseline["roles"][role] - analysis["roleRssDifferenceMiB"][role]) < 1e-10
baseline_exit = read("original/baseline019-exit.json")
assert baseline_exit["exitObserved"] and baseline_exit["exitCode"] == 0 and baseline_exit["retainedHandle"]
assert ticks(read("original/idle-candidate0110-launch.json")["creationTimeUtc"]) >= int(baseline_exit["kernelTimes"]["exitFileTime"])
baseline_action = read("original/baseline019-close-action.json")
assert baseline_action["action"] == "Computer Use Alt+F4" and not baseline_action["forceTerminated"]
assert start <= ticks(baseline_action["atUtc"]) <= end
candidate_exit = read("original/candidate0110-exit.json")
late = read("original/late-cleanup-exit.json")
assert not candidate_exit["exitObserved"] and candidate_exit["exitCode"] is None
assert not late["exitObserved"] and late["exitCode"] is None
assert late["originalExitDeadlineOutcomeUnchanged"] and not late["foregroundPermissionExtended"]
assert not late["formalPairLifecycleAccepted"] and not late["forceTerminated"]
close = read("original/candidate0110-close-window-expired.json")
assert not close["computerUseCloseInputSent"] and close["closeAttemptRefusedByTimeGuard"]
checkpoint = read("original/cleanup-checkpoint.json")
assert checkpoint["nativeCount"] == 1 and not checkpoint["fixtureRestored"]
assert checkpoint["originalProfileIdentityMatches"] and not checkpoint["forceTerminated"]
assert checkpoint["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
final = read("original/cleanup-final-readback.json")
assert final["nativeCount"] == 1 and not final["fixtureRestored"] and not final["restoreAttempted"]
assert final["normalCandidateExitCodeStillUnknown"] and not final["foregroundInputAfterExpirySent"]
assert not final["forceTerminated"] and final["originalExitDeadlineOutcomeUnchanged"]
cached = read("original/candidate0110-sky-preclose-cached.json")
assert cached["cachedObservationOnly"] and not cached["newUiReadOrInputPerformed"]
assert ticks(cached["observationAtUtcFromToolOutput"]) <= end
pr = read("pr-readback.original.json")
assert pr["number"] == 6 and pr["head"]["sha"] == scope["source"] and pr["draft"] and not pr["merged"]
assert "+20.040155MiB" in pr["body"] and "未完成" in pr["body"]
for binding in read("source-bindings.json"):
    directory = PROJECT / binding["evidenceDirectory"]
    assert sha((directory / "manifest.json").read_bytes()) == binding["manifestSha256"]
    assert sha((directory / "verification.json").read_bytes()) == binding["verificationSha256"]
for name in ["formalPairLifecycleAdmitted", "fullPerformanceGatePassed",
             "historicalFailuresCleared", "releaseAccepted", "taskClosed"]:
    assert not scope[name]
if (ROOT / "manifest.json").exists():
    manifest = read("manifest.json")
    assert {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name != "manifest.json"} == {r["path"] for r in manifest["files"]}
    for row in manifest["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
print(json.dumps({"status": "verified-completed-counter-and-role-evidence-lifecycle-incomplete",
                  "samplesPerVersion": 597, "meanRssIncreaseMiB": rss,
                  "cpuIncreasePercentagePoints": cpu,
                  "gpuProcessRssDifferenceMiB": analysis["roleRssDifferenceMiB"]["gpu"],
                  "rssBudgetPassed": False, "formalPairLifecycleAdmitted": False,
                  "cleanupPendingAtCheckpoint": True, "releaseAccepted": False}))
