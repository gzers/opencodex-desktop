"""Bounded read-only source conditions, never a product cycle or install."""
import concurrent.futures
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
assert not (ROOT / "source-conditions.json").exists()
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
config_path = "apps/desktop/tauri/config/runtime.defaults.json"
config_bytes = subprocess.run(
    ["git", "-c", "core.longpaths=true", "-C", str(CODE), "show", SOURCE + ":" + config_path],
    check=True, capture_output=True).stdout
channels = json.loads(config_bytes)["updates"]["desktop"]["channels"]
assert channels == {
    "stable": "https://github.com/gzers/opencodex-desktop/releases/latest/download/latest.json",
    "beta": "https://github.com/gzers/opencodex-desktop/releases/download/beta/latest.json",
}
spec = importlib.util.spec_from_file_location(
    "source_condition_credentials", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
assert "\r" not in token and "\n" not in token


def probe(name, url, authenticated):
    target = ROOT / (name + ".original.body")
    assert not target.exists()
    entries = [
        "url = " + json.dumps(url),
        'header = "User-Agent: OpenCodex-source-precondition-read"',
        'header = "Accept: application/vnd.github+json"' if authenticated else
        'header = "Accept: application/json"',
    ]
    if authenticated:
        assert url.startswith("https://api.github.com/repos/gzers/opencodex-desktop/")
        entries.append("header = " + json.dumps("Authorization: Bearer " + token))
    command = [
        r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
        "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2", "--silent", "--show-error",
        "--connect-timeout", "10", "--max-time", "25", "--max-filesize", "2097152",
        "--proto", "=https", "--proto-redir", "=https",
        "--output", str(target), "--write-out", "%{http_code}",
    ]
    if not authenticated:
        # Public production endpoints receive no Git credentials, including
        # when GitHub redirects to release asset storage.
        command.extend(["--location", "--max-redirs", "5"])
    result = subprocess.run(command, input=("\n".join(entries) + "\n").encode(),
                            capture_output=True, timeout=30)
    record = {
        "name": name, "atUtc": datetime.now(timezone.utc).isoformat(), "configuredUrl": url,
        "httpStatus": int(result.stdout) if result.stdout.strip().isdigit() else None,
        "transportExit": result.returncode,
        "credentialMode": "github-api-only" if authenticated else "none",
        "credentialsSentToProductionEndpoint": False,
        "automaticRetryRequested": False,
        "bodyFile": target.name if target.exists() else None,
        "bodyBytes": target.stat().st_size if target.exists() else 0,
        "bodySha256": hashlib.sha256(target.read_bytes()).hexdigest() if target.exists() else None,
    }
    if result.returncode == 0 and record["httpStatus"] == 200:
        try:
            data = json.loads(target.read_bytes())
            record["jsonParsed"] = True
            record["version"] = data.get("version")
            record["releaseTag"] = data.get("tag_name")
            record["releasePublishedAt"] = data.get("published_at")
            record["draft"] = data.get("draft")
            record["prerelease"] = data.get("prerelease")
            record["platforms"] = sorted(data.get("platforms", {}))
        except (ValueError, TypeError, AttributeError):
            record["jsonParsed"] = False
    return record


requests = [
    ("stable-release", api.BASE + "releases/latest", True),
    ("beta-release", api.BASE + "releases/tags/beta", True),
    ("stable-manifest", channels["stable"], False),
    ("beta-manifest", channels["beta"], False),
]
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
    futures = [pool.submit(probe, *request) for request in requests]
    records = [future.result() for future in futures]
by_name = {r["name"]: r for r in records}
for record in records:
    (ROOT / (record["name"] + "-read.json")).write_text(
        json.dumps(record, indent=2) + "\n", encoding="utf-8")


def higher(record):
    version = record.get("version")
    match = re.fullmatch(r"v?(\d+)\.(\d+)\.(\d+)(?:[-+][0-9A-Za-z.-]+)?", version or "")
    return tuple(map(int, match.groups())) > (0, 1, 10) if match else None


summary = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "configPath": config_path, "configGitBlobSha256": hashlib.sha256(config_bytes).hexdigest(),
    "currentCandidateVersion": "0.1.10", "observations": records,
    "trustedStableManifestAvailable": by_name["stable-manifest"]["transportExit"] == 0
    and by_name["stable-manifest"]["httpStatus"] == 200 and by_name["stable-manifest"].get("jsonParsed", False),
    "trustedBetaManifestAvailable": by_name["beta-manifest"]["transportExit"] == 0
    and by_name["beta-manifest"]["httpStatus"] == 200 and by_name["beta-manifest"].get("jsonParsed", False),
    "stableManifestVersionNumericallyHigher": higher(by_name["stable-manifest"]),
    "betaManifestVersionNumericallyHigher": higher(by_name["beta-manifest"]),
    "scope": "source availability only; direct HTTP reads, not product planner requests",
    "productStarted": False, "schedulerStateChanged": False, "timersReset": False,
    "npmOrLifecycleScriptsExecuted": False, "fZ50Invoked": False,
    "stable24hGatePassed": False, "beta6hGatePassed": False,
    "nativeUpdateProgressGatePassed": False, "releaseRequested": False,
}
(ROOT / "source-conditions.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "source": SOURCE, "observations": [
        {key: row.get(key) for key in ("name", "transportExit", "httpStatus", "version", "releaseTag")}
        for row in records],
    "trustedBetaManifestAvailable": summary["trustedBetaManifestAvailable"],
    "stableManifestVersionNumericallyHigher": summary["stableManifestVersionNumericallyHigher"],
    "productCycleClaimedPassed": False, "timersReset": False,
}))
