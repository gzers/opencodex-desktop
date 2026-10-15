"""Verify exact-source artifacts; extract packages only, never execute an installer."""
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parent
prior = root.parent / "OCX-0110-20261015-WIN-FIRST-CI-ETW"
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
source = "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0"
terminal = json.loads((root / "candidate-terminal.json").read_text(encoding="utf-8-sig"))
run = terminal["run"]
assert run["id"] == 38729632145 and run["head_sha"] == source and run["run_attempt"] == 1
assert run["status"] == "completed" and run["conclusion"] == "success"
jobs = terminal["jobs"]["jobs"]
assert next(job for job in jobs if job["name"] == "package")["conclusion"] == "success"
assert next(job for job in jobs if job["name"] == "query-etw")["conclusion"] == "skipped"
artifacts = terminal["artifacts"]["artifacts"]
token = api.credential()
destination = api.CODE / "test/software/candidate-a2e4ea-38729632145"
assert not destination.is_symlink() and not destination.is_junction()
destination.mkdir(exist_ok=True)
assert destination.resolve().is_relative_to((api.CODE / "test/software").resolve())
verified = []
for name, basename in (
    ("opencodex-desktop-0.1.10-windows-x64-candidate", "installers.original.zip"),
    ("windows-candidate-evidence", "runner-evidence.original.zip"),
):
    matches = [row for row in artifacts if row["name"] == name]
    assert len(matches) == 1
    item = matches[0]
    assert item["workflow_run"]["id"] == run["id"] and item["workflow_run"]["head_sha"] == source
    assert not item["expired"] and item["digest"].startswith("sha256:")
    archive_path = destination / basename if basename.startswith("installers") else root / basename
    if not archive_path.exists():
        admit.download(item["archive_download_url"], archive_path, 32 * 1024 * 1024,
                       token=token, expected=item["digest"][7:])
    assert archive_path.stat().st_size == item["size_in_bytes"]
    assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == item["digest"][7:]
    entries = []
    with zipfile.ZipFile(archive_path) as archive:
        admit.checked_zip(archive)
        for info in archive.infolist():
            if info.is_dir():
                continue
            data = archive.read(info)
            entries.append({"path": info.filename, "bytes": len(data),
                            "sha256": hashlib.sha256(data).hexdigest()})
            if basename.startswith("installers"):
                target = destination / "installers" / info.filename
                target.parent.mkdir(parents=True, exist_ok=True)
                if target.exists():
                    assert target.read_bytes() == data
                else:
                    with target.open("xb") as stream:
                        stream.write(data)
        if basename.startswith("runner"):
            for member_name, saved_name in (
                ("installer-verification.json", "installer-verification.raw.original.json"),
                ("environment.json", "candidate-environment.raw.original.json"),
            ):
                matches = [info for info in archive.infolist() if Path(info.filename).name == member_name]
                assert len(matches) == 1
                data = archive.read(matches[0])
                target = root / saved_name
                if target.exists():
                    assert target.read_bytes() == data
                else:
                    with target.open("xb") as stream:
                        stream.write(data)
    verified.append({
        "artifactId": item["id"], "name": name, "bytes": item["size_in_bytes"],
        "sha256": item["digest"][7:], "zipPath": str(archive_path),
        "apiDigestAndSafePathsAndCrcVerified": True, "entries": entries,
        "crossHostCredentialsForwarded": False,
    })
receipt = json.loads((root / "installer-verification.raw.original.json").read_text(encoding="utf-8-sig"))
environment = json.loads((root / "candidate-environment.raw.original.json").read_text(encoding="utf-8-sig"))
assert receipt["sourceCommit"] == environment["source"] == source
assert receipt["version"] == "0.1.10" and receipt["result"] == "pass"
assert receipt["releaseAccepted"] is False and receipt["codeSigned"] is False
for row in receipt["installers"]:
    matches = list((destination / "installers").rglob(row["name"]))
    assert len(matches) == 1
    package = matches[0]
    assert package.stat().st_size == row["bytes"]
    assert hashlib.sha256(package.read_bytes()).hexdigest() == row["sha256"]
    assert row["result"] == row["firstScreen"] == "pass" and row["uninstallRemovedExecutable"]
    assert row["installedExecutableSha256"] == row["expectedInstalledExecutableSha256"]
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "runId": run["id"],
    "destination": str(destination), "artifacts": verified,
    "hostProductStarted": False, "hostInstallerExecuted": False,
    "runnerFreshInstallIsNotHostUpgradeRetentionAcceptance": True,
    "fullVersionGatePassed": False, "releaseRequested": False,
}
target = root / "candidate-download-result.json"
assert not target.exists()
target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": source, "runId": run["id"], "artifactsVerified": len(verified),
                  "destination": str(destination), "hostInstallerExecuted": False}))
