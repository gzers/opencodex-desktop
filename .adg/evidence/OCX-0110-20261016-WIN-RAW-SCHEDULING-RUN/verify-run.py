"""Verify the one admitted recording, its raw archive, and its limited result."""
import base64
import collections
import gzip
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
SOURCE = "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
RUN = 38876971001


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


ci = read("ci-admission.json")
assert ci["source"] == SOURCE and ci["runId"] == 38858859494 and ci["runAttempt"] == 1
assert ci["wholeCiSucceeded"] and ci["originalCheckoutAndJobsVerified"]
for job in ci["jobs"]:
    data = (ROOT / job["log"]).read_bytes()
    assert len(data) == job["bytes"] and sha(data) == job["sha256"]
    text = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    if job["name"] == "frontend":
        assert re.search(r"Test Files\s+99 passed", text) and re.search(r"Tests\s+535 passed", text)
    else:
        suites = [tuple(map(int, values)) for values in re.findall(
            r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)]
        assert len(suites) == 35 and sum(s[1] for s in suites) == 0
        assert sum(s[0] for s in suites) == (923 if job["name"] == "backend" else 648)
    assert job["certificateVerificationEnabled"] and not job["redirectAuthorizationForwarded"]
terminal, handle = read("recording-terminal.original.json"), read("remote-handle.json")
attempt, request = read("remote-dispatch-attempt.json"), read("remote-dispatch-request.json")
assert handle["runId"] == terminal["run"]["id"] == RUN
assert handle["source"] == terminal["run"]["head_sha"] == SOURCE
assert terminal["run"]["run_attempt"] == 1 and terminal["run"]["conclusion"] == "success"
assert attempt["collectorSource"] == SOURCE and attempt["maximumOriginalLibraryRuns"] == 1
assert not attempt["automaticRetryAllowed"] and attempt["noPrewarm"] and attempt["defaultConcurrency"]
assert attempt["deadlinesSeconds"] == [2, 4, 5, 8] and attempt["outerSeconds"] == 120
assert attempt["serializedMiB"] == 64 and attempt["queueRecords"] == attempt["trackedIrps"] == 32768
assert request["inputs"] == {"purpose": "query-etw", "admission_run": "38858859494",
                             "diagnostic_path": "isolated-replay"}
assert read("remote-dispatch-result.json")["httpStatus"] == 204
assert attempt["admissionSha256"] == sha((ROOT / "ci-admission.json").read_bytes())
download = read("recording-download-verification.json")
assert download["runId"] == RUN and download["artifactId"] == 12035074475
assert download["archive"]["sha256"] == "649ce8c01482b2cf964c4b3d16f54d51c230649c03ba9089134fce447599bda4"
assert download["archive"]["bytes"] == 423839 and download["apiDigestAndCrcVerified"]
assert len(download["entries"]) == 13
for item in download["entries"]:
    data = (ROOT / item["archivedPath"]).read_bytes()
    assert len(data) == item["archivedBytes"] and sha(data) == item["archivedSha256"]
    original = gzip.decompress(data) if item["gzipRoundtrip"] else data
    assert len(original) == item["bytes"] and sha(original) == item["sha256"]
