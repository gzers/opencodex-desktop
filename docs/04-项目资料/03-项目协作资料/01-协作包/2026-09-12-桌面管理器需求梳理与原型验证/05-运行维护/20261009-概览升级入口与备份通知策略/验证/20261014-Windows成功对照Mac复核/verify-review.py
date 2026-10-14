"""Read-only, path-keyed review of the Windows bd2 success comparison.

No execution of Windows programs, mutation of sealed evidence, or gate promotion.
Run on macOS or Windows; output is deterministic for the frozen input bytes.
"""
import collections
import hashlib
import json
import re
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath, PureWindowsPath

REPO = next(p for p in Path(__file__).resolve().parents if (p / "AGENTS.md").is_file())
SOURCE = REPO / ".adg/evidence/OCX-0110-20261014-WIN-BD2-TRACE"
SHA = "bd2f6eb10d58115235e35d575507bb69f44666da"
DOC_SHA = "019dd6c2bdacd6e79c4a5aa6b9b11981197e7d34"


def read(name):
    return json.loads((SOURCE / name).read_text(encoding="utf-8-sig"))


def index(entries):
    result = {}
    for entry in entries:
        path = PurePosixPath(entry["path"])
        assert not path.is_absolute() and ".." not in path.parts
        assert entry["path"] not in result, "Duplicate evidence path"
        result[entry["path"]] = entry
    return result


manifest = read("manifest.json")
files = [p for p in SOURCE.rglob("*") if p.is_file() and
         p.name not in {"manifest.json", "archive-verification.json"}]
