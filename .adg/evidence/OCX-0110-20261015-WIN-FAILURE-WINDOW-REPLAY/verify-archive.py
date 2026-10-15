"""Check offline replay, primary-source notes and immutable archive bytes."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
result = subprocess.run([sys.executable, "-X", "utf8", str(root / "analyze-failure-windows.py")],
                        text=True, capture_output=True, check=True)
replay = json.loads(result.stdout)
assert replay["fixtureTimelinesVerified"] == 35
assert replay["historicalLibrariesReplayedOffline"] == 5
assert not replay["newExecution"] and not replay["actualFailureEtwObtained"]
review = read("decoder-source-review.json")
assert review["tag"] == "v3.1.21" and review["httpStatus"] == 200
assert review["sourceSha256"] == "5a1ff963eb4c5b6dbe8845843c841e8466b5c4855e72b454c7a0cf31ea4a708f"
assert review["waitModeEnumValues"] == {"NonSwap": 0, "Swappable": 1}
assert review["waitModeGetterByteOffset"] == 13
assert not review["independentVersion5LayoutValidationPerformed"] and not review["fullThirdPartySourceVendored"]
assert not review["kernelModeInferenceFrom96Made"]
post = read("checkpoint.original.json")
assert post["source"] == "493da80675e6508e46e0fd83d86ee7216abf96fe"
assert post["codeWorktreeClean"] and post["originalBd2FixtureSourceUnchanged"]
assert post["nativeCount"] == 1 and post["nativeProcesses"][0]["ProcessId"] == 78512
assert post["administratorExitCode"] is None
assert post["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert post["candidateExeSha256"] == "9ca21e30c9c2eee68000e1e8a03c58a42918e46d97387f96a102b7f11afb326a"
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
for key in ("productLaunched", "nodeLaunched", "etwSessionStarted", "uiInputInjected", "agentForcedTermination",
            "macNativeHandoffPerformed", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[key] is False
assert len(post["protectedSandboxHashes"]) == 4
manifest, report_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest, report_path}]
if "--build" in sys.argv:
    assert not manifest.exists() and not report_path.exists()
    manifest.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest.name) == entries
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries),
    "originalInputFilesHashVerified": len(read("input-manifest.json")),
    **replay, "primarySourceReviewBoundedAndRecorded": True,
    "oldEvidenceModified": False, "productionOrCollectorSourceChanged": False,
    "administratorNativeCountAtCheckpoint": post["nativeCount"],
    "administratorExitCode": None, "rootCauseClaimed": False,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path.name) == report
print(json.dumps(report))
