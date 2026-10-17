"""Verify two source-bound first CI runs; retain failures without replay."""
import hashlib
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")
RUNS = {
    "control": CODE / "test/runtime/query-control-approved-20261017T1938",
    "ipc-waits": CODE / "test/runtime/query-idle-waits-20261017T1956",
}
SOURCES = {
    "control": "ee0111e423867e1c3b304d4fe9fd7d4380384df0",
    "ipc-waits": "644fcf35f3f18bbbb46da1e6fed1995c8a572b8c",
}


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert data == Path(row["runtimePath"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    control = read(ROOT / "control/approved-change.json")
    assert control["humanDecision"] == "允许"
    assert control["assertionCount"] == 16 and control["assertionMacrosAndBodiesIdentical"]
    assert control["durationLiteralsIdentical"] and not control["productionQuerySourceChanged"]
    assert not control["rustTestConcurrencyChanged"] and not control["productionRootCauseProved"]
    conclusions = {}
    for key, source in SOURCES.items():
        directory = ROOT / key
        push = read(directory / "push-confirmed.json")
        assert push["source"] == source and push["remoteReadbackMatched"] and not push["force"]
        admission = read(directory / "ci-complete-admission.json")
        assert admission["source"] == source and admission["event"] == "push"
        assert admission["status"] == "completed" and admission["attempt"] == 1
        terminal = read(directory / "ci-terminal.original.json")
        assert terminal["run"]["head_sha"] == source and terminal["run"]["run_attempt"] == 1
        assert terminal["run"]["conclusion"] == admission["conclusion"]
        assert {row["job"] for row in admission["requiredJobLogs"]} == {
            "backend", "backend-windows", "frontend", "release-tools", "windows-smoke-evidence",
        }
        for row in admission["requiredJobLogs"]:
            data = (directory / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
            assert row["actualCheckout"] == source
        conclusions[key] = admission["conclusion"]
    assert conclusions["control"] == "failure"
    failure = read(ROOT / "control/ci-failure-analysis.json")
    assert failure["windowsLibrary"] == {"passed": 555, "failed": 9, "ignored": 4}
    assert len(failure["queryCases"]) == 7 and failure["queryStartupMarkerMissingCases"] == 7
    assert all(row["result"] == "Timeout" for row in failure["queryCases"])
    assert not failure["approvedAdjustmentMadeTheQueryCasesPass"]
    assert not failure["productionQueryRootCauseProved"]
    artifact = read(ROOT / "control/failed-ci-artifact-verification.json")
    data = Path(artifact["path"]).read_bytes()
    assert len(data) == artifact["bytes"] and sha(data) == artifact["sha256"]
    with zipfile.ZipFile(artifact["path"]) as archive:
        assert archive.testzip() is None
        for row in artifact["entries"]:
            payload = archive.read(row["path"])
            assert len(payload) == row["bytes"] and sha(payload) == row["sha256"]
    boundary = read(ROOT / "ipc-waits/source-boundary.json")
    assert boundary["otherSourceAndProductionSectionsIdentical"]
    assert boundary["allDurationLiteralsIdentical"] and boundary["queryFixtureAndProductionUnchanged"]
    for key in SOURCES:
        boundary = read(ROOT / key / "source-boundary.json")
        rows = boundary.get("unchangedCriticalFiles", boundary.get("criticalFiles"))
        for row in rows:
            data = subprocess.check_output(
                ["git", "show", SOURCES[key] + ":" + row["path"]], cwd=CODE,
            )
            assert sha(data) == row["sha256"]
    protection = read(ROOT / "protection.json")
    assert protection["nativeCount"] == 0 and protection["originalPreferencesUnchanged"]
    assert protection["originalPointerUnchanged"] and protection["taskUnchanged"]
    return {
        "controlSource": SOURCES["control"], "controlFirstCi": conclusions["control"],
        "ipcWaitSource": SOURCES["ipc-waits"], "ipcWaitFirstCi": conclusions["ipc-waits"],
        "productionQueryChanged": False, "historicalRootCauseProved": False,
        "hostProductStarted": False, "fullWindowsGatePassed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
else:
    assert not (ROOT / "manifest.json").exists()
    copies = []
    common = [
        "push-confirmed.json", "ci-handle.json", "source-boundary.json",
        "ci-complete-admission.json", "ci-terminal.original.json",
        "ci-complete-backend.original.log", "ci-complete-backend-windows.original.log",
        "ci-complete-frontend.original.log", "ci-complete-release-tools.original.log",
        "ci-complete-windows-smoke-evidence.original.log",
    ]
    for key, run in RUNS.items():
        names = common + ([
            "approved-change.json", "ci-failure-analysis.json", "ci-artifacts.original.json",
            "failed-ci-artifact-verification.json",
        ] if key == "control" else ["change-boundary.json"])
        for name in names:
            data = (run / name).read_bytes()
            assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
            target = ROOT / key / name
            target.parent.mkdir(exist_ok=True)
            with target.open("xb") as stream:
                stream.write(data)
            copies.append({"path": key + "/" + name, "runtimePath": str(run / name),
                           "bytes": len(data), "sha256": sha(data)})
    write(ROOT / "copies.json", copies)
    write(ROOT / "verification.json", verify())
    (ROOT / ".gitattributes").write_bytes(b"* -text\n")
    write(ROOT / "manifest.json", {"files": [
        {"path": path.relative_to(ROOT).as_posix(), "bytes": path.stat().st_size,
         "sha256": sha(path.read_bytes())}
        for path in sorted(ROOT.rglob("*")) if path.is_file()
    ]})
print(json.dumps(read(ROOT / "verification.json")))
