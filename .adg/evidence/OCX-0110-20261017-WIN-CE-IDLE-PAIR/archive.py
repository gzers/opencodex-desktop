"""Preserve this completed pair and independently recompute its recorded subset."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUN = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-ce-pair-20261017T113045")
PREP = ROOT.parent / "OCX-0110-20261017-WIN-CE-FRONT-PREP"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def ticks(value):
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    match = re.search(r"\.(\d+)(?:Z|[+-]\d\d:\d\d)$", value)
    fraction = match.group(1) if match else ""
    assert len(fraction) <= 7
    delta = parsed.replace(microsecond=0) - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return (delta.days * 86400 + delta.seconds) * 10000000 + int(fraction.ljust(7, "0"))


def verify():
    source = ROOT / "original"
    human = read(source / "human-window.json")
    start, end = ticks(human["authorizedAtUtc"]), ticks(human["expiresAtUtc"])
    assert human["explicitHumanStart"] and not human["automaticContinuation"]
    assert human["scope"] == "one-single-instance-foreground-idle-pair"
    assert end - start == 1800 * 10000000
    contract = read(PREP / "contract.json")
    comparison = read(source / "comparison.json")
    results = {}
    for version, prefix in (("baseline019", "baseline"), ("candidate0110", "candidate")):
        launch = read(source / f"idle-{version}-launch.json")
        ready = read(source / f"{version}-sky-settled-ready.json")
        identity = read(source / version / "identity.json")
        terminal = read(source / version / "terminal.json")
        close = read(source / f"{version}-close-action.json")
        stop = read(source / f"{version}-exit.json")
        roles = read(source / f"{version}-settled-roles.original.json")
        collector = read(source / f"{version}-collector-exit.json")
        assert launch["source"] == identity["source"] == contract[prefix + "Source"]
        assert launch["exeSha256"] == identity["exeSha256"] == contract[prefix + "ExeSha256"]
        assert launch["pid"] == identity["pid"] == roles["rootPid"] == stop["pid"]
        assert identity["launchInputSha256"] == sha(source / f"idle-{version}-launch.json")
        assert identity["readinessInputSha256"] == sha(source / f"{version}-sky-settled-ready.json")
        assert identity["moduleSha256"] == contract["methodWorkingFileSha256"]["test/smoke/windows-idle-family.ps1"]
        assert identity["methodSha256"] == contract["methodWorkingFileSha256"]["test/smoke/windows-idle-observe.ps1"]
        assert (identity["clientWidth"], identity["clientHeight"]) == (2360, 1520)
        assert not launch["sandboxUsed"] and not launch["cdpUsed"]
        token = launch["applicationToken"]
        assert not token["elevated"] and token["elevationType"] == 3
        assert token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
        assert "尚未接入 OpenCodex" in ready["accessibility"]["tree"]
        assert ticks(ready["observedAtUtc"]) - ticks(launch["creationTimeUtc"]) >= 60 * 10000000
        assert terminal["measurementCompleted"] and not terminal["observationInvalid"]
        assert terminal["readOrFrameFailures"] == 0 and collector["observerExitCode"] == 0
        assert close["method"] == "sky.press_key Alt_L+F4" and not close["forcedTermination"]
        assert start <= ticks(close["atUtc"]) <= end
        assert stop["exitObserved"] and stop["exitCode"] == 0 and stop["nativeCount"] == 0
        assert stop["retainedHandle"] and not stop["forcedTermination"]
        kernel = stop["kernelTimes"]
        assert kernel["exited"] and kernel["retainedHandle"] and kernel["pid"] == launch["pid"]
        assert int(kernel["creationFileTime"]) == ticks(launch["creationTimeUtc"])
        assert ticks(close["atUtc"]) <= int(kernel["exitFileTime"]) <= end
        assert kernel["sourceMethodSha256"] == contract["kernelTimesMethodSha256"]
        path = source / version / "samples.original.jsonl"
        rows = [json.loads(line, parse_float=Decimal) for line in path.read_text(encoding="utf-8-sig").splitlines()]
        assert len(rows) == terminal["samples"] == comparison["versions"][version]["samples"]
        assert sha(path) == comparison["versions"][version]["rawSha256"]
        span = rows[-1]["elapsedSeconds"] - rows[0]["elapsedSeconds"]
        assert span >= 600 and abs(span - Decimal(str(terminal["counterSpanSeconds"]))) < Decimal("0.000000001")
        role_map = {(m["pid"], ticks(m["creationUtc"])): m["role"] for m in roles["members"]}
        totals = {}
        unbound = set()
        for row in rows:
            assert row["foreground"] and row["visible"] and not row["minimized"] and row["nativeCount"] == 1
            assert start <= ticks(row["atUtc"]) <= end
            assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
            assert abs(row["cpuSeconds"] - sum(Decimal(str(m["cpuSeconds"])) for m in row["members"])) < Decimal("0.000000001")
            for member in row["members"]:
                generation = (member["pid"], ticks(member["creationUtc"]))
                role = role_map.get(generation, "not-in-pre-span-role-snapshot")
                if generation not in role_map:
                    unbound.add((member["pid"], member["creationUtc"], member["name"]))
                totals[role] = totals.get(role, 0) + member["workingSetBytes"]
        cpu = 100 * (rows[-1]["cpuSeconds"] - rows[0]["cpuSeconds"]) / span
        rss = Decimal(sum(r["workingSetBytes"] for r in rows)) / len(rows) / 1048576
        results[version] = {
            "samples": len(rows), "spanSeconds": float(span), "cpuOneLogicalProcessorPercent": float(cpu),
            "meanRssMiB": float(rss), "meanRssMiBByActualRole": {
                role: float(Decimal(total) / len(rows) / 1048576) for role, total in totals.items()
            },
            "roleUnknownGenerations": [{"pid": p, "creationUtc": c, "name": n} for p, c, n in sorted(unbound)],
        }
        for key, value in (("cpuOneLogicalProcessorPercent", cpu), ("meanRssMiB", rss)):
            assert abs(value - Decimal(str(comparison["versions"][version][key]))) < Decimal("0.000000001")
    assert int(read(source / "baseline019-exit.json")["kernelTimes"]["exitFileTime"]) < ticks(read(source / "idle-candidate0110-launch.json")["creationTimeUtc"])
    protection = read(source / "final-protection.json")
    restored = read(source / "fixture-restored.json")
    assert protection["nativeCount"] == 0 and protection["restored"] and restored["originalDirectoryRestored"]
    assert not restored["dataDeleted"] and not restored["registryOrAclChanged"]
    original = protection["profiles"][1]
    assert original["identity"]["fileId"] == contract["baselineOriginalDirectoryId"]
    assert original["identity"]["creationFileTime"] == contract["baselineOriginalDirectoryCreation"]
    assert original["preferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert protection["profiles"][0]["preferencesSha256"] == "2d612148a55152e05047d1f13c4587a3b75f2740b1a51cc2c0f64f30a767495e"
    assert protection["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert protection["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    baseline, candidate = results["baseline019"], results["candidate0110"]
    cpu_delta = candidate["cpuOneLogicalProcessorPercent"] - baseline["cpuOneLogicalProcessorPercent"]
    rss_delta = candidate["meanRssMiB"] - baseline["meanRssMiB"]
    assert abs(cpu_delta - comparison["cpuIncreasePercentagePoints"]) < 1e-9
    assert abs(rss_delta - comparison["meanRssIncreaseMiB"]) < 1e-9
    assert cpu_delta > 0.5 and rss_delta > 10
    assert not comparison["cpuBudgetPassed"] and not comparison["meanRssBudgetPassed"]
    for row in read(ROOT / "copies.json"):
        assert sha(ROOT / row["path"]) == row["sha256"]
        assert (ROOT / row["path"]).read_bytes() == Path(row["runtimePath"]).read_bytes()
    for row in read(ROOT / "source-bindings.json"):
        bound = PROJECT / row["path"]
        assert sha(bound / "manifest.json") == row["manifestSha256"]
        assert sha(bound / "verification.json") == row["verificationSha256"]
    return {
        "status": "valid-completed-idle-pair-both-original-budgets-failed", "versions": results,
        "cpuIncreasePercentagePoints": cpu_delta, "meanRssIncreaseMiB": rss_delta,
        "roleRssDifferenceMiB": {
            role: candidate["meanRssMiBByActualRole"].get(role, 0) - baseline["meanRssMiBByActualRole"].get(role, 0)
            for role in sorted(set(baseline["meanRssMiBByActualRole"]) | set(candidate["meanRssMiBByActualRole"]))
        },
        "bothNormalExitsWithinHumanWindow": True, "configurationRestored": True,
        "decimalArithmeticMatchesOriginalAnalysis": True, "causalAllocationAttribution": False,
        "priorSamplesUsed": False, "fullPerformanceGatePassed": False, "historicalFailuresCleared": False,
        "taskClosed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    result = verify()
    assert result == read(ROOT / "verification.json")
    for item in read(ROOT / "manifest.json")["files"]:
        assert sha(ROOT / item["path"]) == item["sha256"]
    print(json.dumps(result))
    sys.exit(0)

if "--finish-interrupted-archive" in sys.argv:
    assert (ROOT / "copies.json").exists() and not (ROOT / "manifest.json").exists()
    assert not (ROOT / "verification.json").exists()
    write(ROOT / "archival-correction.json", {
        "phase": "offline-evidence-archive", "originalError": "KeyError: (34556, 134366824262266676)",
        "reason": "A child generation observed by the family collector started after the role snapshot.",
        "correction": "Keep its original counters; report its role as unknown rather than infer a role.",
        "nativeAttemptRepeated": False, "rawCountersModified": False,
        "metricFormulaOrBudgetChanged": False, "originalComparisonChanged": False,
    })
    write(ROOT / "verification.json", verify())
    (ROOT / ".gitattributes").write_bytes(b"* -text\n")
    write(ROOT / "manifest.json", {"files": [{
        "path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p),
    } for p in sorted(ROOT.rglob("*")) if p.is_file()]})
    print(json.dumps(read(ROOT / "verification.json")))
    sys.exit(0)

assert not (ROOT / "copies.json").exists()
names = ["human-window.json", "fixture-ready.json", "fixture-restored.json", "foreground-released.json",
         "final-protection.json", "run-observation.ps1", "comparison.json"]
for version in ("baseline019", "candidate0110"):
    names += [f"idle-{version}-launch.json", f"{version}-sky-settled-ready.json",
              f"{version}-settled-roles.original.json", f"{version}-settled-roles-attempt.json",
              f"{version}-exit-observer-ready.json", f"{version}-exit.json", f"{version}-close-action.json",
              f"{version}-collector-invocation.json", f"{version}-collector-exit.json",
              f"{version}-collector.stdout.original.log", f"{version}-collector.stderr.original.log",
              f"{version}/identity.json", f"{version}/terminal.json", f"{version}/progress.json",
              f"{version}/samples.original.jsonl"]
copies = []
for name in names:
    target = ROOT / "original" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    data = (RUN / name).read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig"))
    with target.open("xb") as stream:
        stream.write(data)
    copies.append({"path": "original/" + name, "runtimePath": str(RUN / name), "bytes": len(data),
                   "sha256": hashlib.sha256(data).hexdigest()})
write(ROOT / "copies.json", copies)
write(ROOT / "source-bindings.json", [{
    "path": directory.relative_to(PROJECT).as_posix(), "manifestSha256": sha(directory / "manifest.json"),
    "verificationSha256": sha(directory / "verification.json"),
} for directory in (PREP, ROOT.parent / "OCX-0110-20261017-WIN-CE-CANDIDATE")])
write(ROOT / "verification.json", verify())
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
write(ROOT / "manifest.json", {"files": [{
    "path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p),
} for p in sorted(ROOT.rglob("*")) if p.is_file()]})
print(json.dumps(read(ROOT / "verification.json")))
