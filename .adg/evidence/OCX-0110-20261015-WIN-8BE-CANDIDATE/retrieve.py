import hashlib
import importlib.util
import json
import zipfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "8be87e2d54a895d4fe02e31300100a1ffed90121"
spec = importlib.util.spec_from_file_location(
    "ghread", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
terminal = json.loads((ROOT / "candidate-terminal.json").read_text())
run = terminal["run"]
assert run["head_sha"] == SOURCE and run["status"] == "completed" and run["conclusion"] == "success"
assert run["run_attempt"] == 1 and run["event"] == "workflow_dispatch"
assert next(job for job in terminal["jobs"]["jobs"] if job["name"] == "package")["conclusion"] == "success"
assert next(job for job in terminal["jobs"]["jobs"] if job["name"] == "query-etw")["conclusion"] == "skipped"
token = api.credential()
destination = api.CODE / f"test/software/candidate-8be87e-{run['id']}"
assert not destination.is_symlink() and not destination.is_junction()
destination.mkdir(exist_ok=True)
assert destination.resolve().is_relative_to((api.CODE / "test/software").resolve())
downloads = []
for name, basename in (
    ("opencodex-desktop-0.1.10-windows-x64-candidate", "installers.original.zip"),
    ("windows-candidate-evidence", "runner-evidence.original.zip"),
):
    item, = [row for row in terminal["artifacts"]["artifacts"] if row["name"] == name]
    assert item["workflow_run"]["id"] == run["id"] and item["workflow_run"]["head_sha"] == SOURCE
    assert not item["expired"] and item["digest"].startswith("sha256:")
    target = destination / basename if basename.startswith("installers") else ROOT / basename
    if not target.exists():
        admit.download(item["archive_download_url"], target, 32 * 1024 * 1024,
                       token=token, expected=item["digest"][7:])
    data = target.read_bytes()
    assert len(data) == item["size_in_bytes"] and hashlib.sha256(data).hexdigest() == item["digest"][7:]
    entries = []
    with zipfile.ZipFile(target) as archive:
        admit.checked_zip(archive)
        for entry in archive.infolist():
            if entry.is_dir():
                continue
            content = archive.read(entry)
            entries.append({"path": entry.filename, "bytes": len(content),
                            "sha256": hashlib.sha256(content).hexdigest()})
            if basename.startswith("installers"):
                out = destination / "installers" / entry.filename
                out.parent.mkdir(parents=True, exist_ok=True)
                if out.exists():
                    assert out.read_bytes() == content
                else:
                    with out.open("xb") as stream:
                        stream.write(content)
        if basename.startswith("runner"):
            for member_name, saved_name in (
                ("installer-verification.json", "installer-verification.raw.original.json"),
                ("environment.json", "candidate-environment.raw.original.json"),
            ):
                member, = [entry for entry in archive.infolist() if Path(entry.filename).name == member_name]
                saved = ROOT / saved_name
                content = archive.read(member)
                if saved.exists():
                    assert saved.read_bytes() == content
                else:
                    with saved.open("xb") as stream:
                        stream.write(content)
    downloads.append({"artifactId": item["id"], "name": name, "bytes": len(data),
                      "sha256": hashlib.sha256(data).hexdigest(), "entries": entries,
                      "apiDigestSafePathsAndCrcVerified": True})
receipt = json.loads((ROOT / "installer-verification.raw.original.json").read_text(encoding="utf-8-sig"))
environment = json.loads((ROOT / "candidate-environment.raw.original.json").read_text(encoding="utf-8-sig"))
assert receipt["sourceCommit"] == environment["source"] == SOURCE
assert receipt["version"] == "0.1.10" and receipt["result"] == "pass"
assert receipt["releaseAccepted"] is False and receipt["codeSigned"] is False
for row in receipt["installers"]:
    package, = list((destination / "installers").rglob(row["name"]))
    data = package.read_bytes()
    assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
    assert row["result"] == row["firstScreen"] == "pass" and row["uninstallRemovedExecutable"]
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
result = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runId": run["id"],
          "destination": str(destination), "artifacts": downloads,
          "hostInstallerExecuted": False, "hostProductStarted": False,
          "fullVersionGatePassed": False, "releaseRequested": False}
with (ROOT / "candidate-download-result.json").open("x", encoding="utf-8") as stream:
    json.dump(result, stream, indent=2)
print(json.dumps({"source": SOURCE, "run": run["id"], "artifactsVerified": len(downloads)}))
