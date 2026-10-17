"""Freeze the first CI of the actual owned-root cleanup fix."""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")
RUN = CODE / "test/runtime/uninstall-owned-files-20261017T2020"
SOURCE = "8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b"
NAMES = [
    "real_backup_lock_and_owned_audit_rotations_do_not_block_confirmed_removal",
    "active_backup_transaction_preserves_data_until_its_real_lock_is_released",
    "reserved_names_must_be_regular_files_and_unknown_rotations_are_retained",
]


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert data == Path(row["runtimePath"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    original = ROOT / "original"
    boundary = read(original / "change-boundary.json")
    assert boundary["source"] == SOURCE and boundary["newCrossPlatformTests"] == 3
    assert boundary["queryProductionFixtureAndIpcWaitsUnchanged"]
    assert boundary["oldUnknownFutureAndLinkRefusalsKept"]
    assert boundary["backupTransactionsRequireNonblockingLockBeforeAnyDeletion"]
    smoke = read(original / "smoke-syntax.json")
    assert smoke["syntaxPassed"] and smoke["sentinelCount"] == 7
    assert not smoke["hostProductStarted"] and not smoke["hostCleanupInvoked"]
    pushed = read(original / "push-confirmed.json")
    assert pushed["source"] == SOURCE and pushed["remoteReadbackMatched"] and not pushed["force"]
    terminal = read(original / "ci-terminal.original.json")
    admission = read(original / "ci-complete-admission.json")
    assert terminal["run"]["head_sha"] == SOURCE and terminal["run"]["run_attempt"] == 1
    assert terminal["run"]["id"] == admission["runId"] == 39032153138
    assert terminal["run"]["event"] == "push" and admission["source"] == SOURCE
    assert admission["status"] == "completed"
    assert terminal["run"]["conclusion"] == admission["conclusion"]
    assert {r["job"] for r in admission["requiredJobLogs"]} == {
        "backend", "backend-windows", "frontend", "release-tools", "windows-smoke-evidence",
    }
    results = {}
    for row in admission["requiredJobLogs"]:
        data = (original / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert row["actualCheckout"] == SOURCE
        if row["job"] in ("backend", "backend-windows"):
            text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data.decode("utf-8-sig"))
            passed = [
                name for name in NAMES
                if "test modules::data_root::uninstall::tests::" + name + " ... ok" in text
            ]
            results[row["job"]] = {"conclusion": row["conclusion"], "newCoreCasesPassed": passed}
            if row["conclusion"] == "success":
                assert len(passed) == 3
                expected = (938, 0, 6) if row["job"] == "backend" else (667, 0, 7)
                tests = row["rustResults"]
                assert len(tests) == 35
                assert tuple(sum(int(r[i]) for r in tests) for i in (1, 2, 3)) == expected
    base = boundary["base"]
    for path in [
        "apps/desktop/tauri/src/infrastructure/windows_query_fixture.rs",
        "apps/desktop/tauri/src/infrastructure/official_version_source.rs",
        "apps/desktop/tauri/src/modules/ipc/service.rs",
        ".github/workflows/ci.yml", ".github/workflows/windows-candidate.yml",
    ]:
        assert subprocess.check_output(["git", "show", base + ":" + path], cwd=CODE) == (
            subprocess.check_output(["git", "show", SOURCE + ":" + path], cwd=CODE)
        )
    protection = read(ROOT / "protection.json")
    assert protection["nativeCount"] == 0 and protection["originalPreferencesUnchanged"]
    assert protection["originalPointerUnchanged"] and protection["taskUnchanged"]
    return {
        "source": SOURCE, "firstCiRunId": 39032153138, "firstCiConclusion": admission["conclusion"],
        "sameSourceFullCiAdmitted": admission["wholeToolCiAdmitted"], "backendCoreResults": results,
        "hostCleanupExecuted": False, "historicalQueryRootCauseProved": False,
        "fullInstallerGatePassed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
else:
    assert not (ROOT / "manifest.json").exists()
    names = [
        "change-boundary.json", "smoke-syntax.json", "push-confirmed.json", "ci-handle.json",
        "ci-terminal.original.json", "ci-complete-admission.json",
        "ci-complete-backend.original.log", "ci-complete-backend-windows.original.log",
        "ci-complete-frontend.original.log", "ci-complete-release-tools.original.log",
        "ci-complete-windows-smoke-evidence.original.log",
    ]
    copies = []
    for name in names:
        data = (RUN / name).read_bytes()
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
        target = ROOT / "original" / name
        target.parent.mkdir(exist_ok=True)
        with target.open("xb") as stream:
            stream.write(data)
        copies.append({"path": "original/" + name, "runtimePath": str(RUN / name),
                       "bytes": len(data), "sha256": sha(data)})
    write(ROOT / "copies.json", copies)
    write(ROOT / "verification.json", verify())
    (ROOT / ".gitattributes").write_bytes(b"* -text\n")
    write(ROOT / "manifest.json", {"files": [
        {"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
         "sha256": sha(p.read_bytes())}
        for p in sorted(ROOT.rglob("*")) if p.is_file()
    ]})
print(json.dumps(read(ROOT / "verification.json")))
