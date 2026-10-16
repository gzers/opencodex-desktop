"""Archive read-only WPR preparation; no recording or product gate admission."""
import hashlib
import json
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
import zipfile
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop")
RUNTIME = CODE / "test/runtime/memory-profile-prep-20261016T2300"
SOURCE = "494fc830e30cc6b56cf62f34884d44e23bfb763d"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-WPR-ALLOCATION-PREP-20261016.json"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert data == Path(row["runtimePath"]).read_bytes()
    identity = read(ROOT / "source-identity.json")
    assert identity["source"] == SOURCE
    for row in identity["files"]:
        blob = subprocess.check_output(["git", "show", SOURCE + ":" + row["path"]], cwd=CODE)
        assert sha(blob) == row["gitLfSha256"]
    admission = read(ROOT / "original/method-admission.json")
    assert admission["source"] == admission["actualCheckout"] == SOURCE
    assert admission["runId"] == 38920950285 and admission["jobId"] == 117040410248
    assert admission["windowsReadOnlyPreparationJobSucceeded"] and admission["checks"] == 18
    assert not admission["wholeCiAdmitted"] and not admission["recordingExecutionAdmitted"]
    raw = read(ROOT / "original/ci-method-job-and-artifacts.original.json")
    job = raw["job"]
    assert job["head_sha"] == SOURCE and job["status"] == "completed" and job["conclusion"] == "success"
    assert raw["run"]["event"] == "push" and raw["run"]["run_attempt"] == job["run_attempt"] == 1
    step, = [s for s in job["steps"] if s["name"] == "Verify read-only WPR allocation profile preparation"]
    assert step["conclusion"] == "success"
    artifact, = [a for a in raw["artifacts"]["artifacts"] if a["id"] == admission["artifactId"]]
    archive = ROOT / "original/ci-method.original.zip"
    assert sha(archive.read_bytes()) == admission["artifactSha256"] == artifact["digest"][7:]
    assert archive.stat().st_size == admission["artifactBytes"] == artifact["size_in_bytes"]
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None
        for name in admission["extractedMembers"]:
            member, = [m for m in zipped.infolist() if m.filename.endswith("memory-profile/" + name)]
            assert zipped.read(member) == (ROOT / "original/ci-profile" / name).read_bytes()
    log = (ROOT / "original/ci-method-job.original.log").read_bytes()
    assert sha(log) == admission["jobLogSha256"]
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", log.decode("utf-8-sig"))
    assert checkout == SOURCE
    for folder in ["validation", "ci-profile"]:
        base = ROOT / "original" / folder
        result = read(base / "result.json")
        assert result["passed"] and result["checks"] == 18 and result["readOnlyWprParserValidated"]
        assert result["configuredBufferBytes"] == 33554432
        assert not any(result[k] for k in ["recordingStarted", "eventLossVerified", "fileGrowthBoundEnforced",
                                          "executionAdmitted", "desktopProductStarted", "queryReplayed",
                                          "fullPerformanceGatePassed"])
        for name, field in [("host-gpu-va-export.original.wprp", "exportSha256"),
                            ("ocx-gpu-allocation.wprp", "generatedProfileSha256")]:
            assert sha((base / name).read_bytes()) == result[field]
        method = identity["files"][0]
        assert result["sourceMethodSha256"] in {method["gitLfSha256"], method["gitCheckoutCrLfSha256"]}
        source = ET.parse(base / "host-gpu-va-export.original.wprp").getroot().find("Profiles")
        profile = ET.parse(base / "ocx-gpu-allocation.wprp").getroot().find("Profiles")
        collectors = list(profile.findall("SystemCollector")) + list(profile.findall("EventCollector"))
        assert len(collectors) == 2
        assert sum(int(c.find("BufferSize").get("Value")) * int(c.find("Buffers").get("Value"))
                   for c in collectors) * 1024 == 33554432
        actual, = profile.findall("Profile")
        assert actual.get("Id") == "OcxGpuAllocation.Verbose.File" and actual.get("LoggingMode") == "File"
        keywords = {r.get("Value") for r in profile.findall("SystemProvider/Keywords/Keyword")}
        assert keywords == {"ProcessThread", "Loader", "VirtualAllocation", "MemoryInfo", "MemoryInfoWS"}
        stacks = {r.get("Value") for r in profile.findall("SystemProvider/Stacks/Stack")}
        assert stacks == {"VirtualAllocation", "ProcessCreate"}
        graphics = profile.findall("EventProvider")
        assert len(graphics) == 3
        assert {r.get("Name") for r in graphics} == {
            "ca11c036-0102-4a2d-a6ad-f03cfed5d3c9", "db6f6ddb-ac77-4e88-8253-819df9bbf140",
            "802ec45a-1e99-4b83-9920-87c98277ba9d"}
        for row in graphics:
            original, = [r for r in source.findall("EventProvider") if r.get("Name") == row.get("Name")]
            assert {k: v for k, v in row.attrib.items() if k != "Id"} == {
                k: v for k, v in original.attrib.items() if k != "Id"}
            assert ET.tostring(row.find("Keywords")) == ET.tostring(original.find("Keywords"))
            assert ET.tostring(row.find("CaptureStateOnStart")) == ET.tostring(original.find("CaptureStateOnStart"))
    state = read(ROOT / "original/final-readback.original.json")
    assert state["nativeCount"] == 0 and state["toolSource"] == SOURCE
    assert state["originalDirectory"]["fileId"] == "0ebd4e59-001000000005cf4b"
    assert state["originalDirectory"]["creationFileTime"] == "134366003622018367"
    assert state["restoredPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert state["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert state["taskFileSha256"] == sha(TASK.read_bytes()) == (
        "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93")
    assert not state["tracingStarted"] and not state["productStarted"] and not state["queryReplayed"]
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {
        "status": "verified-read-only-profile-preparation", "source": SOURCE,
        "localChecks": 18, "windowsCiMethodChecks": 18, "ciRunId": 38920950285,
        "configuredCollectorBufferBytes": 33554432, "graphicsProviders": 3,
        "wholeCiAdmitted": False, "recordingExecutionAdmitted": False,
        "eventLossVerified": False, "fileGrowthBoundEnforced": False,
        "originalProfilePreserved": True, "nativeCountAtReadback": 0,
        "rssFailureCleared": False, "fullProductGatePassed": False,
    }


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "copies.json").exists()
copies = []
for path in sorted(RUNTIME.rglob("*")):
    if not path.is_file():
        continue
    data = path.read_bytes()
    if path.suffix != ".zip":
        text = data.decode("utf-8-sig")
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", text)
    target = ROOT / "original" / path.relative_to(RUNTIME)
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": target.relative_to(ROOT).as_posix(), "runtimePath": str(path),
                   "bytes": len(data), "sha256": sha(data)})
