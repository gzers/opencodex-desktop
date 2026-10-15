"""Verify the two unsuccessful attempts without turning them into a valid pair."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
FIRST = ROOT.parent / "OCX-0110-20261016-WIN-IDLE-8BE-019"


def read(root, name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))


def verify_manifest(root):
    files = {p.relative_to(root).as_posix(): p for p in root.rglob("*")
             if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
    manifest = read(root, "manifest.json")
    assert set(files) == {entry["path"] for entry in manifest["files"]}
    for entry in manifest["files"]:
        data = files[entry["path"]].read_bytes()
        assert len(data) == entry["bytes"]
        assert hashlib.sha256(data).hexdigest() == entry["sha256"]
    return len(files)


first_count = verify_manifest(FIRST)
assert not list(FIRST.glob("*-samples.original.jsonl"))
assert not (FIRST / "comparison.json").exists()
first_stop = read(FIRST, "controlled-stop.original.json")
assert first_stop["exitObserved"] and first_stop["exitCode"] == 0
assert not first_stop["forcedTermination"] and first_stop["samplesCollected"] == 0
assert not first_stop["productCrashClaimed"]
first_result = {"status": "verified-readiness-method-failure",
                "files": first_count, "measuredVersions": 0,
                "productNormalExit": True, "fullPerformanceGatePassed": False}

settled_count = verify_manifest(ROOT)
terminal = read(ROOT, "terminal.json")
assert terminal["completed"] == ["baseline019"]
assert not terminal["measurementCompleted"]
assert not terminal["fullPerformanceGatePassed"]
assert not (ROOT / "comparison.json").exists()
baseline = read(ROOT, "baseline019-measurement.json")
baseline_rows = [json.loads(line) for line in
                 (ROOT / "baseline019-samples.original.jsonl")
                 .read_text(encoding="utf-8").splitlines()]
baseline_span = baseline_rows[-1]["elapsedSeconds"] - baseline_rows[0]["elapsedSeconds"]
assert baseline_span == baseline["observedCounterSpanSeconds"] >= 600
assert baseline["samples"] == len(baseline_rows)
assert baseline["childDiscoveryReadFailures"] == 0
assert all(r["foreground"] and r["visible"] and not r["minimized"]
           and r["nativeCount"] == 1 for r in baseline_rows)
baseline_stop = read(ROOT, "baseline019-exit.json")
assert baseline_stop["exitObserved"] and baseline_stop["exitCode"] == 0
assert not baseline_stop["forcedTermination"]

candidate_rows = [json.loads(line) for line in
                  (ROOT / "candidate0110-samples.original.jsonl")
                  .read_text(encoding="utf-8").splitlines()]
candidate_span = candidate_rows[-1]["elapsedSeconds"] - candidate_rows[0]["elapsedSeconds"]
assert candidate_span < 600 and not candidate_rows[-1]["foreground"]
stop = read(ROOT, "interrupted-normal-stop.json")
assert stop["ownedWindowVerified"] and stop["closeMessageSent"]
assert stop["exitObserved"] and stop["exitCode"] == 0
assert not stop["forcedTermination"]
for rows in (baseline_rows, candidate_rows):
    assert all(b["elapsedSeconds"] > a["elapsedSeconds"] for a, b in zip(rows, rows[1:]))
    for row in rows:
        assert row["cpuSeconds"] == sum(m["cpuSeconds"] for m in row["members"])
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])

result = {"status": "verified-completed-baseline-and-interrupted-candidate",
          "files": settled_count, "baselineCounterSpanSeconds": baseline_span,
          "baselineSamples": len(baseline_rows),
          "candidateCounterSpanSeconds": candidate_span,
          "candidateSamples": len(candidate_rows),
          "candidateNormalExit": True, "productCrashClaimed": False,
          "candidateMeasurementValid": False, "fullPerformanceGatePassed": False}
for root, outcome in ((FIRST, first_result), (ROOT, result)):
    (root / "verification.json").write_text(json.dumps(outcome, indent=2) + "\n",
                                            encoding="utf-8")
print(json.dumps({"first": first_result, "settled": result}))
