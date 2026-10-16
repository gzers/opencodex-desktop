"""Compare observed original build versions, not untracked baseline lock guesses."""
import hashlib
import json
import re
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DOCS = Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop")
CODE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")
BASELINE = "de6862d40389d527527b9db8ef52e186a58b625f"
CANDIDATE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
CANDIDATE_EVIDENCE = DOCS / ".adg/evidence/OCX-0110-20261016-WIN-B4-CANDIDATE"
CANDIDATE_LOG = CANDIDATE_EVIDENCE / "original/candidate-package.original.log"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def clean(data):
    return re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))


def versions(text):
    result = {}
    for name, version in re.findall(r"\bCompiling\s+([a-zA-Z0-9_-]+)\s+v([0-9][0-9A-Za-z.+_-]*)", text):
        if name.startswith(("tauri", "wry", "tao", "webview2", "windows")):
            result.setdefault(name, set()).add(version)
    return {k: sorted(v) for k, v in sorted(result.items())}


baseline_bytes = (ROOT / "baseline-release-windows.original.log").read_bytes()
candidate_bytes = CANDIDATE_LOG.read_bytes()
baseline_text, candidate_text = clean(baseline_bytes), clean(candidate_bytes)
manifest = read(CANDIDATE_EVIDENCE / "manifest.json")
row, = [r for r in manifest["files"] if r["path"] == "original/candidate-package.original.log"]
assert row["bytes"] == len(candidate_bytes) and row["sha256"] == sha(candidate_bytes)
run = read(ROOT / "release-run.original.json")
jobs = read(ROOT / "release-jobs.original.json")
asset = read(ROOT / "release-asset.original.json")
job, = [j for j in jobs["jobs"] if "windows" in j["name"].lower()]
assert run["head_sha"] == job["head_sha"] == BASELINE
assert run["id"] == job["run_id"] == 37871899116 and run["run_attempt"] == job["run_attempt"] == 1
assert run["event"] == "push" and run["head_branch"] == "v0.1.9"
assert run["path"] == ".github/workflows/release.yml"
assert run["conclusion"] == job["conclusion"] == "success"
assert job["started_at"] <= asset["created_at"] <= job["completed_at"]
assert asset["id"] == 623593980 and asset["name"] == "OpenCodeX.Desktop_0.1.9_x64_en-US.msi"
assert "Uploading OpenCodeX Desktop_0.1.9_x64_en-US.msi..." in baseline_text
assert asset["digest"] == "sha256:b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a"
for text, source in [(baseline_text, BASELINE), (candidate_text, CANDIDATE)]:
    checkout, = re.findall(r"log -1 --format=%H\s*\r?\n[^\r\n]*?\b([0-9a-f]{40})\b", text)
    assert checkout == source
baseline_versions, candidate_versions = versions(baseline_text), versions(candidate_text)
core = ["tauri", "tauri-runtime", "tauri-runtime-wry", "tao", "wry", "webview2-com", "webview2-com-sys"]
assert all(name in baseline_versions and name in candidate_versions for name in core)
lock_bytes = subprocess.check_output(["git", "show", CANDIDATE + ":apps/desktop/tauri/Cargo.lock"], cwd=CODE)
locked = tomllib.loads(lock_bytes.decode("utf-8"))
for name in core:
    expected = sorted({r["version"] for r in locked["package"] if r["name"] == name})
    assert expected == candidate_versions[name]
comparison = {name: {"baselineObserved": baseline_versions[name], "candidateObserved": candidate_versions[name],
                     "sameVersion": baseline_versions[name] == candidate_versions[name]} for name in core}
assert all(row["sameVersion"] for row in comparison.values())
value = {
    "baselineSource": BASELINE, "baselineRunId": run["id"], "baselineJobId": job["id"],
    "baselineAssetId": asset["id"], "baselineAssetSha256": asset["digest"][7:],
    "baselineApiAssetName": asset["name"],
    "baselineUploadLogFilename": "OpenCodeX Desktop_0.1.9_x64_en-US.msi",
    "apiAssetAndUploadLogFilenamesIdentical": False,
    "baselineLogSha256": sha(baseline_bytes), "candidateSource": CANDIDATE,
    "candidateLogPath": str(CANDIDATE_LOG), "candidateLogSha256": sha(candidate_bytes),
    "candidateLockSha256": sha(lock_bytes), "coreCompiledVersionComparison": comparison,
    "baselineOtherCompiledVersions": baseline_versions, "candidateOtherCompiledVersions": candidate_versions,
    "versionObservation": "original Compiling lines after ANSI removal; not a package download inference",
    "baselineCargoLockRecovered": False, "allDependencyVersionsEqual": baseline_versions == candidate_versions,
    "nativeUpdaterVersions": {"baseline": baseline_versions.get("tauri-plugin-updater"),
                             "candidate": candidate_versions.get("tauri-plugin-updater")},
    "matchingVersionsProveSameFeaturesCompilerOrBinaries": False,
    "actualBaselineWebViewChildImageCaptured": False, "historicalChildImageBackfilled": False,
    "individualGlAllocationAttributed": False, "originalRssFailureCleared": False,
    "newProductStarted": False, "uiInputInjected": False, "queryReplayed": False,
    "signingOrPublicationRequested": False,
}
with (ROOT / "build-version-analysis.json").open("xb") as out:
    out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
print(json.dumps({"coreCompiledVersionComparison": comparison,
                  "nativeUpdaterVersions": value["nativeUpdaterVersions"],
                  "allocationCauseAttributed": False}))
