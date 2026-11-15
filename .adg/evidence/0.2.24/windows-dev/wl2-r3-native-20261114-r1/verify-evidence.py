"""Offline integrity replay only. No native action or acceptance is minted."""
from pathlib import Path
import hashlib
import json
import re

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

for stage in ["build01", "build02", "build03", "native01", "native02", "native03", "product01", "product02"]:
    result = read(f"{stage}/result.json")
    assert result["exitCode"] == 0
    assert result["accepted"] is False
assert "8 passed; 0 failed" in (root / "build03/stdout.log").read_text(encoding="utf-8-sig")
assert "4 passed; 0 failed" in (root / "native03/stdout.log").read_text(encoding="utf-8-sig")
result = read("result.json")
assert result["allAccepted"] is False
assert result["complete"] is False
assert result["productionExecuted"] is False
assert result["newOrdinaryAccounts"] == result["knownPrerequisiteFailuresRepeated"] == 0
assert result["W2"] is False and result["dualSID"] is False and result["released"] is False
assert result["firstUnresolvedPrerequisite"]["currentCompleteSDPositiveAvailable"] is False
assert result["testCounts"] == {"finalLibrary": 8, "finalNative": 4}
assert result["finalProductBuild"]["exitCode"] == 0
assert result["finalProductBuild"]["elapsedMs"] == 137535
source = read("final-source-manifest.json")
assert source["head"] == result["source"]
for entry in source["files"]:
    payload = root / "final-source" / entry["path"]
    assert hashlib.sha256(payload.read_bytes()).hexdigest() == entry["sha256"]
for stage in ["build03", "native03", "product02"]:
    comparison = next(r for r in source["executionComparisons"] if r["run"] == stage)
    assert comparison["finalRustMatchesRawBytes"] is True
    assert comparison["rustFileCount"] == 34
    actual_source = read(f"{stage}/source-manifest.json")
    for item in actual_source:
        payload = root / stage / "executed-source" / item["path"]
        assert hashlib.sha256(payload.read_bytes()).hexdigest() == item["sha256"]
        final = root / "final-source" / item["path"]
        if final.is_file() and item["path"].endswith(".rs"):
            assert final.read_bytes() == payload.read_bytes()
assert read("artifact-provenance-limits.json")["accepted"] is False
artifacts = read("artifacts.json")
assert len(artifacts) == 5
for artifact in artifacts:
    assert artifact["bytes"] > 0
    assert re.fullmatch("[a-f0-9]{64}", artifact["sha256"])
implementation = read("implementation-status.json")
assert implementation["allAccepted"] is False
assert implementation["productionExecuted"] is False
assert any("NOT implemented" in r["implementation"] for r in implementation["items"])
assert read("marenben-session-inventory01.json")["runtimeTokenAndNoJobNotProven"] is True
cleanup = read("cleanup.json")
for field in ["accountsCreated", "ProfilesCreated", "officialRuntimeProcessesCreated", "brokerProcessesCreated", "PublicRootsCreated", "tasksCreated", "MSIRuns"]:
    assert cleanup["thisRound"][field] == 0
assert cleanup["cleanupAllHistoricalComplete"] is False
matrix = read("status-matrix.json")
assert matrix["allAccepted"] is False
assert len(matrix["items"]) == len({r["id"] for r in matrix["items"]}) == 37
print(json.dumps({"payloadFiles": len(listed), "payloadBytes": manifest["payloadBytes"],
                  "source": source["head"], "statusItems": 37,
                  "evidenceIntegrity": True, "liveRetest": False,
                  "allAccepted": False, "productionExecuted": False}))
