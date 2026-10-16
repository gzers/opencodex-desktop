"""Retrieve this diagnostic's one artifact; preserve byte-identical originals."""
import hashlib
import importlib.util
import json
import sys
import zipfile
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("remote_artifact_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
admit = load("remote_artifact_download", api.CODE / "test/smoke/windows-etw-ci-admit.py")
terminal = json.loads((ROOT / "remote-terminal.original.json").read_text(encoding="utf-8"))
run = terminal["run"]
assert run["status"] == "completed" and run["run_attempt"] == 1 and run["id"] == 38829382543
assert run["head_sha"] == "7704a17902c98ceed047cc3dec2a361937b5e1f7"
artifact, = [a for a in terminal["artifacts"]["artifacts"]
             if a["name"] == "windows-query-etw-" + run["head_sha"]]
assert artifact["workflow_run"]["id"] == run["id"] and not artifact["expired"]
archive_path = ROOT / "remote-artifact.original.zip"
if not archive_path.exists():
    admit.download(artifact["archive_download_url"], archive_path, 32 * 1024 * 1024,
                   token=api.credential(), expected=artifact["digest"][7:])
data = archive_path.read_bytes()
assert len(data) == artifact["size_in_bytes"] and hashlib.sha256(data).hexdigest() == artifact["digest"][7:]
entries = []
with zipfile.ZipFile(archive_path) as archive:
    admit.checked_zip(archive)
    for info in archive.infolist():
        if info.is_dir():
            continue
        assert info.filename.startswith(("out/windows-query-etw/", "runtime/ci-bd2-query-etw/capture/"))
        payload = archive.read(info)
        target = ROOT / "remote-original" / info.filename
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            assert target.read_bytes() == payload
        else:
            target.write_bytes(payload)
        entries.append({"path": info.filename, "bytes": len(payload),
                        "sha256": hashlib.sha256(payload).hexdigest()})
result = {"source": run["head_sha"], "runId": run["id"], "artifactId": artifact["id"],
          "archiveBytes": len(data), "archiveSha256": hashlib.sha256(data).hexdigest(),
          "apiDigestAndCrcVerified": True, "entries": entries,
          "crossHostCredentialsForwarded": False, "originalLibraryRunRepeated": False}
(ROOT / "remote-download-verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"runId": run["id"], "entries": len(entries),
                  "archiveBytes": len(data), "artifactId": artifact["id"]}))
