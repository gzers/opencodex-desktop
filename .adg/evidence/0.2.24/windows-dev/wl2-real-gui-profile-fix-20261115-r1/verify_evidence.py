"""Offline consistency of code inputs and same-SID evidence; not native proof."""
from pathlib import Path
import base64
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
inventory = load(ROOT / "payload-manifest.json")
assert inventory["productionAcceptance"] is False
for row in inventory["files"]:
    path = ROOT / row["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    raw = path.read_bytes()
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"], row["path"]
for name in ["library01", "product01"]:
    run = ROOT / name
    assert load(run / "result.json")["exitCode"] == 0
    for row in load(run / "source-manifest.json"):
        raw = (run / "executed-source" / row["path"]).read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
assert "test result: ok. 7 passed; 0 failed; 4 ignored" in (ROOT / "library01/stdout.log").read_text(encoding="utf-8-sig")
assert load(ROOT / "prepare01-short-sha-rejected.json")["beforeAnyTaskRootCreation"] is True
assert load(ROOT / "prepare02-result.json")["rawExit"] == 0
prepared = load(ROOT / "prepared-location.json")
assert prepared["parseErrors"] == 0 and prepared["oldTaskRootChanged"] is False
for transform in prepared["scriptTransforms"]:
    source = ROOT / "product01/executed-source/test/windows024-native" / transform["name"]
    raw = source.read_bytes()
    assert sha(raw) == transform["sourceSha256"]
    expected = raw.decode("utf-8-sig").replace(
        r"C:\Users\Public\ocx-h024-actual-gui-20261115-r1",
        r"C:\Users\Public\ocx-h024-actual-gui-fixed-20261115-r1")
    assert (ROOT / "prepared-scripts" / transform["name"]).read_bytes() == expected.encode("utf-8")
if (ROOT / "desktop-result.json").exists():
    export = load(ROOT / "same-sid-export01.json")
    for row in export["files"]:
        raw = base64.b64decode(row["base64"], validate=True)
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
        assert raw == (ROOT / "received" / row["relative"]).read_bytes()
    desktop = load(ROOT / "desktop-result.json")
    assert desktop["sameSidExportComplete"] is True
    assert desktop["productionExecuted"] is False and desktop["brokerRawExit"] is None
status = load(ROOT / "final-status.json")
assert status["allAccepted"] is False and status["productionExecuted"] is False
assert status["startOpen"] is False and status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert status["cleanup"]["overallCleanupComplete"] is False
print(json.dumps(dict(payloads=len(inventory["files"]), integrity="verified", productionAcceptance=False)))
