"""Verify retained supplemental baseline records, not current files or performance."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent / "baseline-git-proof-20261015"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
manifest = read("manifest.json")
assert len({row["path"] for row in manifest}) == len(manifest)
for row in manifest:
    path = root / row["path"]
    assert path.resolve().is_relative_to(root.resolve())
    assert path.stat().st_size == row["bytes"]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]
result = read("result.json")
assert result["result"] == "baseline_provenance_verified"
assert result["baselineSource"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert result["annotatedTagObject"] == "e5deeed6512c2cdaf7662a88f831bd0702a7c4ca"
refs = {line.split()[1]: line.split()[0]
        for line in (root / "git-ls-remote.stdout.original.txt").read_text().splitlines()}
assert refs["refs/tags/v0.1.9^{}"] == result["baselineSource"]
assert refs["refs/tags/v0.1.9"] == result["annotatedTagObject"]
release = read("release.original.json")
assert release["id"] == result["officialReleaseId"] and release["tag_name"] == "v0.1.9"
assert not release["draft"]
asset = next(row for row in release["assets"] if row["id"] == result["officialAssetId"])
assert asset["name"] == "OpenCodeX.Desktop_0.1.9_x64_en-US.msi"
assert asset["digest"] == "sha256:" + result["baselineMsiSha256"]
extraction = read("extraction-receipt.original.json")
assert extraction["packageSha256"] == result["baselineMsiSha256"]
assert extraction["result"] == "pass" and not extraction["installerExecuted"]
assert extraction["properties"]["ProductVersion"] == "0.1.9"
assert [{key: row[key] for key in ("name", "bytes", "sha256")}
        for row in extraction["extractedExecutables"]] == result["baselinePayloads"]
assert result["nativeApplicationsStarted"] == 0 and not result["installerExecuted"]
assert not result["performanceMeasured"] and not result["performanceGatePassed"]
assert result["previousApiReadFailureRetained"]
assert result["coldAndWarmSamplesObtained"] == result["controlled600SecondPairsObtained"] == 0
print(json.dumps({"baselineProvenanceArchive": "passed", "manifestFiles": len(manifest),
                  "officialReleaseAssetDigestMatched": True, "nativeApplicationsStarted": 0,
                  "performanceGatePassed": False}))
