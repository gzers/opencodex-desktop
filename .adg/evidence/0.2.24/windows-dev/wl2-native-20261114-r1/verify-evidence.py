"""Offline replay of immutable W-L2 observations; never runs a Windows action."""
from pathlib import Path
import hashlib
import json

root = Path(__file__).resolve().parent


def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
listed = set()
for row in manifest["files"]:
    relative = Path(row["path"])
    assert not relative.is_absolute() and ".." not in relative.parts
    assert row["path"] not in listed
    listed.add(row["path"])
    path = root / relative
    assert path.is_file() and not path.is_symlink()
    assert path.stat().st_size == row["bytes"], row["path"]
    with path.open("rb") as stream:
        assert hashlib.file_digest(stream, "sha256").hexdigest() == row["sha256"], row["path"]
actual = {p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()}
assert actual == listed | {"manifest.json"}
assert len(listed) == manifest["payloadFiles"]
assert sum(row["bytes"] for row in manifest["files"]) == manifest["payloadBytes"]

assert read("baseline-build/result.json")["exitCode"] == 0
assert read("primitive-build01/result.json")["exitCode"] == 101
assert read("native-build01/result.json")["exitCode"] == 101
for name in ["primitive-build02", "native-build02", "product-build03"]:
    assert read(f"{name}/result.json")["exitCode"] == 0
result = read("result.json")
assert result["allAccepted"] is False
assert result["productionExecuted"] is False
assert result["fourNativeTraitsImplemented"] is False
assert result["cleanupComplete"] is False
facts = read("public-raw/out01/native/wl2-independent-prerequisites.json")
child = read("public-raw/out01/native/child-result.json")
assert child["sid"] == facts["sid"] and child["elevated"] is False
assert child["exitCode"] == 101
assert facts["security"]["rawWin32"] == 1314
assert facts["security"]["completeSecurityReadable"] is False
assert facts["ownerOuterJob"]["outerJobSafe"] is False
assert facts["productionExecuted"] is False
assert facts["originalBefore"] == facts["originalAfter"]
assert facts["nativeNamespaces"]["scm"]["raw_code"] == 1060
assert facts["nativeNamespaces"]["scheduler"]["raw_code"] == 0x80070002
assert read("public-raw/out01/native/wl1-final-owner.json")["jobCreated"] is False
assert read("public-raw/out01/daily-path-result.json")["unchanged"] is True
assert read("cleanup-account-profile.json")["nativeDeleteProfileWin32"] == 87
cleanup = read("cleanup-account-profile02.json")
assert cleanup["accountRemoved"] is True
assert cleanup["nativeDeleteProfileReturned"] is True
assert cleanup["nativeDeleteProfileWin32"] == 0
assert cleanup["profileRemoved"] is False and cleanup["profileAfter"] == []
rejected = read("public-delete-rejected.json")
assert rejected["commandExecuted"] is False
assert rejected["retried"] is False
assert rejected["alternativeMechanismUsed"] is False
source = read("final-source-manifest.json")
assert source["head"] == result["source"]
for entry in source["files"]:
    payload = root / "final-source" / entry["path"]
    assert hashlib.sha256(payload.read_bytes()).hexdigest() == entry["sha256"]
for comparison in source["executionComparisons"]:
    assert comparison["finalRustMatchesRawBytes"] is True
matrix = read("status-matrix.json")
assert matrix["allAccepted"] is False
assert len(matrix["items"]) == len({r["id"] for r in matrix["items"]}) == 36
print(json.dumps({"payloadFiles": len(listed), "payloadBytes": manifest["payloadBytes"],
                  "source": source["head"], "statusItems": 36,
                  "evidenceIntegrity": True, "allAccepted": False,
                  "productionExecuted": False}))
