"""Retrieve the exact retention run, including failure originals; never rerun it."""
import hashlib
import importlib.util
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("api", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
spec = importlib.util.spec_from_file_location("admit", api.CODE / "test/smoke/windows-etw-ci-admit.py")
admit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admit)
terminal = json.loads((root / "retention-terminal.json").read_text(encoding="utf-8-sig"))
run = terminal["run"]
assert run["head_sha"] == api.SOURCE and run["status"] == "completed" and run["run_attempt"] == 1
token = api.credential()
for kind in ("msi", "nsis"):
    matches = [row for row in terminal["artifacts"]["artifacts"]
               if row["name"] == "windows-retention-" + kind + "-" + api.SOURCE]
    assert len(matches) == 1
    item = matches[0]
    assert item["workflow_run"]["id"] == run["id"] and item["workflow_run"]["head_sha"] == api.SOURCE
    assert not item["expired"] and item["digest"].startswith("sha256:")
    destination = root / kind
    destination.mkdir(exist_ok=True)
    metadata = destination / "artifact.original.json"
    if metadata.exists():
        assert json.loads(metadata.read_text())["id"] == item["id"]
    else:
        metadata.write_text(json.dumps(item, indent=2) + "\n", encoding="utf-8")
    archive_path = destination / "artifact.original.zip"
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
            content = archive.read(info)
            target = destination / "original" / info.filename
            target.parent.mkdir(parents=True, exist_ok=True)
            if target.exists():
                assert target.read_bytes() == content
            else:
                with target.open("xb") as stream:
                    stream.write(content)
            entries.append({"path": info.filename, "bytes": len(content),
                            "sha256": hashlib.sha256(content).hexdigest()})
    report = {"runId": run["id"], "harnessSource": api.SOURCE, "kind": kind, "artifactId": item["id"],
              "artifactBytes": item["size_in_bytes"], "artifactSha256": item["digest"][7:],
              "apiDigestSafePathsAndCrcVerified": True, "entries": entries,
              "crossHostCredentialsForwarded": False, "executionRepeated": False}
    report_path = destination / "download-verification.json"
    if report_path.exists():
        assert json.loads(report_path.read_text()) == report
    else:
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"kind": kind, "artifactId": item["id"], "entries": len(entries),
                      "apiDigestVerified": True}))
