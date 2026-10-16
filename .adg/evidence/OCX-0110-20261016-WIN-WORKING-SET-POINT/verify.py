"""Read-only verification of the single late GPU metadata snapshot."""
import hashlib
import json
import re
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def ticks(value):
    digits = re.search(r"\.(\d+)(?:Z|[+-]\d\d:\d\d)$", value).group(1)
    assert len(digits) <= 7
    value = datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    delta = value.replace(microsecond=0) - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return (delta.days * 86400 + delta.seconds) * 10_000_000 + int(digits.ljust(7, "0"))


for row in read("original-copies.json"):
    data = (ROOT / row["path"]).read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    assert data == Path(row["runtimePath"]).read_bytes()
identity = read("source-identity.json")
source = identity["source"]
assert source == "3a2ff08a6b480d237e93502626861090aa14423d"
assert identity["toolSourceIsDifferentFromProductCandidate"]
for row in identity["sourceFiles"]:
    blob = subprocess.check_output(["git", "show", source + ":" + row["path"]], cwd=identity["workspace"])
    assert b"\r\n" not in blob
    assert sha(blob) == row["gitLfSha256"]
    assert sha(blob.replace(b"\n", b"\r\n")) == row["gitCheckoutCrLfSha256"]
    assert row["workingSha256"] in {row["gitLfSha256"], row["gitCheckoutCrLfSha256"]}
prior = read("prior-evidence-binding.json")
prior_root = PROJECT / prior["directory"]
for path, field in [("manifest.json", "manifestSha256"),
                    ("verification.json", "verificationSha256"), ("scope.json", "scopeSha256")]:
    assert sha((prior_root / path).read_bytes()) == prior[field]
prior_scope = json.loads((prior_root / "scope.json").read_text())
assert not prior_scope["fullPerformanceGatePassed"] and not prior_scope["formalPairLifecycleAdmitted"]
assert prior_scope["meanRssIncreaseMiB"] > 10 and not prior_scope["fixtureRestoredAtArchive"]
roles = json.loads((prior_root / "original/candidate0110-settled-roles.original.json").read_text(encoding="utf-8-sig"))
gpu, = [m for m in roles["members"] if m["role"] == "gpu"]
admission = read("original/method-admission.json")
assert admission["source"] == source and admission["runId"] == 38910385568
assert admission["windowsMethodJobSucceeded"] and admission["controlledChecks"] == 34
assert admission["actualCheckout"] == source and not admission["wholeCiAdmitted"]
assert not admission["productGateAccepted"] and not admission["hostProductStarted"]
assert admission["ciMethodSha256"] == identity["sourceFiles"][0]["gitCheckoutCrLfSha256"]
assert admission["localMethodSha256"] == identity["sourceFiles"][0]["workingSha256"]
api = read("original/method-job-and-artifacts.original.json")
run, job = api["run"], api["job"]
assert run["head_sha"] == job["head_sha"] == source and run["run_attempt"] == job["run_attempt"] == 1
assert run["event"] == "push" and job["run_id"] == run["id"] == 38910385568
assert job["id"] == admission["jobId"] == 117004428361
assert job["status"] == "completed" and job["conclusion"] == "success"
step, = [s for s in job["steps"] if s["name"] == "Verify bound read-only working-set metadata"]
assert step["status"] == "completed" and step["conclusion"] == "success"
artifact, = [a for a in api["artifacts"]["artifacts"] if a["id"] == admission["artifactId"]]
zip_bytes = (ROOT / "original/method-evidence.original.zip").read_bytes()
assert len(zip_bytes) == artifact["size_in_bytes"] == admission["artifactBytes"]
assert artifact["digest"] == "sha256:" + sha(zip_bytes) == "sha256:" + admission["artifactSha256"]
assert artifact["workflow_run"]["head_sha"] == source and not artifact["expired"]
with zipfile.ZipFile(ROOT / "original/method-evidence.original.zip") as archive:
    assert archive.testzip() is None
    assert len({m.filename.casefold() for m in archive.infolist()}) == len(archive.infolist())
    for member in archive.infolist():
        assert not member.filename.startswith(("/", "\\")) and ".." not in Path(member.filename).parts
        if not member.is_dir():
            text = archive.read(member).decode("utf-8-sig")
            assert not re.search(r"S-1-\d+(?:-\d+){1,}", text)
    for name in ["result.json", "controlled-self.original.json", "controlled-pages.original.json"]:
        member, = [m for m in archive.infolist() if m.filename.endswith("working-set/" + name)]
        assert archive.read(member) == (ROOT / ("original/ci-" + name)).read_bytes()
log = (ROOT / "original/method-job.original.log").read_bytes()
assert sha(log) == admission["logSha256"]
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", log.decode("utf-8-sig"))
assert checkout == source
for label in ["ci-", "local-20261016T214301/"]:
    result = read("original/" + label + "result.json")
    assert result["passed"] and result["checks"] == 34 and result["pageProbeOwnAllocationsReleased"]
    assert not result["nativeProductStarted"] and not result["productSnapshotTaken"]
    pages = read("original/" + label + "controlled-pages.original.json")
    assert {(p["Kind"], p["RegionType"], p["Valid"]) for p in pages} == {
        ("controlled-private", 0x20000, True), ("controlled-mapped", 0x40000, True),
        ("controlled-image", 0x1000000, True)}
    controlled = read("original/" + label + "controlled-self.original.json")
    assert controlled["controlledSelf"] and controlled["rootPid"] is None
    assert not controlled["snapshot"]["TargetMemoryContentsRead"]
    assert controlled["sourceMethodSha256"] == (
        admission["ciMethodSha256"] if label == "ci-" else admission["localMethodSha256"])
