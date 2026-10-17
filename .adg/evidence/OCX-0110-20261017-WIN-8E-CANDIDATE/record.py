"""Freeze the unique, fully admitted 8e candidate without executing it."""
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")
RUN = CODE / "test/runtime/uninstall-owned-files-20261017T2020"
CI = ROOT.parent / "OCX-0110-20261017-WIN-UNINSTALL-OWNED-FILES"
SOURCE = "8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b"
RUN_ID = 39033532953
EXE = "870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a"


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
    assert sha((CI / "manifest.json").read_bytes()) == read(ROOT / "ci-binding.json")["manifestSha256"]
    admission = read(CI / "original/ci-complete-admission.json")
    assert admission["source"] == SOURCE and admission["runId"] == 39032153138
    assert admission["wholeToolCiAdmitted"] and admission["conclusion"] == "success"
    for row in admission["requiredJobLogs"]:
        data = (CI / "original" / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert row["actualCheckout"] == SOURCE and row["conclusion"] == "success"
    original = ROOT / "original"
    request = read(original / "candidate-dispatch-request.json")
    assert request["ref"] == "feature/0.1.10-windows-uninstall-data"
    assert request["inputs"]["purpose"] == "candidate"
    assert request["inputs"]["admission_run"] == "39032153138"
    assert not read(original / "candidate-dispatch-attempt.json")["automaticRetryAllowed"]
    assert read(original / "candidate-dispatch-response.json")["status"] == 204
    assert read(original / "candidate-dispatch-before.original.json")["workflow_runs"] == []
    terminal = read(original / "candidate-terminal.original.json")
    run, jobs, artifacts = terminal["run"], terminal["jobs"], terminal["artifacts"]
    assert run["id"] == RUN_ID and run["head_sha"] == SOURCE and run["run_attempt"] == 1
    assert run["status"] == "completed" and run["conclusion"] == "success"
    assert run["event"] == "workflow_dispatch" and run["path"] == ".github/workflows/windows-candidate.yml"
    assert jobs["total_count"] == len(jobs["jobs"]) == 3
    package, = [job for job in jobs["jobs"] if job["name"] == "package"]
    assert package["id"] == 117398708696 and package["conclusion"] == "success"
    assert package["head_sha"] == SOURCE
    assert all(job["conclusion"] == "skipped" for job in jobs["jobs"] if job["name"] != "package")
    log = (original / "candidate-package.original.log").read_bytes()
    log_proof = read(original / "candidate-package-log-verification.json")
    assert len(log) == log_proof["bytes"] and sha(log) == log_proof["sha256"]
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    tests = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert len(tests) == 35
    assert tuple(sum(int(row[i]) for row in tests) for i in (0, 1, 2)) == (667, 0, 7)
    downloads = read(original / "candidate-download-result.json")
    assert downloads["source"] == SOURCE and downloads["runId"] == RUN_ID
    assert len(downloads["artifacts"]) == artifacts["total_count"] == 2
    expected = {
        12101766414: (7729981, "26a206fb4ede180878c263774dd3b99e8222e86c4ccfda421884635032b5dd51"),
        12101038569: (1653736, "67466dd77809f93575ef903b0296299a8a822a4cd02527e3cb9a316d060b2f57"),
    }
    for row in downloads["artifacts"]:
        api, = [a for a in artifacts["artifacts"] if a["id"] == row["artifactId"]]
        size, digest = expected[row["artifactId"]]
        data = Path(row["path"]).read_bytes()
        assert len(data) == size == api["size_in_bytes"] and sha(data) == digest
        assert api["digest"] == "sha256:" + digest and not api["expired"]
        assert api["workflow_run"]["id"] == RUN_ID and api["workflow_run"]["head_sha"] == SOURCE
        with zipfile.ZipFile(row["path"]) as archive:
            assert archive.testzip() is None
            for member in row["entries"]:
                payload = archive.read(member["path"])
                assert len(payload) == member["bytes"] and sha(payload) == member["sha256"]
    receipt = read(original / "candidate-installer-verification.original.json")
    environment = read(original / "candidate-environment.original.json")
    assert receipt["sourceCommit"] == environment["source"] == SOURCE
    assert receipt["version"] == "0.1.10" and receipt["result"] == "pass"
    assert not receipt["codeSigned"] and not receipt["releaseAccepted"]
    assert {row["kind"] for row in receipt["installers"]} == {"msi", "nsis"}
    for row in receipt["installers"]:
        assert row["installExitCode"] == row["uninstallExitCode"] == 0
        assert row["result"] == row["firstScreen"] == "pass"
        assert row["uninstallRemovedExecutable"] and row["defaultUninstallRetainedData"]
        assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
        cleanup = row["explicitCleanupHelper"]
        assert cleanup["unconfirmedExitCode"] == 2 and cleanup["confirmedExitCode"] == 0
        assert all(cleanup[key] for key in (
            "startupPointerRemoved", "anchorAndActiveRootRemoved", "externalHomeUnchanged", "applicationRetained",
        ))
        assert not cleanup["applicationRootBindingTested"] and not cleanup["hostAcceptanceClaimed"]
        path, = (Path(downloads["destination"]) / "installers").rglob(row["name"])
        data = path.read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    descriptor = read(original / "candidate-descriptor.json")
    assert descriptor["source"] == SOURCE and descriptor["runId"] == RUN_ID
    data = Path(descriptor["executable"]).read_bytes()
    assert len(data) == descriptor["bytes"] == 8832000 and sha(data) == descriptor["exeSha256"] == EXE
    msi, = [row for row in receipt["installers"] if row["kind"] == "msi"]
    nsis, = [row for row in receipt["installers"] if row["kind"] == "nsis"]
    assert msi["installedExecutableSha256"] == EXE
    offset = descriptor["markerStartOffset"]
    assert offset == msi["bundleMarkerOffset"] == nsis["bundleMarkerOffset"] == 8149749
    prefix = b"__TAURI_BUNDLE_TYPE_VAR_"
    assert data[offset:offset + len(prefix) + 3] == prefix + b"MSI"
    for suffix, digest in [
        (b"NSS", nsis["installedExecutableSha256"]), (b"UNK", receipt["builtExecutableSha256"]),
    ]:
        variant = bytearray(data)
        variant[offset + len(prefix):offset + len(prefix) + 3] = suffix
        assert sha(variant) == digest
    extraction = read(original / "candidate-msi-readonly.original.json")
    assert extraction["result"] == "pass" and extraction["databaseMode"] == "read-only"
    assert not extraction["installerExecuted"]
    assert descriptor["msiReadonlyMethodSha256"] == "98a35f58c8d1ee9608e81630de88a3f1a62fd56eb9bca766e4b5dd2b6284e6e8"
    assert not descriptor["installerExecuted"] and not descriptor["productStarted"] and not descriptor["originalExeWritten"]
    protection = read(ROOT / "protection.json")
    assert protection["nativeCount"] == 0 and protection["originalPreferencesUnchanged"]
    assert protection["originalPointerUnchanged"] and protection["taskUnchanged"]
    return {
        "status": "verified-exact-8e-candidate", "source": SOURCE, "runId": RUN_ID,
        "sameSourceFullCiRunId": 39032153138, "exeSha256": EXE, "exeBytes": len(data),
        "artifactCount": 2, "packageWindowsTestsPassed": 667,
        "runnerMsiAndNsisInstallFirstScreenAndUninstallPassed": True,
        "runnerUnconfirmedCleanupRefusedAndConfirmedCleanupPassed": True,
        "runnerExternalHomeAndDefaultUninstallRetentionPassed": True,
        "hostInstallerOrProductExecuted": False, "hostUninstallerCheckboxTested": False,
        "ordinaryRestartReadbackCompleted": False, "newPerformanceMeasurement": False,
        "historicalFailureRootCauseProved": False, "fullWindowsGatePassed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
else:
    assert not (ROOT / "manifest.json").exists()
    names = [
        "candidate-terminal.original.json", "candidate-package.original.log",
        "candidate-package-log-verification.json", "candidate-download-result.json",
        "candidate-installer-verification.original.json", "candidate-environment.original.json",
        "candidate-msi-readonly.original.json", "candidate-descriptor.json",
        "candidate-dispatch-attempt.json", "candidate-dispatch-before.original.json",
        "candidate-dispatch-request.json", "candidate-dispatch-response.json", "candidate-handle.json",
        "candidate-artifacts.py", "extract-candidate.ps1",
    ]
    copies = []
    for name in names:
        data = (RUN / name).read_bytes()
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
        target = ROOT / "original" / name
        target.parent.mkdir(exist_ok=True)
        with target.open("xb") as stream:
            stream.write(data)
        copies.append({"path": "original/" + name, "runtimePath": str(RUN / name),
                       "bytes": len(data), "sha256": sha(data)})
    write(ROOT / "copies.json", copies)
    write(ROOT / "ci-binding.json", {"directory": CI.name, "manifestSha256": sha((CI / "manifest.json").read_bytes())})
    write(ROOT / "verification.json", verify())
    (ROOT / ".gitattributes").write_bytes(b"* -text\n")
    write(ROOT / "manifest.json", {"files": [
        {"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
         "sha256": sha(p.read_bytes())}
        for p in sorted(ROOT.rglob("*")) if p.is_file()
    ]})
print(json.dumps(read(ROOT / "verification.json")))
