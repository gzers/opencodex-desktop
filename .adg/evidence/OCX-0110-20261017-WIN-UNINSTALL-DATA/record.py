"""Record the scoped product fix and actual CI checkpoint; never run cleanup."""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")
RUNTIME = CODE / "test/runtime/windows-uninstall-data-20261017T1420"
CURRENT = RUNTIME / "revision5"
SOURCE = "0e81500f262289a9bf0cf722c3ca9fa70471c1b0"


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
        assert sha(data) == row["sha256"] and len(data) == row["bytes"]
        assert data == Path(row["runtimePath"]).read_bytes()
    original = ROOT / "original"
    boundary = read(original / "source-boundary.json")
    assert boundary["source"] == SOURCE and boundary["base"] == "99a62e4ad35e9404cb08922571ad8b77f89214dd"
    assert len(boundary["files"]) == 9
    for row in boundary["files"]:
        data = subprocess.check_output(["git", "show", SOURCE + ":" + row["path"]], cwd=CODE)
        assert sha(data) == row["sha256"]
    checkpoint = read(original / "current-ci-checkpoint.original.json")
    run, jobs = checkpoint["run"], checkpoint["jobs"]["jobs"]
    assert run["id"] == 39010269240 and run["head_sha"] == SOURCE and run["run_attempt"] == 1
    assert run["event"] == "push" and run["path"] == ".github/workflows/ci.yml"
    assert {j["name"] for j in jobs} == {
        "frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence",
    }
    mac = next(j for j in jobs if j["name"] == "backend")
    windows = next(j for j in jobs if j["name"] == "backend-windows")
    assert mac["conclusion"] == "success"
    assert next(s for s in windows["steps"] if s["name"] == "Windows backend regression")["conclusion"] == "success"
    text = (original / "current-mac-backend.original.log").read_text(encoding="utf-8-sig")
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    cases = [line for line in text.splitlines() if "data_root::uninstall::tests::" in line]
    assert len(cases) == 8 and all(line.endswith(" ... ok") for line in cases)
    counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert len(counts) == 35 and tuple(sum(int(r[i]) for r in counts) for i in range(3)) == (935, 0, 6)
    first = read(original / "first-ci-terminal-summary.json")
    assert first["source"] == "3d9634ac830a207513dfd7368ab7b8c8bf4f5107"
    assert first["conclusion"] == "failure"
    assert "Diff in " in (original / "first-format-failure.original.log").read_text(encoding="utf-8-sig")
    visibility = (original / "windows-visibility-failure.original.log").read_text(encoding="utf-8-sig")
    assert "E0364" in visibility and "remove_for_uninstall" in visibility
    assert not boundary["hostProductStarted"] and not boundary["hostUninstallOrCleanupExecuted"]
    return {
        "source": SOURCE, "branch": boundary["branch"], "ciRunId": 39010269240,
        "scopedCodeAndTestFiles": 9, "macBackendSuites": 35, "macBackendPassed": 935,
        "macCleanupCoreCasesPassed": 8, "windowsBackendRegressionStepPassed": True,
        "wholeCiAdmittedAtCheckpoint": False, "installerHookCompiledAtCheckpoint": False,
        "realInstalledHelperSmokePassed": False, "hostCleanupExecuted": False,
        "newCandidateDispatchedAtCheckpoint": False, "nativeDataRootSelectionPassed": False,
        "fullInstallerGatePassed": False, "taskClosed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        assert sha((ROOT / row["path"]).read_bytes()) == row["sha256"]
    print(json.dumps(read(ROOT / "verification.json")))
    sys.exit(0)

assert not (ROOT / "manifest.json").exists()
latest = sorted(CURRENT.glob("ci-terminal-check-*.original.json"))[-1]
selected = [
    (CURRENT / "source-boundary.json", "source-boundary.json"),
    (CURRENT / "ci-handle.json", "current-ci-handle.json"),
    (latest, "current-ci-checkpoint.original.json"),
    (CURRENT / "ci-complete-backend.original.log", "current-mac-backend.original.log"),
    (RUNTIME / "ci-complete-admission.json", "first-ci-terminal-summary.json"),
    (RUNTIME / "first-ci-format-failure.original.log", "first-format-failure.original.log"),
    (RUNTIME / "revision2/windows-clippy-failure.original.log", "windows-visibility-failure.original.log"),
]
copies = []
for path, name in selected:
    data = path.read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig"))
    target = ROOT / "original" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as stream:
        stream.write(data)
    copies.append({"path": "original/" + name, "runtimePath": str(path),
                   "bytes": len(data), "sha256": sha(data)})
write(ROOT / "copies.json", copies)
write(ROOT / "verification.json", verify())
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
write(ROOT / "manifest.json", {"files": [{
    "path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p.read_bytes()),
} for p in sorted(ROOT.rglob("*")) if p.is_file()]})
print(json.dumps(read(ROOT / "verification.json")))
