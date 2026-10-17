"""Freeze the single failed candidate and diagnostic ZIP, without replaying it."""
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/test/runtime/windows-uninstall-data-20261017T1420/revision6")
CI = ROOT.parent / "ci"
SOURCE = "1dccf22ec3b93a0b08115eaf794fb45ec9814d2a"
RUN = 39012344655


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
    terminal = read(ROOT / "candidate-failed-terminal.original.json")
    run, jobs = terminal["run"], terminal["jobs"]["jobs"]
    assert run["id"] == RUN and run["head_sha"] == SOURCE and run["run_attempt"] == 1
    assert run["event"] == "workflow_dispatch" and run["conclusion"] == "failure"
    assert run["status"] == "completed"
    package, = [j for j in jobs if j["name"] == "package"]
    assert package["id"] == 117337352360 and package["conclusion"] == "failure"
    phases = {s["name"]: s["conclusion"] for s in package["steps"]}
    assert phases["Windows regression with the package lock"] == "failure"
    assert phases["Build NSIS and MSI candidates"] == "skipped"
    assert phases["Install, verify final EXE and uninstall on isolated runner"] == "skipped"
    assert phases["Upload verified Windows installers"] == "skipped"
    raw = (ROOT / "candidate-failed-package.original.log").read_bytes()
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", raw.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    counts = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert counts == [("FAILED", "562", "2", "4")]
    names = sorted(set(re.findall(r"test ([^\r\n]+) \.\.\. FAILED", text)))
    analysis = read(ROOT / "candidate-failure-analysis.json")
    assert names == analysis["failures"] and len(names) == 2
    assert analysis["source"] == SOURCE and analysis["runId"] == RUN
    assert analysis["logSha256"] == sha(raw) and analysis["logBytes"] == len(raw)
    for mode in ("success", "nonzero"):
        case = analysis["cases"][mode]
        assert case["queryResult"] == "Timeout" and 2 <= case["queryElapsedSeconds"] < 4
        assert not case["queryReadinessMarkersObserved"]
        assert case["controlMarkerObservedAfterOriginalObserverDeadline"]
        for snapshot in case["controlProcessRetainedSnapshots"]:
            assert snapshot["exit_code"] == 259 and snapshot["wait_status"] == 258
            assert snapshot["process_times"]["exit_filetime_100ns"] == 0
    assert not analysis["rootCauseOrSchedulingIoCausalityProved"]
    assert not analysis["testBudgetChanged"] and not analysis["candidateRerunRequested"]
    diagnostic = read(ROOT / "failed-candidate-artifact-verification.json")
    assert diagnostic["artifactId"] == 12094175208 and diagnostic["runId"] == RUN
    assert diagnostic["bytes"] == 278234
    assert diagnostic["sha256"] == "a3142b734c9089dc2d0f7f5efa956f680440b9a1b37c3c5c1448863027767818"
    data = Path(diagnostic["path"]).read_bytes()
    assert len(data) == diagnostic["bytes"] and sha(data) == diagnostic["sha256"]
    with zipfile.ZipFile(Path(diagnostic["path"])) as archive:
        assert archive.testzip() is None
        assert len(diagnostic["entries"]) == 5
        for item in diagnostic["entries"]:
            payload = archive.read(item["path"])
            assert sha(payload) == item["sha256"] and len(payload) == item["bytes"]
    environment = read(ROOT / "failed-candidate-environment.original.json")
    assert environment["source"] == SOURCE
    ci = read(CI / "ci-complete-admission.json")
    assert ci["source"] == SOURCE and ci["runId"] == 39011042117 and ci["conclusion"] == "success"
    binding = read(ROOT / "ci-binding.json")
    assert sha((CI / "manifest.json").read_bytes()) == binding["manifestSha256"]
    protection = read(ROOT / "failed-candidate-host-protection.json")
    assert protection["nativeCount"] == 0
    assert protection["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert protection["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert protection["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    return {
        "status": "single-candidate-failed-before-installer-build",
        "source": SOURCE, "runId": RUN, "jobId": package["id"],
        "sameSourceFullCiStillSuccess": True, "libTestsPassed": 562,
        "libTestsFailed": 2, "libTestsIgnored": 4, "failedVersionQueryCases": names,
        "queryReadinessMarkersAbsent": True, "rootCauseProved": False,
        "installerBuilt": False, "installedCleanupHelperTested": False,
        "diagnosticZipVerified": True, "candidateAdmitted": False,
        "sameShaRerunRequested": False, "testBudgetChanged": False,
        "originalBd2FailureReplaced": False, "hostProductOrCleanupStarted": False,
        "taskClosed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        assert sha((ROOT / row["path"]).read_bytes()) == row["sha256"]
    print(json.dumps(read(ROOT / "verification.json")))
    sys.exit(0)

assert not (ROOT / "manifest.json").exists()
names = [
    "candidate-failed-terminal.original.json", "candidate-failed-package.original.log",
    "candidate-failure-analysis.json", "failed-candidate-artifact-verification.json",
    "failed-candidate-environment.original.json", "failed-candidate-host-protection.json",
]
copies = []
for name in names:
    data = (RUNTIME / name).read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
    with (ROOT / name).open("xb") as stream:
        stream.write(data)
    copies.append({"path": name, "runtimePath": str(RUNTIME / name),
                   "bytes": len(data), "sha256": sha(data)})
write(ROOT / "copies.json", copies)
write(ROOT / "ci-binding.json", {"manifestSha256": sha((CI / "manifest.json").read_bytes())})
write(ROOT / "verification.json", verify())
write(ROOT / "manifest.json", {"files": [{
    "path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p.read_bytes()),
} for p in sorted(ROOT.rglob("*")) if p.is_file()]})
print(json.dumps(read(ROOT / "verification.json")))
