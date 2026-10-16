"""Read-only source, raw CI log, test, and pixel-pair evidence verification."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
WORKTREE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


scope = read("verification-scope.json")
source = scope["source"]
assert source == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
assert subprocess.check_output(["git", "rev-parse", source + "^"], cwd=WORKTREE).decode().strip() == scope["parent"]
changes = subprocess.check_output(["git", "diff-tree", "--no-commit-id", "--name-only", "-r", source], cwd=WORKTREE).decode().splitlines()
assert changes == ["apps/desktop/ui/src/features/runtime/motion/meshGlow.ts",
                   "apps/desktop/ui/tests/mesh-performance.test.ts"]
before, after = [subprocess.check_output(["git", "show", revision + ":" + changes[0]], cwd=WORKTREE).decode()
                 for revision in [scope["parent"], source]]
for name in ["VERT", "NOISE", "ORBIT_GLSL", "FRAG"]:
    pattern = r"const " + name + r" = ([\s\S]*?)(?=\n\n)"
    assert re.search(pattern, before).group(1) == re.search(pattern, after).group(1)
for copy in read("original-copies.json"):
    data = (ROOT / copy["path"]).read_bytes()
    assert len(data) == copy["bytes"] and sha(data) == copy["sha256"]
    assert data == Path(copy["runtimePath"]).read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig"))
before_test = read("original/before-test.json")
assert (before_test["numPassedTests"], before_test["numFailedTests"]) == (3, 3)
for case in [case for suite in before_test["testResults"] for case in suite["assertionResults"]
             if case["status"] == "failed"]:
    assert "[ 300, 150 ]" in "".join(case["failureMessages"])
assert read("original/after-targeted-test.json")["numPassedTests"] == 23
full = read("original/full-test-correct-cwd.json")
assert full["success"] and full["numPassedTests"] == 538 and full["numFailedTests"] == 0
assert len(full["testResults"]) == 99
assert not read("original/full-test.json")["success"]
terminal = read("original/ci-terminal.original.json")
assert terminal["run"]["id"] == 38891188021 and terminal["run"]["head_sha"] == source
assert terminal["run"]["conclusion"] == "success" and terminal["run"]["run_attempt"] == 1
assert {j["name"] for j in terminal["jobs"]["jobs"] if j["conclusion"] == "success"} == {
    "frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert [j["conclusion"] for j in terminal["jobs"]["jobs"] if j["name"] == "build"] == ["skipped"]
correction = read("original/all-job-checkout-correction.json")
assert correction["missingMarkerClaimWasIncorrect"] and len(correction["jobs"]) == 5
commit = read("original/checkout-e7272dddb093dcffa0a27dd8dad5e7422288218a.original.json")
tree = subprocess.check_output(["git", "rev-parse", source + "^{tree}"], cwd=WORKTREE).decode().strip()
assert commit["tree"]["sha"] == tree == scope["sourceTree"]
for row in correction["jobs"]:
    raw = (ROOT / "original" / row["log"]).read_bytes()
    assert sha(raw) == row["sha256"]
    actual, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", raw.decode("utf-8-sig"))
    assert actual == row["actualCheckout"] == commit["sha"]
for name, passed, ignored in [("backend", 923, 6), ("backend-windows", 650, 7)]:
    text = (ROOT / f"original/ci-38891188021-{name}.original.log").read_text(encoding="utf-8-sig")
    rows = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert len(rows) == 35
    assert tuple(sum(int(x[i]) for x in rows) for i in range(3)) == (passed, 0, ignored)
front = re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", (ROOT / "original/ci-38891188021-frontend.original.log").read_text(encoding="utf-8-sig"))
assert re.search(r"Test Files\s+99 passed", front) and re.search(r"Tests\s+538 passed", front)
pixels = read("original/pixel-check/result.json")
mapping = read("pixel-map.json")
assert pixels["comparisons"] == 24 and len(pixels["rows"]) == 8 and len(mapping) == 48
raw_distinct = set()
for row in pixels["rows"]:
    assert row["before"]["init"]["creationSize"] == [300, 150]
    assert row["after"]["init"]["creationSize"] == row["after"]["init"]["finalSize"]
    assert row["before"]["init"]["finalSize"] == row["after"]["init"]["finalSize"]
    assert row["before"]["init"]["attributes"] == row["after"]["init"]["attributes"]
    for original, changed in zip(row["before"]["frames"], row["after"]["frames"], strict=True):
        assert original["rawSha256"] == changed["rawSha256"]
        assert original["seconds"] == changed["seconds"]
        raw_distinct.add(original["rawSha256"])
        a, b = [mapping[frame["png"]] for frame in [original, changed]]
        assert a == b
        png = (ROOT / a["path"]).read_bytes()
        assert png.startswith(b"\x89PNG\r\n\x1a\n") and len(png) == a["bytes"]
        assert sha(png) == a["sha256"] == original["pngSha256"] == changed["pngSha256"]
        assert not original["contextLost"] and not changed["contextLost"]
        assert original["glError"] == changed["glError"] == 0
assert len(raw_distinct) > 8
browser_exit = read("original/pixel-check/browser-exit.json")
assert browser_exit["code"] == 0 and browser_exit["signal"] is None and not browser_exit["forceTerminated"]
assert not browser_exit["descendantCleanupFullyProven"]
pr = read("original/pr-readback.original.json")
assert pr["number"] == 6 and pr["draft"] and not pr["merged"] and pr["head"]["sha"] == source
dispatch = read("original/candidate-dispatch-readback.json")
assert dispatch["httpAccepted"] and dispatch["runId"] == 38893610415 and dispatch["source"] == source
assert not dispatch["automaticRetry"] and not dispatch["queryReplayRequested"]
for key in ["nativePerformanceMeasured", "nativeProductLaunched", "nativeGpuAllocationAttributed",
            "originalRssBudgetPassed", "queryReplayed", "dpiAcceptanceAdded", "candidateArtifactsAdmitted",
            "historicalFailuresCleared", "taskClosed", "releaseAccepted"]:
    assert not scope[key], key
if (ROOT / "manifest.json").exists():
    manifest = read("manifest.json")
    expected = {row["path"] for row in manifest["files"]}
    actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name != "manifest.json"}
    assert actual == expected
    for row in manifest["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
print(json.dumps({"status": "verified", "source": source, "ciRunId": 38891188021,
                  "localAndCiFrontendTests": 538, "macTests": 923, "windowsTests": 650,
                  "pixelPairs": 24, "uniquePngFiles": scope["uniquePngFiles"],
                  "nativePerformanceProven": False, "candidateArtifactsAdmitted": False}))
