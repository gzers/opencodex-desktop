"""Read-only verification of full tool CI and actual original build versions."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
TOOL = "3a2ff08a6b480d237e93502626861090aa14423d"
BASELINE = "de6862d40389d527527b9db8ef52e186a58b625f"
CANDIDATE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def text(data):
    return re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))


def checkout(value):
    result, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", value)
    return result


def compiled(value):
    result = {}
    for name, version in re.findall(r"\bCompiling\s+([a-zA-Z0-9_-]+)\s+v([0-9][0-9A-Za-z.+_-]*)", value):
        result.setdefault(name, set()).add(version)
    return result


for row in read("original-copies.json"):
    data = (ROOT / row["path"]).read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    assert data == Path(row["runtimePath"]).read_bytes()
for row in read("prior-evidence-bindings.json"):
    assert sha((PROJECT / row["directory"] / "manifest.json").read_bytes()) == row["manifestSha256"]
terminal = read("original/ci/terminal.original.json")
run, jobs = terminal["run"], terminal["jobs"]
assert run["id"] == 38910385568 and run["head_sha"] == TOOL and run["run_attempt"] == 1
assert run["event"] == "push" and run["path"] == ".github/workflows/ci.yml"
assert run["status"] == "completed" and run["conclusion"] == "success"
assert jobs["total_count"] == len(jobs["jobs"]) == 6
required = {"windows-smoke-evidence", "frontend", "backend", "backend-windows", "release-tools"}
assert {j["name"] for j in jobs["jobs"] if j["conclusion"] == "success"} == required
assert [j["name"] for j in jobs["jobs"] if j["conclusion"] == "skipped"] == ["build"]
assert all(j["status"] == "completed" and j["head_sha"] == TOOL for j in jobs["jobs"])
admission = read("original/ci/admission.json")
assert admission["source"] == TOOL and admission["runId"] == run["id"] and admission["wholeToolCiAdmitted"]
assert not admission["sameSourceAsProductCandidate"] and not admission["fullProductGateAccepted"]
counts = {}
for row in admission["requiredJobLogs"]:
    data = (ROOT / "original/ci" / row["path"]).read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    clean = text(data)
    assert checkout(clean) == row["actualCheckout"] == TOOL
    job, = [j for j in jobs["jobs"] if j["id"] == row["jobId"]]
    assert job["name"] == row["job"] and job["conclusion"] == "success"
    results = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", clean)
    if results:
        assert all(int(r[1]) == 0 for r in results)
        counts[job["name"]] = {"groups": len(results), "passed": sum(int(r[0]) for r in results),
                               "ignored": sum(int(r[2]) for r in results)}
    if job["name"] == "frontend":
        assert re.search(r"Test Files\s+99 passed \(99\)", clean)
        assert re.search(r"\bTests\s+535 passed \(535\)", clean)
assert counts == {"backend": {"groups": 35, "passed": 923, "ignored": 6},
                  "backend-windows": {"groups": 35, "passed": 648, "ignored": 7}}
baseline_run = read("original/baseline-build/release-run.original.json")
baseline_jobs = read("original/baseline-build/release-jobs.original.json")
asset = read("original/baseline-build/release-asset.original.json")
analysis = read("original/baseline-build/build-version-analysis.json")
baseline_data = (ROOT / "original/baseline-build/baseline-release-windows.original.log").read_bytes()
baseline_text = text(baseline_data)
assert baseline_run["id"] == 37871899116 and baseline_run["head_sha"] == BASELINE
assert baseline_run["run_attempt"] == 1 and baseline_run["conclusion"] == "success"
assert baseline_run["event"] == "push" and baseline_run["head_branch"] == "v0.1.9"
job, = [j for j in baseline_jobs["jobs"] if "windows" in j["name"].lower()]
assert job["id"] == analysis["baselineJobId"] and job["head_sha"] == BASELINE
assert job["run_id"] == baseline_run["id"] and job["conclusion"] == "success"
assert asset["id"] == analysis["baselineAssetId"] == 623593980
assert asset["digest"] == "sha256:" + analysis["baselineAssetSha256"]
assert analysis["baselineAssetSha256"] == "b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a"
assert job["started_at"] <= asset["created_at"] <= job["completed_at"]
assert asset["name"] == analysis["baselineApiAssetName"] == "OpenCodeX.Desktop_0.1.9_x64_en-US.msi"
assert "Uploading " + analysis["baselineUploadLogFilename"] + "..." in baseline_text
assert not analysis["apiAssetAndUploadLogFilenamesIdentical"]
assert checkout(baseline_text) == BASELINE and sha(baseline_data) == analysis["baselineLogSha256"]
candidate_path = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-B4-CANDIDATE/original/candidate-package.original.log"
candidate_data = candidate_path.read_bytes()
assert sha(candidate_data) == analysis["candidateLogSha256"] and checkout(text(candidate_data)) == CANDIDATE
baseline_versions, candidate_versions = compiled(baseline_text), compiled(text(candidate_data))
for name, row in analysis["coreCompiledVersionComparison"].items():
    assert sorted(baseline_versions[name]) == row["baselineObserved"]
    assert sorted(candidate_versions[name]) == row["candidateObserved"]
    assert row["sameVersion"] and baseline_versions[name] == candidate_versions[name]
assert len(analysis["coreCompiledVersionComparison"]) == 7
assert analysis["nativeUpdaterVersions"] == {"baseline": ["2.13.2"], "candidate": ["2.13.1"]}
assert not analysis["matchingVersionsProveSameFeaturesCompilerOrBinaries"]
assert not analysis["actualBaselineWebViewChildImageCaptured"] and not analysis["historicalChildImageBackfilled"]
assert not analysis["individualGlAllocationAttributed"] and not analysis["originalRssFailureCleared"]
cleanup = read("original/baseline-build/cleanup-followup.original.json")
assert cleanup["nativeCount"] == 1 and cleanup["nativeProcesses"][0]["ProcessId"] == 76372
assert not cleanup["fixtureRestored"] and not cleanup["restoreAttempted"]
assert cleanup["originalDirectory"]["fileId"] == "0ebd4e59-001000000005cf4b"
assert cleanup["originalDirectory"]["creationFileTime"] == "134366003622018367"
assert cleanup["currentDirectory"]["fileId"] == "0ebd4e59-00090000000d6120"
assert cleanup["originalAndLateExitTimeoutResultsUnchanged"] and cleanup["normalCandidateExitCodeStillUnknown"]
assert not cleanup["newProductStarted"] and not cleanup["foregroundInputSent"]
assert cleanup["taskFileSha256"] == sha((PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").read_bytes())
if (ROOT / "manifest.json").exists():
    for row in read("manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
print(json.dumps({"status": "verified-tool-ci-and-original-build-versions", "toolSource": TOOL,
                  "toolCiRunId": run["id"], "requiredJobsPassed": 5, "frontendTests": 535,
                  "rust": counts, "coreGraphicsPackagesWithSameObservedVersion": 7,
                  "allDependenciesEqual": False, "baselineChildImageCaptured": False,
                  "rssFailureCleared": False, "fixtureRestoredAtFollowup": False,
                  "fullProductGateAccepted": False, "releaseAccepted": False}, indent=2))
