"""Keep compact product validation and original failures; no product launch."""
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")
RUNTIME = CODE / "test/runtime/webgl-lifetime-20261017T1027"
SOURCE = "ce670ce76bb1c29f70989ee54838d5b31e97059c"
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RCP = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-WEBGL-LIFETIME-20261017.json"


def read(p):
    return json.loads(p.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(p, value):
    p.parent.mkdir(parents=True, exist_ok=True)
    with p.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert data == Path(row["runtimePath"]).read_bytes()
        assert sha(data) == row["sha256"] and len(data) == row["bytes"]
    original = ROOT / "original"
    handle = read(original / "ci-handle.json")
    assert handle == {"source": SOURCE, "runId": 38995008914, "attempt": 1}
    state = read(original / "ci-initial.original.json")
    assert state["run"]["head_sha"] == SOURCE and state["run"]["id"] == handle["runId"]
    assert state["run"]["run_attempt"] == 1 and state["run"]["event"] == "push"
    assert read(original / "remote-readback.json")["remoteHead"] == SOURCE
    boundary = read(original / "source-boundary.json")
    assert boundary["source"] == SOURCE and boundary["originalRssBudgetMiB"] == 10
    assert boundary["vertexAndFragmentSourcesUnchanged"] and not boundary["renderResolutionOrCssChanged"]
    assert not boundary["nativeBackendChanged"] and not boundary["nativeRssMeasured"]
    before = (original / "before-test.log").read_text(encoding="utf-8-sig")
    assert "2 failed" in before and "300" in before and "150" in before
    assert "expected 'css' to be 'mesh'" in before
    checks = read(original / "checks.json")
    assert {r["name"] for r in checks} == {"tests", "typecheck", "build"}
    assert all(r["exitCode"] == 0 for r in checks)
    tests = (original / "tests.log").read_text(encoding="utf-8-sig")
    assert "100 passed (100)" in tests and "540 passed (540)" in tests
    pixels = read(original / "pixel-result.json")
    assert pixels["comparisons"] == 24 and pixels["scenes"] == 8 and len(pixels["rows"]) == 8
    for pair in pixels["rows"]:
        assert pair["allPixelsIdentical"]
        assert pair["before"]["init"]["finalSize"] == pair["after"]["init"]["finalSize"]
        for before, after in zip(pair["before"]["frames"], pair["after"]["frames"], strict=True):
            assert before["rawSha256"] == after["rawSha256"] and before["pngSha256"] == after["pngSha256"]
            assert before["glError"] == after["glError"] == 0
            assert not before["contextLost"] and not after["contextLost"]
            for frame in (before, after):
                png = RUNTIME / "pixel-check" / frame["png"]
                assert sha(png.read_bytes()) == frame["pngSha256"] and png.stat().st_size == frame["pngBytes"]
    pixel_binding = read(original / "pixel-binding.json")
    blob = subprocess.check_output(["git", "show", SOURCE + ":apps/desktop/ui/src/features/runtime/motion/meshGlow.ts"], cwd=CODE)
    assert pixel_binding["afterSha256"] in (sha(blob), sha(blob.replace(b"\n", b"\r\n")))
    assert pixel_binding["headlessOnly"] and not pixel_binding["nativeProductStarted"]
    assert not pixel_binding["officialQueryNodeExecuted"] and not pixel_binding["rssMeasured"]
    exited = read(original / "pixel-browser-exit.json")
    assert exited["code"] == 0 and exited["signal"] is None
    assert not exited["forceTerminated"] and not exited["descendantCleanupFullyProven"]
    assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert sha(data) == row["sha256"] and len(data) == row["bytes"]
    return {"source": SOURCE, "parent": boundary["parent"], "firstCiRunId": 38995008914,
            "regressionsFailedBeforeFix": 2, "frontendFiles": 100, "frontendTests": 540,
            "typecheckAndBuildPassed": True, "identicalRawAndCompositePixelCases": 24,
            "headlessBrowserRootExitCode": 0, "wholeCiAdmittedAtCheckpoint": False,
            "newCandidateAdmitted": False, "productNativeStarted": False,
            "rssBenefitMeasured": False, "rssFailureCleared": False, "fullProductGatePassed": False}


if "--verify" in sys.argv:
    print(json.dumps(verify()))
    sys.exit(0)
assert not RCP.exists() and not (ROOT / "copies.json").exists()
names = {"before-test.log": "before-test.log", "tests.log": "tests.log", "typecheck.log": "typecheck.log",
         "build.log": "build.log", "checks.json": "checks.json", "source-boundary.json": "source-boundary.json",
         "remote-readback.json": "remote-readback.json", "ci-handle.json": "ci-handle.json",
         "ci-initial.original.json": "ci-initial.original.json",
         "pixel-check/result.json": "pixel-result.json", "pixel-check/binding.json": "pixel-binding.json",
         "pixel-check/browser.json": "pixel-browser.json", "pixel-check/browser-exit.json": "pixel-browser-exit.json"}
copies = []
for name, dest in names.items():
    path = RUNTIME / name
    data = path.read_bytes()
    target = ROOT / "original" / dest
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": "original/" + dest, "runtimePath": str(path), "bytes": len(data), "sha256": sha(data)})
write(ROOT / "copies.json", copies)
result = verify()
write(ROOT / "verification.json", result)
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object
task = read(TASK)
validate_object(task)
receipt = seal_object({
    "kind": "RCP", "id": RCP.stem, "task_id": task["id"], "task_digest": task["content_digest"],
    "task_summary_digest": task["summary_digest"], "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [ROOT.relative_to(PROJECT).as_posix(), RCP.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [{"name": "local:windows-webgl-lifetime-product-fix", "status": "passed",
                "evidence": ROOT.relative_to(PROJECT).as_posix() + "/verification.json"},
               {"name": "review:scheduler-performance-0110", "status": "failed",
                "evidence": ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/original/counter-role-analysis.json"}],
    "acceptance_results": [],
    "fact_impacts": [{"target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(), "after_digest": sha(IMP.read_bytes()),
                      "summary": "固定八项收尾清单；ce670产品上下文释放/故障识别修复，540前端与24像素对照通过，首次CI运行中，原RSS预算失败保持。"}],
    "completed_at": datetime.now(timezone.utc).isoformat(), "revision": 1,
    "summary": "交付可复现的产品资源生命周期修复，不把工具证据或预算变更当产品通过。",
})
write(RCP, receipt)
write(ROOT / "manifest.json", {"files": [{"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
                                         "sha256": sha(p.read_bytes())} for p in sorted(ROOT.rglob("*")) if p.is_file()]})
verify()
print(json.dumps(result))
