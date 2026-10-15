import hashlib
import json
import re
import subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parent
CODE = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")

def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))

manifest = read("manifest.json")
actual = {path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*")
          if path.is_file() and path.relative_to(ROOT).as_posix() not in {"manifest.json", "verification.json"}}
assert actual == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
before_source = subprocess.check_output([
    "git", "show", "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f:apps/desktop/ui/src/features/runtime/motion/meshGlow.ts"], cwd=CODE)
pixel = read("real-webgl-depth-pixels.original.json")
assert pixel["sourceSha256"] in {
    hashlib.sha256(before_source).hexdigest(),
    hashlib.sha256(before_source.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")).hexdigest(),
}
assert not pixel["originalProtocolResult"].get("error")
result = pixel["originalProtocolResult"]["result"]["result"]["value"]
assert result["caseCount"] == len(result["cases"]) == 24 and not result["liveProductCanvasTouched"]
for case in result["cases"]:
    assert case["leftDepthAttribute"] and not case["rightDepthAttribute"]
    assert case["leftDepthBits"] > 0 and case["rightDepthBits"] == 0
    assert case["nonzeroBytes"] > 0 and case["differentBytes"] == case["maxDifference"] == 0
    assert case["rgbaSha256"][0] == case["rgbaSha256"][1]
local = read("local-validation.json")
assert local["uiTestExit"] == local["typecheckExit"] == local["buildExit"] == 0
text = (ROOT / "ui-test.original.log").read_text(encoding="utf-8-sig")
assert re.search(r"Tests\s+533 passed", re.sub(r"\x1b\[[0-9;]*m", "", text))
ci = read("ci-log-verification.json")
assert ci["head"] == "8be87e2d54a895d4fe02e31300100a1ffed90121" and ci["actualCheckoutTreesEqual"]
assert len(ci["runs"]) == 2
for run in ci["runs"]:
    assert len(run["jobs"]) == 2
    for job in run["jobs"]:
        data = (ROOT / job["originalLog"]).read_bytes()
        assert len(data) == job["bytes"] and hashlib.sha256(data).hexdigest() == job["sha256"]
        assert job["suites"] == 35 and job["failed"] == 0 and job["tree"] == ci["headTree"]
        assert job["passed"] == (923 if job["job"] == "backend" else 648)
pr = read("pr-after.original.json")
payload = read("pr-payload.json")
assert pr["head"]["sha"] == ci["head"] and pr["draft"] and not pr["merged"] and pr["auto_merge"] is None
assert pr["title"] == payload["title"] and pr["body"] == payload["body"]
summary = {"status": "verified-optimization-and-ci-only", "files": len(actual),
           "source": ci["head"], "actualGitTreesEqual": True, "realPixelCases": 24,
           "localUiTests": 533, "macPassedPerRun": 923, "windowsPassedPerRun": 648,
           "newExeMemoryBudgetPassed": False, "historicalFailureCleared": False,
           "prDraft": True, "fullVersionGatePassed": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
