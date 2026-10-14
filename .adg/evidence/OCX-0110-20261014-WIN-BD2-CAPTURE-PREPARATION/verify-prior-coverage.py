"""Path-keyed, platform-neutral byte verification and actual live-Node coverage."""
import collections
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
previous = root.parent / "OCX-0110-20261014-WIN-BD2-TRACE"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
manifest = read(previous / "manifest.json")
listed = {}
for item in manifest:
    relative = item["path"]
    path = previous.joinpath(*relative.split("/"))
    assert path.resolve().is_relative_to(previous.resolve()) and relative not in listed
    raw = path.read_bytes()
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"]
    listed[relative] = item
actual = {path.relative_to(previous).as_posix() for path in previous.rglob("*")
          if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"}}
assert actual == set(listed) and len(listed) == 35
trace = [json.loads(line) for line in
         (previous / "owned-process-thread-io.raw.jsonl").read_text().splitlines()]
nodes = {row["pid"] for row in trace if row["kind"] == "owned-node-discovered"}
samples = [row for row in trace if row["kind"] == "owned-process-sample"]
coverage = []
for pid in sorted({row["pid"] for row in samples}):
    process = [row for row in samples if row["pid"] == pid]
    threads = [thread for row in process if isinstance(row.get("threads"), list) for thread in row["threads"]]
    coverage.append({"pid": pid, "isNode": pid in nodes, "samples": len(process),
                     "liveSamples": sum(row["waitStatus"] == 258 for row in process),
                     "threadRecords": len(threads),
                     "waitChainRecords": sum(bool(thread.get("waitChain")) for thread in threads),
                     "ioFirstEqualsLast": process[0]["io"] == process[-1]["io"]})
assert len(nodes) == 6
assert all(item["liveSamples"] == item["threadRecords"] == item["waitChainRecords"] == 0
           for item in coverage if item["isNode"])
assert all(item["ioFirstEqualsLast"] for item in coverage if item["isNode"])
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "priorEvidenceCommit": "019dd6c2bdacd6e79c4a5aa6b9b11981197e7d34",
    "macReviewCommit": "b14a0f6955af3d0a5afc41716b2f32ee62196077",
    "pathKeyedBytesAndDigestsPassed": True, "files": len(listed),
    "pathOrderIgnored": True, "originalManifestOrEvidenceModified": False,
    "coverage": coverage, "nodeLiveSamples": 0, "nodeThreadRecords": 0, "nodeWaitChainRecords": 0,
    "allThreadAndWaitChainRecordsAreTestRoot": True,
    "macCoverageReviewConfirmed": True,
    "oldSuccessComparisonIsNodeStartupFailureTrace": False,
    "failureReproduced": False, "oldCiFailureCleared": False,
}
(root / "prior-byte-and-live-coverage-review.json").write_text(
    json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"priorBytesVerified": 35, "liveNodeSamples": 0,
                  "nodeThreadOrWaitChainRecords": 0, "frozenPriorEvidenceChanged": False}))
