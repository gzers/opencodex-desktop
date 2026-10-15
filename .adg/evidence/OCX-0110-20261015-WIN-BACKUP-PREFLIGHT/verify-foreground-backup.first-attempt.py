"""Offline verification of the bounded native backup session; no application or fixture writes."""
import hashlib
import json
import sys
from datetime import datetime, timedelta
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parent
SOURCE = "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
EXE_SHA = "f43e81a9d96f25a29a937a54d2c779ce2459c3ecb30a73b131c57729f70cb929"
ORIGINAL = "bk_20261015014652_9e631596"
LIGHT_GUARD = "bk_20261015015101_2a4ddd2a"
MANUAL_REMOVED = {"bk_20261015015159_1cc59ec0", "bk_20261015015200_df084d8f"}
AUTO_REMOVED = "bk_20261015015201_a254f99c"
TARGET = "bk_20261015015618_1393e76a"
FAILED_GUARD = "bk_20261015015856_334223b7"
UNRELATED = "bk_20261015015922_b3f9f2c5"
RECOVERED_GUARD = "bk_20261015020030_5ac0e0d6"
DARK_SHA = "25a41960e9033165e15052d927816914cbe266c67f97a1830240eff8ee547eab"
LIGHT_SHA = "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
CORRUPT_SHA = "cbf910baaeda1ded82aef83b20588030d5feb850c7b3d4fe5012a22ddddf9197"


