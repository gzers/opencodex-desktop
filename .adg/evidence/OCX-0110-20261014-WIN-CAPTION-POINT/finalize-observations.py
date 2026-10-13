"""Summarize bounded observations, extract selected CI metadata, keep open gates."""
import hashlib
import json
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "5a36cda04018fb367c2f77e3a2a1bdc22d991e03"
candidate = "be7395acfce0ec1e36b83300b576db408f8c35bb"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
snapshot = read("native-caption-point.json")
artifacts = read("ci-artifact-verification.json")
copies = []
selected = {
    "process-token.json", "windows-startup/cdp-diagnostics.json",
    "windows-startup/startup-diagnostics.json", "windows-startup/startup.json",
    "windows-startup/webview.json", "windows-backend/environment.json",
    "windows-backend/test-executable-sha256.json",
}
for artifact in artifacts["artifacts"]:
    with zipfile.ZipFile(artifact["downloadedToIgnoredRuntime"]) as archive:
        for item in archive.infolist():
            if item.filename not in selected:
                continue
            raw = archive.read(item)
            destination = "ci-selected-" + item.filename.replace("/", "-")
            with (root / destination).open("xb") as stream:
                stream.write(raw)
            copies.append({"artifactId": artifact["id"], "archivePath": item.filename,
                           "savedAs": destination, "bytes": len(raw),
                           "sha256": hashlib.sha256(raw).hexdigest()})


def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout.decode().strip()


points = [point for button in snapshot["buttons"].values() for point in button["probes"]]
jobs = read("github-jobs-38431616359.json")["jobs"]
assert len(points) == 9
assert all(not point["windowUnderPoint"]["rootMatchesTarget"] for point in points)
assert all(not point["windowUnderPoint"]["belongsToTargetPid"] for point in points)
assert not snapshot["window"]["foreground"]
assert git("diff", "--name-only", "eae85322ebca3b755c034db97cbebf2ea5bf71fc", source, "--", "apps") == ""
summary = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "diagnosticSource": source, "candidateSource": candidate,
    "probeBranchBase": "eae85322ebca3b755c034db97cbebf2ea5bf71fc",
    "probeBranchChangesProductRuntimeSinceMain": False,
    "probeBranchContainsFocusFix": False,
    "candidateSourceAcceptedAsDiagnosticSource": False,
    "ciRequiredJobs": {job["name"]: job["conclusion"] for job in jobs},
    "selectedCiOriginals": copies,
    "localPortableContracts": 16,
    "captionPoints": 9, "captionPointRootsMatchingTarget": 0,
    "parentHitsByButton": {name: [point["value"] for point in button["probes"]]
                           for name, button in snapshot["buttons"].items()},
    "observationOnly": True, "rootCauseIdentified": False,
    "outsideTargetWindowIdsDisclosed": False,
    "nextForegroundSequence": [
        "Use a new human start signal and reconnect/lock guard; retain the single identified instance.",
        "Select the actual main HWND and observe a fresh complete screenshot without assuming focus.",
        "Maximize with one authorized native action; immediately reread identity/state and a complete screenshot.",
        "Run the read-only probe while maximized; require WindowFromPoint/GA_ROOT ownership before clicking.",
        "Use screenshot-verified physical coordinates for one restore click; observe maximized flag and bounds.",
        "If not restored, preserve the actual point and before/after evidence before using system-menu recovery.",
        "Then test minimize/restore, Snap, tray and exit in separate observed steps; no AX-only success inference.",
    ],
    "remainingPrerequisites": {
        "foregroundNative": "new human start signal and stable unlocked desktop",
        "administratorAndDpi": "real elevated token and physical 100/125/150% display changes, foreground window",
        "installationAndOfficial019": "verified installation isolation and real KnownFolder user profile; do not share daily MSI product registration",
        "officialScripts": "FZ50 explicit permission for pinned third-party postinstall, still absent",
        "newSourceAcceptance": "Mac fix commit, same-SHA CI and new candidate artifact; current EXE cannot prove future source",
        "realCyclesAndBackoff": "final source and independently verified raw command/network observation; auto-check remains false",
        "pairedPerformance": "final source, isolated official019 profile and controlled physical-host foreground window; original budgets unchanged",
    },
    "anyFullRequiredGatePromoted": False, "taskClosed": False, "releaseAuthorized": False,
}
with (root / "diagnostic-summary-and-next-steps.json").open("x", encoding="utf-8") as stream:
    json.dump(summary, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
print(json.dumps({"selectedCiMetadataOriginals": len(copies), "localContracts": 16,
                  "captionPoints": 9, "rootCauseIdentified": False}))
