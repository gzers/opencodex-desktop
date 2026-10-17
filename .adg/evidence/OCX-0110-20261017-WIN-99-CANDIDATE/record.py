"""Freeze the single existing 99a62e4a candidate; no product or installer execution."""
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/test/runtime/empty-root-fix-20261017T1320/revision2")
CI = ROOT.parent / "OCX-0110-20261017-WIN-CE-DATA-ROOT/ci"
SOURCE = "99a62e4ad35e9404cb08922571ad8b77f89214dd"
RUN = 39007300430
EXE = "43e6deb5b2ea4a1b886c51a0c75c0eb9ca4b19a32126d1d44e5dca55a40535c8"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def verify():
    original = ROOT / "original"
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert data == Path(row["runtimePath"]).read_bytes()
    binding = read(ROOT / "ci-binding.json")
    assert sha((CI / "manifest.json").read_bytes()) == binding["manifestSha256"]
    ci = read(CI / "current-ci-complete-admission.json")
    assert ci["source"] == SOURCE and ci["runId"] == 39006330384
    assert ci["status"] == "completed" and ci["conclusion"] == "success"
    assert {j["job"] for j in ci["requiredJobLogs"]} == {
        "backend", "backend-windows", "frontend", "release-tools", "windows-smoke-evidence",
    }
    for job in ci["requiredJobLogs"]:
        data = (CI / job["path"]).read_bytes()
        assert job["actualCheckout"] == SOURCE and job["conclusion"] == "success"
        assert sha(data) == job["sha256"] and len(data) == job["bytes"]
    terminal = read(original / "candidate-terminal.original.json")
    run, jobs = terminal["run"], terminal["jobs"]
    assert run["id"] == RUN and run["head_sha"] == SOURCE and run["run_attempt"] == 1
    assert run["status"] == "completed" and run["conclusion"] == "success"
    assert run["event"] == "workflow_dispatch" and run["path"] == ".github/workflows/windows-candidate.yml"
    assert jobs["total_count"] == len(jobs["jobs"]) == 3
    package, = [j for j in jobs["jobs"] if j["name"] == "package"]
    assert package["id"] == 117322280951 and package["head_sha"] == SOURCE and package["conclusion"] == "success"
    assert all(j["conclusion"] == "skipped" for j in jobs["jobs"] if j["name"] != "package")
    log = (original / "candidate-package.original.log").read_bytes()
    proof = read(original / "candidate-package-log-verification.json")
    assert sha(log) == proof["sha256"] and len(log) == proof["bytes"]
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert len(counts) == 35 and tuple(sum(int(r[i]) for r in counts) for i in range(3)) == (654, 0, 7)
    downloads = read(original / "candidate-download-result.json")
    assert downloads["source"] == SOURCE and downloads["runId"] == RUN
    artifacts = terminal["artifacts"]
    assert len(downloads["artifacts"]) == artifacts["total_count"] == len(artifacts["artifacts"]) == 2
    expected_archives = {
        12090994649: (7669517, "87294d844f9ca109eb54587f3ec4a8605c81092e6e398df25ed4aca3cbb112a9"),
        12092230799: (1675741, "b19f495e92270946fdd95abe0a0bcba2b0137001b3fc47afad0c254456270ce0"),
    }
    for row in downloads["artifacts"]:
        api, = [a for a in artifacts["artifacts"] if a["id"] == row["artifactId"]]
        size, digest = expected_archives[row["artifactId"]]
        assert api["digest"] == "sha256:" + digest and api["size_in_bytes"] == size
        assert api["workflow_run"]["id"] == RUN and api["workflow_run"]["head_sha"] == SOURCE
        assert not api["expired"]
        data = Path(row["path"]).read_bytes()
        assert sha(data) == row["sha256"] == digest and len(data) == row["bytes"] == size
        with zipfile.ZipFile(Path(row["path"])) as archive:
            assert archive.testzip() is None
            assert {r["path"] for r in row["entries"]} == {m.filename for m in archive.infolist() if not m.is_dir()}
            for member in row["entries"]:
                payload = archive.read(member["path"])
                assert len(payload) == member["bytes"] and sha(payload) == member["sha256"]
    receipt = read(original / "candidate-installer-verification.original.json")
    environment = read(original / "candidate-environment.original.json")
    assert receipt["sourceCommit"] == environment["source"] == SOURCE
    assert receipt["result"] == "pass" and receipt["version"] == "0.1.10"
    assert not receipt["releaseAccepted"] and not receipt["codeSigned"]
    assert {r["kind"] for r in receipt["installers"]} == {"msi", "nsis"}
    for installer in receipt["installers"]:
        assert installer["result"] == installer["firstScreen"] == "pass"
        assert installer["installExitCode"] == installer["uninstallExitCode"] == 0
        assert installer["uninstallRemovedExecutable"]
        assert installer["installedExecutableSha256"] == installer["expectedInstalledExecutableSha256"]
        package_path, = Path(downloads["destination"]).joinpath("installers").rglob(installer["name"])
        assert sha(package_path.read_bytes()) == installer["sha256"]
    descriptor = read(original / "candidate-descriptor.json")
    assert descriptor["source"] == SOURCE and descriptor["runId"] == RUN and descriptor["exeSha256"] == EXE
    data = Path(descriptor["executable"]).read_bytes()
    assert sha(data) == EXE and len(data) == descriptor["bytes"] == 8831488
    msi, = [i for i in receipt["installers"] if i["kind"] == "msi"]
    nsis, = [i for i in receipt["installers"] if i["kind"] == "nsis"]
    assert msi["installedExecutableSha256"] == EXE and msi["sha256"] == descriptor["msiPackageSha256"]
    offset = descriptor["markerStartOffset"]
    assert offset == msi["bundleMarkerOffset"] == nsis["bundleMarkerOffset"] == 8149245
    prefix = b"__TAURI_BUNDLE_TYPE_VAR_"
    assert data[offset:offset + len(prefix) + 3] == prefix + b"MSI"
    for suffix, expected in [(b"NSS", nsis["installedExecutableSha256"]), (b"UNK", receipt["builtExecutableSha256"])]:
        changed = bytearray(data)
        changed[offset + len(prefix):offset + len(prefix) + 3] = suffix
        assert sha(changed) == expected
    extraction = read(original / "candidate-msi-readonly.original.json")
    assert extraction["result"] == "pass" and extraction["databaseMode"] == "read-only"
    assert not extraction["installerExecuted"] and extraction["packageSha256"] == msi["sha256"]
    assert descriptor["msiReadonlyMethodSha256"] == "98a35f58c8d1ee9608e81630de88a3f1a62fd56eb9bca766e4b5dd2b6284e6e8"
    assert not descriptor["installerExecuted"] and not descriptor["productStarted"] and not descriptor["originalExeWritten"]
    return {
        "status": "verified-exact-99-candidate", "source": SOURCE, "runId": RUN,
        "exeSha256": EXE, "bytes": len(data), "artifactCount": 2,
        "sameSourceFullCiRunId": 39006330384, "packageWindowsTestsPassed": 654,
        "runnerMsiAndNsisInstallFirstScreenAndUninstallPassed": True,
        "sourceLogsZipMembersCrcAndPayloadVerified": True, "msiReadOnlyExtraction": True,
        "hostInstallerExecuted": False, "hostProductStarted": False,
        "nativeStorageAcceptancePassed": False, "fullVersionGatePassed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        assert sha((ROOT / row["path"]).read_bytes()) == row["sha256"]
    print(json.dumps(read(ROOT / "verification.json")))
    sys.exit(0)

assert not (ROOT / "manifest.json").exists() and not (ROOT / "copies.json").exists()
names = [
    "candidate-terminal.original.json", "candidate-package.original.log",
    "candidate-package-log-verification.json", "candidate-download-result.json",
    "candidate-installer-verification.original.json", "candidate-environment.original.json",
    "candidate-msi-readonly.original.json", "candidate-descriptor.json",
    "candidate-artifacts.py", "extract-candidate.ps1",
]
copies = []
for name in names:
    data = (RUNTIME / name).read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
    target = ROOT / "original" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as stream:
        stream.write(data)
    copies.append({"path": "original/" + name, "runtimePath": str(RUNTIME / name),
                   "bytes": len(data), "sha256": sha(data)})
write(ROOT / "copies.json", copies)
write(ROOT / "ci-binding.json", {
    "directory": CI.relative_to(PROJECT).as_posix(), "manifestSha256": sha((CI / "manifest.json").read_bytes()),
})
write(ROOT / "verification.json", verify())
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
write(ROOT / "manifest.json", {"files": [{
    "path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p.read_bytes()),
} for p in sorted(ROOT.rglob("*")) if p.is_file()]})
print(json.dumps(read(ROOT / "verification.json")))
