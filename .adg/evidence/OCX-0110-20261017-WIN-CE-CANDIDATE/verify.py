"""Verify the sole candidate's existing artifacts; never execute a payload."""
import hashlib
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
SOURCE = "ce670ce76bb1c29f70989ee54838d5b31e97059c"
RUN = 38996189227
EXE = "4fe09bce265b51860da17f2cd1466a50430513e5ed29050f85caf5d913652bb5"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify():
    original = ROOT / "original"
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert data == Path(row["runtimePath"]).read_bytes()
    state = read(original / "candidate-terminal.original.json")
    run, jobs = state["run"], state["jobs"]
    assert run["id"] == RUN and run["head_sha"] == SOURCE and run["run_attempt"] == 1
    assert run["event"] == "workflow_dispatch" and run["path"] == ".github/workflows/windows-candidate.yml"
    assert run["status"] == "completed" and run["conclusion"] == "success"
    assert jobs["total_count"] == len(jobs["jobs"]) == 3
    job, = [j for j in jobs["jobs"] if j["name"] == "package"]
    assert job["id"] == 117289165536 and job["head_sha"] == SOURCE and job["conclusion"] == "success"
    assert all(j["conclusion"] == "skipped" for j in jobs["jobs"] if j["name"] != "package")
    log = (original / "candidate-package.original.log").read_bytes()
    log_proof = read(original / "candidate-package-log-verification.json")
    assert len(log) == log_proof["bytes"] and sha(log) == log_proof["sha256"]
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.decode("utf-8-sig"))
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == SOURCE
    tests = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert len(tests) == 35 and tuple(sum(int(r[i]) for r in tests) for i in range(3)) == (651, 0, 7)
    downloads = read(original / "candidate-download-result.json")
    assert downloads["source"] == SOURCE and downloads["runId"] == RUN
    assert len(downloads["artifacts"]) == 2
    api_artifacts = state["artifacts"]
    assert api_artifacts["total_count"] == len(api_artifacts["artifacts"]) == 2
    for row in downloads["artifacts"]:
        api, = [a for a in api_artifacts["artifacts"] if a["id"] == row["artifactId"]]
        assert api["workflow_run"]["id"] == RUN and api["workflow_run"]["head_sha"] == SOURCE
        assert not api["expired"] and api["digest"] == "sha256:" + row["sha256"]
        data = Path(row["path"]).read_bytes()
        assert len(data) == row["bytes"] == api["size_in_bytes"] and sha(data) == row["sha256"]
        with zipfile.ZipFile(Path(row["path"])) as archive:
            assert archive.testzip() is None
            for member in row["entries"]:
                payload = archive.read(member["path"])
                assert len(payload) == member["bytes"] and sha(payload) == member["sha256"]
    receipt = read(original / "candidate-installer-verification.original.json")
    environment = read(original / "candidate-environment.original.json")
    assert receipt["sourceCommit"] == environment["source"] == SOURCE
    assert receipt["result"] == "pass" and receipt["version"] == "0.1.10"
    assert not receipt["codeSigned"] and not receipt["releaseAccepted"]
    descriptor = read(original / "candidate-descriptor.json")
    assert descriptor["source"] == SOURCE and descriptor["runId"] == RUN
    assert descriptor["exeSha256"] == EXE and descriptor["sameSourceCiAndArtifactVerified"]
    exe_bytes = Path(descriptor["executable"]).read_bytes()
    assert len(exe_bytes) == descriptor["bytes"] == 8830976 and sha(exe_bytes) == EXE
    msi, = [r for r in receipt["installers"] if r["kind"] == "msi"]
    nsis, = [r for r in receipt["installers"] if r["kind"] == "nsis"]
    for installer in (msi, nsis):
        assert installer["result"] == installer["firstScreen"] == "pass"
        assert installer["installExitCode"] == installer["uninstallExitCode"] == 0
        assert installer["uninstallRemovedExecutable"]
        assert installer["installedExecutableSha256"] == installer["expectedInstalledExecutableSha256"]
        package, = Path(downloads["destination"]).joinpath("installers").rglob(installer["name"])
        assert sha(package.read_bytes()) == installer["sha256"]
    assert msi["installedExecutableSha256"] == EXE
    assert descriptor["msiPackageSha256"] == msi["sha256"]
    offset = descriptor["markerStartOffset"]
    prefix = b"__TAURI_BUNDLE_TYPE_VAR_"
    assert offset == msi["bundleMarkerOffset"] == nsis["bundleMarkerOffset"] == 8148685
    assert exe_bytes[offset:offset + len(prefix) + 3] == prefix + b"MSI"
    def variant(suffix):
        payload = bytearray(exe_bytes)
        payload[offset + len(prefix):offset + len(prefix) + 3] = suffix
        return sha(payload)
    assert variant(b"NSS") == descriptor["inMemoryNssVariantSha256"] == nsis["installedExecutableSha256"]
    assert variant(b"UNK") == descriptor["inMemoryBuiltVariantSha256"] == receipt["builtExecutableSha256"]
    extraction = read(original / "candidate-msi-readonly.original.json")
    assert extraction["databaseMode"] == "read-only" and not extraction["installerExecuted"]
    assert extraction["result"] == "pass" and extraction["packageSha256"] == msi["sha256"]
    corrected = read(original / "extract-method-check-correction.json")
    assert corrected["lfNormalizedWorkingFileSha256"] == corrected["expectedLfSha256"]
    assert not corrected["scriptChanged"] and not corrected["extractionStartedBeforeCheck"]
    binding = read(ROOT / "ci-binding.json")
    ci = PROJECT / binding["directory"]
    assert sha((ci / "manifest.json").read_bytes()) == binding["manifestSha256"]
    admission = read(ci / "admission.json")
    assert admission["source"] == SOURCE and admission["runId"] == 38995008914
    assert admission["status"] == "completed" and admission["conclusion"] == "success"
    assert sha((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_bytes()) == \
        "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            data = (ROOT / row["path"]).read_bytes()
            assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    return {"status": "verified-exact-ce-candidate", "source": SOURCE, "runId": RUN,
            "exeSha256": EXE, "bytes": len(exe_bytes), "artifactCount": 2,
            "packageWindowsTestsPassed": 651, "sameSourceCiAndArtifactsVerified": True,
            "runnerMsiAndNsisSmokePassed": True, "hostInstallerExecuted": False,
            "hostProductStarted": False, "nativePerformanceGatePassed": False,
            "historicalFailuresCleared": False, "fullVersionGatePassed": False, "releaseAccepted": False}


if __name__ == "__main__":
    print(json.dumps(verify()))
