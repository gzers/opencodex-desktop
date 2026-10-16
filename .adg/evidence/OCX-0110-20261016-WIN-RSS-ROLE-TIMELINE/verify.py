"""Verify the role timeline against the frozen raw sample bytes."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


value = read("role-timeline-analysis.json")
assert value["kind"] == "offline-role-timeline"
assert value["baseline"]["source"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert value["candidate"]["source"] == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
assert value["baseline"]["samples"] == value["candidate"]["samples"] == 597
assert value["baseline"]["spanSeconds"] >= 600
assert value["candidate"]["spanSeconds"] >= 600
for version in ["baseline", "candidate"]:
    row = value[version]
    sample = Path(row["sampleFile"])
    roles = Path(row["roleFile"])
    assert sample.read_bytes() and sha(sample.read_bytes()) == row["sampleFileSha256"]
    assert sha(roles.read_bytes()) == row["roleFileSha256"]
    assert row["roles"]["gpu"]["meanMiB"] > 0
assert value["roleAccountingMatchesPriorRss"]
assert value["gpuMeanDeltaMiB"] > 20
assert value["gpuFirst60DeltaMiB"] > 20
assert value["gpuLast60DeltaMiB"] > 15
assert value["candidateGpuStartToEndMiB"] < 0
assert value["baselineGpuStartToEndMiB"] > 0
assert value["formalRssIncreaseMiB"] == 20.040155203098834
assert value["formalRssBudgetMiB"] == 10
assert not value["formalRssBudgetPassed"]
assert not value["individualAllocationAttributed"]
print(json.dumps({
    "status": "verified-frozen-role-timeline",
    "baselineSamples": value["baseline"]["samples"],
    "candidateSamples": value["candidate"]["samples"],
    "gpuMeanDeltaMiB": value["gpuMeanDeltaMiB"],
    "gpuFirst60DeltaMiB": value["gpuFirst60DeltaMiB"],
    "gpuLast60DeltaMiB": value["gpuLast60DeltaMiB"],
    "candidateGpuStartToEndMiB": value["candidateGpuStartToEndMiB"],
    "baselineGpuStartToEndMiB": value["baselineGpuStartToEndMiB"],
    "formalRssBudgetPassed": False,
    "allocationCauseAttributed": False,
}))
