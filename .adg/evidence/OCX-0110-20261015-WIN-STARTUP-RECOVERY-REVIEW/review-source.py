"""Read exact Git blobs and locate a startup-recovery wiring gap; no native execution."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "3c51367f1be89abf66f69805d29b2a660a570a2c"
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=code).decode().strip() == source
assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=code).strip()
files = (
    "apps/desktop/tauri/src/lib.rs",
    "apps/desktop/tauri/src/main.rs",
    "apps/desktop/tauri/src/modules/data_root/bootstrap.rs",
    "apps/desktop/tauri/src/modules/instance/mod.rs",
    "apps/desktop/tauri/src/commands/data_root.rs",
    "apps/desktop/ui/src/features/data-root/store.ts",
    "apps/desktop/tauri/tests/app_startup_wiring.rs",
)
identities = {}
texts = {}
for path in files:
    raw = subprocess.check_output(["git", "show", source + ":" + path], cwd=code)
    blob = subprocess.check_output(["git", "rev-parse", source + ":" + path], cwd=code).decode().strip()
    assert raw == subprocess.check_output([
        "git", "show", "43142bdbcc7a0b9f1325103febae3f44dce407ab:" + path], cwd=code)
    texts[path] = raw.decode("utf-8")
    identities[path] = {"gitBlob": blob, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def locate(path, needle):
    text = texts[path]
    assert text.count(needle) == 1, (path, needle)
    return {"path": path, "line": text[:text.index(needle)].count("\n") + 1,
            "gitBlob": identities[path]["gitBlob"]}


lib, main, bootstrap, lock, commands, store, startup_tests = files
locations = {
    "setupClosure": locate(lib, ".setup(move |app| {"),
    "installationDefaultAnchor": locate(lib, "crate::modules::data_root::bootstrap::default_anchor("),
    "rootResolutionBeforeState": locate(lib, "let (runtime_config, locks) = crate::modules::data_root::bootstrap::resolve_locked_with_boundary("),
    "firstRootState": locate(lib, "app.manage(crate::state::SharedDataRootAnchor(anchor));"),
    "buildErrorExpect": locate(lib, '.expect("failed to build OpenCodeX Desktop")'),
    "mainCallsRunOnly": locate(main, "    opencodex_desktop_lib::run();"),
    "lockBeforeInitialize": locate(bootstrap, "let mut locks = vec![crate::modules::instance::AppInstanceLock::acquire(anchor)?];"),
    "managerStateDirectoryCreation": locate(lock, "std::fs::create_dir_all(&state_dir).map_err(|error| AppError::FileSystem {"),
    "existingSwitchRequiresInitializedAnchor": locate(commands, "pub async fn switch_data_root(\n    request: DataRootSwitchRequest,\n    anchor: tauri::State<'_, SharedDataRootAnchor>,"),
    "settingsSwitchAction": locate(store, "    async switchTo(targetPath: string, migrateData: boolean) {"),
    "existingStartupTestUnixOnly": locate(startup_tests, "#![cfg(unix)]"),
}
assert (locations["setupClosure"]["line"] < locations["installationDefaultAnchor"]["line"]
        < locations["rootResolutionBeforeState"]["line"] < locations["firstRootState"]["line"]
        < locations["buildErrorExpect"]["line"])
resolution_start = texts[lib].index("let (runtime_config, locks)")
resolution_end = texts[lib].index("app.manage(crate::state::SharedDataRootAnchor(anchor));")
assert ")?;" in texts[lib][resolution_start:resolution_end]
assert "initialize(anchor)?;" in texts[bootstrap]
report = {
    "source": source, "sameProductFilesAsFixed431Candidate": True,
    "method": "Exact Git blob reading plus located control-flow edges; human-reviewed source finding",
    "sourceFiles": identities, "locations": locations,
    "finding": {
        "id": "WIN-0110-STARTUP-ROOT-RECOVERY",
        "status": "open-source-wiring-gap",
        "contract": "IMP-20 section 4: unwritable Windows root must explicitly offer custom-root guidance; no silent AppData fallback or daily administrator requirement",
        "scenario": "New installation without legacy AppData; data root or manager-state cannot be created/opened under the ordinary user token",
        "observedSourceBehavior": [
            "Installer-relative default root is selected before root/store initialization.",
            "Root lock/initialization errors propagate out of setup before SharedDataRootAnchor is managed.",
            "This application has no startup root-recovery branch around the fallible setup root resolution.",
            "Pinned Tauri 2.12.1 runs setup on Ready and panics on a returned setup error; the project's build expect is not that error handler.",
            "Existing settings root-switch command requires the initialized anchor; it cannot substitute for failed startup admission.",
        ],
        "nativeOrdinaryTokenFailureReproduced": False,
        "actualWindowsErrorUiObserved": False,
        "allPossibleFrameworkErrorPresentationExcluded": False,
        "productFixed": False,
        "requiredNextWork": [
            "Implement and test an explicit startup root-recovery path, including a durable validated user choice before normal service initialization.",
            "Retain old roots/bindings on cancellation or failure; never convert malformed roots, instance conflicts or migration protection into a silent fallback.",
            "Verify ordinary denied-write, chosen-root startup and subsequent ordinary/administrator same-root readback on the matching candidate.",
        ],
    },
    "unitOrNativeTestsExecuted": False, "productLaunched": False,
    "userProfileOrAclChanged": False, "codeModifiedByThisReview": False,
    "installerRetentionResultInvalidated": False, "fullStorageGatePassed": False,
    "historicalQueryFailureCleared": False, "fullVersionGatePassed": False,
    "frameworkVerificationRef": "framework-source-review.json",
}
path = root / "source-review.json"
if "--build" in sys.argv:
    assert not path.exists()
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert json.loads(path.read_text(encoding="utf-8")) == report
print(json.dumps({
    "sourceFilesReviewed": len(files), "exactSource": source,
    "sameCandidateProductFiles": True, "finding": report["finding"]["id"],
    "nativeFailureReproduced": False, "productFixed": False,
    "reviewIntegrity": "passed", "storageGatePassed": False,
}))
