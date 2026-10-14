"""Supplement the failed API tag read with remote Git provenance and local byte checks."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
output = root / "baseline-git-proof-20261015"
assert not output.exists(), "Keep this supplemental observation immutable."
output.mkdir()
record = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "result": "unverified",
    "previousApiReadFailureRetained": True,
    "nativeApplicationsStarted": 0,
    "installerExecuted": False,
    "performanceMeasured": False,
    "performanceGatePassed": False,
}
try:
    release_bytes = (root / "baseline-release.original.json").read_bytes()
    (output / "release.original.json").write_bytes(release_bytes)
    release = json.loads(release_bytes.decode("utf-8-sig"))
    assert release["tag_name"] == "v0.1.9" and not release["draft"]
    remote = subprocess.run(
        ["git", "-c", "http.sslBackend=openssl", "ls-remote", "origin",
         "refs/tags/v0.1.9", "refs/tags/v0.1.9^{}"],
        cwd=code, capture_output=True, timeout=30,
    )
    (output / "git-ls-remote.stdout.original.txt").write_bytes(remote.stdout)
    (output / "git-ls-remote.stderr.original.txt").write_bytes(remote.stderr)
    assert remote.returncode == 0
    refs = {line.split()[1]: line.split()[0] for line in remote.stdout.decode().splitlines()}
    assert refs["refs/tags/v0.1.9^{}"] == "de6862d40389d527527b9db8ef52e186a58b625f"
    extraction_bytes = (code / "test/software/baseline019-v2/msi-readonly.json").read_bytes()
    (output / "extraction-receipt.original.json").write_bytes(extraction_bytes)
    extraction = json.loads(extraction_bytes.decode("utf-8-sig"))
    assert extraction["result"] == "pass" and not extraction["installerExecuted"]
    assert extraction["properties"]["ProductVersion"] == "0.1.9"
    package = Path(extraction["package"])
    matching = [asset for asset in release["assets"] if asset["name"] == package.name]
    assert len(matching) == 1
    asset = matching[0]
    package_hash = hashlib.sha256(package.read_bytes()).hexdigest()
    assert asset["digest"] == "sha256:" + package_hash
    assert asset["size"] == package.stat().st_size and package_hash == extraction["packageSha256"]
    payloads = []
    for row in extraction["extractedExecutables"]:
        executable = Path(row["path"])
        digest = hashlib.sha256(executable.read_bytes()).hexdigest()
        assert digest == row["sha256"] and executable.stat().st_size == row["bytes"]
        payloads.append({"name": row["name"], "bytes": row["bytes"], "sha256": digest})
    record.update({
        "result": "baseline_provenance_verified",
        "baselineSource": refs["refs/tags/v0.1.9^{}"],
        "annotatedTagObject": refs["refs/tags/v0.1.9"],
        "officialReleaseId": release["id"],
        "officialAssetId": asset["id"],
        "officialAssetDigestMatched": True,
        "baselineMsiSha256": package_hash,
        "baselinePayloads": payloads,
        "remoteGitTagAndReleaseAssetChecksAreSeparate": True,
        "coldAndWarmSamplesObtained": 0,
        "controlled600SecondPairsObtained": 0,
    })
except Exception as error:
    record["result"] = "preflight_incomplete"
    record["error"] = {"type": type(error).__name__, "message": str(error)}
finally:
    record["finishedAtUtc"] = datetime.now(timezone.utc).isoformat()
    (output / "result.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    manifest = [
        {"path": path.name, "bytes": path.stat().st_size,
         "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        for path in sorted(output.iterdir(), key=lambda item: item.name)
    ]
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(record, ensure_ascii=False))
