"""Recompute selected-table facts from originals, without running installers."""
import collections
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REVIEW = ROOT.parent / "OCX-0110-20261016-WIN-823-INSTALLER-REUSE-REVIEW"
RETENTION = ROOT.parent / "OCX-0110-20261015-WIN-INSTALLER-RETENTION"
CANDIDATE = ROOT.parent / "OCX-0110-20261016-WIN-823-CANDIDATE"
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
OLD = "43142bdbcc7a0b9f1325103febae3f44dce407ab"
NEW = "823a8a0615153068594daef530c2e80f1d95fc29"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blob(source, path):
    return subprocess.run(
        ["git", "-c", "core.longpaths=true", "-C", str(CODE), "show", source + ":" + path],
        check=True, capture_output=True).stdout


def rows_key(rows):
    return collections.Counter(json.dumps(r, sort_keys=True) for r in rows)


def indexed(rows, column):
    result = {row[column]: row for row in rows}
    assert len(result) == len(rows), "Duplicate row identity"
    return result


manifest = read(REVIEW / "manifest.json")
for item in manifest["files"]:
    path = REVIEW / item["path"]
    assert path.stat().st_size == item["bytes"] and digest(path) == item["sha256"]
raw = read(REVIEW / "msi-tables.json")
packages = {p["label"]: p for p in raw["packages"]}
assert len(packages) == 2 and packages["431"]["source"] == OLD and packages["823"]["source"] == NEW
for package in packages.values():
    assert package["databaseMode"] == "read-only"
    assert digest(Path(package["package"])) == package["packageSha256"]
assert not raw["installerExecuted"] and not raw["productStarted"]
assert not raw["registryWritten"] and not raw["hostInstallerGatePassed"]
old, new = packages["431"]["tables"], packages["823"]["tables"]
summary_diff = read(REVIEW / "table-diff.json")
equal_tables = []
for name in old:
    assert name in new
    assert summary_diff[name]["431Count"] == len(old[name])
    assert summary_diff[name]["823Count"] == len(new[name])
    assert summary_diff[name]["equal"] == (old[name] == new[name])
    if rows_key(old[name]) == rows_key(new[name]):
        equal_tables.append(name)
assert set(equal_tables) == {
    "Directory", "RemoveFile", "CustomAction", "InstallExecuteSequence", "Upgrade",
    "Registry", "RemoveRegistry", "CreateFolder", "ServiceInstall", "ServiceControl"}
properties = [indexed(t["Property"], "Property") for t in (old, new)]
assert set(properties[0]) == set(properties[1])
assert [k for k in properties[0] if properties[0][k] != properties[1][k]] == ["ProductCode"]
files = [indexed(t["File"], "File") for t in (old, new)]
assert set(files[0]) == set(files[1]) == {"Path", "Bin_ocxd"}
file_changes = {}
for key in files[0]:
    fields = [k for k in files[0][key] if files[0][key][k] != files[1][key][k]]
    assert fields == ["FileSize"]
    file_changes[key] = {k: [files[0][key][k], files[1][key][k]] for k in fields}
components = [indexed(t["Component"], "Component") for t in (old, new)]
assert set(components[0]) == set(components[1]) and len(components[0]) == 6
component_changes = {}
for key in components[0]:
    fields = [k for k in components[0][key] if components[0][key][k] != components[1][key][k]]
    if fields:
        component_changes[key] = fields
assert component_changes == {"ocxd": ["ComponentId"]}
input_hashes = {}
for path in (
    "apps/desktop/tauri/tauri.conf.json", "apps/desktop/tauri/tauri.windows.conf.json",
    "test/smoke/windows-installer.ps1", "test/smoke/windows-bundle-identity.mjs"):
    before, after = blob(OLD, path), blob(NEW, path)
    assert before == after
    input_hashes[path] = hashlib.sha256(after).hexdigest()
workflow_path = ".github/workflows/windows-candidate.yml"
before, after = blob(OLD, workflow_path), blob(NEW, workflow_path)
package_sections = [data.split(b"\n  package:\n", 1)[1].split(b"\n  query-etw:", 1)[0]
                    for data in (before, after)]
assert package_sections[0] == package_sections[1]
raw_references = {}
retained = {}
for kind in ("msi", "nsis"):
    directory = RETENTION / kind / "original"
    admission_path, = directory.rglob("admission.json")
    result_path, = directory.rglob("retention-result.json")
    admission, result = read(admission_path), read(result_path)
    assert admission["candidateSource"] == result["candidateSource"] == OLD
    assert admission["candidateRun"] == 38709701621
    assert result["result"] == "pass" and not result["productLaunched"]
    assert not result["hostMachineAcceptanceClaimed"] and not result["explicitDataPurgeTested"]
    installs = [s for s in result["steps"] if "exitCode" in s]
    phases = [s for s in result["steps"] if "files" in s]
    assert len(installs) == len(phases) == 5
    assert all(step["exitCode"] == 0 for step in installs)
    assert all(step["allBytesUnchanged"] and len(step["files"]) == 6 for step in phases)
    assert all(phase["files"] == phases[0]["files"] for phase in phases)
    if kind == "msi":
        assert admission["candidateSha256"] == packages["431"]["packageSha256"]
    retained[kind] = {"actualTestedSource": OLD, "installerSteps": 5,
                      "retainedSeededFiles": 6, "readbackPhases": 5,
                      "productLaunched": False, "customRootWasSeededSibling": True}
    for path in (admission_path, result_path):
        raw_references[path.relative_to(ROOT.parent).as_posix()] = digest(path)
descriptor = read(CANDIDATE / "candidate-descriptor.json")
candidate_msi = read(CANDIDATE / "msi-readonly.original.json")
assert descriptor["source"] == NEW and descriptor["runId"] == 38841117656
assert packages["823"]["packageSha256"] == candidate_msi["packageSha256"]
protect = read(REVIEW / "protection.json")
assert protect["nativeCount"] == 0 and not protect["hostInstallerExecuted"]
assert protect["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
report = {
    "status": "crosschecked-original-selected-msi-tables",
    "reviewManifestSha256": digest(REVIEW / "manifest.json"),
    "checkerSha256": digest(Path(__file__)),
    "oldSource": OLD, "candidateSource": NEW,
    "originalTablesRecomputed": True, "packageBytesReverified": True,
    "equalSelectedTables": sorted(equal_tables), "differentProperty": "ProductCode",
    "fileChanges": file_changes, "componentChanges": component_changes,
    "identicalSourceInputs": input_hashes,
    "packageWorkflowSectionSha256": hashlib.sha256(package_sections[0]).hexdigest(),
    "retentionRunId": 38716371739, "retainedHistoricalResults": retained,
    "historicalRawReferences": raw_references,
    "scope": "Selected MSI fields and source inputs; historical tests retain their original source.",
    "msiBinaryCustomActionStreamsCompared": False,
    "nsisBinaryStructureCompared": False,
    "new823UpgradeOrRetentionExecutionClaimed": False,
    "nativeApplicationRootOrPermissionsProven": False,
    "hostInstallerExecuted": False, "productStarted": False,
    "fullInstallerGatePassed": False, "historicalFailuresCleared": False,
    "releaseAccepted": False,
}
target = ROOT / "crosscheck.json"
if target.exists():
    assert read(target) == report, "Preserve the original crosscheck result"
else:
    with target.open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(json.dumps(report, indent=2) + "\n")
print(json.dumps({k: v for k, v in report.items()
                  if k not in {"historicalRawReferences", "identicalSourceInputs", "fileChanges"}}))
