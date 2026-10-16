"""Verify a bounded interpretation audit without promoting the old replay."""
import collections
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OLD = ROOT.parent / "OCX-0110-20261016-WIN-QUERY-THREAD-RUN"
CAPTURE = OLD / "remote-original/runtime/ci-bd2-query-etw/capture"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = read(ROOT / "manifest.json")
files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(files) == {row["path"] for row in manifest["files"]}
for row in manifest["files"]:
    assert files[row["path"]].stat().st_size == row["bytes"] and sha(files[row["path"]]) == row["sha256"]
audit = read(ROOT / "corpus-audit.json")
reflection = read(ROOT / "pinned-decoder-reflection.json")
access = read(ROOT / "getter-access.json")
assert audit["decoderSha256"] == reflection["decoderSha256"] == "401e948dac99d0f9756ef96d5a8b8e4736e19e9f82ce8cdd744495951e3a94bb"
assert audit["referenceHashes"] == {
    "priorManifest": sha(OLD / "manifest.json"), "priorAnalysis": sha(OLD / "analysis.json"),
    "rawEvents": sha(CAPTURE / "owned-etw.events.jsonl"), "captureResult": sha(CAPTURE / "capture-result.json")}
properties = {p["property"]: p for p in reflection["properties"]}
valid_reason = {e["value"] for e in properties["OldThreadWaitReason"]["enumValues"]}
valid_mode = {e["value"] for e in properties["OldThreadWaitMode"]["enumValues"]}
assert valid_mode == {0, 1}
assert {p["property"]: p["byteOffset"] for p in access["properties"]} == {
    "OldThreadWaitReason": 12, "OldThreadWaitMode": 13, "OldThreadState": 14}
assert all(p["callName"] == "GetByteAt" and p["signature"] == "Int32 GetByteAt(Int32)"
           and not p["versionConditionalInGetter"] and not p["bitMaskInGetter"] for p in access["properties"])
counts = collections.Counter()
for line in (CAPTURE / "owned-etw.events.jsonl").read_text(encoding="utf-8-sig").splitlines():
    row = json.loads(line)
    if row.get("kind") != "owned-thread-scheduled-out":
        continue
    name = "named-root-query-threads" if row["pid"] == 6688 else "owned-descendants"
    assert row["original"]["version"] == 5 and not row["versionSpecificWaitSemanticsVerified"]
    counts[(name, "total")] += 1
    counts[(name, "reason")] += row["waitReasonDecodedRaw"] not in valid_reason
    counts[(name, "mode")] += row["waitModeDecodedRaw"] not in valid_mode
for name in ("named-root-query-threads", "owned-descendants"):
    group = audit["groups"][name]
    assert counts[(name, "total")] == group["count"]
    assert counts[(name, "reason")] == group["outsidePinnedReasonEnum"]
    assert counts[(name, "mode")] == group["outsidePinnedModeEnum"]
assert counts[("named-root-query-threads", "total")] == 564
assert counts[("named-root-query-threads", "reason")] == 545
assert counts[("named-root-query-threads", "mode")] == 42
assert counts[("owned-descendants", "total")] == 3095
assert audit["fullRawCswitchPayloadRecords"] == 0 and not audit["etlAvailableInPreservedCapture"]
assert audit["doNotInferMaskOrWaitCause"] and audit["managedEnumMembershipIsNotKernelSchemaValidation"]
assert not audit["v5SemanticsVerified"] and not audit["actualFailedQueryWindowObtained"]
assert not audit["newRecordingStarted"] and not audit["originalLibraryReplayed"] and not audit["productStarted"]
protect = read(ROOT / "protection.json")
assert protect["nativeCount"] == 0 and not protect["productStarted"]
assert protect["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
result = {
    "status": "verified-pinned-decoder-and-cswitch-corpus-audit", "files": len(files),
    "namedRootOutEvents": 564, "rootReasonValuesOutsidePinnedEnum": 545,
    "rootModeValuesOutsidePinnedEnum": 42, "descendantOutEvents": 3095,
    "v5WaitSemanticsVerified": False, "oldFailureWindowObtained": False,
    "newRecordingOrReplayExecuted": False, "nativeProductStarted": False,
    "historicalFailureCleared": False, "releaseAccepted": False,
}
(ROOT / "verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
