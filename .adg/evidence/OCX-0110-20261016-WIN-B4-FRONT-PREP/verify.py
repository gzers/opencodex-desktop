"""Read-only verification of preparation; never launch a desktop process."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DIAGNOSTIC = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


contract = read("contract.json")
source = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
image = "f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db"
assert contract["candidateSource"] == source and contract["candidateExeSha256"] == image
assert sha(Path(contract["candidateExecutable"])) == image
assert sha(Path(contract["baselineExecutable"])) == contract["baselineExeSha256"] == "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"
assert contract["baselineSource"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert contract["budgetsUnchanged"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
assert (contract["settleSecondsPerVersion"], contract["minimumCompletedCounterSpanSeconds"]) == (60, 600)
assert (contract["clientWidth"], contract["clientHeight"]) == (2360, 1520)
assert (contract["theme"], contract["visualEffects"], contract["glowRender"], contract["interfaceScale"]) == ("light", "high", "mesh", 100)
assert contract["humanWindowScope"] == "one-single-instance-foreground-idle-pair"
assert contract["minimumWholePairWindowSeconds"] == 1500
assert not contract["recoveryIncludedInThisWindow"] and not contract["dpiAcceptanceRequired"]
assert contract["collectorSource"] == "f2271f361b9da562b62c71afcdbeb15e61d183e2"
assert contract["actualCollectorWorkspaceHead"] == "768e231811a2363b32f5fa67c8e74cbc31d751e1"
for relative, expected in contract["methodWorkingFileSha256"].items():
    assert sha(DIAGNOSTIC / relative) == expected
for path_key, digest_key in [
    ("candidateVerificationPath", "candidateVerificationSha256"),
    ("candidateManifestPath", "candidateManifestSha256"),
    ("sidMethodPath", "sidMethodSha256"),
    ("rolesMethodPath", "rolesMethodSha256"),
    ("directoryIdentityMethod", "directoryIdentityMethodSha256"),
]:
    assert sha(Path(contract[path_key])) == contract[digest_key], path_key
assert sha(ROOT / "retained-process-times.ps1") == contract["kernelTimesMethodSha256"]
assert sha(ROOT / "preparation-readback.json") == contract["preparationReadbackSha256"]
for origin in read("method-origins.json"):
    assert sha(Path(origin["origin"])) == origin["originSha256"]
    assert sha(ROOT / origin["path"]) == origin["preparedSha256"]
syntax = read("all-syntax-checks.json")
assert len(syntax) == 9 and all(row["syntaxPassed"] and not row["errors"] for row in syntax)
guards = read("checks/guard-checks.json")
assert len(guards["cases"]) == 4
assert {c["entry"] for c in guards["cases"]} == {"prepare-fixture.ps1", "launch.ps1", "observe.ps1", "capture-roles.ps1"}
assert guards["beforeAfterProtectionEqual"] and guards["nativeCount"] == 0
assert not any(guards[k] for k in ["humanReceiptCreated", "productStarted",
                                  "configurationMoved", "registryOrAclChanged", "fullNativeGatePassed"])
for case in guards["cases"]:
    log = ROOT / "checks" / case["log"]
    assert case["exitCode"] != 0 and case["missingHumanReceiptRefused"]
    assert sha(log) == case["logSha256"]
    assert "human-window.json" in log.read_text(encoding="utf-8-sig")
before, after = read("checks/before-protection.json"), read("checks/after-protection.json")
assert before == after
assert before["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
analysis_guard = read("checks/analysis-guard.json")
assert analysis_guard["missingInputRejected"] and not analysis_guard["comparisonCreated"]
assert sha(ROOT / "checks/analysis-missing-input.original.log") == analysis_guard["logSha256"]
binding = read("guard-runtime-binding.json")
guard_runtime = Path(binding["guardRuntime"])
assert not (guard_runtime / "human-window.json").exists()
assert not (guard_runtime / "fixture-ready.json").exists()
assert not (guard_runtime / "comparison.json").exists()
for copy in read("check-original-copies.json"):
    destination, original = ROOT / copy["path"], Path(copy["runtimePath"])
    assert destination.read_bytes() == original.read_bytes()
    assert destination.stat().st_size == copy["bytes"] and sha(destination) == copy["sha256"]
protection = read("final-protection.json")
assert protection["nativeCount"] == 0 and not protection["productStarted"]
assert not protection["configurationMoved"] and not protection["newHumanReceiptCreated"]
assert protection["profiles"] == before["profiles"]
assert protection["startupPointerSha256"] == before["startupPointerSha256"]
assert protection["taskSha256"] == before["taskSha256"]
assert len(protection["code"]) == 3
assert all(not code["trackedChanges"] and code["sharedGitBlobs"] == protection["docsSharedGitBlobs"]
           for code in protection["code"])
human = read("human-window.template.json")
assert not human["explicitHumanStart"] and not human["automaticContinuation"]
assert human["authorizedAtUtc"] is None and human["expiresAtUtc"] is None and human["userMessage"] is None
assert human["scope"] == contract["humanWindowScope"]
pr = read("pr-candidate-readback.original.json")
assert pr["number"] == 6 and pr["draft"] and not pr["merged"] and pr["state"] == "open"
assert pr["head"]["sha"] == source and image in pr["body"]
if (ROOT / "manifest.json").exists():
    manifest = read("manifest.json")
    actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name != "manifest.json"}
    assert actual == {row["path"] for row in manifest["files"]}
    for row in manifest["files"]:
        target = ROOT / row["path"]
        assert target.stat().st_size == row["bytes"] and sha(target) == row["sha256"]
print(json.dumps({"status": "verified-preparation-only", "source": source,
                  "syntaxScripts": 9, "missingHumanRefusals": 4,
                  "missingComparisonRefused": True, "configurationUnchanged": True,
                  "nativeCountAtGuards": 0, "newProductStarted": False,
                  "formalPairExecuted": False, "fullNativeGatePassed": False}))