assert all(not p.is_symlink() for p in files)
actual = [{"path": p.relative_to(SOURCE).as_posix(), "bytes": p.stat().st_size,
           "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
assert index(manifest) == index(actual), "Missing, extra, or changed evidence bytes"
assert len(manifest) == 35
names = [entry["path"] for entry in manifest]
assert names == sorted(names, key=PureWindowsPath)
assert names != sorted(names, key=PurePosixPath)

result = read("single-run-result.json")
analysis = read("owned-trace-analysis.json")
admission = read("test-artifact-admission.json")
assert result["source"] == analysis["source"] == admission["source"] == SHA
assert admission["runId"] == 38537161445
assert admission["testExecutableSha256"] == result["testBinarySha256"]
assert admission["artifactApiSourceMatches"] and admission["runnerTestExecutableHashMatches"]
assert admission["independentHeadCheckoutLogMatches"] and admission["zipCrcPassed"]
assert not admission["oldFailureCleared"]
assert result["arguments"] == [] and result["defaultConcurrencyPreserved"]
assert result["testCapturePreserved"] and not result["nodePrewarmed"]
assert result["testExitCode"] == 101 and not result["forcedOuterCleanup"]
assert result["protectedSandboxBefore"] == result["protectedSandboxAfter"]
assert result["dailyExeBefore"] == result["dailyExeAfter"]
assert not result["desktopStarted"] and not result["wprOrSchedulingEtwRecorded"]
assert not read("final-owned-census.json")["formalMacNativeHandoffIssued"]

raw = (SOURCE / "owned-process-thread-io.raw.jsonl").read_bytes()
assert hashlib.sha256(raw).hexdigest() == analysis["rawTraceSha256"]
rows = [json.loads(line) for line in raw.decode().splitlines()]
counts = dict(collections.Counter(row["kind"] for row in rows))
assert len(rows) == 836 and counts == analysis["rawRecordCounts"]
samples = [row for row in rows if row["kind"] == "owned-process-sample"]
nodes = [row for row in rows if row["kind"] == "owned-node-discovered"]
root_pid = rows[0]["rootPid"]
node_ids = {row["pid"] for row in nodes}
assert len(nodes) == len(node_ids) == 6 and root_pid not in node_ids
assert all(row["imageAndCreationVerified"] for row in nodes)
assert {row["pid"] for row in samples} == node_ids | {root_pid}


def filetime(iso):
    match = re.fullmatch(r"(.+)\.(\d{1,7})Z", iso)
    assert match, iso
    base = datetime.fromisoformat(match[1]).replace(tzinfo=timezone.utc)
    delta = base - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return (delta.days * 86400 + delta.seconds) * 10_000_000 + int(match[2].ljust(7, "0"))


coverage = []
for pid in [root_pid] + sorted(node_ids):
    observed = [row for row in samples if row["pid"] == pid]
    threads = [t for row in observed if isinstance(row.get("threads"), list) for t in row["threads"]]
    assert len({row["creationFileTime100ns"] for row in observed}) == 1
    assert all(row["retainedHandle"] and row["timesOk"] for row in observed)
    if pid in node_ids:
        assert all(row["waitStatus"] == 0 and row["exitCodeOk"] and
                   row["exitCode"] != 259 and row["exitFileTime100ns"] > 0 for row in observed)
        assert not threads and all(row.get("threads") is None for row in observed)
        assert len({json.dumps(row["io"], sort_keys=True) for row in observed}) == 1
        assert all(row["ioOk"] for row in observed)
        after_exit = (filetime(observed[0]["atUtc"]) - observed[0]["exitFileTime100ns"]) / 10_000
        assert after_exit > 0
    else:
        after_exit = None
    coverage.append({
        "pid": pid, "isNode": pid in node_ids, "samples": len(observed),
        "waitStatusCounts": dict(collections.Counter(str(row["waitStatus"]) for row in observed)),
        "threadRecords": len(threads), "wctRecords": sum(bool(t.get("waitChain")) for t in threads),
        "firstNodeSampleAfterExitMs": round(after_exit, 4) if after_exit is not None else None,
    })
threads = [t for row in samples if isinstance(row.get("threads"), list) for t in row["threads"]]
chains = [t["waitChain"] for t in threads if t.get("waitChain")]
assert len(threads) == 2664 and len(chains) == 80
assert dict(collections.Counter(f"{c['ok']}:{c['error']}" for c in chains)) == analysis["wctOutcomeCounts"]
assert all(not row.get("threads") for row in samples if row["pid"] in node_ids)
stdout = (SOURCE / "test-stdout.original.log").read_text(encoding="utf-8-sig")
assert "529 passed; 1 failed; 4 ignored; 0 measured; 0 filtered out" in stdout
assert len(analysis["queryFixtureCasesPassed"]) == 7
assert all(f"test {name} ... ok" in stdout for name in analysis["queryFixtureCasesPassed"])
assert "---- commands::workspace::tests::local_documents_are_frozen stdout ----" in stdout
assert "assertion failed: license.opened" in stdout

print(json.dumps({
    "sourceSha": SHA, "evidenceDocsSha": DOC_SHA,
    "manifest": {"pathKeyedBytesAndSha256": "passed", "fileCount": len(manifest),
                 "frozenManifestSha256": hashlib.sha256((SOURCE / "manifest.json").read_bytes()).hexdigest(),
                 "windowsPathOrderMatches": True, "posixPathOrderMatches": False,
                 "legacyMacStrictListComparison": "failed: order-only; original unchanged"},
    "rawTraceSha256": hashlib.sha256(raw).hexdigest(), "rawCounts": counts,
    "processCoverage": coverage, "allThreadAndWctRecordsBelongToTestRoot": True,
    "nodeLiveSamples": 0, "nodeLiveThreadRecords": 0, "nodeLiveWctRecords": 0,
    "nodeIoIntervalProgressNotObserved": True, "queryFixtureCasesPassedLocally": 7,
    "wholeLibrary": {"passed": 529, "failed": 1, "ignored": 4, "exitCode": 101},
    "failedSceneObtained": False, "rootCauseIdentified": False,
    "oldCiFailureCleared": False, "macPerformanceFailureCleared": False,
    "fullGatePromoted": False, "macNativeStarted": False, "originalEvidenceModified": False,
}, ensure_ascii=False, indent=2))
