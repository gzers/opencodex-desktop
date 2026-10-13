"""Verify frozen exact-SHA CI archives and bounded native observation claims."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "5a36cda04018fb367c2f77e3a2a1bdc22d991e03"
candidate = "be7395acfce0ec1e36b83300b576db408f8c35bb"
image = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
result = read("background-result.json")
assert result["source"] == source and result["candidateSource"] == candidate
assert result["ciRuns"] == [{"id": 38431616359, "headSha": source, "status": "completed", "conclusion": "success"}]
assert result["protectedBytesUnchanged"] and result["originalApplicationPidAndCreationRetained"]
for key in ["inputIssued", "installerRun", "newNativeInstanceStarted",
            "completeNativeGatePassed", "fullPerformanceOrCyclePassed"]:
    assert result[key] is False
assert read("protected-before.json") == read("protected-after.json")
assert read("protected-after.json")["automaticCheck"] is False
assert read("local-native-census.json")["isolatedNativeCount"] == 1
native = read("native-caption-point.json")
assert native["pid"] == 10904 and native["hwnd"] == 6685560
assert native["identity"]["sha256"] == image
assert native["identity"]["creationFileTime"] == 134363770515757733
assert native["window"]["dpi"] == 192 and not native["window"]["foreground"]
assert not native["window"]["maximized"] and not native["window"]["minimized"]
assert native["captionQuery"]["ok"] and native["window"]["dwm"]["backdrop"]["value"] == 1
points = [p for b in native["buttons"].values() for p in b["probes"]]
assert len(points) == 9 and all(p["ok"] for p in points)
assert all(not p["windowUnderPoint"]["rootMatchesTarget"] for p in points)
assert all(not p["windowUnderPoint"]["belongsToTargetPid"] for p in points)
assert all(p["windowUnderPoint"]["targetFamilyHwnd"] is None for p in points)
assert all(p["windowUnderPoint"]["targetFamilyClass"] is None for p in points)
assert not native["inputIssued"] and not native["applicationLifecycleChanged"] and not native["acceptancePassed"]
summary = read("diagnostic-summary-and-next-steps.json")
assert summary["diagnosticSource"] == source and summary["candidateSource"] == candidate
assert summary["localPortableContracts"] == 16 and not summary["rootCauseIdentified"]
assert summary["ciRequiredJobs"] == {
    "release-tools": "success", "windows-smoke-evidence": "success", "backend": "success",
    "frontend": "success", "backend-windows": "success", "build": "skipped",
}
assert not summary["probeBranchChangesProductRuntimeSinceMain"] and not summary["probeBranchContainsFocusFix"]
assert not summary["candidateSourceAcceptedAsDiagnosticSource"] and not summary["anyFullRequiredGatePromoted"]
assert "Ran 16 tests" in (root / "portable-contracts.stderr.txt").read_text()
assert read("portable-contracts-result.json")["exitCode"] == 0
artifacts = read("ci-artifact-verification.json")
assert artifacts["source"] == source and len(artifacts["artifacts"]) == 2
assert not artifacts["authorizationHeaderForwardedToStorageHost"]
for item in artifacts["artifacts"]:
    assert item["digestMatches"] and item["zipCrcVerified"]
    assert item["actualSha256"] == item["expectedSha256"]
    assert not item["archiveContentsExecuted"] and not item["installerOrNewCandidate"]
for item in summary["selectedCiOriginals"]:
    raw = (root / item["savedAs"]).read_bytes()
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"]
smoke = read("ci-selected-windows-startup-startup.json")
assert smoke["expectedCommit"] == source and smoke["sourceCommitMatched"] and smoke["result"] == "pass"
assert smoke["mode"] == "any" and smoke["applicationToken"]["elevated"] is True
assert smoke["sha256"] != image and smoke["pid"] != native["pid"]
assert read("session-after.json")["applicationToken"]["elevated"] is False
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "archive-verification.json"})
manifest = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size,
             "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == manifest, "Frozen evidence bytes changed"
report = {"archiveIntegrity": "passed", "files": len(files), "diagnosticCiExactShaVerified": True,
          "ciArchivesVerified": 2, "windowsPortableContracts": 16, "readonlyNativePoints": 9,
          "inputIssued": False, "restoreAccepted": False, "fullRequiredGatePromoted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
