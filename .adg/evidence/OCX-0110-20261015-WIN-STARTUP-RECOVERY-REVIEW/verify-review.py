"""Verify the source finding's provenance without calling it a runtime test."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
result = subprocess.run([sys.executable, "-B", "-X", "utf8", str(root / "review-source.py")],
                        capture_output=True, text=True, check=True)
assert json.loads(result.stdout)["reviewIntegrity"] == "passed"
review = read("source-review.json")
assert review["source"] == "3c51367f1be89abf66f69805d29b2a660a570a2c"
assert len(review["sourceFiles"]) == 7 and review["sameProductFilesAsFixed431Candidate"]
finding = review["finding"]
assert finding["id"] == "WIN-0110-STARTUP-ROOT-RECOVERY" and finding["status"] == "open-source-wiring-gap"
assert not finding["nativeOrdinaryTokenFailureReproduced"] and not finding["actualWindowsErrorUiObserved"]
assert not finding["productFixed"] and len(finding["requiredNextWork"]) == 3
for name in ("unitOrNativeTestsExecuted", "productLaunched", "userProfileOrAclChanged",
             "codeModifiedByThisReview", "installerRetentionResultInvalidated", "fullStorageGatePassed",
             "historicalQueryFailureCleared", "fullVersionGatePassed"):
    assert review[name] is False
framework = read("framework-source-review.json")
assert framework["version"] == "2.12.1" and framework["httpStatus"] == 200
assert framework["sourceUrl"] == "https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.1/crates/tauri/src/app.rs"
assert framework["lockedCrateChecksum"] == "ed99ee9694a2deb776d91cae48ac7411ddfc89ecae2f9b5041111d8c88f2ace9"
assert framework["setupRunsAtReadyAndFailurePanics"] and framework["setupHookErrorMapsToSetupError"]
assert not framework["projectBuildExpectIsSetupFailureHandler"]
assert not framework["fullThirdPartySourceCopied"] and not framework["nativeApplicationExecuted"]
assert len(framework["sourceSha256"]) == 64 and framework["sourceBytes"] > 10000
assert framework["readyCallsSetupLine"] < framework["readySetupErrorPanicsLine"] < framework["setupFunctionLine"]
old = read("source-review.initial.json")
assert old["sourceFiles"] == review["sourceFiles"] and old["locations"] == review["locations"]
post = read("final-protection.original.json")
assert post["source"] == review["source"] and post["codeWorktreeClean"]
assert post["nativeCount"] == 0 and post["nativeProcesses"] == []
assert post["diagnosticExecutionLocation"] == "local read-only exact-source review; no new diagnostic or native execution"
assert post["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert post["originalTaskBytes"] == 3878 and len(post["protectedSandboxHashes"]) == 4
for name in ("localProductOrOriginalLibraryOrFixedNodeStarted", "localEtwSessionStarted", "localUiInputInjected",
             "hostInstallerExecuted", "newAccountOrProfileCreated", "defenseOrGlobalEnvironmentChanged",
             "macNativeHandoffPerformed", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[name] is False
manifest_path, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path.name) == entries
report = {
    "evidenceIntegrity": "passed", "manifestFiles": len(entries), "source": review["source"],
    "sourceFilesReviewed": 7, "frameworkPinnedVersion": "2.12.1",
    "finding": finding["id"], "findingStatus": finding["status"],
    "setupFailureAtReadyFrameworkPathVerified": True,
    "initialBuildExpectInferenceCorrectedAndRetained": True,
    "collectorTemplateScopeLabelCorrectedAndInitialRecordRetained": True,
    "nativeFailureReproduced": False, "nativeErrorUiObserved": False, "productionFixImplemented": False,
    "nativeCountAtCheckpoint": 0, "installerRetentionEvidenceInvalidated": False,
    "fullStorageGatePassed": False, "historicalQueryFailureCleared": False,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path.name) == report
print(json.dumps(report))
