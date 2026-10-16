"""Freeze or read-only verify the one b4d candidate, without executing it."""
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/test/runtime/mesh-initial-size-20261016T1857")
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
RUN = 38893610415
EXE = "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def write(name, value):
    target = ROOT / name
    assert not target.exists(), name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


freeze = sys.argv[1:] == ["--freeze"]
assert not sys.argv[1:] or freeze
if freeze:
    assert not (ROOT / "manifest.json").exists(), "Frozen evidence must not be rewritten"
    names = [
        "candidate-terminal.original.json", "candidate-download-result.json",
        "candidate-environment.original.json", "candidate-installer-verification.original.json",
        "candidate-msi-readonly.original.json", "candidate-descriptor.json",
        "candidate-package-log-verification.json", "candidate-package.original.log",
        "candidate-dispatch-request.json", "candidate-dispatch-attempt.json",
        "candidate-dispatch-readback.json", "prefetch-result.json",
        "candidate-artifacts.py", "prefetch-candidate.py", "extract-candidate.ps1",
        "extract-candidate.original.log", "pr-final-description-readback.original.json",
    ]
    copies = []
    for name in names:
        data = (RUNTIME / name).read_bytes()
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
        target = ROOT / "original" / name
        assert not target.exists()
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        copies.append({"path": "original/" + name, "runtimePath": str(RUNTIME / name),
                       "bytes": len(data), "sha256": sha(data)})
    write("original-copies.json", copies)
    ci = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-MESH-INITIAL-SIZE"
    write("ci-binding.json", {
        "source": SOURCE, "runId": 38891188021, "evidenceDirectory": ci.relative_to(PROJECT).as_posix(),
        "manifestSha256": sha((ci / "manifest.json").read_bytes()),
        "verificationSha256": sha((ci / "verification.json").read_bytes()),
        "actualCheckout": "e7272dddb093dcffa0a27dd8dad5e7422288218a",
        "wholeTreeEquivalent": True,
    })
    (ROOT / ".gitattributes").write_bytes(b"* -text\n")

terminal = read("original/candidate-terminal.original.json")
run = terminal["run"]
assert run["id"] == RUN and run["head_sha"] == SOURCE
assert run["event"] == "workflow_dispatch" and run["run_attempt"] == 1
assert run["status"] == "completed" and run["conclusion"] == "success"
assert [j["conclusion"] for j in terminal["jobs"]["jobs"] if j["name"] == "package"] == ["success"]
for name in ["query-etw", "retention"]:
    assert [j["conclusion"] for j in terminal["jobs"]["jobs"] if j["name"] == name] == ["skipped"]
download = read("original/candidate-download-result.json")
assert download["source"] == SOURCE and download["runId"] == RUN
assert len(download["artifacts"]) == 2 and not download["hostInstallerExecuted"]
for artifact in download["artifacts"]:
    metadata, = [a for a in terminal["artifacts"]["artifacts"] if a["id"] == artifact["artifactId"]]
    data = Path(artifact["path"]).read_bytes()
    assert len(data) == artifact["bytes"] == metadata["size_in_bytes"]
    assert sha(data) == artifact["sha256"] == metadata["digest"].removeprefix("sha256:")
    assert metadata["workflow_run"]["head_sha"] == SOURCE and not metadata["expired"]
    with zipfile.ZipFile(artifact["path"]) as archive:
        assert archive.testzip() is None
        entries = [m for m in archive.infolist() if not m.is_dir()]
        assert len(entries) == len(artifact["entries"])
        for entry in artifact["entries"]:
            payload = archive.read(entry["path"])
            assert len(payload) == entry["bytes"] and sha(payload) == entry["sha256"]
