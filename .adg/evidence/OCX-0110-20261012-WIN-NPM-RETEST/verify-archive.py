"""Verify the frozen no-script retest and concrete pending approval plan."""

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def write(name, value):
    (ROOT / name).write_bytes(
        (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )


if sys.argv[1:] == ["--build"]:
    files = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name in {"manifest.json", "archive-verification.json"}:
            continue
        assert path.suffix.lower() not in {".exe", ".msi", ".cab", ".zip", ".dll", ".tgz"}
        payload = path.read_bytes()
        files.append(
            {
                "path": path.relative_to(ROOT).as_posix(),
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": files})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-archive.py [--build]")

for item in read("manifest.json")["entries"]:
    path = ROOT / item["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    payload = path.read_bytes()
    assert len(payload) == item["bytes"]
    assert hashlib.sha256(payload).hexdigest() == item["sha256"], item["path"]

source = "ee65f1158c4f0bb7c283ff09e24120a7884472d0"
for name in ["run-38208548177-final.json", "run-38208582603-final.json"]:
    run = read(name)["run"]
    assert run["head_sha"] == source and run["conclusion"] == "success"
assert all(
    download["sourceCommit"] == source and download["matched"]
    and download["expectedDigest"] == "sha256:" + download["sha256"]
    for download in read("download-verification.json")
)
launch = read("native-launch-input.json")
assert launch["ciInstalledExeMatched"] and launch["sourceCommit"] == source
assert not launch["installerExecutedLocally"]
live = read("ordinary-live-session-checkpoint.json")
assert live["sourceCommit"] == source and live["sourceCommitMatched"]
assert not live["applicationToken"]["elevated"] and live["applicationToken"]["elevationType"] == 3
retest = read("panel282-retest-summary.json")
assert retest["sourceCommit"] == source and retest["state"] == "failed"
assert "scripts_required" in retest["error"]["message"]
assert retest["sourceKindAfter"] == "unresolved"
assert retest["listenerRemoved"] and not retest["scriptsAllowed"]
log = (ROOT / "panel282-npm-retest-success.log").read_text(encoding="utf-8-sig")
assert "verbose exit 0" in log and "added 102 packages" not in log
# npm's verbose file and the captured native progress contain different details.
operation = read("panel282-install-observation.json")["result"]["operation"]
assert any(event["payload"].get("line") == "added 102 packages in 14s" for event in operation["events"])
identity = read("panel282-package-inspection-identity.json")
assert identity["registryIntegrityMatched"] and not identity["codeExecuted"]
assert identity["sha256"] == "3d2e06f075a91529f30ad71c02442886e0ba4e14497080008d0b0e97fb2e7ff7"
script_plan = read("install-script-plan.json")
assert not script_plan["scriptsExecuted"]
assert len(script_plan["installScriptPackages"]) == 1
assert script_plan["installScriptPackages"][0]["path"] == "node_modules/bun"
assert script_plan["installScriptPackages"][0]["version"] == "1.4.2"
inspection = read("bun142-script-inspection.json")
assert not inspection["codeExecuted"]
assert inspection["scriptSha256"] == "c317425810b3b70862713d51b9640e1e02ca9061a9c2f7bbdfe61e8e622ef55d"
confirmation = read("install-script-confirmation-plan.json")
assert confirmation["state"] == "awaiting_explicit_second_confirmation"
assert not confirmation["approvalProvided"] and not confirmation["scriptsExecuted"]
assert confirmation["version"] == "2.82.0" and confirmation["privilege"] == "ordinary"
assert confirmation["notAnOsSecuritySandbox"]
checkpoint = read("checkpoint.json")
assert not checkpoint["task182Present"] and not checkpoint["explicitScriptPermissionPresent"]
assert not checkpoint["realUserDataModified"] and not checkpoint["localInstallerExecuted"]
write(
    "archive-verification.json",
    {
        "result": "passed",
        "scope": "archive identity and partial no-script npm retest only",
        "filesVerified": len(read("manifest.json")["entries"]),
        "nativeSource": source,
        "canonicalNpmPathFailure": "not observed on real retest; npm install exit 0",
        "completeRuntimeInstall": "blocked by scripts_required; not accepted",
        "explicitScriptPermission": "pending; no scripts executed",
        "cwdFixSource": "e1a1aaa54ccc56530d5da507d0666b6ec8d2c138",
        "cwdFixAcceptance": "pending exact CI and native retest",
        "formalTaskReceipt": False,
        "fullNativeGateAccepted": False,
        "performanceAccepted": False,
        "realCyclesAccepted": False,
        "existingPerformanceFailureCleared": False,
        "releaseAccepted": False,
    },
)
print(json.dumps({"archiveIntegrity": "passed", "files": len(read("manifest.json")["entries"])}))
