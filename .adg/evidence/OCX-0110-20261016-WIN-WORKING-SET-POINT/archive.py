"""Archive diagnostics as new evidence; leave the earlier failed pair frozen."""
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop")
RUNTIME = CODE / "test/runtime/working-set-gpu-20261016T2145"
SOURCE = "3a2ff08a6b480d237e93502626861090aa14423d"
assert not (ROOT / "original-copies.json").exists(), "Do not replace frozen evidence"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(name, value):
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


copies = []


def copy(path, relative):
    data = path.read_bytes()
    if path.suffix != ".zip":
        text = data.decode("utf-8-sig")
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", text), path.name
        assert not re.search(r"\bgh[pousr]_[A-Za-z0-9]{20,}", text), path.name
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": relative, "runtimePath": str(path),
                   "bytes": len(data), "sha256": sha(data)})


for path in sorted(RUNTIME.iterdir()):
    if path.is_file() and path.suffix in {".json", ".log", ".zip", ".py", ".ps1"}:
        copy(path, "original/" + path.name)
for stamp in ["20261016T214156", "20261016T214301"]:
    local = CODE / ("test/runtime/working-set-validation-" + stamp)
    for path in sorted(local.iterdir()):
        assert path.suffix == ".json", path.name
        copy(path, "original/local-" + stamp + "/" + path.name)
    copy(local.with_name(local.name + ".original.log"),
         "original/local-" + stamp + "/harness.original.log")
write("original-copies.json", copies)
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=CODE).decode().strip() == SOURCE
files = []
for name in ["test/smoke/windows-working-set.ps1", "test/smoke/windows-working-set.test.ps1",
             ".github/workflows/ci.yml"]:
    blob = subprocess.check_output(["git", "show", SOURCE + ":" + name], cwd=CODE)
    current = (CODE / name).read_bytes()
    assert current in (blob, blob.replace(b"\n", b"\r\n")), name
    files.append({"path": name, "gitLfSha256": sha(blob),
                  "gitCheckoutCrLfSha256": sha(blob.replace(b"\n", b"\r\n")),
                  "workingSha256": sha(current)})
write("source-identity.json", {
    "source": SOURCE, "branch": "feature/0.1.10-windows-working-set-pages",
    "workspace": str(CODE), "sourceFiles": files,
    "productCandidateSource": "b4d487e4e04cd4c80fd5570663e1ed21dd483b19",
    "toolSourceIsDifferentFromProductCandidate": True,
})
prior = ROOT.parent / "OCX-0110-20261016-WIN-B4-IDLE-ROLES"
write("prior-evidence-binding.json", {
    "directory": prior.relative_to(PROJECT).as_posix(),
    "manifestSha256": sha((prior / "manifest.json").read_bytes()),
    "verificationSha256": sha((prior / "verification.json").read_bytes()),
    "scopeSha256": sha((prior / "scope.json").read_bytes()),
    "priorFailuresPreserved": True,
})
snapshot = json.loads((RUNTIME / "gpu-working-set.original.json").read_text(encoding="utf-8-sig"))["snapshot"]
regions = {}
shareable = 0
for row in snapshot["Buckets"]:
    regions[row["RegionType"]] = regions.get(row["RegionType"], 0) + int(row["Bytes"])
    if row["ShareableAtRecheck"] is True:
        shareable += int(row["Bytes"])
write("point-analysis.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "kind": "late-nonatomic-point-classification",
    "rootPid": 76372, "gpuPid": 17236,
    "gpuCreationUtc": "2026-10-16T13:00:18.4261052Z",
    "startedAtUtc": snapshot["StartedAtUtc"], "finishedAtUtc": snapshot["FinishedAtUtc"],
    "listedResidentPages": snapshot["ListedResidentPages"],
    "listedResidentBytes": int(snapshot["ListedResidentBytes"]),
    "regionBytes": regions,
    "regionMiB": {k: v / 1048576 for k, v in regions.items()},
    "shareableAtRecheckBytes": shareable,
    "privateCommitBytesSeparateMetric": int(snapshot["ProcessPrivateCommitAfterBytes"]),
    "processWorkingSetAfterBytes": int(snapshot["ProcessWorkingSetAfterBytes"]),
    "workingSetAfterMinusListedBytes": int(snapshot["ProcessWorkingSetAfterBytes"]) - int(snapshot["ListedResidentBytes"]),
    "currentSceneReobserved": False, "baselinePageClassificationAvailable": False,
    "counterSpanBackfilled": False, "shareableMeansDeduplicatedSharedBytes": False,
    "rssMetricReplaced": False, "individualGlAllocationAttributed": False,
    "originalRssIncreaseMiB": 20.040155203098834, "originalRssBudgetMiB": 10,
    "originalRssBudgetPassed": False, "fullPairLifecycleAdmitted": False,
    "historicalFailuresCleared": False, "fullPerformanceGatePassed": False,
    "foregroundInputSent": False, "newProductStarted": False, "queryReplayRequested": False,
})
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
print(json.dumps({"archivedOriginals": len(copies), "regionMiB": {k: v / 1048576 for k, v in regions.items()}}))
