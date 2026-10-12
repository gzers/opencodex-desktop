"""Verify this preflight archive; this is not a TASK receipt or native gate."""

import hashlib
import json
from pathlib import Path


EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[2]
COLLAB = (
    "docs/04-项目资料/03-项目协作资料/01-协作包/"
    "2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/"
    "20261009-概览升级入口与备份通知策略/验证/"
    "20261012-Windows接续核验/README.md"
)
COMMIT = "01a1d85504781f4cb189a68ed85d1d803a1409ff"


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8-sig"))


def write(name, value):
    (EVIDENCE / name).write_text(
        json.dumps(value, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


snapshot = read("local-preflight.json")
ci = read("ci-final-summary.json")
runs = read("same-sha-runs-final.json")
artifacts = read("ci-artifacts-final.json")
token = read("process-token/process-token.json")
tests = read("local-test-commands.json")
push = read("push-preflight.json")
checks = {
    "source_identity": snapshot["codeCommit"] == snapshot["remoteMain"] == COMMIT,
    "ci_identity_and_success": ci["head_sha"] == COMMIT
    and ci["status"] == "completed"
    and ci["conclusion"] == "success"
    and len(ci["jobs"]) == 6
    and all(job["conclusion"] == "success" for job in ci["jobs"]),
    "no_same_sha_installer_run": runs["sha"] == COMMIT
    and not runs["candidateInstallerRunPresent"],
    "artifact_metadata_only": artifacts["total_count"] == 2
    and {item["name"] for item in artifacts["artifacts"]}
    == {"windows-startup-evidence", "windows-process-token-evidence"}
    and all(
        item["workflow_run"]["head_sha"] == COMMIT
        and item["digest"].startswith("sha256:")
        and not item["expired"]
        for item in artifacts["artifacts"]
    ),
    "task_authority_not_reconstructed": not snapshot["taskAuthority"]["exists"]
    and snapshot["taskAuthority"]["gitHistoryCount"] == 0
    and not snapshot["taskAuthority"]["reconstructed"]
    and not snapshot["taskAuthority"]["formalReceiptCreated"],
    "shared_files_match": all(item["equal"] for item in snapshot["sharedFiles"]),
    "token_tool_only": token["sourceCommit"] == COMMIT
    and token["result"] == "pass"
    and len(token["checks"]) == 9
    and all(item["result"] == "pass" for item in token["checks"])
    and token["childToken"]["elevated"] is False,
    "tool_regression_counts": tests["codeCommit"] == COMMIT
    and [item["passed"] for item in tests["commands"]] == [9, 12]
    and all(item["failed"] == item["exitCode"] == 0 for item in tests["commands"]),
    "push_preflight_failed_without_mutation": push["exitCode"] == 128
    and not push["mutatesRemote"],
    "native_gate_not_started": snapshot["candidateApplicationInstancesStarted"] == 0
    and not snapshot["installerExecuted"]
    and not snapshot["productSourceChanged"]
    and not snapshot["taskStateChanged"]
    and not snapshot["realDataReadOrModified"]
    and not snapshot["releaseAccepted"],
}
assert all(checks.values()), checks

paths = sorted(
    (
        path
        for path in EVIDENCE.rglob("*")
        if path.is_file()
        and path.name not in {"manifest.json", "archive-verification.json"}
    ),
    key=lambda path: path.as_posix(),
)
paths.extend([ROOT / COLLAB, ROOT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"])
entries = []
for path in paths:
    payload = path.read_bytes()
    hash_mode = "raw"
    if path.is_relative_to(ROOT / "docs"):
        # Human Markdown follows the repository's checkout line-ending policy.
        # Raw API/command evidence is protected by this archive's .gitattributes.
        payload = payload.replace(b"\r\n", b"\n")
        hash_mode = "canonical-lf"
    entries.append(
        {
            "path": path.relative_to(ROOT).as_posix(),
            "hashMode": hash_mode,
            "bytes": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(),
        }
    )
write(
    "manifest.json",
    {
        "schema": "ocx-windows-preflight-archive-v1",
        "codeCommit": COMMIT,
        "artifactDownloadVerified": False,
        "entries": entries,
    },
)
for entry in entries:
    payload = (ROOT / entry["path"]).read_bytes()
    if entry["hashMode"] == "canonical-lf":
        payload = payload.replace(b"\r\n", b"\n")
    assert hashlib.sha256(payload).hexdigest() == entry["sha256"]
    assert len(payload) == entry["bytes"]
write(
    "archive-verification.json",
    {
        "result": "passed",
        "scope": "archive integrity and preflight observations only",
        "formalTaskReceipt": False,
        "nativeGateResult": "not_run",
        "pairedPerformanceResult": "not_run",
        "realCycleObservationStarted": False,
        "existingPerformanceFailureCleared": False,
        "filesVerified": len(entries),
        "checks": checks,
    },
)
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