write(ROOT / "copies.json", copies)
files = []
for name in ["test/smoke/windows-memory-profile.ps1", "test/smoke/windows-memory-profile.test.ps1",
             ".github/workflows/ci.yml"]:
    blob = subprocess.check_output(["git", "show", SOURCE + ":" + name], cwd=CODE)
    assert b"\r\n" not in blob
    files.append({"path": name, "gitLfSha256": sha(blob),
                  "gitCheckoutCrLfSha256": sha(blob.replace(b"\n", b"\r\n"))})
write(ROOT / "source-identity.json", {"source": SOURCE, "branch": "feature/0.1.10-windows-working-set-pages",
                                     "files": files, "implementationCopiedIntoDocs": False})
result = verify()
write(ROOT / "verification.json", result)
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object
task = read(TASK)
validate_object(task)
assert task["revision"] == 2 and task["state"] == "in_progress" and task["result"] == "pending"
relative = ROOT.relative_to(PROJECT).as_posix()
receipt = seal_object({
    "kind": "RCP", "id": RECEIPT.stem, "task_id": task["id"],
    "task_digest": task["content_digest"], "task_summary_digest": task["summary_digest"],
    "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [relative, RECEIPT.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-read-only-wpr-profile-preparation", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-wpr-recording-execution-admission", "status": "failed",
         "evidence": relative + "/original/method-admission.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{
        "target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
        "after_digest": sha(IMP.read_bytes()),
        "summary": "494只读WPR配置32MiB及三图形provider准备完成，本机和同源CI方法各18项通过；整轮CI/真实录制未准入，原失败保持。",
    }],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "仅只读WPR分配配置准备通过；尚缺真实录制、文件增长/丢事件证明，RSS及完整门禁未通过。",
    "revision": 1,
})
write(RECEIPT, receipt)
manifest = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "manifest.json":
        data = path.read_bytes()
        manifest.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": sha(data)})
write(ROOT / "manifest.json", {"files": manifest})
verify()
print(json.dumps({"originalCopies": len(copies), "receiptResult": receipt["result"], **result}))