receipt = read("original/candidate-installer-verification.original.json")
environment = read("original/candidate-environment.original.json")
descriptor = read("original/candidate-descriptor.json")
assert receipt["sourceCommit"] == environment["source"] == descriptor["source"] == SOURCE
assert receipt["version"] == descriptor["productVersion"] == "0.1.10" and receipt["result"] == "pass"
assert not receipt["releaseAccepted"] and not receipt["codeSigned"]
payload = Path(descriptor["executable"]).read_bytes()
assert len(payload) == descriptor["bytes"] == 8830976
assert sha(payload) == descriptor["sha256"] == descriptor["exeSha256"] == EXE
for row in receipt["installers"]:
    path, = (Path(download["destination"]) / "installers").rglob(row["name"])
    data = path.read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    assert row["result"] == row["firstScreen"] == "pass"
    assert row["installExitCode"] == row["uninstallExitCode"] == 0 and row["uninstallRemovedExecutable"]
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
msi, = [row for row in receipt["installers"] if row["kind"] == "msi"]
nss, = [row for row in receipt["installers"] if row["kind"] == "nsis"]
assert msi["installedExecutableSha256"] == EXE
assert descriptor["msiPackageSha256"] == msi["sha256"]
offset = descriptor["markerStartOffset"]
prefix = b"__TAURI_BUNDLE_TYPE_VAR_"
assert payload[offset:offset + len(prefix) + 3] == prefix + b"MSI"
variant = bytearray(payload)
variant[offset + len(prefix):offset + len(prefix) + 3] = b"NSS"
assert sha(variant) == nss["installedExecutableSha256"] == descriptor["inMemoryNssVariantSha256"]
variant[offset + len(prefix):offset + len(prefix) + 3] = b"UNK"
assert sha(variant) == receipt["builtExecutableSha256"] == descriptor["inMemoryBuiltVariantSha256"]
assert descriptor["exactThreeByteBundleSuffixVariantsPassed"] and descriptor["runnerMsiPayloadHashMatched"]
assert not descriptor["installerExecuted"] and not descriptor["productStarted"] and descriptor["nativeCount"] == 0
log_data = (ROOT / "original/candidate-package.original.log").read_bytes()
log_verification = read("original/candidate-package-log-verification.json")
assert len(log_data) == log_verification["bytes"] and sha(log_data) == log_verification["sha256"]
text = re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", log_data.decode("utf-8-sig"))
checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
assert checkout == log_verification["actualCheckout"] == SOURCE
counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
assert len(counts) == 35 and tuple(sum(int(c[i]) for c in counts) for i in range(3)) == (650, 0, 7)
ci_binding = read("ci-binding.json")
ci = PROJECT / ci_binding["evidenceDirectory"]
assert sha((ci / "manifest.json").read_bytes()) == ci_binding["manifestSha256"]
assert sha((ci / "verification.json").read_bytes()) == ci_binding["verificationSha256"]
assert json.loads((ci / "verification.json").read_text())["source"] == SOURCE
for copy in read("original-copies.json"):
    data = (ROOT / copy["path"]).read_bytes()
    assert len(data) == copy["bytes"] and sha(data) == copy["sha256"]
    assert data == Path(copy["runtimePath"]).read_bytes()
summary = {
    "status": "verified-exact-b4-candidate", "source": SOURCE, "runId": RUN,
    "exeSha256": EXE, "bytes": len(payload), "artifactCount": 2,
    "packageWindowsTestsPassed": 650, "sameSourceCiAndArtifactsVerified": True,
    "hostInstallerExecuted": False, "hostProductStarted": False,
    "nativePerformanceGatePassed": False, "historicalFailuresCleared": False,
    "fullVersionGatePassed": False, "releaseAccepted": False,
}
if freeze:
    write("verification.json", summary)
    write("manifest.json", {
        "frozen": True,
        "files": [{"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
                   "sha256": sha(p.read_bytes())}
                  for p in sorted(ROOT.rglob("*")) if p.is_file() and p.name != "manifest.json"],
        "source": SOURCE, "hostProductStarted": False,
    })
else:
    for row in read("manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
print(json.dumps(summary))
