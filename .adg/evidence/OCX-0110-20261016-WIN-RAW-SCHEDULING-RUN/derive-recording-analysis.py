"""Reuse the reviewed generation audit and add the new raw-byte-window audit."""
import ast
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
origin = ROOT.parent / "OCX-0110-20261016-WIN-QUERY-THREAD-RUN/analyze-remote.py"
old = origin.read_text(encoding="utf-8")
new = old
for before, after in [
    ("import collections\n", "import collections\nimport base64\nimport gzip\n"),
    ("7704a17902c98ceed047cc3dec2a361937b5e1f7", "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"),
    ("38829382543", "38876971001"), ("38826901050", "38858859494"),
    ("remote-terminal.original.json", "recording-terminal.original.json"),
    ("remote-download-verification.json", "recording-download-verification.json"),
    ('path = ROOT / "analysis.json"', 'path = ROOT / "recording-analysis.json"'),
]:
    assert before in new
    new = new.replace(before, after)
old_entries = '''for entry in download["entries"]:
    data = (ROOT / "remote-original" / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]'''
new_entries = '''for entry in download["entries"]:
    archived = (ROOT / entry["archivedPath"]).read_bytes()
    assert len(archived) == entry["archivedBytes"]
    assert hashlib.sha256(archived).hexdigest() == entry["archivedSha256"]
    data = gzip.decompress(archived) if entry["gzipRoundtrip"] else archived
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]'''
assert old_entries in new
new = new.replace(old_entries, new_entries)
old_raw = 'raw = (CAPTURE / "owned-etw.events.jsonl").read_bytes()'
assert old_raw in new
new = new.replace(old_raw, 'raw = gzip.decompress((CAPTURE / "owned-etw.events.jsonl.gz").read_bytes())')
anchor = 'result = {\n    "collectorSource": SOURCE'
assert anchor in new
new = new.replace(anchor, '''raw_windows = [e for e in events if "rawSchedulingWindow" in e]
assert len(raw_windows) == capture["eventsWithRawSchedulingWindow"] == 5319
assert capture["rawSchedulingByteOffset"] == capture["rawSchedulingByteLength"] == 8
assert not capture["completeCswitchPayloadRetained"] and not capture["counterpartThreadIdPrefixRetained"]
versions = collections.Counter()
raw_tuples = collections.Counter()
for event in raw_windows:
    window = event["rawSchedulingWindow"]
    assert event["kind"] in {"owned-thread-scheduled-in", "owned-thread-scheduled-out"}
    assert event["original"]["opcode"] == 36
    assert window["payloadLength"] >= 16 and window["byteOffset"] == window["byteLength"] == 8
    payload = base64.b64decode(window["bytesBase64"], validate=True)
    assert len(payload) == 8
    assert window["threadIdPrefixExcluded"] and window["unrecognizedTailExcluded"]
    assert not window["completePayloadRetained"] and not window["versionSpecificSemanticsVerified"]
    assert not event["counterpartThreadIdSaved"]
    if event["kind"] == "owned-thread-scheduled-out":
        assert payload[4] == event["waitReasonDecodedRaw"]
        assert payload[5] == event["waitModeDecodedRaw"]
        assert payload[6] == event["stateDecodedRaw"]
        assert not event["versionSpecificWaitSemanticsVerified"]
        raw_tuples[tuple(payload[4:8])] += 1
    versions[str(event["original"]["version"])] += 1
result = {
    "rawSchedulingByteWindows": len(raw_windows), "rawSchedulingByteOffset": 8, "rawSchedulingByteLength": 8,
    "rawWindowEventVersions": dict(versions),
    "decodedGetterByteOffsetsCoherent": True,
    "rawOffsets12To15TuplesWithoutSemanticLabels": [
        {"bytes": list(values), "count": count} for values, count in raw_tuples.most_common()],
    "completeCswitchPayloadRetained": False, "counterpartThreadIdPrefixRetained": False,
    "collectorSource": SOURCE''')
ast.parse(new)
target = ROOT / "analyze-recording.py"
with target.open("xb") as output:
    output.write(new.encode("utf-8"))
record = {"priorMethod": str(origin), "priorMethodSha256": hashlib.sha256(old.encode()).hexdigest(),
    "derivedMethodSha256": hashlib.sha256(new.encode()).hexdigest(),
    "changes": ["bind actual 549/38876971001/38858859494 identities", "verify gzip roundtrip originals",
                "audit 5319 raw scheduling windows and getter byte coherence without assigning v5 semantics"],
    "priorLifecycleAndCaptureAssertionsRelaxed": False, "historicalFailureCleared": False,
    "newRecordingRequestedByAnalysis": False}
with (ROOT / "analysis-method-derivation.json").open("xb") as output:
    output.write((json.dumps(record, indent=2) + "\n").encode())
print(json.dumps(record))
