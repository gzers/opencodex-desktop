"""External beta source readiness only, never a product query/cycle acceptance."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
endpoint = "https://github.com/gzers/opencodex-desktop/releases/download/beta/latest.json"
def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))
api = read("authenticated-beta-api-observation.json")
public = read("public-beta-download-observation.json")
assert api["url"] == "https://api.github.com/repos/gzers/opencodex-desktop/releases/tags/beta"
assert api["authenticatedRequest"] and not api["credentialSaved"]
assert api["remoteMainSha"] == source and api["statusCode"] == 404
assert api["frozenBetaEndpoint"] == public["url"] == endpoint
assert public["statusCode"] == 404 and not public["authenticatedRequest"]
assert not public["downloadExecuted"] and not public["sourceModified"] and not public["productCheckInvoked"]
assert read("authenticated-beta-api-body.json")["message"] == "Not Found"
assert read("authenticated-beta-api-body.json")["status"] == "404"
assert read("public-beta-download-body.txt") == {"error": "Not Found"}
for metadata, filename in [(api, "authenticated-beta-api-body.json"), (public, "public-beta-download-body.txt")]:
    assert hashlib.sha256((root / filename).read_bytes()).hexdigest() == metadata["responseBodySha256"]
    assert datetime.fromisoformat(metadata["responseObservedUtc"].replace("Z", "+00:00")) >= \
        datetime.fromisoformat(metadata["requestStartedUtc"].replace("Z", "+00:00"))
prior = read("previous-beta-api-observation.json")
assert prior["httpStatus"] == 404 and prior["sourceEndpoint"] == endpoint
prior_time = datetime.fromisoformat(prior["collectedAtUtc"].replace("Z", "+00:00"))
new_time = datetime.fromisoformat(api["requestStartedUtc"].replace("Z", "+00:00"))
assert (new_time - prior_time).total_seconds() > 6 * 3600
preserved = read("cycle-preservation.json")
assert preserved["sourceCommit"] == source and preserved["bytesUnchanged"]
assert preserved["beforeHashes"] == preserved["afterHashes"]
assert preserved["beforeHashes"]["schedule"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert preserved["beforeHashes"]["preferences"] == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert not any(preserved[key] for key in ["appChannelSwitched", "newBetaCycleStarted", "releaseCreated",
                                         "timerOrClockChanged", "performanceAccepted", "fullNativeGateAccepted"])
life = read("retained-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
assert life["cycleExecSession"] == 46484 and life["managerDeadlineEpoch"] == 1791903339 and life["panelDeadlineEpoch"] == 1791903340
assert not life["cycleAccepted"] and not life["uiInputsPerformed"] and not life["productCheckInvoked"]
assert {item["processId"] for item in life["processes"]} == {37276, 24300, 3920}
assert all(item["actualLiveHandle"] for item in life["processes"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert (root / "previous-beta-api-observation.json").read_bytes() == (
    root.parent / "OCX-0110-20261012-WIN-REAL-BETA-CYCLE/beta-source-api.json").read_bytes()
assert api["frozenRuntimeDefaultsBlob"] == (root / "runtime-defaults-blob.txt").read_text().strip()
defaults = read("runtime.defaults.json")
assert defaults["updates"]["desktop"]["channels"]["beta"] == endpoint
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "archive-verification.json"})
assert all(p.suffix.lower() not in {".exe", ".dll", ".msi", ".zip"} for p in files)
entries = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size,
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "sourceCommit": source, "filesVerified": len(entries),
    "scope": "authenticated beta release and frozen public download endpoint external readiness recheck; both404, original final-source lifecycle and cycle preserved",
    "elapsedSincePreviousSourceCheckSeconds": (new_time - prior_time).total_seconds(),
    "betaSourceAccessible": False, "betaSixHourSuccessCycleStarted": False,
    "betaSixHourCycleAccepted": False, "stableOrPanelCycleAccepted": False,
    "nativeProductQueryAccepted": False, "rawNetworkOrQueryCountAccepted": False,
    "pairedPerformanceAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "authenticatedBetaApi": 404,
                  "frozenPublicEndpoint": 404, "betaCycleAccepted": False}))
