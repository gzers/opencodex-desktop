"""Prepare a self-contained offline fixture; no application or installer runs."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CODE = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
SOURCE = "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
KIT = CODE / "test/software/acceptance-kit-02a0d4-20261015"
read = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
digest = lambda b: hashlib.sha256(b).hexdigest()

def check_file(path, size, sha):
    assert path.is_file() and not path.is_symlink() and not path.is_junction()
    raw = path.read_bytes()
    assert len(raw) == size and digest(raw) == sha, path.name
    return raw

assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=CODE).decode().strip() == SOURCE
assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=CODE).strip()
assert not KIT.exists()
assert KIT.resolve().is_relative_to((CODE / "test/software").resolve())
for ancestor in KIT.parent.parents:
    assert not ancestor.is_symlink() and not ancestor.is_junction()
runtime = read(ROOT / "runtime-source.original.json")
assert runtime["signatureStatus"] == "Valid" and "Microsoft Corporation" in runtime["signerSubject"]
assert runtime["source"] == SOURCE
pshome = Path(runtime["runtimeSource"])
assert digest((pshome / "pwsh.exe").read_bytes()) == runtime["executableSha256"]
baseline_proof = ROOT.parent / "OCX-0110-20261015-WIN-FG-TRAY-ADMIN/baseline-git-proof-20261015"
for row in read(baseline_proof / "manifest.json"):
    check_file(baseline_proof / row["path"], row["bytes"], row["sha256"])
baseline = read(CODE / "test/software/baseline019-v2/msi-readonly.json")
release = read(baseline_proof / "release.original.json")
assert release["tag_name"] == "v0.1.9" and not release["draft"]
asset, = [row for row in release["assets"] if row["name"] == Path(baseline["package"]).name]
assert asset["digest"] == "sha256:" + baseline["packageSha256"]
check_file(Path(baseline["package"]), asset["size"], baseline["packageSha256"])
assert baseline["result"] == "pass" and baseline["properties"]["ProductVersion"] == "0.1.9"
assert not baseline["installerExecuted"]
candidate_root = ROOT.parent / "OCX-0110-20261015-WIN-02-CANDIDATE"
candidate = read(candidate_root / "msi-readonly.original.json")
identity = read(candidate_root / "candidate-payload-identity.json")
assert identity["source"] == SOURCE and identity["runId"] == 38737628201
assert candidate["result"] == "pass" and not candidate["installerExecuted"]
assert candidate["properties"]["ProductVersion"] == "0.1.10"
expected_main = {
    "baseline019": "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608",
    "candidate0110": "7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9",
}
KIT.mkdir()
rows = []

def put(relative, raw):
    path = KIT / relative
    assert path.resolve().is_relative_to(KIT.resolve())
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as stream:
        stream.write(raw)
    assert path.read_bytes() == raw
    rows.append({"path": relative, "bytes": len(raw), "sha256": digest(raw)})

payloads = {}
for label, receipt in (("baseline019", baseline), ("candidate0110", candidate)):
    assert {row["name"] for row in receipt["extractedExecutables"]} == {"opencodex-desktop.exe", "ocxd.exe"}
    payloads[label] = []
    for row in receipt["extractedExecutables"]:
        raw = check_file(Path(row["path"]), row["bytes"], row["sha256"])
        if row["name"] == "opencodex-desktop.exe":
            assert row["sha256"] == expected_main[label]
        relative = f"bin/{label}/{row['name']}"
        put(relative, raw)
        payloads[label].append({"path": relative, "sha256": row["sha256"],
                                "bytes": row["bytes"], "version": row["productVersion"]})
methods = {
    "windows-process-token.ps1": CODE / "test/smoke/windows-process-token.ps1",
    "windows-acceptance-profile.ps1": CODE / "test/smoke/windows-acceptance-profile.ps1",
    "process-user-sid.ps1": ROOT.parent / "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1",
    "inspect-profile.ps1": ROOT / "methods/inspect-profile.ps1",
}
for name, path in methods.items():
    put("methods/" + name, path.read_bytes())
excluded = set(runtime["excludedRelativePaths"])
runtime_count = 0
for path in sorted(pshome.rglob("*")):
    assert not path.is_symlink() and not path.is_junction(), "Runtime aliases must not enter the kit"
    if not path.is_file():
        continue
    relative = path.relative_to(pshome).as_posix()
    if relative in excluded or relative.startswith("preview/"):
        continue
    assert not path.name.lower().endswith("profile.ps1") and path.name.lower() != "powershell.config.json"
    put("runtime/powershell/" + relative, path.read_bytes())
    runtime_count += 1
manifest = {
    "schemaVersion": 1, "source": SOURCE, "baselineSource": "de6862d40389d527527b9db8ef52e186a58b625f",
    "candidateRun": 38737628201, "hostDailySidSha256": runtime["hostDailySidSha256"],
    "expectedOrdinaryAccountName": "ocx0110_accept", "files": sorted(rows, key=lambda row: row["path"]),
    "runtimeVersion": runtime["version"], "payloads": payloads,
    "baselineReleaseAssetId": asset["id"], "baselinePackageSha256": baseline["packageSha256"],
    "budgets": {
        "startupSamplesPerVersionPerColdAndWarmModeMin": 20,
        "startupP95IncreaseMsMax": 100, "startupP95IncreasePercentMax": 5,
        "idleSecondsPerVersionMin": 600, "cpuIncreasePercentagePointsMax": 0.5,
        "rssIncreaseMiBMax": 10, "mainThreadTaskMsMax": 50,
        "validCacheRoutes": 100, "validCacheRestarts": 10, "automaticQueriesAndQueryChildrenMax": 0,
        "realStablePeriodHours": 24, "realBetaPeriodHours": 6,
    },
    "preflightOnly": True, "launchPermissionGranted": False,
    "profileApprovalAndFirstLoginRequired": True, "foregroundApprovalRequired": True,
    "coldWarmDefinitionsStillRequireOriginalMethodBinding": True,
    "installerOrPerformanceAcceptanceClaimed": False,
}
raw = (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode()
with (KIT / "kit-manifest.json").open("xb") as stream:
    stream.write(raw)
receipt = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "kitPath": str(KIT), "kitManifestSha256": digest(raw), "kitFiles": len(rows),
    "kitBytes": sum(row["bytes"] for row in rows), "runtimeFilesCopied": runtime_count,
    "runtimeProfileAndConfigurationFilesExcluded": True,
    "payloads": payloads, "offlineBytesVerified": True,
    "sourceEvidence": {
        "official019": str(baseline_proof.relative_to(ROOT.parent)),
        "candidate0110": candidate_root.name,
    },
    "productStarted": False, "installerExecuted": False, "accountOrProfileCreated": False,
    "systemPathOrAclOrKnownFoldersChanged": False, "kitTransferredToTestProfile": False,
    "performanceMeasured": False, "launchPermissionGranted": False, "fullVersionGatePassed": False,
}
target = ROOT / "kit-build.original.json"
assert not target.exists()
target.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": SOURCE, "kitFiles": len(rows), "kitBytes": receipt["kitBytes"],
                  "runtimeFilesCopied": runtime_count, "productStarted": False,
                  "kitManifestSha256": receipt["kitManifestSha256"]}))