attempt = read("original/gpu-read-attempt.json")
binding = read("original/gpu-read-binding.json")
point = read("original/gpu-working-set.original.json")
assert attempt["toolSource"] == binding["toolSource"] == source
assert not attempt["automaticRetry"] and not attempt["foregroundPermissionGranted"]
assert not attempt["newProductStartAllowed"] and not attempt["fullPerformanceGateAccepted"]
assert point["targetPid"] == gpu["pid"] == attempt["gpuPid"] == 17236
assert ticks(point["targetCreationUtc"]) == ticks(gpu["creationUtc"]) == ticks(attempt["gpuCreationUtc"])
assert point["rootPid"] == attempt["rootPid"] == binding["rootLaunch"]["pid"] == roles["rootPid"] == 76372
assert binding["rootLaunch"]["exeSha256"] == roles["exeSha256"] == (
    "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db")
assert not binding["rootLaunch"]["applicationToken"]["elevated"]
assert binding["rootLaunch"]["applicationToken"]["elevationType"] == 3
assert binding["currentSceneNotReobserved"] and binding["pointSnapshotCannotBackfillCounterSpanClassification"]
assert point["sourceMethodSha256"] == admission["localMethodSha256"]
assert point["targetExeSha256"] == binding["gpuLaunch"]["exeSha256"]
snapshot = point["snapshot"]
assert int(snapshot["CreationFileTime"]) == ticks(gpu["creationUtc"])
assert snapshot["ProcessId"] == point["targetPid"] and snapshot["PageSize"] == 4096
assert 0 <= ticks(snapshot["FinishedAtUtc"]) - ticks(snapshot["StartedAtUtc"]) <= 20000 * 10000
assert ticks(snapshot["StartedAtUtc"]) > ticks(binding["formalCounterSpanEndedAtUtc"])
assert snapshot["NonAtomic"] and not snapshot["RawAddressesSaved"]
assert not snapshot["TargetMemoryContentsRead"] and not snapshot["TargetMemoryChanged"]
assert not snapshot["SharedPagesDeduplicated"]
assert sum(row["Pages"] for row in snapshot["Buckets"]) == snapshot["ListedResidentPages"]
assert sum(int(row["Bytes"]) for row in snapshot["Buckets"]) == int(snapshot["ListedResidentBytes"])
assert snapshot["ValidAtRecheckPages"] + snapshot["InvalidAtRecheckPages"] == snapshot["ListedResidentPages"]
for row in snapshot["Buckets"]:
    assert row["Pages"] * 4096 == int(row["Bytes"])
    assert row["ValidAtRecheck"] or row["ShareableAtRecheck"] is None
analysis = read("point-analysis.json")
regions = {}
for row in snapshot["Buckets"]:
    regions[row["RegionType"]] = regions.get(row["RegionType"], 0) + int(row["Bytes"])
assert regions == analysis["regionBytes"]
assert {k: v / 1048576 for k, v in regions.items()} == analysis["regionMiB"]
assert not analysis["baselinePageClassificationAvailable"] and not analysis["counterSpanBackfilled"]
assert not analysis["individualGlAllocationAttributed"] and not analysis["rssMetricReplaced"]
assert not analysis["originalRssBudgetPassed"] and not analysis["fullPairLifecycleAdmitted"]
assert analysis["originalRssIncreaseMiB"] == prior_scope["meanRssIncreaseMiB"]
for field in ["newProductStarted", "uiInputInjected", "formalRssMetricReplaced",
              "allocationStackAttributed", "fullPerformanceGatePassed"]:
    assert not point[field]
cleanup = read("original/cleanup-followup.original.json")
assert cleanup["nativeCount"] == 1 and not cleanup["fixtureRestored"] and not cleanup["restoreAttempted"]
assert cleanup["nativeProcesses"][0]["ProcessId"] == point["rootPid"]
assert cleanup["originalDirectory"]["fileId"] == "0ebd4e59-001000000005cf4b"
assert cleanup["originalDirectory"]["creationFileTime"] == "134366003622018367"
assert cleanup["currentDirectory"]["fileId"] == "0ebd4e59-00090000000d6120"
assert cleanup["originalAndLateExitTimeoutResultsUnchanged"] and cleanup["normalCandidateExitCodeStillUnknown"]
assert cleanup["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
assert cleanup["taskFileSha256"] == sha((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_bytes())
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        data = (ROOT / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and sha(data) == item["sha256"]
print(json.dumps({
    "status": "verified-single-late-point-only", "toolSource": source,
    "windowsMethodJobSucceeded": True, "controlledChecks": 34,
    "gpuPid": point["targetPid"], "listedResidentPages": snapshot["ListedResidentPages"],
    "regionMiB": analysis["regionMiB"], "nonAtomic": True,
    "baselinePageClassificationAvailable": False, "rssFailureCleared": False,
    "fixtureRestoredAtFollowup": False, "allocationCauseAttributed": False,
    "fullPerformanceGatePassed": False, "releaseAccepted": False,
}, indent=2))
