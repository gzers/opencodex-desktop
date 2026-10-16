"""Prepare only; no process launch, profile move, registry or ACL mutation."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
OLD = ROOT.parent / "OCX-0110-20261016-WIN-823-FRONT-PREP"
CANDIDATE = ROOT.parent / "OCX-0110-20261016-WIN-B4-CANDIDATE"
DIAGNOSTIC = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop")
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
IMAGE = "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db"
assert not (ROOT / "contract.json").exists(), "Do not overwrite preparation"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(name, value):
    target = ROOT / name
    assert not target.exists(), name
    target.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


def change(text, old, new, count=1):
    assert text.count(old) == count, old
    return text.replace(old, new)


contract = json.loads((OLD / "contract.json").read_text(encoding="utf-8-sig"))
descriptor = json.loads((CANDIDATE / "original/candidate-descriptor.json").read_text(encoding="utf-8-sig"))
assert descriptor["source"] == SOURCE and descriptor["sha256"] == IMAGE
assert sha(Path(descriptor["executable"])) == IMAGE
diagnostic_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=DIAGNOSTIC).decode().strip()
assert diagnostic_head == "768e231811a2363b32f5fa67c8e74cbc31d751e1"
for relative, expected in contract["methodWorkingFileSha256"].items():
    assert sha(DIAGNOSTIC / relative) == expected
readback = json.loads((ROOT / "preparation-readback.json").read_text(encoding="utf-8-sig"))
assert readback["nativeCount"] == 0 and not readback["productStarted"]
assert readback["fixtureReadback"][0]["preferencesSha256"] == "2d612148a55152e05047d1f13c4587a3b75f2740b1a51cc2c0f64f30a767495e"
assert all(row["runtime"]["source"] == "unresolved" and not row["runtime"]["path"]
           for row in readback["fixtureReadback"])
for key in list(contract):
    if key.startswith("recovery"):
        del contract[key]
contract.update({
    "candidateSource": SOURCE, "candidateExecutable": descriptor["executable"],
    "candidateExeSha256": IMAGE, "candidateVerificationPath": str(CANDIDATE / "verification.json"),
    "candidateVerificationSha256": sha(CANDIDATE / "verification.json"),
    "candidateManifestPath": str(CANDIDATE / "manifest.json"),
    "candidateManifestSha256": sha(CANDIDATE / "manifest.json"),
    "actualCollectorWorkspaceHead": diagnostic_head,
    "humanWindowScope": "one-single-instance-foreground-idle-pair",
    "minimumWholePairWindowSeconds": 1500,
    "rolesMethodPath": str(DIAGNOSTIC / "test/smoke/windows-gpu-resource.ps1"),
    "rolesMethodSha256": sha(DIAGNOSTIC / "test/smoke/windows-gpu-resource.ps1"),
    "rolesMethodSource": diagnostic_head,
    "directoryIdentityMethod": str(ROOT.parent / "OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP/directory-identity.ps1"),
    "directoryIdentityMethodSha256": sha(ROOT.parent / "OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP/directory-identity.ps1"),
    "preparationReadbackSha256": sha(ROOT / "preparation-readback.json"),
    "baselineOriginalDirectoryId": readback["fixtureReadback"][1]["nativeDirectory"]["fileId"],
    "baselineOriginalDirectoryCreation": readback["fixtureReadback"][1]["nativeDirectory"]["creationFileTime"],
    "recoveryIncludedInThisWindow": False, "dpiAcceptanceRequired": False,
    "formalPairAllowedOnlyBecauseProductChanged": True,
})
write("contract.json", contract)
origins = []
for name in ["run-binding.ps1", "prepare-fixture.ps1", "launch.ps1", "observe.ps1",
             "retained-process-times.ps1", "analyze-pair.py"]:
    origin = OLD / name
    text = origin.read_text(encoding="utf-8-sig")
    if name == "run-binding.ps1":
        text = "# Preparation is not authorization; create a receipt only after a fresh human signal.\n" + text[text.index("function Assert-PreparedIdleRun"):]
        text = change(text, "verified-exact-823-candidate", "verified-exact-b4-candidate")
        text = change(text, "one-single-instance-foreground-recovery-and-idle-pair",
                      contract["humanWindowScope"])
        text = change(text, "    if($RequireCurrentHumanWindow){",
                      """    if((Get-FileHash -LiteralPath $taskContract.directoryIdentityMethod).Hash.ToLowerInvariant() -cne $taskContract.directoryIdentityMethodSha256){throw 'Directory identity method changed'}
    if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'preparation-readback.json')).Hash.ToLowerInvariant() -cne $taskContract.preparationReadbackSha256){throw 'Fixture preparation binding changed'}
    if($RequireCurrentHumanWindow){""")
    elif name == "launch.ps1":
        text = change(text, "'f2271f361b9da562b62c71afcdbeb15e61d183e2'", "'" + diagnostic_head + "'")
        text = change(text, "OCX-0110-20261016-WIN-823-CANDIDATE/candidate-descriptor.json",
                      "OCX-0110-20261016-WIN-B4-CANDIDATE/original/candidate-descriptor.json")
        text = change(text, "collectorSource=$taskSource",
                      "collectorSource=$taskSource;collectorMethodSource=$taskContract.collectorSource")
    elif name == "prepare-fixture.ps1":
        text = change(text, "if(Test-Path -LiteralPath (Join-Path $taskRun 'fixture-ready.json'))",
                      """$taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw | ConvertFrom-Json
if((([datetimeoffset]$taskHuman.expiresAtUtc)-[datetimeoffset]::UtcNow).TotalSeconds -lt $taskContract.minimumWholePairWindowSeconds){throw 'Insufficient remaining window for the full pair; do not start a partial attempt'}
if(Test-Path -LiteralPath (Join-Path $taskRun 'fixture-ready.json'))""")
        text = change(text, "# Both resolved absolute endpoints have been checked before this recursive move.",
                      """. $taskContract.directoryIdentityMethod
$taskOriginalDirectory=Get-RecoveryDirectoryIdentity -Path $taskBaseline
if($taskOriginalDirectory.fileId -cne $taskContract.baselineOriginalDirectoryId -or
   $taskOriginalDirectory.creationFileTime -cne $taskContract.baselineOriginalDirectoryCreation){throw 'Original baseline profile generation changed'}
# Both resolved absolute endpoints have been checked before this recursive move.""")
        text = change(text, "$taskCloneHash=", "$taskCloneDirectory=Get-RecoveryDirectoryIdentity -Path $taskBaseline\n$taskCloneHash=")
        text = change(text, "fixturePreferencesSha256=$taskCloneHash;",
                      "originalDirectory=$taskOriginalDirectory;cloneDirectory=$taskCloneDirectory;fixturePreferencesSha256=$taskCloneHash;")
    elif name == "analyze-pair.py":
        text = change(text, '"823a8a0615153068594daef530c2e80f1d95fc29"', '"' + SOURCE + '"')
        text = text.replace("019/823", "019/b4")
    target = ROOT / name
    assert not target.exists()
    target.write_bytes(origin.read_bytes() if name == "retained-process-times.ps1" else text.encode("utf-8"))
    origins.append({"path": name, "origin": str(origin), "originSha256": sha(origin),
                    "preparedSha256": sha(target)})
write("method-origins.json", origins)
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
print(json.dumps({"source": SOURCE, "methodsPrepared": len(origins), "productStarted": False,
                  "profileMoved": False, "foregroundGranted": False}))