job = download["originalJob"]
job_data = (ROOT / job["path"]).read_bytes()
assert sha(job_data) == job["sha256"] and len(job_data) == job["bytes"]
assert job["actualCheckout"] == SOURCE and job["certificateVerificationEnabled"]
assert not job["crossHostCredentialsForwarded"]
capture_root = ROOT / "remote-original/runtime/ci-bd2-query-etw/capture"
capture = json.loads((capture_root / "capture-result.json").read_text(encoding="utf-8-sig"))
raw = gzip.decompress((capture_root / "owned-etw.events.jsonl.gz").read_bytes())
events = [json.loads(line) for line in raw.splitlines()]
assert len(events) == 17296 and len(raw) == capture["actualSerializedBytes"] == 8153824
assert capture["recordedEvents"] == len(events)
assert capture["testExitCode"] == 0 and capture["boundedOriginalRunCount"] == 1
assert capture["droppedRecords"] == capture["kernelEventsLost"] == capture["decoderCallbackErrors"] == 0
assert not capture["knownImageCoverageGap"] and not capture["rootQueryThreadCoverageGap"]
assert capture["ownedSessionStopSucceeded"] and capture["processTraceJoined"] and capture["writerJoined"]
assert not capture["forcedOuterCleanup"] and not capture["nodePrewarmed"]
assert capture["maximumSerializedBytes"] == 64 * 1024 * 1024
assert capture["maximumQueueRecords"] == capture["maximumTrackedIrps"] == 32768
admission = read("remote-original/out/windows-query-etw/admission.json")
assert admission["testSha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert admission["nodeSha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
assert admission["nodeVersion"] == "24.21.0" and not admission["prewarm"]
assert admission["originalLibraryCommandArguments"] == [] and admission["originalLibraryRunsAllowed"] == 1
windows = [e for e in events if "rawSchedulingWindow" in e]
groups = collections.defaultdict(list)
for event in windows:
    window = event["rawSchedulingWindow"]
    payload = base64.b64decode(window["bytesBase64"], validate=True)
    assert len(payload) == 8 and window["byteOffset"] == window["byteLength"] == 8
    assert window["threadIdPrefixExcluded"] and window["unrecognizedTailExcluded"]
    assert not window["completePayloadRetained"] and not window["versionSpecificSemanticsVerified"]
    if event["kind"] == "owned-thread-scheduled-out":
        assert list(payload[4:7]) == [event["waitReasonDecodedRaw"], event["waitModeDecodedRaw"],
                                      event["stateDecodedRaw"]]
    groups[(event["original"]["qpc"], window["bytesBase64"])].append(event)
minimum = sum(max(collections.Counter(e["kind"] for e in rows).values()) for rows in groups.values())
assert len(windows) == 6131 and len(groups) == 5318 and minimum == capture["eventsWithRawSchedulingWindow"] == 5319
source_data = subprocess.check_output(["git", "show", f"{SOURCE}:test/smoke/windows-owned-etw.cs"], cwd=PROJECT)
assert source_data.count(b"eventsWithRawSchedulingWindow++;") == 1
analysis = read("recording-analysis.json")
assert analysis["eventSha256"] == sha(raw) and analysis["callbackCounterConsistentWithSerializedRows"]
assert not analysis["uniqueKernelEventIdentityReconstructed"]
assert analysis["tests"] == {"passed": 530, "failed": 0, "ignored": 4}
assert len(analysis["namedQueryFixtures"]) == 7 and all(f["result"] == "ok" for f in analysis["namedQueryFixtures"])
assert analysis["processStarts"] == analysis["processStops"] == 31
assert not any(analysis[key] for key in ["actualFailureWindowObtained", "targetQueryFailureReproduced",
    "rootCauseDetermined", "historicalFailureCleared", "versionSpecificWaitSemanticsVerified",
    "fullPerformanceGatePassed", "fullVersionGatePassed", "anotherOriginalRunRequested"])
checkpoint = read("checkpoint-manifest.json")
for item in checkpoint["files"]:
    data = (ROOT / item["path"]).read_bytes()
    assert len(data) == item["bytes"] and sha(data) == item["sha256"]
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        data = (ROOT / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and sha(data) == item["sha256"]
print(json.dumps({
    "status": "verified-one-success-control-only", "runId": RUN, "source": SOURCE,
    "originalArtifacts": len(download["entries"]), "tests": analysis["tests"],
    "eventCount": len(events), "eventBytes": len(raw), "rawWindowRows": len(windows),
    "sourceCallbackCount": capture["eventsWithRawSchedulingWindow"],
    "callbackCounterConsistent": True, "droppedOrDecodeErrors": 0,
    "originalFailedArtifactAndOfficialNodePinned": True,
    "actualFailureWindowObtained": False, "rootCauseDetermined": False,
    "historicalFailureCleared": False, "anotherOriginalRunRequested": False,
    "releaseAccepted": False,
}, indent=2))
