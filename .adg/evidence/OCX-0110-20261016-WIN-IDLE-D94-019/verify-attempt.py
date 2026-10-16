"""Verify failure evidence and the native UI subset; never certify this pair."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


manifest = read("manifest.json")
actual = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
          if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
assert set(actual) == {e["path"] for e in manifest["files"]}
for entry in manifest["files"]:
    data = actual[entry["path"]].read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
before = (ROOT / "attempt-result.json").read_bytes()
subprocess.run([sys.executable, "-B", str(ROOT / "analyze-attempt.py")],
               capture_output=True, text=True, check=True)
assert before == (ROOT / "attempt-result.json").read_bytes()
assert not (ROOT / "comparison.json").exists()
rejection = read("complete-pair-rejection.json")
assert rejection["comparisonRefused"] and rejection["measurementRerun"] is False
result = read("attempt-result.json")
assert result["validPairForRecordedSubset"] is False
assert not result["newCandidateMemoryBudgetDetermined"] and not result["oldBaselineOrInterruptedSamplesSubstituted"]
ui = read("native-ui-observations.json")
assert ui["source"] == result["source"] and ui["observedAfterMeasurementStopped"]
assert ui["qualityScreenshotShowsHighAndWebgl"] and ui["refreshRetainsExpandedMonth"]
assert not ui["fullPixelAlignmentMatrixPassed"] and not ui["keyboardFoldingPassed"]
assert not ui["backupRestoreOrCleanupClicked"] and not ui["settingsSavedOrProxyChanged"]
for name in ui["evidence"]:
    observation = read(name)
    assert observation["window"]["app"].endswith(
        "candidate-d94c21-38827408691\\msi-readonly\\opencodex-desktop.exe")
    for image in observation["images"]:
        assert (ROOT / image["filename"]).stat().st_size > 1000
collapsed = read("candidate-backup-tree-collapsed.json")["accessibility"]["tree"]
expanded = read("candidate-backup-tree-expanded.json")["accessibility"]["tree"]
refreshed = read("candidate-backup-tree-refreshed.json")["accessibility"]["tree"]
assert "展开 10" in collapsed and "manual-preferences" not in collapsed
assert "收起 10" in expanded and "manual-preferences" in expanded
assert "收起 10" in refreshed and "restore-protection" in refreshed
assert "已固定 · 受保护" in refreshed
state = read("at-rest-protection.json")
assert state["nativeCount"] == 0 and state["codeWorktreeClean"]
assert all(state["sharedFilesIdentical"].values()) and state["candidateFixturePreferencesUnchangedSincePreparation"]
assert state["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert state["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
close = read("sky-close-confirmed.json")
assert not close["remainingCandidateWindows"] and not close["remainingBaselineWindows"]
summary = {"status": "verified-rejected-performance-attempt-and-native-ui-subset",
           "files": len(actual), "source": result["source"],
           "baselineCounterSpanSeconds": result["versions"]["baseline019"]["counterSpanSeconds"],
           "baselineChildDiscoveryReadFailures": 1,
           "candidateCounterSpanSeconds": result["versions"]["candidate0110"]["counterSpanSeconds"],
           "candidateForegroundLostSamples": 1, "bothNormalExits": True,
           "nativeHighWebglSettingsObserved": True, "monthExpansionAndRefreshObserved": True,
           "validPerformancePair": False, "newCandidateMemoryBudgetDetermined": False,
           "fullPerformanceGatePassed": False, "historicalFailuresErased": False,
           "fullNativeGatePassed": False, "releaseAccepted": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
