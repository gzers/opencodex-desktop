"""Seal the original a5 CI terminal state without replaying any job."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop/test/runtime/controlled-memory-validation-20261016T2330")
SOURCE = "a5a1aba290793b23c6d27c35edd0ab227d4f5e50"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-MEMORY-TOOL-FULL-CI-20261016.json"


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
    admission = read(ROOT / "original/admission.json")
    state = read(ROOT / "original/terminal.original.json")
    run, jobs = state["run"], state["jobs"]
    assert run["id"] == admission["runId"] == 38925613291
    assert run["head_sha"] == admission["source"] == SOURCE and run["event"] == "push"
    assert run["run_attempt"] == 1 and run["status"] == "completed" and run["conclusion"] == "success"
    assert admission["wholeToolCiAdmitted"] and not admission["fullProductGateAccepted"]
    assert jobs["total_count"] == len(jobs["jobs"]) == 6
    assert all(j["status"] == "completed" and j["head_sha"] == SOURCE for j in jobs["jobs"])
    required = {"windows-smoke-evidence", "frontend", "backend", "backend-windows", "release-tools"}
    assert {j["name"] for j in jobs["jobs"] if j["conclusion"] == "success"} == required
    assert [j["name"] for j in jobs["jobs"] if j["conclusion"] == "skipped"] == ["build"]
    counts = {}
    for row in admission["requiredJobLogs"]:
        data = (ROOT / "original" / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
        checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
        assert checkout == row["actualCheckout"] == SOURCE
        job, = [j for j in jobs["jobs"] if j["id"] == row["jobId"]]
        assert job["name"] == row["job"] and job["conclusion"] == "success"
        tests = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        if tests:
            assert all(int(r[1]) == 0 for r in tests)
            counts[job["name"]] = {"groups": len(tests), "passed": sum(int(r[0]) for r in tests),
                                  "ignored": sum(int(r[2]) for r in tests)}
        if job["name"] == "frontend":
            assert re.search(r"Test Files\s+99 passed \(99\)", text) and re.search(r"\bTests\s+535 passed \(535\)", text)
    assert counts == {"backend": {"groups": 35, "passed": 923, "ignored": 6},
                      "backend-windows": {"groups": 35, "passed": 648, "ignored": 7}}
    binding = read(ROOT / "controlled-proof-binding.json")
    proof = PROJECT / binding["directory"]
    assert sha((proof / "manifest.json").read_bytes()) == binding["manifestSha256"]
    assert sha((proof / "verification.json").read_bytes()) == binding["verificationSha256"]
    previous = read(proof / "verification.json")
    assert previous["source"] == SOURCE and not previous["currentToolWholeCiAdmitted"]
    assert not previous["productAllocationCauseAttributed"] and not previous["originalBd2FailureCleared"]
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-first-same-source-full-tool-ci", "source": SOURCE,
            "runId": run["id"], "attempt": 1, "requiredJobsPassed": 5, "mainOnlyBuildSkipped": True,
            "frontendTests": 535, "rust": counts, "wholeToolCiAdmitted": True,
            "historicalMethodAdmissionRewritten": False, "productGpuBindingAdmitted": False,
            "rssFailureCleared": False, "originalBd2FailureCleared": False,
            "fullProductGatePassed": False, "releaseAccepted": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RECEIPT.exists() and not (ROOT / "copies.json").exists()
copies = []


def copy(path, name):
    data = path.read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig"))
    target = ROOT / "original" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": target.relative_to(ROOT).as_posix(), "runtimePath": str(path),
                   "bytes": len(data), "sha256": sha(data)})


admission = read(RUNTIME / "ci-terminal-admission.json")
copy(RUNTIME / "ci-terminal-admission.json", "admission.json")
terminal, = [p for p in RUNTIME.glob("ci-*.original.json")
             if re.fullmatch(r"ci-\d{8}T\d+Z.original.json", p.name)
             and read(p)["run"]["id"] == 38925613291 and read(p)["run"]["status"] == "completed"]
copy(terminal, "terminal.original.json")
copy(RUNTIME / "collect-ci-terminal.py", "collect-ci-terminal.py")
for row in admission["requiredJobLogs"]:
    copy(RUNTIME / row["path"], row["path"])
write(ROOT / "copies.json", copies)
proof = ROOT.parent / "OCX-0110-20261016-WIN-CONTROLLED-MEMORY-PROOF"
write(ROOT / "controlled-proof-binding.json", {
    "directory": proof.relative_to(PROJECT).as_posix(), "manifestSha256": sha((proof / "manifest.json").read_bytes()),
    "verificationSha256": sha((proof / "verification.json").read_bytes()),
})
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
        {"name": "local:windows-controlled-memory-tool-full-ci", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{
        "target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(), "after_digest": sha(IMP.read_bytes()),
        "summary": "a5首次完整CI五必需job及原日志同SHA核验通过；仅工具CI准入，真实GPU入口/产品RSS/bd2历史失败未解除。",
    }],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "a5同源完整工具CI通过，旧方法时点快照保持；产品与全部原门禁未关闭。",
    "revision": 1,
})
write(RECEIPT, receipt)
files = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "manifest.json":
        data = path.read_bytes()
        files.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": sha(data)})
write(ROOT / "manifest.json", {"files": files})
verify()
print(json.dumps({"originalCopies": len(copies), "receiptResult": receipt["result"], **result}))
