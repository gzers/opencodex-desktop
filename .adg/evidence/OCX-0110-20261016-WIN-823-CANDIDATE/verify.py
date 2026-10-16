"""Verify the exact 823 candidate and readonly MSI payload, never run it."""
import hashlib
import json
import zipfile
import sys
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


preview = sys.argv[1:] == ["--preview"]
assert not sys.argv[1:] or preview
actual = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
          if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
if not preview:
    manifest = read("manifest.json")
    assert set(actual) == {e["path"] for e in manifest["files"]}
    for entry in manifest["files"]:
        data = actual[entry["path"]].read_bytes()
        assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
handle = read("candidate-handle.json")
terminal, download, descriptor = (read(name) for name in (
    "candidate-terminal.json", "candidate-download-result.json", "candidate-descriptor.json"))
assert terminal["run"]["id"] == handle["runId"] == download["runId"] == descriptor["runId"] == 38841117656
assert terminal["run"]["head_sha"] == handle["source"] == descriptor["source"] == download["source"] == SOURCE
assert terminal["run"]["conclusion"] == "success" and terminal["run"]["run_attempt"] == 1
assert terminal["run"]["event"] == "workflow_dispatch"
assert read("dispatch-result.json")["httpStatus"] == 204
for name in ("retention", "query-etw"):
    job, = [j for j in terminal["jobs"]["jobs"] if j["name"] == name]
    assert job["conclusion"] == "skipped"
package, = [j for j in terminal["jobs"]["jobs"] if j["name"] == "package"]
assert package["conclusion"] == "success"
receipt = read("installer-verification.raw.original.json")
assert receipt["sourceCommit"] == SOURCE and receipt["version"] == "0.1.10"
assert receipt["result"] == "pass" and not receipt["releaseAccepted"] and not receipt["codeSigned"]
for artifact in download["artifacts"]:
    api, = [row for row in terminal["artifacts"]["artifacts"] if row["id"] == artifact["artifactId"]]
    assert api["digest"] == "sha256:" + artifact["sha256"]
    assert api["size_in_bytes"] == artifact["bytes"] and not api["expired"]
    archive = Path(download["destination"]) / "installers.original.zip" if artifact["name"].startswith("opencodex-desktop") else ROOT / "runner-evidence.original.zip"
    data = archive.read_bytes()
    assert len(data) == artifact["bytes"] and hashlib.sha256(data).hexdigest() == artifact["sha256"]
    with zipfile.ZipFile(archive) as bundle:
        assert bundle.testzip() is None
        for entry in artifact["entries"]:
            data = bundle.read(entry["path"])
            assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
data = Path(descriptor["executable"]).read_bytes()
assert len(data) == descriptor["bytes"]
assert hashlib.sha256(data).hexdigest() == descriptor["sha256"]
msi, = [i for i in receipt["installers"] if i["kind"] == "msi"]
assert msi["installedExecutableSha256"] == descriptor["sha256"] == msi["expectedInstalledExecutableSha256"]
assert descriptor["sameShaCiAndArtifactVerified"] and not descriptor["installerExecuted"]
marker = read("bundle-marker-verification.json")
assert marker["source"] == SOURCE and marker["runId"] == handle["runId"]
assert marker["differencesLimitedToExactThreeByteBundleSuffix"] and not marker["originalExeWritten"]
nsis, = [i for i in receipt["installers"] if i["kind"] == "nsis"]
assert nsis["bundleMarkerOffset"] == msi["bundleMarkerOffset"] == marker["markerStartOffset"]
prefix = b"__TAURI_BUNDLE_TYPE_VAR_"
offset = marker["markerStartOffset"]
assert data[offset:offset + len(prefix) + 3] == prefix + b"MSI"
variant = bytearray(data)
variant[offset + len(prefix):offset + len(prefix) + 3] = b"NSS"
assert hashlib.sha256(variant).hexdigest() == nsis["installedExecutableSha256"]
assert nsis["installedExecutableSha256"] == nsis["expectedInstalledExecutableSha256"] == marker["inMemoryNssVariantSha256"]
checker = subprocess.run([
    "git", "-C", str(Path(download["destination"]).parents[2]),
    "show", SOURCE + ":test/smoke/windows-bundle-identity.mjs"
], check=True, capture_output=True).stdout
assert hashlib.sha256(checker).hexdigest() == marker["checkerGitBlobSha256"]
package_log = read("package-log-verification.json")
assert package_log["source"] == SOURCE and package_log["runId"] == handle["runId"]
assert package_log["actualCheckout"] == SOURCE and package_log["passed"] == 650
assert package_log["suites"] == 35 and package_log["failed"] == 0 and package_log["ignored"] == 7
admission = read("ci-build-admission.json")
assert admission["source"] == SOURCE and admission["wholeCiRun"] == 38838969102
ci_root = ROOT.parent / admission["evidenceDirectory"]
assert hashlib.sha256((ci_root / "verification.json").read_bytes()).hexdigest() == admission["verificationSha256"]
assert hashlib.sha256((ci_root / "manifest.json").read_bytes()).hexdigest() == admission["manifestSha256"]
ci = json.loads((ci_root / "verification.json").read_text(encoding="utf-8"))
assert ci["source"] == SOURCE and ci["wholeCiSucceeded"]
static = read("admission-manifest.json")
for entry in static["files"]:
    payload = (ROOT / entry["path"]).read_bytes()
    assert len(payload) == entry["bytes"] and hashlib.sha256(payload).hexdigest() == entry["sha256"]
assert read("admission-verification.json")["maximumDispatches"] == 1
assert read("dispatch-attempt.json")["ciRunId"] == 38838969102
assert read("at-rest-protection.json")["nativeCount"] == 0
summary = {"status": "preview-candidate-checks-passed" if preview else "verified-exact-823-candidate", "files": len(actual),
           "runId": handle["runId"], "source": SOURCE, "exeSha256": descriptor["sha256"],
           "bytes": descriptor["bytes"], "hostInstallerExecuted": False,
           "hostProductStarted": False, "nativeMemoryGatePassed": False,
           "historicalQueryFailureCleared": False, "fullVersionGatePassed": False,
           "releaseRequested": False}
if not preview:
    (ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
