"""Recompute captured allocation/stack pairs and post-capture module matches."""
import argparse
from collections import Counter, defaultdict
from datetime import datetime
import hashlib
import json
from pathlib import Path


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def instant(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def analyze(root):
    capture = read(root / "gpu-capture/result.json")
    raw = (root / "gpu-capture/owned-memory.events.jsonl").read_bytes()
    events = [json.loads(line) for line in raw.splitlines()]
    assert capture["passed"] and capture["boundGpuObservation"] and not capture["controlledSelf"]
    assert len(events) == capture["records"] == 581 and len(raw) == capture["serializedBytes"] == 269538
    assert all(row["pid"] == capture["targetPid"] for row in events)
    allocations = [e for e in events if e["kind"] == "virtual-allocation"]
    stacks = [e for e in events if e["kind"] == "allocation-stack"]
    frees = [e for e in events if e["kind"] == "virtual-free"]
    assert len(allocations) == len(stacks) == 231 and len(frees) == 119
    by_key = {(s["tid"], s["eventRelativeMilliseconds"]): s for s in stacks}
    assert len(by_key) == len(stacks)
    modules = read(root / "gpu-stack-module-map.original.json")
    assert modules["targetPid"] == capture["targetPid"]
    assert not modules["atomicWithCapture"] and not modules["kernelAddressesResolved"]
    action = read(root / "overview-return-action.json")
    settled = read(root / "overview-during-recording.original.json")
    ready = read(root / "gpu-capture/recording-ready.json")
    assert instant(capture["startedAtUtc"]) <= instant(ready["atUtc"]) < instant(action["startedAtUtc"])
    assert instant(action["returnedAtUtc"]) < instant(settled["observedAtUtc"]) < instant(capture["finishedAtUtc"])
    assert "#overview" in settled["accessibility"]["tree"]
    segments = defaultdict(lambda: {"allocationEvents": 0, "eventLengthSum": 0, "freeEvents": 0})
    inclusive = defaultdict(lambda: {"stackPairs": 0, "eventLengthSum": 0})
    exclusive = defaultdict(lambda: {"stackPairs": 0, "eventLengthSum": 0})
    patterns = Counter()
    resolved = []
    all_ips = set()
    mapped_ips = set()
    kernel_ips = set()

    def phase(at):
        if instant(at) < instant(action["startedAtUtc"]):
            return "before-return-action"
        if instant(at) <= instant(settled["observedAtUtc"]):
            return "return-action-through-first-overview-observation"
        return "after-first-overview-observation"

    for allocation in allocations:
        stack = by_key[(allocation["tid"], allocation["relativeMilliseconds"])]
        assert stack["frameCount"] == len(stack["instructionPointersHex"]) <= 128
        assert stack["boundGpuFramesOnly"] and not stack["ownedControllerFramesOnly"]
        frames = []
        names = []
        for address in stack["instructionPointersHex"]:
            value = int(address, 16)
            all_ips.add(address)
            matches = [m for m in modules["modules"]
                       if int(m["baseAddressHex"], 16) <= value <
                       int(m["baseAddressHex"], 16) + m["imageBytes"]]
            assert len(matches) <= 1
            if matches:
                module = matches[0]
                mapped_ips.add(address)
                frames.append({"module": module["name"],
                               "moduleOffsetHex": format(value - int(module["baseAddressHex"], 16), "x")})
                if not names or names[-1] != module["name"]:
                    names.append(module["name"])
            else:
                is_kernel_range = value >= 0xFFFF000000000000
                if is_kernel_range:
                    kernel_ips.add(address)
                frames.append({"module": None, "kernelAddressRange": is_kernel_range,
                               "instructionPointerHex": address})
        lower_names = {name.lower() for name in names}
        for name in set(names):
            inclusive[name]["stackPairs"] += 1
            inclusive[name]["eventLengthSum"] += allocation["length"]
        if "igd10um64xe.dll" in lower_names:
            group = "contains-recorded-driver-module-range"
        elif "d3d11.dll" in lower_names or "dxgi.dll" in lower_names or "dxcore.dll" in lower_names:
            group = "contains-graphics-runtime-range-without-recorded-driver-range"
        elif "uiautomationcore.dll" in lower_names:
            group = "contains-uiautomation-range-without-graphics-range"
        else:
            group = "other-observed-module-ranges"
        exclusive[group]["stackPairs"] += 1
        exclusive[group]["eventLengthSum"] += allocation["length"]
        patterns[tuple(names)] += 1
        segment = segments[phase(allocation["atUtc"])]
        segment["allocationEvents"] += 1
        segment["eventLengthSum"] += allocation["length"]
        resolved.append({"atUtc": allocation["atUtc"], "tid": allocation["tid"],
                         "relativeMilliseconds": allocation["relativeMilliseconds"],
                         "length": allocation["length"], "flags": allocation["flags"],
                         "phase": phase(allocation["atUtc"]), "classification": group,
                         "moduleSequence": names, "frames": frames})
    for free in frees:
        segments[phase(free["atUtc"])]["freeEvents"] += 1
    assert len(all_ips) == modules["distinctCapturedIps"] == 744
    for module in modules["modules"]:
        count = sum(int(module["baseAddressHex"], 16) <= int(ip, 16) <
                    int(module["baseAddressHex"], 16) + module["imageBytes"] for ip in all_ips)
        assert count == module["matchedDistinctIps"]
    before = read(root / "settings-roles.original.json")
    after = read(root / "overview-after-roles.original.json")
    before_gpu, = [m for m in before["members"] if m["role"] == "gpu"]
    after_gpu, = [m for m in after["members"] if m["role"] == "gpu"]
    assert before_gpu["pid"] == after_gpu["pid"] == capture["targetPid"]
    assert before_gpu["creationUtc"] == after_gpu["creationUtc"]
    counters_before, = [r for r in before["gpuCounterInstances"] if r["pid"] == capture["targetPid"]]
    counters_after, = [r for r in after["gpuCounterInstances"] if r["pid"] == capture["targetPid"]]
    summary = {
        "status": "verified-actual-bound-gpu-allocation-stack-observation",
        "targetPid": capture["targetPid"], "rootPid": capture["rootPid"],
        "captureStartedAtUtc": capture["startedAtUtc"], "captureFinishedAtUtc": capture["finishedAtUtc"],
        "readyAtUtc": ready["atUtc"], "returnActionAtUtc": action["startedAtUtc"],
        "firstOverviewObservationAtUtc": settled["observedAtUtc"],
        "eventsSha256": hashlib.sha256(raw).hexdigest(),
        "records": len(events), "serializedBytes": len(raw),
        "allocationStackPairs": len(resolved), "freeEvents": len(frees),
        "allocationEventLengthSum": sum(a["length"] for a in allocations),
        "freeEventLengthSum": sum(f["length"] for f in frees),
        "allocationFlagsCounts": dict(Counter(str(a["flags"]) for a in allocations)),
        "phases": dict(segments), "inclusiveModulePresence": dict(inclusive),
        "exclusiveModuleGroups": dict(exclusive),
        "largestAllocations": [{k: row[k] for k in ("atUtc", "length", "tid", "phase", "moduleSequence")}
                               for row in sorted(resolved, key=lambda r: -r["length"])[:12]],
        "topModuleSequences": [{"sequence": list(seq), "stackPairs": count}
                               for seq, count in patterns.most_common(12)],
        "distinctInstructionPointers": len(all_ips), "matchedPostCaptureModuleRanges": len(mapped_ips),
        "unresolvedKernelRangeIps": len(kernel_ips),
        "unmappedNonKernelIps": len(all_ips - mapped_ips - kernel_ips),
        "moduleMapCapturedAtUtc": modules["finishedAtUtc"],
        "moduleMapIsEventTimeLoaderProof": False, "symbolFunctionNamesResolved": False,
        "gpuPointWorkingSetBefore": before_gpu["workingSetBytes"],
        "gpuPointWorkingSetAfter": after_gpu["workingSetBytes"],
        "gpuPointWorkingSetDifference": after_gpu["workingSetBytes"] - before_gpu["workingSetBytes"],
        "gpuPointSharedUsageBefore": counters_before["SharedUsage"],
        "gpuPointSharedUsageAfter": counters_after["SharedUsage"],
        "pointSnapshotsAreFormalIdleAverage": False,
        "lengthSumsAreNetCommittedOrResidentMemory": False,
        "allocationAndFreeIdentityCanBeMatched": False,
        "inclusiveModuleGroupsAreAdditive": False, "observationOverheadAbsent": False,
        "allEventsAreCausedByWebgl": False, "rssDifferenceCauseEstablished": False,
        "originalRssFailureCleared": False, "fullProductGatePassed": False,
    }
    return summary, resolved


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, default=Path(__file__).resolve().parent / "original")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    summary, _ = analyze(args.input)
    if args.output:
        with args.output.open("x", encoding="utf-8") as out:
            json.dump(summary, out, ensure_ascii=False, indent=2)
            out.write("\n")
    print(json.dumps(summary))
