"""Local red/green focus regression, separate from retained native source acceptance."""
import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
red_initial = read("red.json")
red = read("red-final.json")
green = read("green.json")
checks = {name: read(name + ".json") for name in ["typecheck", "frontend-all", "build"]}
assert red_initial["exitCode"] == red["exitCode"] == 1
assert green["exitCode"] == 0 and all(value["exitCode"] == 0 for value in checks.values())
assert red_initial["componentWorktreeBlob"] == red["componentWorktreeBlob"]
assert red["componentWorktreeBlob"] != green["componentWorktreeBlob"]
assert red["testsWorktreeBlob"] == green["testsWorktreeBlob"] == "3da03dbbd22ac21e05b55b27b7790a794d784dc8"
assert red_initial["testsWorktreeBlob"] != red["testsWorktreeBlob"]
assert green["componentWorktreeBlob"] == "a5d587f4b7a4828cac9234feb9c7335ba0c38155"
for value in [red_initial, red, green, *checks.values()]:
    assert value["branch"] == "feature/0.1.10-update-center-focus"
    assert value["head"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
    assert not value["actualNativeUiAccepted"] and not value["nativeCycleOrPerformanceAccepted"]
    assert not value["productRestarted"]
for value in checks.values():
    assert value["componentWorktreeBlob"] == green["componentWorktreeBlob"] and value["testsWorktreeBlob"] == green["testsWorktreeBlob"]
def stdout(phase):
    return re.sub(r"\x1b\[[0-9;]*m", "", (root / (phase + ".stdout.txt")).read_text(encoding="utf-8-sig"))
for phase in ["red", "red-final"]:
    assert re.search(r"3 failed\s*\|\s*8 passed\s*\(11\)", stdout(phase))
assert re.search(r"11 passed\s*\(11\)", stdout("green"))
assert re.search(r"95 passed\s*\(95\)", stdout("frontend-all"))
assert re.search(r"483 passed\s*\(483\)", stdout("frontend-all"))
identity = read("source-and-ci-boundary.json")
assert identity["nativeSourceCommit"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert identity["featureCommit"] == "d975b2351920bc027c931c8135b5605fde7a46c2"
assert identity["componentBlob"] == green["componentWorktreeBlob"] and identity["testsBlob"] == green["testsWorktreeBlob"]
assert not any(identity[key] for key in ["mainIntegrated", "fullCiAccepted", "featureNativeUiAccepted", "productOrCycleRestarted", "releaseAccepted"])
runs = read("ci-after-feature-push.json")
if isinstance(runs, dict):
    runs = [runs]
assert len(runs) == 1 and runs[0]["id"] == 38349316806
assert runs[0]["head_sha"] == identity["featureCommit"] and runs[0]["status"] == "in_progress"
assert runs[0]["conclusion"] is None
disposition = read("regression-disposition.json")
assert disposition["featureCommit"] == identity["featureCommit"] and disposition["featureCiRun"] == 38349316806
assert not any(disposition[key] for key in ["scheduleCodeModified", "retainedApplicationPatchedOrRestarted",
    "mainIntegrated", "fullCiAccepted", "fixNativeUiAccepted", "pairedPerformanceAccepted", "realCyclesAccepted",
    "fullNativeGateAccepted", "formalTaskReceipt", "releaseAccepted"])
actual_native = json.loads((root.parent / "OCX-0110-20261013-WIN-EAE-NOON-NATIVE/06-all-update-objects.json").read_text(encoding="utf-8-sig"))
assert actual_native["sourceCommit"] == identity["nativeSourceCommit"]
assert actual_native["result"]["activeElement"]["tag"] == "BODY"
assert not actual_native["result"]["activeElement"]["inUpdateDialog"]
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries),
    "scope": "actual eae native focus finding plus identical-test-blob red/green source fix, local483 frontend tests/typecheck/build; feature CI and native fixed build pending",
    "nativeSourceCommit": identity["nativeSourceCommit"], "featureCommit": identity["featureCommit"],
    "finalRedFailedTests": 3, "finalRedPassedTests": 8, "greenPassedTests": 11,
    "frontendPassedTests": 483, "frontendPassedFiles": 95,
    "mainIntegrated": False, "fullCiAccepted": False, "fixedNativeUiAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "localRedGreen": "passed",
                  "featureCommit": identity["featureCommit"], "nativeFixAcceptance": False}))
