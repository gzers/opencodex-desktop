"""Verify preparation and its limits, never manufacture recording admission."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


source = "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
code, ci = read("code-provenance.json"), read("ci-terminal.original.json")
assert code["source"] == source and code["appsTreeUnchanged"]
for name, expected in code["sourceBlobs"].items():
    data = subprocess.check_output(["git", "show", f"{source}:{name}"], cwd=PROJECT)
    assert hashlib.sha256(data).hexdigest() == expected
run, jobs = ci["run"], ci["jobs"]
assert run["head_sha"] == source and run["id"] == 38858859494 and run["run_attempt"] == 1
assert run["status"] == "completed" and run["conclusion"] == "success" and run["event"] == "push"
assert len(jobs["jobs"]) == jobs["total_count"]
required = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}
assert {j["name"] for j in jobs["jobs"]} == required | {"build"}
assert all(j["status"] == "completed" and j["conclusion"] == "success"
           for j in jobs["jobs"] if j["name"] in required)
assert next(j for j in jobs["jobs"] if j["name"] == "build")["conclusion"] == "skipped"
pure = read("pure-checks.original.log")
fields = ["pureScopeChecks", "pureLifecycleChecks", "pureNativeStopChecks",
          "controllerProcessImageChecks", "descendantHotScopeChecks", "utf8LineBudgetChecks",
          "namedQueryThreadScopeChecks", "executableAdmissionChecks", "runnerEnvironmentChecks",
          "regressionEventChecks", "rawSchedulingByteWindowChecks"]
assert pure["status"] == "passed" and sum(pure[key] for key in fields) == 159
assert pure["rawSchedulingByteWindowChecks"] == 12
assert not pure["etwSessionStarted"] and not pure["testOrNodeStarted"]
gpu, gpu_source = read("gpu-method-checks.original.json"), read("gpu-method-provenance.json")
assert gpu["status"] == "passed" and gpu["checks"] == 21
assert not gpu["productStarted"] and not gpu["nativeSnapshotExecutionTested"]
assert not gpu_source["pushExecuted"] and not gpu_source["sourceCiPassed"]
assert gpu_source["localSource"] == "47144dbe1e813352f0e4618150ab89a15614a4cf"
assert gpu_source["appsTreeUnchanged"]
for name, expected in gpu_source["sourceBlobs"].items():
    data = subprocess.check_output(["git", "show", f"{gpu_source['localSource']}:{name}"], cwd=PROJECT)
    assert hashlib.sha256(data).hexdigest() == expected
fixed = read("fixed-inputs-readback.json")
assert fixed["testSha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert fixed["nodeSha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
assert fixed["officialNodeVersion"] == "24.21.0" and not fixed["libraryOrNodeExecuted"]
assert fixed["deadlinesSeconds"] == [2, 4, 5, 8] and fixed["outerSeconds"] == 120
assert fixed["serializedLimitMiB"] == 64 and fixed["queueLimit"] == fixed["trackedIrpLimit"] == 32768
blocker, protected = read("host-transfer-blocker.json"), read("final-protection-readback.json")
assert blocker["localDnsErrorObserved"] == 10055 and not blocker["etwReplayStarted"]
assert not blocker["originalCiJobLogsFullyRetained"] and not blocker["etwReplayAdmitted"]
assert not blocker["globalNetworkSettingsChanged"] and not blocker["externalProcessTerminated"]
assert protected["nativeCount"] == 0 and not protected["productStatus"]
assert protected["productHead"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert protected["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protected["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not (ROOT / "ci-admission.json").exists()
assert not (ROOT / "remote-dispatch-attempt.json").exists()
for path in ROOT.glob("*transfer-*.original.log"):
    assert path.read_bytes() == b"", "Do not publish signed redirect material"
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        data = (ROOT / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
print(json.dumps({
    "status": "verified-preparation-with-explicit-transfer-blocker",
    "source": source, "wholeCiApiSucceeded": True, "localPureChecks": 159,
    "gpuMethodLocalChecks": 21, "gpuMethodPushedOrCiPassed": False,
    "rawCiLogsFullyRetained": False, "newRecordingAdmitted": False,
    "newRecordingStarted": False, "actualFailureWindowObtained": False,
    "historicalFailureCleared": False, "releaseAccepted": False,
}, indent=2))
