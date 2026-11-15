"""Check exact archived payload bytes and each invocation's captured source.

This replays byte integrity only. It never issues a runtime/Closed certificate.
No network, original Profile access or cleanup is performed.
"""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent


def load(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(data):
    return hashlib.sha256(data).hexdigest()


def confined(relative):
    path = ROOT / relative
    assert path.resolve().is_relative_to(ROOT.resolve()), relative
    return path


inventory = load(ROOT / "payload-inventory.json")
assert inventory["runtimeAcceptance"] is False
for row in inventory["files"]:
    raw = confined(row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and digest(raw) == row["sha256"], row["path"]

runs = []
for source in sorted(ROOT.glob("*/source-manifest.json")):
    directory = source.parent
    manifest = load(source)
    assert isinstance(manifest, list) and manifest
    for row in manifest:
        path = directory / "executed-source" / row["path"]
        assert path.resolve().is_relative_to((directory / "executed-source").resolve())
        raw = path.read_bytes()
        assert len(raw) == row["bytes"] and digest(raw) == row["sha256"], str(path)
    result = load(directory / "result.json")
    assert result["accepted"] is False
    if directory.name in {"library02", "library04", "library09"}:
        assert result["exitCode"] == 101
        assert "error[" in (directory / "stderr.log").read_text(encoding="utf-8-sig") or "error:" in (directory / "stderr.log").read_text(encoding="utf-8-sig")
    else:
        assert result["exitCode"] == 0, directory.name
    runs.append(directory.name)

print(json.dumps({"payloadFiles": len(inventory["files"]), "runs": runs,
                  "runtimeAcceptance": False, "integrity": "verified"}))
