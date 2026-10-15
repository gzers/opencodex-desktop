import hashlib
import json
import zipfile
from pathlib import Path
ROOT = Path(__file__).resolve().parent
def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))
manifest = read("manifest.json")
actual = {path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*")
          if path.is_file() and path.relative_to(ROOT).as_posix() not in {"manifest.json", "verification.json"}}
assert actual == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
terminal, download, descriptor = read("candidate-terminal.json"), read("candidate-download-result.json"), read("candidate-descriptor.json")
assert terminal["run"]["id"] == descriptor["runId"] == download["runId"] == 38767255195
assert terminal["run"]["head_sha"] == descriptor["source"] == download["source"] == "8be87e2d54a895d4fe02e31300100a1ffed90121"
assert terminal["run"]["conclusion"] == "success" and terminal["run"]["run_attempt"] == 1
assert read("dispatch-result.json")["httpStatus"] == 204
receipt = read("installer-verification.raw.original.json")
assert receipt["result"] == "pass" and not receipt["releaseAccepted"] and not receipt["codeSigned"]
for artifact in download["artifacts"]:
    api, = [row for row in terminal["artifacts"]["artifacts"] if row["id"] == artifact["artifactId"]]
    assert api["digest"] == "sha256:" + artifact["sha256"] and api["size_in_bytes"] == artifact["bytes"]
    archive = Path(download["destination"]) / "installers.original.zip" if artifact["name"].startswith("opencodex-desktop") else ROOT / "runner-evidence.original.zip"
    data = archive.read_bytes()
    assert len(data) == artifact["bytes"] and hashlib.sha256(data).hexdigest() == artifact["sha256"]
    with zipfile.ZipFile(archive) as bundle:
        assert bundle.testzip() is None
        for entry in artifact["entries"]:
            data = bundle.read(entry["path"])
            assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
data = Path(descriptor["executable"]).read_bytes()
assert len(data) == descriptor["bytes"] == 8808448
assert hashlib.sha256(data).hexdigest() == descriptor["sha256"] == "26d5458d1e25ab6c9da74229e638f1c2b475ca1e1934dd0538b542d1f6bf38c9"
msi, = [item for item in receipt["installers"] if item["kind"] == "msi"]
assert msi["installedExecutableSha256"] == descriptor["sha256"] == msi["expectedInstalledExecutableSha256"]
assert descriptor["sameShaCiAndArtifactVerified"] and not descriptor["installerExecuted"]
summary = {"status": "verified-exact-source-candidate", "files": len(actual),
           "runId": descriptor["runId"], "source": descriptor["source"],
           "exeSha256": descriptor["sha256"], "apiArtifactsVerified": 2,
           "hostInstallerExecuted": False, "fullInstallerGatePassed": False,
           "memoryBudgetPassed": False, "releaseRequested": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
