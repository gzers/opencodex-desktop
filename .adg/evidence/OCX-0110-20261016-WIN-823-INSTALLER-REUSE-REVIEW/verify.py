"""Verify read-only MSI table comparison; no installer or application execution."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
OLD_SOURCE = "43142bdbcc7a0b9f1325103febae3f44dce407ab"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


files = {p.relative_to(ROOT).as_posix(): p for p in ROOT.rglob("*")
         if p.is_file() and p.name not in {"manifest.json", "verification.json"}}
manifest = read("manifest.json")
assert set(files) == {row["path"] for row in manifest["files"]}
for row in manifest["files"]:
    assert files[row["path"]].stat().st_size == row["bytes"]
    assert sha(files[row["path"]]) == row["sha256"]
tables = read("msi-tables.json")
packages = {row["label"]: row for row in tables["packages"]}
assert {label: row["source"] for label, row in packages.items()} == {
    "431": OLD_SOURCE, "823": SOURCE}
assert packages["431"]["databaseMode"] == packages["823"]["databaseMode"] == "read-only"
assert packages["431"]["packageSha256"] == "2dc08092319fb91fc2866d010de850ca23bcc7716232ef2ac212b94c4d118dbe"
assert packages["823"]["packageSha256"] == "e64491715bc5e510d06eb8a18f723b6700be800e4555a44c4e2207059bf0318c"
diff = read("table-diff.json")
for table in ("Directory", "RemoveFile", "CustomAction", "InstallExecuteSequence",
              "Upgrade", "Registry", "RemoveRegistry", "CreateFolder",
              "ServiceInstall", "ServiceControl"):
    assert diff[table]["equal"]
assert diff["Property"]["431Count"] == diff["Property"]["823Count"] == 19
assert diff["File"]["431Count"] == diff["File"]["823Count"] == 2
assert diff["Property"]["only431"] and diff["Property"]["only823"]
assert diff["Property"]["only431"][0].find("ProductCode") >= 0
assert diff["Property"]["only823"][0].find("ProductCode") >= 0
assert diff["File"]["only431"] and diff["File"]["only823"]
component = {row["label"]: {
    row["Component"]: row["ComponentId"] for row in row["tables"]["Component"]
} for row in tables["packages"]}
assert component["431"]["Path"] == component["823"]["Path"]
assert component["431"]["ocxd"] != component["823"]["ocxd"]
retention = json.loads(
    (ROOT.parent / "OCX-0110-20261015-WIN-INSTALLER-RETENTION/verification.json")
    .read_text(encoding="utf-8-sig"))
assert retention["bothInstallerUpgradeAndDefaultUninstallFixtureRetentionPassed"]
assert not retention["hostInstallerExecuted"] and not retention["applicationRootResolutionOrOrdinaryPermissionsAccepted"]
assert retention["fixedCandidateSource"] == OLD_SOURCE
descriptor = json.loads(
    (ROOT.parent / "OCX-0110-20261016-WIN-823-CANDIDATE/candidate-descriptor.json")
    .read_text(encoding="utf-8-sig"))
assert descriptor["source"] == SOURCE and not descriptor["installerExecuted"]
assert not descriptor["productStarted"]
for path, expected in {
    "apps/desktop/tauri/tauri.windows.conf.json": None,
    "test/smoke/windows-bundle-identity.mjs": None,
}.items():
    blob = subprocess.run(["git", "-C", str(CODE), "show", SOURCE + ":" + path],
                          check=True, capture_output=True).stdout
    assert blob
summary = {
    "status": "verified-readonly-msi-reuse-boundary",
    "files": len(files),
    "oldSource": OLD_SOURCE,
    "candidateSource": SOURCE,
    "sameUpgradeAndRemovalStructure": True,
    "sameCustomActionAndSequenceStructure": True,
    "sameDirectoryAndRegistryStructure": True,
    "productCodeDiffExpected": True,
    "payloadFileSizeDiffExpected": True,
    "ocxdComponentGuidDiffObserved": True,
    "oldRetentionEvidenceReusableFor": [
        "installer upgrade sequencing",
        "default uninstall preserving seeded files",
        "reinstall after uninstall",
    ],
    "oldRetentionEvidenceDoesNotProve": [
        "823 host installation",
        "823 ordinary/admin token",
        "823 startup root binding",
        "823 native UI or performance",
    ],
    "readOnlyDatabaseMode": True,
    "hostInstallerExecuted": False,
    "hostProductStarted": False,
    "fullInstallerGatePassed": False,
    "releaseAccepted": False,
}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
