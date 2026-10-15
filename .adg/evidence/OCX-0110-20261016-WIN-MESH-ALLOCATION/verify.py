"""Verify allocation observations without claiming that native RSS is repaired."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def digest(data):
    return hashlib.sha256(data).hexdigest()


manifest = read("manifest.json")
files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(files) == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = files[entry["path"]].read_bytes()
    assert len(data) == entry["bytes"] and digest(data) == entry["sha256"]

bindings = read("fixed/source-bindings-corrected.json")
line_endings = read("source-line-endings.json")["files"]
for name, expected in bindings["sourceDigests"].items():
    mode, rel = name.split("/", 1)
    source = SOURCE if mode == "fixed-worktree" else "8be87e2d54a895d4fe02e31300100a1ffed90121"
    blob = subprocess.run(["git", "-C", str(CODE), "show",
                           f"{source}:apps/desktop/ui/src/{rel}"],
                          check=True, capture_output=True).stdout
    if digest(blob) != expected:
        forms = line_endings[name]
        assert digest(blob) == forms["normalizedGitSha256"]
        assert expected == forms["rawSha256"]
        endings = [kind for count, kind in forms["newlineRuns"] for _ in range(count)]
        parts = blob.split(b"\n")
        assert len(endings) == len(parts) - 1
        reconstructed = b"".join(part + (b"\r\n" if ending == "crlf" else b"\n")
                                  for part, ending in zip(parts[:-1], endings)) + parts[-1]
        assert digest(reconstructed) == expected

shell = read("shell/results.json")
assert not shell["nativeApplicationStarted"] and not shell["rssMeasured"]
assert len(shell["results"]) == 6
fixed = read("fixed/results.json")
assert not fixed["nativeApplicationStarted"] and not fixed["rssMeasured"]
assert len(fixed["results"]) == 8
for row in fixed["results"]:
    assert row["meshCount"] == 1 and row["remainingCanvases"] == 0
    assert not row["errors"] and row["dpr"] == 2
    assert (row["canvas"]["width"], row["canvas"]["height"]) == (1496, 1342)
    allocations = row["allocations"]
    if row["mode"] == "candidate0110":
        assert [(a["width"], a["height"]) for a in allocations] == [
            (1426, 150), (1426, 960), (1496, 960), (1496, 1342)]
        assert [a["teleported"] for a in allocations[:2]] == ["motion-mark", "motion-mark"]
    else:
        assert row["mode"] == "fixed-worktree"
        assert [(a["width"], a["height"]) for a in allocations] == [(1496, 150), (1496, 1342)]
        assert all(a["teleported"] == "route-section active ov" for a in allocations)
for case in read("pixel-comparison.json")["cases"]:
    theme = case["theme"]
    old = Image.open(ROOT / f"fixed/candidate0110-mid-{theme}.original.png").convert("RGBA")
    new = Image.open(ROOT / f"fixed/fixed-worktree-mid-{theme}.original.png").convert("RGBA")
    assert old.size == new.size == (2360, 1520)
    assert old.tobytes() == new.tobytes()
    assert digest(old.tobytes()) == case["oldPixelSha256"] == case["fixedPixelSha256"]
    assert case["differingChannels"] == case["maximumDifference"] == 0
before = (ROOT / "regression-before.original.log").read_text(encoding="utf-8-sig")
assert re.search(r"2 failed\s*\|\s*6 passed", before)
all_tests = (ROOT / "ui-all.original.log").read_text(encoding="utf-8-sig")
assert re.search(r"Test Files\s+99 passed", all_tests)
assert re.search(r"Tests\s+535 passed", all_tests)
assert (ROOT / "typecheck.original.log").stat().st_size == 0
assert "built in" in (ROOT / "build.original.log").read_text(encoding="utf-8-sig")
state = read("at-rest-protection.json")
assert state["source"] == SOURCE and state["nativeCount"] == 0 and state["codeWorktreeClean"]
assert all(state["sharedFilesIdentical"].values())
assert state["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert state["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
pr = read("pr-after.original.json")
assert pr["head"]["sha"] == SOURCE and pr["draft"] and pr["state"] == "open"
assert pr["merged_at"] is None and pr["auto_merge"] is None
summary = {
    "status": "verified-final-source-allocation-diagnostic",
    "files": len(files), "source": SOURCE, "finalWorktreeBrowserCases": 8,
    "equalPixelThemes": 2, "finalBuffer": [1496, 1342],
    "intermediateOldContainerAllocationRemoved": True,
    "uiFilesPassed": 99, "uiTestsPassed": 535,
    "nativeRssMeasured": False, "historicalMemoryFailureCleared": False,
    "fullPerformanceGatePassed": False, "releaseAccepted": False,
}
if (ROOT / "ci-log-verification.json").exists():
    ci = read("ci-log-verification.json")
    assert ci["source"] == SOURCE and ci["actualCheckoutTreesEqual"]
    assert {r["event"] for r in ci["runs"]} == {"push", "pull_request"}
    for run in ci["runs"]:
        assert len(run["jobs"]) == 3
        for job in run["jobs"]:
            assert job["tree"] == ci["tree"]
            data = (ROOT / job["log"]).read_bytes()
            assert len(data) == job["bytes"] and digest(data) == job["sha256"]
            assert job["counts"]["failed"] == 0
    summary["exactSourceCiVerified"] = True
else:
    summary["exactSourceCiVerified"] = False
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