def read(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def instant(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def safe_path(base, name):
    relative = PurePosixPath(name)
    assert not relative.is_absolute() and ".." not in relative.parts and "\\" not in name
    result = base.joinpath(*relative.parts)
    assert not result.is_symlink() and result.resolve().is_relative_to(base.resolve())
    return result


launch = read("ordinary-launch.json")
restart = read("ordinary-restart.json")
first_exit = read("first-exit-observer-result.json")
last_exit = read("restart-exit-observer-result.json")
post = read("foreground-final-postcheck.json")
authorization = read("foreground-authorization-20261015T0943.json")
start = instant(authorization["atUtc"])
deadline = start + timedelta(minutes=authorization["windowMinutes"])
assert authorization["origin"] == "human" and authorization["foregroundStartGranted"]
assert authorization["userText"] == "开始前台验证" and authorization["windowMinutes"] == 30
assert authorization["source"] == SOURCE and authorization["scope"] == "isolated-backup-tree"
assert launch["dataRoot"] == restart["dataRoot"] and launch["runtime"] == restart["runtime"]
assert launch["executable"] == restart["executable"]
assert launch["pid"] == 18460 and restart["pid"] == 66488 and restart["priorPid"] == launch["pid"]
for identity, exit_record in ((launch, first_exit), (restart, last_exit)):
    assert identity["source"] == SOURCE and identity["sha256"] == EXE_SHA
    token = identity["applicationToken"]
    assert token["status"] == "observed" and token["elevated"] is False and token["elevationType"] == 3
    assert token["processId"] == identity["pid"] == exit_record["pid"]
    assert token["executable"] == identity["executable"]
    assert instant(token["creationTimeUtc"]) == instant(identity["creationTimeUtc"])
    assert instant(exit_record["creationTimeUtc"]) == instant(identity["creationTimeUtc"])
    assert start < instant(identity["creationTimeUtc"]) < instant(exit_record["atUtc"]) < deadline
    assert exit_record["exitObserved"] and exit_record["exitCode"] == 0
    assert not exit_record["timeout"] and not exit_record["agentForcedTermination"]
    assert not identity["hostInstallerExecuted"] and not identity["forcedScaleUsed"]
    assert not identity["cdpUsedForUiInput"]
    assert instant(identity["authorization"]["startedAtUtc"]) == start
    assert instant(identity["authorization"]["expiresAtUtc"]) == deadline
zero_between = read("first-exit-postcheck.json")
assert zero_between["nativeCount"] == restart["nativeCountBefore"] == 0
assert zero_between["nativeProcesses"] == []
assert instant(first_exit["atUtc"]) < instant(zero_between["atUtc"]) < instant(restart["creationTimeUtc"])
assert not restart["preferencesChangedByLauncher"] and not restart["backupsChangedByLauncher"]
assert post["nativeCount"] == 0 and post["nativeProcesses"] == []
assert post["firstExitCode"] == post["restartExitCode"] == 0
assert post["sequentialNativeInstances"] == 2 and not post["agentForcedTermination"]
assert post["source"] == SOURCE and post["exeSha256"] == EXE_SHA
assert post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
for key in ("systemDpiChanged", "systemTransparencyChanged", "cdpUsedForUiInput",
            "thirdPartyLifecycleScriptsExecuted", "fullVersionGatePassed",
            "releaseRequested", "macNativeHandoffPerformed"):
    assert post[key] is False
screen = read("ordinary-first-screen.json")
assert screen["pid"] == launch["pid"] and screen["hwnd"] == 5637288
assert screen["visible"] and screen["foreground"] and not screen["applicationMenuAttached"]
assert screen["backdropHresult"] == 0 and screen["backdropRawValue"] == 2
assert screen["remoteSessionMetric"] == 1 and screen["nativeDpi"] == 192
assert not screen["physicalDpiGateClaimed"] and not screen["forcedScaleUsed"]

states = {}
payload_checks = 0
expected_mismatches = []
for folder in sorted(ROOT.glob("backup-state-*")):
    snap = json.loads((folder / "snapshot.json").read_text(encoding="utf-8-sig"))
    phase = snap["phase"]
    assert folder.name == "backup-state-" + phase and phase not in states
    assert snap["source"] == SOURCE and snap["dataRoot"] == launch["dataRoot"]
    identity, exit_record = (restart, last_exit) if phase in {
        "after-restart", "user-unpinned", "final-repinned"
    } else (launch, first_exit)
    assert snap["pid"] == identity["pid"] and snap["creationTimeUtc"] == identity["creationTimeUtc"]
    assert instant(identity["creationTimeUtc"]) < instant(snap["atUtc"]) < instant(exit_record["atUtc"])
    listed = {row["path"]: row for row in snap["files"]}
    assert len(listed) == len(snap["files"])
    actual = {path.relative_to(folder).as_posix() for path in folder.rglob("*")
              if path.is_file() and path.name != "snapshot.json"}
    assert set(listed) == actual
    for name, row in listed.items():
        path = safe_path(folder, name)
        assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"], (phase, name)
    assert sum(row["bytes"] for row in listed.values()) == snap["bytes"]
    assert not snap["inputInjected"] and not snap["sourceDataChangedByReader"]
    assert not snap["backupDatesChangedByReader"]
    records = {}
    manifests_by_id = {}
    for path in folder.glob("backups/**/backup-manifest.json"):
        manifest = json.loads(path.read_text(encoding="utf-8-sig"))
        backup_id = manifest["backup_id"]
        assert backup_id == path.parent.name and backup_id not in records
        records[backup_id] = manifest
        manifests_by_id[backup_id] = path
        payload = path.with_name("preferences.json")
        assert payload.stat().st_size == manifest["bytes"]
        actual_hash = digest(payload)
        if phase in {"corrupt-restore-rejected", "unrelated-success"} and backup_id == TARGET:
            assert actual_hash == CORRUPT_SHA and manifest["sha256"] == DARK_SHA
            expected_mismatches.append({"phase": phase, "backupId": backup_id})
        else:
            assert actual_hash == manifest["sha256"], (phase, backup_id)
            payload_checks += 1
    assert records == {row["backup_id"]: row for row in snap["manifests"]}
    prefs = folder / "manager-state/preferences.json"
    value = json.loads(prefs.read_text(encoding="utf-8-sig"))
    assert value["appearance"]["theme"] == snap["preferencesTheme"]
    assert value["schema_version"] == snap["preferencesSchema"] == 2
    assert not value["updates"]["app_update_auto_check"] and not value["shell"]["auto_panel"]
    assert not value["maintenance"]["startup_cleanup"] and not value["shell"]["launch_with_codex"]
    states[phase] = {"snapshot": snap, "records": records, "folder": folder,
                     "prefs": prefs, "manifests": manifests_by_id}
assert len(states) == 18 and len(expected_mismatches) == 2


def records(phase):
    return states[phase]["records"]


def bytes_at(phase, relative):
    return (states[phase]["folder"] / relative).read_bytes()


def payload(phase, backup_id):
    return states[phase]["manifests"][backup_id].with_name("preferences.json")


def pinned(phase, backup_id=ORIGINAL):
    return records(phase)[backup_id]["management"]["pinned"]


def guard(phase, backup_id, related, transaction_state):
    value = records(phase)[backup_id]["management"]
    assert value == {"version": 2, "kind": "restore-protection", "pinned": False,
                     "related_backup_id": related, "transaction_state": transaction_state}


assert set(records("first-created")) == {ORIGINAL} and not pinned("first-created")
assert pinned("user-pinned") and pinned("changed-light") and pinned("restored-dark")
assert digest(states["first-created"]["prefs"]) == DARK_SHA
assert digest(states["changed-light"]["prefs"]) == LIGHT_SHA
assert digest(states["restored-dark"]["prefs"]) == DARK_SHA
assert states["first-created"]["prefs"].read_bytes() == payload("restored-dark", ORIGINAL).read_bytes()
assert states["changed-light"]["prefs"].read_bytes() == payload("restored-dark", LIGHT_GUARD).read_bytes()
guard("restored-dark", LIGHT_GUARD, ORIGINAL, "committed")
assert set(records("fourteen-fresh")) >= set(records("restored-dark"))
assert len(records("fourteen-fresh")) == 14
fresh_time = instant(states["fourteen-fresh"]["snapshot"]["atUtc"])
assert all(timedelta(0) <= fresh_time - instant(row["created_at"]) < timedelta(minutes=10)
           for row in records("fourteen-fresh").values())
generations = read("retention-real-ui-generations.json")
assert generations["source"] == SOURCE and not generations["invokeOrCdpUsed"]
assert [(row["priorCount"], row["nextCount"]) for row in generations["actions"]] == [
    (index, index + 1) for index in range(2, 14)
]

# Age fixtures may change only created_at, never payload/id/stored_at/directory.
fixture = read("retention-age-fixture/result.json")
assert fixture["syntheticAgesAreNotRealPeriodEvidence"] and not fixture["systemClockChanged"]
for row in fixture["changes"]:
    before_path = ROOT / "retention-age-fixture" / (row["backupId"] + ".before-manifest.json")
    after_path = ROOT / "retention-age-fixture" / (row["backupId"] + ".after-manifest.json")
    assert digest(before_path) == row["beforeSha256"] and digest(after_path) == row["afterSha256"]
    before = json.loads(before_path.read_text(encoding="utf-8-sig"))
    after = json.loads(after_path.read_text(encoding="utf-8-sig"))
    assert before == records("fourteen-fresh")[row["backupId"]]
    assert after == records("ages-fixture")[row["backupId"]]
    assert before["created_at"] != after["created_at"]
    assert {k: v for k, v in before.items() if k != "created_at"} == {
        k: v for k, v in after.items() if k != "created_at"
    }
    assert instant(fixture["atUtc"]) - instant(after["created_at"]) == timedelta(days=row["syntheticAgeDays"])
    assert payload("fourteen-fresh", row["backupId"]).read_bytes() == payload("ages-fixture", row["backupId"]).read_bytes()
assert set(records("ages-fixture")) - set(records("manual-cleanup-two")) == MANUAL_REMOVED
assert not (set(records("manual-cleanup-two")) - set(records("ages-fixture")))
assert len(records("manual-cleanup-two")) == 12 and pinned("manual-cleanup-two")
assert read("manual-retention-analysis.json")["removedIds"] == sorted(MANUAL_REMOVED)
auto_fixture = read("automatic-age-fixture/result.json")
assert auto_fixture["backupId"] == AUTO_REMOVED and not auto_fixture["realCycleEvidenceClaimed"]
assert not auto_fixture["systemClockChanged"] and auto_fixture["onlyCreatedAtChanged"]
before_auto = records("manual-cleanup-two")[AUTO_REMOVED]
after_auto = records("before-automatic-policy")[AUTO_REMOVED]
assert before_auto["created_at"] != after_auto["created_at"]
assert {k: v for k, v in before_auto.items() if k != "created_at"} == {
    k: v for k, v in after_auto.items() if k != "created_at"
}
assert records("before-automatic-policy") == records("saved-automatic-no-delete")
assert set(records("saved-automatic-no-delete")) - set(records("automatic-triggered")) == {AUTO_REMOVED}
assert set(records("automatic-triggered")) - set(records("saved-automatic-no-delete")) == {TARGET}
assert pinned("automatic-triggered") and len(records("automatic-triggered")) == 12
policy = lambda phase: json.loads(bytes_at(phase, "manager-state/backup-cleanup-policy.json"))
assert policy("saved-automatic-no-delete") == {
    "schema_version": 1, "mode": "automatic", "keep_recent": 10, "keep_days": 30
}
assert policy("before-corruption") == policy("final-repinned") == {
    "schema_version": 1, "mode": "manual", "keep_recent": 10, "keep_days": 30
}

# Deliberate same-size, JSON-equivalent corruption must be rejected before overwrite.
original_bytes = (ROOT / "restore-integrity-fixture/payload-original.json").read_bytes()
corrupt_bytes = (ROOT / "restore-integrity-fixture/payload-corrupted-same-size.json").read_bytes()
assert len(original_bytes) == len(corrupt_bytes) == 1374
assert hashlib.sha256(original_bytes).hexdigest() == DARK_SHA
assert hashlib.sha256(corrupt_bytes).hexdigest() == CORRUPT_SHA
assert json.loads(original_bytes) == json.loads(corrupt_bytes) and original_bytes != corrupt_bytes
assert records("before-corruption")[TARGET] == records("corrupt-restore-rejected")[TARGET]
assert bytes_at("before-corruption", "manager-state/preferences.json") == bytes_at(
    "corrupt-restore-rejected", "manager-state/preferences.json")
guard("corrupt-restore-rejected", FAILED_GUARD, TARGET, "active")
assert set(records("unrelated-success")) - set(records("corrupt-restore-rejected")) == {UNRELATED}
assert payload("exact-restore-recovered", TARGET).read_bytes() == original_bytes
guard("exact-restore-recovered", RECOVERED_GUARD, TARGET, "committed")
guard("exact-restore-recovered", FAILED_GUARD, TARGET, "active")


def notifications(phase):
    return json.loads(bytes_at(phase, "manager-state/notifications.json"))["items"]


event_analysis = read("restore-event-exact-resolution-analysis.json")
failure_id = event_analysis["sameFailureId"]
for phase, resolved in (("corrupt-restore-rejected", False), ("unrelated-success", False),
                        ("exact-restore-recovered", True), ("after-restart", True),
                        ("final-repinned", True)):
    event = next(row for row in notifications(phase) if row["notification_id"] == failure_id)
    assert event["event_identity"] == event_analysis["sameEventIdentity"]
    assert event["event_identity"]["channel"] == "local"
    assert event["resolved"] is resolved and event["read"] is False and event["deleted"] is False
    assert event["resolved_at"] == (event_analysis["resolvedAt"] if resolved else None)
    warning = next(row for row in notifications(phase) if row["notification_id"] == "runtime-not-found")
    assert not warning["resolved"] and not warning["deleted"]
before_restart = {row["path"]: row for row in states["before-first-exit"]["snapshot"]["files"]
                  if row["path"].startswith("backups/")}
after_restart = {row["path"]: row for row in states["after-restart"]["snapshot"]["files"]
                 if row["path"].startswith("backups/")}
assert before_restart == after_restart and len(records("after-restart")) == 15
assert bytes_at("before-first-exit", "manager-state/preferences.json") == bytes_at(
    "after-restart", "manager-state/preferences.json")
assert not pinned("user-unpinned") and pinned("final-repinned")
assert set(records("after-restart")) == set(records("user-unpinned")) == set(records("final-repinned"))
guard("final-repinned", FAILED_GUARD, TARGET, "active")

# UI artifacts support observations, not pixel measurement or full prototype coverage.
ui = {}
for path in ROOT.glob("*.sky.json"):
    state = read(path.name)
    assert start <= instant(state["atUtc"]) < deadline
    assert state["window"]["id"] in {5637288, 6165676}
    assert state["accessibility"]["tree"] and state["accessibility"]["document_text"]
    for shot in state["screenshots"]:
        assert safe_path(ROOT, shot["file"]).stat().st_size > 1000
    ui[path.name.removesuffix(".sky.json")] = state["accessibility"]["document_text"]
assert "尚无备份" in ui["backup-empty-ui"] and all(
    title in ui["backup-empty-ui"] for title in ("名称", "用途", "操作", "backups"))
assert "没有可清理的备份" in ui["retention-over-ten-fresh-no-cleanup"]
assert "将删除 2 份备份" in ui["cleanup-aged-two-preview"]
for name in ("tree-backup-files-expanded", "tree-keyboard-enter-expand", "tree-refresh-expansion-kept"):
    assert "backup-manifest.json" in ui[name] and "preferences.json" in ui[name]
    assert "收起 " + ORIGINAL in ui[name]
assert "展开 " + ORIGINAL in ui["tree-keyboard-space-collapse"]
assert "backup-manifest.json" not in ui["tree-keyboard-space-collapse"]
assert "恢复未成功完成" in ui["restore-corrupt-rejected"] or "恢复失败" in ui["restore-corrupt-rejected"]
assert "偏好已恢复" in ui["restore-exact-candidate-recovered"]
regression = read("authorization-time-type-regression.json")
assert regression["positiveChecks"] == 6 and regression["negativeChecks"] == 4
assert len(regression["cases"]) == 10 and all(row["accepted"] == row["expectedAccepted"]
                                            for row in regression["cases"])
assert not regression["applicationStarted"] and not regression["foregroundWindowExtended"]
assert not regression["syntheticAuthorizationFileWritten"]
assert read("authorization-json-time-parse.original.json")["nativeCount"] == 0
assert not read("authorization-json-time-parse.original.json")["runtimeExists"]
assert not read("integrity-fixture-preparation-error.json")["fixtureTargetChanged"]

# Freeze separately from the historical 17-file preparation manifest.
manifest_path = ROOT / "foreground-manifest.json"
report_path = ROOT / "foreground-verification.json"
current_paths = {path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*")
                 if path.is_file() and path not in {manifest_path, report_path}}
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest = [{"path": name, "bytes": (ROOT / name).stat().st_size, "sha256": digest(ROOT / name)}
                for name in sorted(current_paths)]
else:
    manifest = read(manifest_path.name)
assert {row["path"] for row in manifest} == current_paths
assert len(manifest) == len(current_paths)
for row in manifest:
    path = safe_path(ROOT, row["path"])
    assert path.stat().st_size == row["bytes"] and digest(path) == row["sha256"], row["path"]
report = {
    "archiveIntegrity": "passed", "scope": "Bounded native backup session only",
    "source": SOURCE, "exeSha256": EXE_SHA, "manifestFiles": len(manifest),
    "snapshots": len(states), "validPayloadDigestChecks": payload_checks,
    "expectedCorruptPayloadSnapshots": expected_mismatches, "uiStateArtifacts": len(ui),
    "ordinaryApplicationTokensVerified": 2, "sequentialInstances": 2,
    "nativeInstancesAtFinalCheckpoint": 0, "bothObservedExitCodes": [0, 0],
    "darkLightDarkBytesVerified": True, "manualCleanupRemoved": sorted(MANUAL_REMOVED),
    "automaticCleanupRemoved": AUTO_REMOVED, "policySaveDeletedNothing": True,
    "pinnedSynthetic90DayBackupRetained": True, "unpinRepinPersistedWithoutDeletion": True,
    "corruptRestoreRejectedWithoutOverwrite": True, "sameLocalEventResolved": True,
    "unrelatedSuccessDidNotResolve": True, "restartPersistenceVerified": True,
    "priorFailedGuardStillActive": True, "timestampTypeRegressionChecks": 10,
    "treeKeyboardExpansionAndRefreshObserved": True, "treePixelAlignmentMeasured": False,
    "treeRefreshFocusRetentionVerified": False, "prototypeCheckboxCoverageVerified": False,
    "physicalDpiVerified": False, "managerChannelIsolationVerified": False,
    "realCycleVerified": False, "pairedPerformanceVerified": False,
    "installerRetentionVerified": False, "fullVersionGatePassed": False,
}
if "--build" in sys.argv:
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path.name) == report
print(json.dumps(report))
