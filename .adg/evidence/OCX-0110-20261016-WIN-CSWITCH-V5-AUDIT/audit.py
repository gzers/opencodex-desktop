"""Audit preserved decoded CSwitch fields; never record or replay a process."""
import collections
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OLD = ROOT.parent / "OCX-0110-20261016-WIN-QUERY-THREAD-RUN"
CAPTURE = OLD / "remote-original/runtime/ci-bd2-query-etw/capture"
assert not (ROOT / "manifest.json").exists()
assert not (ROOT / "corpus-audit.json").exists()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


reflection = read(ROOT / "pinned-decoder-reflection.json")
access = read(ROOT / "getter-access.json")
assert reflection["decoderSha256"] == "401e948dac99d0f9756ef96d5a8b8e4736e19e9f82ce8cdd744495951e3a94bb"
properties = {p["property"]: p for p in reflection["properties"]}
defined = {
    key: {e["value"] for e in properties[name]["enumValues"]}
    for key, name in (("reason", "OldThreadWaitReason"), ("mode", "OldThreadWaitMode"), ("state", "OldThreadState"))
}
assert defined["mode"] == {0, 1}
assert {p["property"]: p["byteOffset"] for p in access["properties"]} == {
    "OldThreadWaitReason": 12, "OldThreadWaitMode": 13, "OldThreadState": 14}
assert all(p["callName"] == "GetByteAt" and p["returnType"] == "System.Int32"
           and not p["versionConditionalInGetter"] and not p["bitMaskInGetter"]
           for p in access["properties"])
capture = read(CAPTURE / "capture-result.json")
prior = read(OLD / "analysis.json")
assert capture["testPid"] == 6688 and capture["boundedOriginalRunCount"] == 1
assert capture["testExitCode"] == 0 and prior["tests"] == {"passed": 530, "failed": 0, "ignored": 4}
assert not capture["versionSpecificWaitSemanticsVerified"] and not capture["systemWideEtlWritten"]
groups = {}
for name in ("named-root-query-threads", "owned-descendants"):
    groups[name] = {"count": 0, "versions": collections.Counter(),
                    "reason": collections.Counter(), "mode": collections.Counter(),
                    "state": collections.Counter(), "outsidePinnedReasonEnum": 0,
                    "outsidePinnedModeEnum": 0, "outsidePinnedStateEnum": 0}
payload_fields = {"rawPayload", "rawPayloadHex", "payloadBytes", "eventData", "eventDataHex", "userData", "userDataHex"}
events = 0
payload_saved = 0
for line in (CAPTURE / "owned-etw.events.jsonl").read_text(encoding="utf-8-sig").splitlines():
    row = json.loads(line)
    events += 1
    if row.get("kind") != "owned-thread-scheduled-out":
        continue
    assert row["versionSpecificWaitSemanticsVerified"] is False
    group_name = "named-root-query-threads" if row["pid"] == capture["testPid"] else "owned-descendants"
    if group_name == "named-root-query-threads":
        assert row["fixture"] in capture["namedQueryFixtures"]
    group = groups[group_name]
    group["count"] += 1
    group["versions"][row["original"]["version"]] += 1
    for key, field in (("reason", "waitReasonDecodedRaw"), ("mode", "waitModeDecodedRaw"), ("state", "stateDecodedRaw")):
        value = row[field]
        assert isinstance(value, int) and 0 <= value <= 255
        group[key][value] += 1
        if value not in defined[key]:
            group["outsidePinned" + key.capitalize() + "Enum"] += 1
    payload_saved += bool(payload_fields & (set(row) | set(row["original"])))
assert events == capture["recordedEvents"] == 19058
assert sum(group["count"] for group in groups.values()) == 3659
assert not payload_saved and not list(CAPTURE.glob("*.etl"))
for group in groups.values():
    for key in ("versions", "reason", "mode", "state"):
        group[key] = dict(sorted(group[key].items()))
result = {
    "sourceCollector": prior["collectorSource"], "originalFailedSource": "bd2f6eb10d58115235e35d575507bb69f44666da",
    "successfulReplayRun": prior["runId"], "originalFailedRun": prior["originalFailedRun"],
    "pinnedDecoder": reflection["assemblyName"], "decoderInformationalVersion": reflection["informationalVersion"],
    "decoderSha256": reflection["decoderSha256"], "directGetterOffsets": access["properties"],
    "groups": groups, "fullRawCswitchPayloadRecords": payload_saved, "etlAvailableInPreservedCapture": False,
    "referenceHashes": {
        "priorManifest": digest(OLD / "manifest.json"), "priorAnalysis": digest(OLD / "analysis.json"),
        "rawEvents": digest(CAPTURE / "owned-etw.events.jsonl"), "captureResult": digest(CAPTURE / "capture-result.json")},
    "finding": "Selected integers come from fixed byte offsets through the pinned decoder; no v5-specific semantics established.",
    "doNotInferMaskOrWaitCause": True, "managedEnumMembershipIsNotKernelSchemaValidation": True,
    "allOriginalSemanticsFlagsRetainedFalse": True, "v5SemanticsVerified": False,
    "newRecordingStarted": False, "originalLibraryReplayed": False, "productStarted": False,
    "actualFailedQueryWindowObtained": False, "historicalFailureCleared": False,
    "performanceGatePassed": False, "releaseAccepted": False,
}
(ROOT / "corpus-audit.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "eventsRead": events,
    "groups": {name: {key: group[key] for key in (
        "count", "versions", "outsidePinnedReasonEnum", "outsidePinnedModeEnum", "outsidePinnedStateEnum")}
        for name, group in groups.items()},
    "rawCswitchPayloadAvailable": False, "v5SemanticsVerified": False,
    "recordingRepeated": False, "actualFailedWindowObtained": False,
}))
