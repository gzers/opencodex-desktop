"""Verify the exact d94 candidate and readonly MSI payload, never run it."""
import hashlib
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
actual = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
          if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(actual) == {e["path"] for e in manifest["files"]}
for entry in manifest["files"]:
    data = actual[entry["path"]].read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
handle = read("candidate-handle.json")
terminal, download, descriptor = (read(name) for name in (
    "candidate-terminal.json", "candidate-download-result.json", "candidate-descriptor.json"))
assert terminal["run"]["id"] == handle["runId"] == download["runId"] == descriptor["runId"] == 38827408691
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
package_log = read("package-log-verification.json")
assert package_log["source"] == SOURCE and package_log["runId"] == handle["runId"]
assert package_log["actualCheckout"] == SOURCE and package_log["passed"] == 648
assert package_log["suites"] == 35 and package_log["failed"] == 0 and package_log["ignored"] == 7
admission = read("ci-build-admission.json")
assert admission["source"] == SOURCE and admission["wholeCiRun"] == 38824151573
ci_root = ROOT.parent / admission["evidenceDirectory"]
assert hashlib.sha256((ci_root / "ci-admission.json").read_bytes()).hexdigest() == admission["admissionSha256"]
assert read("at-rest-protection.json")["nativeCount"] == 0
summary = {"status": "verified-exact-d94-candidate", "files": len(actual),
           "runId": handle["runId"], "source": SOURCE, "exeSha256": descriptor["sha256"],
           "bytes": descriptor["bytes"], "hostInstallerExecuted": False,
           "hostProductStarted": False, "nativeMemoryGatePassed": False,
           "historicalQueryFailureCleared": False, "fullVersionGatePassed": False,
           "releaseRequested": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
