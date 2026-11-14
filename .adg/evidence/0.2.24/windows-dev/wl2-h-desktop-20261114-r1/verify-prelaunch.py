"""Verify immutable preparation only; this does not execute the desktop test."""
from pathlib import Path
import hashlib
import json

root = Path(__file__).resolve().parent


def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))


m = read("manifest.json")
listed = set()
for row in m["files"]:
    p = Path(row["path"])
    assert not p.is_absolute() and ".." not in p.parts
    assert row["path"] not in listed
    listed.add(row["path"])
    path = root / p
    assert path.is_file() and not path.is_symlink()
    assert path.stat().st_size == row["bytes"]
    with path.open("rb") as stream:
        assert hashlib.file_digest(stream, "sha256").hexdigest() == row["sha256"]
actual = {p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()}
assert actual == listed | {"manifest.json"}
assert len(listed) == m["payloadFiles"]
assert sum(r["bytes"] for r in m["files"]) == m["payloadBytes"]
assert read("build01/result.json")["exitCode"] == 0
assert read("product01/result.json")["exitCode"] == 101
assert "E0601" in (root / "product01/stderr.log").read_text(encoding="utf-8-sig")
assert read("product02/result.json")["exitCode"] == 0
assert "1 passed; 0 failed" in (root / "build01/stdout.log").read_text(encoding="utf-8-sig")
result = read("prelaunch-result.json")
assert result["accepted"] is False and result["allAccepted"] is False
assert result["HComponentExecuted"] is False and result["productionExecuted"] is False
assert result["knownOrdinarySDRetests"] == 0
assert result["interactiveSession"] == 7
source = read("final-source-manifest.json")
assert source["head"] == result["source"]
for row in source["files"]:
    path = root / "final-source" / row["path"]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]
for check in source["comparisons"]:
    assert check["matchesFinalRawRust"] is True
assert read("artifact-provenance-limits.json")["accepted"] is False
prepared = read("prepared-artifacts.json")
assert prepared["productionAccepted"] is False
assert len(prepared["files"]) == 2
assert read("public-prelaunch-inventory.json")["launchedFromController"] is False
status = read("prelaunch-status.json")
assert status["allAccepted"] is False
assert len(status["items"]) == len({r["id"] for r in status["items"]}) == 13
print(json.dumps({"payloadFiles": len(listed), "payloadBytes": m["payloadBytes"],
                  "source": source["head"], "preparationIntegrity": True,
                  "HComponentExecuted": False, "allAccepted": False}))
