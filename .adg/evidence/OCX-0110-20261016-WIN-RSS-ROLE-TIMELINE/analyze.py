"""Offline role timeline analysis over the already frozen 597+597 samples."""
import hashlib
import json
import statistics
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-b4-pair-20261016T204338")
PRIOR = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def analyze(version):
    roles = read(RUNTIME / f"{version}-settled-roles.original.json")
    lookup = {(m["pid"], m["creationUtc"]): m["role"] for m in roles["members"]}
    sample_path = RUNTIME / version / "samples.original.jsonl"
    raw = sample_path.read_bytes()
    rows = [json.loads(line) for line in raw.decode("utf-8-sig").splitlines()]
    values = {role: [] for role in sorted(set(lookup.values()))}
    times = []
    for row in rows:
        times.append(row["elapsedSeconds"])
        sums = {role: 0.0 for role in values}
        for member in row["members"]:
            role = lookup.get((member["pid"], member["creationUtc"]))
            if role:
                sums[role] += member["workingSetBytes"] / 1048576
        for role, value in sums.items():
            values[role].append(value)
    assert len(rows) == 597 and all(len(series) == 597 for series in values.values())
    assert times[-1] - times[0] >= 600
    window = 60
    summary = {}
    for role, series in values.items():
        summary[role] = {
            "meanMiB": statistics.mean(series),
            "firstSampleMiB": series[0],
            "lastSampleMiB": series[-1],
            "minMiB": min(series),
            "maxMiB": max(series),
            "first60MeanMiB": statistics.mean(series[:window]),
            "last60MeanMiB": statistics.mean(series[-window:]),
            "slopeMiBPerMinute": (series[-1] - series[0]) / (times[-1] - times[0]) * 60,
        }
    return {
        "version": version,
        "source": {
            "baseline019": "de6862d40389d527527b9db8ef52e186a58b625f",
            "candidate0110": "b4d487e4e04cd4c80fd5570663e1ed21dd483b19",
        }[version],
        "samples": len(rows),
        "spanSeconds": times[-1] - times[0],
        "sampleFile": str(sample_path),
        "sampleFileSha256": sha(raw),
        "roleFile": str(RUNTIME / f"{version}-settled-roles.original.json"),
        "roleFileSha256": sha((RUNTIME / f"{version}-settled-roles.original.json").read_bytes()),
        "roles": summary,
    }


baseline = analyze("baseline019")
candidate = analyze("candidate0110")
assert baseline["source"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert candidate["source"] == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
role_delta = {}
for role in sorted(set(baseline["roles"]) | set(candidate["roles"])):
    role_delta[role] = {
        "meanMiB": candidate["roles"][role]["meanMiB"] - baseline["roles"][role]["meanMiB"],
        "first60MeanMiB": candidate["roles"][role]["first60MeanMiB"] - baseline["roles"][role]["first60MeanMiB"],
        "last60MeanMiB": candidate["roles"][role]["last60MeanMiB"] - baseline["roles"][role]["last60MeanMiB"],
        "startToEndDeltaMiB": (
            candidate["roles"][role]["lastSampleMiB"] - candidate["roles"][role]["firstSampleMiB"]
            - baseline["roles"][role]["lastSampleMiB"] + baseline["roles"][role]["firstSampleMiB"]
        ),
    }
value = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "kind": "offline-role-timeline",
    "priorEvidence": str(PRIOR),
    "priorManifestSha256": sha((PRIOR / "manifest.json").read_bytes()),
    "priorVerificationSha256": sha((PRIOR / "verification.json").read_bytes()),
    "baseline": baseline,
    "candidate": candidate,
    "roleDeltaCandidateMinusBaseline": role_delta,
    "gpuMeanDeltaMiB": role_delta["gpu"]["meanMiB"],
    "gpuFirst60DeltaMiB": role_delta["gpu"]["first60MeanMiB"],
    "gpuLast60DeltaMiB": role_delta["gpu"]["last60MeanMiB"],
    "candidateGpuStartToEndMiB": candidate["roles"]["gpu"]["lastSampleMiB"] - candidate["roles"]["gpu"]["firstSampleMiB"],
    "baselineGpuStartToEndMiB": baseline["roles"]["gpu"]["lastSampleMiB"] - baseline["roles"]["gpu"]["firstSampleMiB"],
    "roleAccountingMatchesPriorRss": abs(sum(row["meanMiB"] for row in role_delta.values()) - 20.040155203098834) < 1e-9,
    "interpretation": "The candidate GPU role starts higher and does not grow monotonically across the frozen span; this narrows the observation toward startup/resource composition but does not identify an allocation cause.",
    "formalRssBudgetMiB": 10,
    "formalRssIncreaseMiB": 20.040155203098834,
    "formalRssBudgetPassed": False,
    "individualAllocationAttributed": False,
    "queryReplayRequested": False,
    "newProductStarted": False,
}
(ROOT / "role-timeline-analysis.json").write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
print(json.dumps({
    "gpuMeanDeltaMiB": value["gpuMeanDeltaMiB"],
    "gpuFirst60DeltaMiB": value["gpuFirst60DeltaMiB"],
    "gpuLast60DeltaMiB": value["gpuLast60DeltaMiB"],
    "candidateGpuStartToEndMiB": value["candidateGpuStartToEndMiB"],
    "baselineGpuStartToEndMiB": value["baselineGpuStartToEndMiB"],
    "formalRssBudgetPassed": False,
}))
