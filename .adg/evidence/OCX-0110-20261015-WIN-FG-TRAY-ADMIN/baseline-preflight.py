"""Read-only baseline provenance; does not start an app or claim performance acceptance."""
import concurrent.futures
import hashlib
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
baseline_source = "de6862d40389d527527b9db8ef52e186a58b625f"
candidate_source = "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
result_path = root / "baseline-preflight-result.json"
assert not result_path.exists(), "Inspect the existing result instead of repeating."
prior_env = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}
record = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "result": "unverified",
    "baselineSource": baseline_source,
    "candidateSource": candidate_source,
    "nativeApplicationsStarted": 0,
    "installerExecuted": False,
    "performanceMeasured": False,
    "performanceGatePassed": False,
}


def save_new(name, value):
    with (root / name).open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(value, handle, ensure_ascii=False, indent=2)
        handle.write("\n")


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(
        ["git", "credential", "fill"], cwd=code, capture_output=True, text=True,
        input="protocol=https\nhost=github.com\n\n", check=True, timeout=20,
    )
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def get(path):
        config = (
            f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
            f'header = "Authorization: Bearer {token}"\n'
            'header = "Accept: application/vnd.github+json"\n'
        )
        response = subprocess.run(
            [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
             "--connect-timeout", "10", "--max-time", "25", "--fail", "--silent", "--show-error"],
            input=config.encode(), capture_output=True, timeout=30,
        )
        if response.returncode:
            raise RuntimeError("Baseline metadata read unavailable; no download, dispatch, or restart.")
        return json.loads(response.stdout)

    requests = {
        "baseline-release.original.json": "releases/tags/v0.1.9",
        "baseline-tag-ref.original.json": "git/ref/tags/v0.1.9",
    }
    fetched = {}
    failures = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        futures = {pool.submit(get, path): name for name, path in requests.items()}
        for future, name in futures.items():
            try:
                value = future.result()
                save_new(name, value)
                fetched[name] = value
            except Exception as error:
                failures.append({"name": name, "error": type(error).__name__, "message": str(error)})
    if failures:
        record["readFailures"] = failures
        raise RuntimeError("Metadata preflight incomplete; inspect retained read results.")

    release = fetched["baseline-release.original.json"]
    tag = fetched["baseline-tag-ref.original.json"]
    assert release["tag_name"] == "v0.1.9" and not release["draft"]
    assert tag["ref"] == "refs/tags/v0.1.9"
    tag_object = tag["object"]
    if tag_object["type"] == "tag":
        annotated = get("git/tags/" + tag_object["sha"])
        save_new("baseline-annotated-tag.original.json", annotated)
        tag_object = annotated["object"]
    assert tag_object["type"] == "commit" and tag_object["sha"] == baseline_source

    extraction_path = code / "test/software/baseline019-v2/msi-readonly.json"
    extraction_bytes = extraction_path.read_bytes()
    extraction = json.loads(extraction_bytes.decode("utf-8-sig"))
    save_new("baseline-extraction-receipt.original.json", extraction)
    package = Path(extraction["package"])
    assets = [asset for asset in release["assets"] if asset["name"] == package.name]
    assert len(assets) == 1, "The local MSI must be an exact named asset of the official release."
    asset = assets[0]
    package_hash = hashlib.sha256(package.read_bytes()).hexdigest()
    assert package_hash == extraction["packageSha256"]
    assert package.stat().st_size == asset["size"]
    assert asset.get("digest") == "sha256:" + package_hash, "A missing digest is not an exact identity proof."
    assert extraction["properties"]["ProductVersion"] == "0.1.9"
    assert extraction["result"] == "pass" and not extraction["installerExecuted"]
    payloads = []
    for row in extraction["extractedExecutables"]:
        executable = Path(row["path"])
        actual_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
        assert actual_hash == row["sha256"] and executable.stat().st_size == row["bytes"]
        payloads.append({"name": row["name"], "bytes": row["bytes"], "sha256": actual_hash})
    candidate = code / "test/software/candidate-e6a4b32-38591604133/msi-readonly/opencodex-desktop.exe"
    candidate_hash = hashlib.sha256(candidate.read_bytes()).hexdigest()
    assert candidate_hash == "2c9cad960fcee27b6eb1ed6ff9301b0d89101d29d24c16f0aa8a6b06ae01f6b3"
    record.update({
        "result": "baseline_provenance_verified",
        "officialReleaseId": release["id"],
        "officialAssetId": asset["id"],
        "officialAssetDigestMatched": True,
        "baselineMsiSha256": package_hash,
        "baselineExtractionReceiptSha256": hashlib.sha256(extraction_bytes).hexdigest(),
        "baselinePayloads": payloads,
        "candidateExeSha256": candidate_hash,
        "originalBudgets": {
            "startupSamplesPerVersionPerColdOrWarmModeMin": 20,
            "interactiveStartupP95IncreaseMsMax": 100,
            "interactiveStartupP95IncreasePercentMax": 5,
            "idleSampleSecondsPerVersionMin": 600,
            "cpuIncreasePercentagePointsMax": 0.5,
            "rssIncreaseMiBMax": 10,
            "mainThreadTaskMsMax": 50,
            "freshCacheRoutes": 100,
            "freshCacheRestarts": 10,
            "freshCacheAutomaticRequestsAndQueryProcessesMax": 0,
            "realStablePeriodHours": 24,
            "realBetaPeriodHours": 6,
        },
        "measurementDependencies": [
            "Ordinary tray instance exits normally and native instance count becomes zero.",
            "Foreground window and same-machine controlled scene for each sequential version.",
            "Original cold/warm definitions and original CPU/RSS metering; no post-measurement threshold changes.",
            "A read-only-ready observer and independent original request/process counts.",
        ],
    })
except Exception as error:
    record["result"] = "preflight_incomplete"
    record["error"] = {"type": type(error).__name__, "message": str(error)}
finally:
    for key, value in prior_env.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
    record["finishedAtUtc"] = datetime.now(timezone.utc).isoformat()
    save_new(result_path.name, record)
    print(json.dumps(record, ensure_ascii=False))
