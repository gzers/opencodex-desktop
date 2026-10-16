"""Verify configured source observations only; no cycle or update acceptance."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
manifest = read("manifest.json")
assert set(files) == {row["path"] for row in manifest["files"]}
for row in manifest["files"]:
    data = files[row["path"]].read_bytes()
    assert len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"]
summary = read("source-conditions.json")
assert summary["source"] == SOURCE and summary["currentCandidateVersion"] == "0.1.10"
config = subprocess.run(["git", "-C", str(CODE), "show", SOURCE + ":" + summary["configPath"]],
                        check=True, capture_output=True).stdout
assert hashlib.sha256(config).hexdigest() == summary["configGitBlobSha256"]
channels = json.loads(config)["updates"]["desktop"]["channels"]
assert len(summary["observations"]) == 4
records = {record["name"]: record for record in summary["observations"]}
for name, record in records.items():
    assert record == read(name + "-read.json")
    assert record["transportExit"] == 0 and record["httpStatus"] in (200, 404)
    data = (ROOT / record["bodyFile"]).read_bytes()
    assert len(data) == record["bodyBytes"] and hashlib.sha256(data).hexdigest() == record["bodySha256"]
    assert not record["automaticRetryRequested"] and not record["credentialsSentToProductionEndpoint"]
    assert record["credentialMode"] == ("github-api-only" if name.endswith("-release") else "none")
assert records["stable-manifest"]["configuredUrl"] == channels["stable"]
assert records["beta-manifest"]["configuredUrl"] == channels["beta"]
assert records["stable-release"]["httpStatus"] == records["stable-manifest"]["httpStatus"] == 200
assert records["beta-release"]["httpStatus"] == records["beta-manifest"]["httpStatus"] == 404
assert records["stable-release"]["releaseTag"] == "v0.1.9"
assert records["stable-manifest"]["version"] == "0.1.9"
assert json.loads((ROOT / records["stable-manifest"]["bodyFile"]).read_bytes())["version"] == "0.1.9"
assert not summary["trustedBetaManifestAvailable"] and summary["stableManifestVersionNumericallyHigher"] is False
assert not summary["productStarted"] and not summary["schedulerStateChanged"] and not summary["timersReset"]
assert not summary["npmOrLifecycleScriptsExecuted"] and not summary["fZ50Invoked"]
assert not summary["stable24hGatePassed"] and not summary["beta6hGatePassed"]
assert not summary["nativeUpdateProgressGatePassed"] and not summary["releaseRequested"]
protect = read("protection.json")
assert protect["nativeCount"] == 0 and protect["managerFixtureHashesUnchanged"]
assert protect["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protect["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
result = {
    "status": "verified-current-update-source-conditions", "files": len(files), "source": SOURCE,
    "observedAtUtc": summary["observedAtUtc"], "stableManifestVersion": "0.1.9",
    "betaReleaseHttpStatus": 404, "betaManifestHttpStatus": 404,
    "higherManagerVersionObserved": False, "productStarted": False,
    "schedulerOrTimersChanged": False, "cycleGatePassed": False,
    "nativeUpdateProgressGatePassed": False, "historicalFailuresCleared": False,
    "releaseRequested": False,
}
(ROOT / "verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
