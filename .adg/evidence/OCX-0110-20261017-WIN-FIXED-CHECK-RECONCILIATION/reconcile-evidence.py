"""Reuse fixed-check evidence; no product starts, foreground input or new test gates."""
import hashlib
import json
import subprocess
import sys
import tempfile
import winreg
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
CODE = Path(r"C:\Users\15119\.codex\worktrees\windows-mesh-initial-size\opencodex-desktop")
SOURCE = "8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b"
TASK = ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(name, value):
    (ROOT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def witness(path):
    data = path.read_bytes()
    return {"path": path.relative_to(REPO).as_posix(), "bytes": len(data), "sha256": sha(data)}


def git(*args):
    return subprocess.run(["git", "-c", "core.longpaths=true", "-C", str(CODE), *args],
                          check=True, capture_output=True).stdout


def replay(relative, temporary=False):
    method = ROOT.parent / relative
    if temporary:
        # These historical methods write a report. Replay unmodified bytes in
        # a task-owned temporary directory, never in the historical archive.
        runtime = CODE / "test/runtime"
        assert runtime.is_dir() and not runtime.is_symlink()
        with tempfile.TemporaryDirectory(prefix="fixed-check-replay-", dir=runtime) as directory:
            target = Path(directory).resolve()
            assert target.is_relative_to(runtime.resolve())
            for path in method.parent.rglob("*"):
                if path.is_file():
                    assert not path.is_symlink()
                    copy = target / path.relative_to(method.parent)
                    assert copy.resolve().is_relative_to(target)
                    copy.parent.mkdir(parents=True, exist_ok=True)
                    copy.write_bytes(path.read_bytes())
            result = subprocess.run([sys.executable, "-X", "utf8", "-B", str(target / method.name)],
                                    capture_output=True, timeout=60)
    else:
        result = subprocess.run([sys.executable, "-X", "utf8", "-B", str(method)],
                                capture_output=True, timeout=60)
    assert result.returncode == 0, (relative, result.stderr.decode("utf-8", errors="replace"))
    return {"method": witness(method), "exitCode": result.returncode,
            "temporaryByteIdenticalReplay": temporary,
            "stdout": result.stdout.decode("utf-8"), "stderr": result.stderr.decode("utf-8")}


def check_manifest(directory, filename="manifest.json"):
    base = ROOT.parent / directory
    manifest = read(base / filename)
    rows = manifest if isinstance(manifest, list) else manifest.get("files", manifest.get("entries"))
    assert rows is not None
    for row in rows:
        path = base / row["path"]
        assert path.resolve().is_relative_to(base.resolve()) and not path.is_symlink()
        assert path.stat().st_size == row["bytes"] and sha(path.read_bytes()) == row["sha256"], path
    return {"manifest": witness(base / filename), "verifiedListedFiles": len(rows)}


def build():
    assert not (ROOT / "manifest.json").exists()
    assert git("rev-parse", "HEAD").decode().strip() == SOURCE
    assert git("status", "--porcelain", "--untracked-files=no") == b""
    task = read(REPO / TASK)
    assert task["state"] == "in_progress" and task["result"] == "pending" and task["revision"] == 2
    assert len(task["required_checks"]) == 8
    methods = [
        ("OCX-0110-20261015-WIN-BACKUP-PREFLIGHT/verify-foreground-backup.py", False),
        ("OCX-0110-20261015-WIN-FG-493-UPDATES-TREE/verify-complete-archive.py", False),
        ("OCX-0110-20261015-WIN-FG-TRAY-ADMIN/verify-resume-checkpoint.py", False),
        ("OCX-0110-20261015-WIN-FG-493-MANUAL-WINDOW/verify-manual-window.py", False),
        ("OCX-0110-20261013-WIN-RESTART-DUE-NATIVE/verify-evidence.py", True),
        ("OCX-0110-20261012-WIN-ADMIN-TOKEN/verify-evidence.py", True),
    ]
    replays = [replay(path, temporary) for path, temporary in methods]
    write("offline-replays.json", replays)
    reports = [
        "OCX-0110-20261015-WIN-BACKUP-PREFLIGHT/foreground-verification.json",
        "OCX-0110-20261015-WIN-FG-493-UPDATES-TREE/archive-complete-verification.json",
        "OCX-0110-20261015-WIN-FG-493-UPDATES-TREE/archive-verification.json",
        "OCX-0110-20261015-WIN-EVENT-RETRY-02/result.json",
        "OCX-0110-20261012-WIN-RESTORE-EVENT-SCOPE/archive-verification.json",
        "OCX-0110-20261012-WIN-EAE-RESTORE-EVENT/archive-verification.json",
        "OCX-0110-20261011-TREE-EVENT-AUDIT/result-summary.json",
        "OCX-0110-20261011-TREE-EVENT-AUDIT/manifest.json",
        "OCX-0110-20261013-WIN-RESTART-DUE-NATIVE/archive-verification.json",
        "OCX-0110-20261012-WIN-REAL-TIMER-BACKOFF/archive-verification.json",
        "OCX-0110-20261015-WIN-FG-TRAY-ADMIN/foreground-resume-verification-20261015.json",
        "OCX-0110-20261015-WIN-FG-493-CONTINUE/checkpoint-verification.json",
        "OCX-0110-20261015-WIN-FG-493-CONTINUE/session-analysis.json",
        "OCX-0110-20261015-WIN-FG-493-MANUAL-WINDOW/verification.json",
        "OCX-0110-20261012-WIN-ADMIN-TOKEN/archive-verification.json",
        "OCX-0110-20261017-WIN-8E-CANDIDATE/verification.json",
        "OCX-0110-20261017-WIN-8E-NATIVE-CLEAR/verification.json",
        "OCX-0110-20261016-WIN-UPDATE-SOURCE-PRECONDITIONS/source-conditions.json",
    ]
    manifests = [check_manifest(name) for name in (
        "OCX-0110-20261015-WIN-EVENT-RETRY-02",
        "OCX-0110-20261012-WIN-RESTORE-EVENT-SCOPE",
        "OCX-0110-20261012-WIN-EAE-RESTORE-EVENT",
        "OCX-0110-20261012-WIN-REAL-TIMER-BACKOFF",
        "OCX-0110-20261017-WIN-8E-CANDIDATE",
        "OCX-0110-20261017-WIN-8E-NATIVE-CLEAR",
    )]
    scopes = [
        ("backup-tree-ui", "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff", [
            "apps/desktop/ui/src/features/backup", "apps/desktop/ui/src/components/ui/UiTreeTable.vue",
            "apps/desktop/ui/src/components/ui/treeTable.ts"]),
        ("backup-safety-policy-browser", "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff", [
            "apps/desktop/tauri/src/modules/backup/safety.rs", "apps/desktop/tauri/src/modules/backup/policy.rs",
            "apps/desktop/tauri/src/modules/backup/browser.rs", "apps/desktop/tauri/src/commands/backup.rs"]),
        ("updates-and-notifications", "493da80675e6508e46e0fd83d86ee7216abf96fe", [
            "apps/desktop/ui/src/features/updates", "apps/desktop/ui/src/features/notifications",
            "apps/desktop/tauri/src/modules/update", "apps/desktop/tauri/src/modules/notifications",
            "apps/desktop/tauri/src/commands/event_delivery.rs", "apps/desktop/tauri/config/event-registry.json"]),
        ("native-window-tray", "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963", [
            "apps/desktop/tauri/src/modules/tray", "apps/desktop/tauri/src/commands/tray.rs",
            "apps/desktop/tauri/src/commands/window.rs", "apps/desktop/tauri/src/infrastructure/window_appearance.rs",
            "apps/desktop/tauri/src/infrastructure/tray_controller.rs",
            "apps/desktop/tauri/src/infrastructure/windows_tray_icon.rs"]),
        ("cycle-ui-scheduler", "be7395acfce0ec1e36b83300b576db408f8c35bb", [
            "apps/desktop/ui/src/features/updates/scheduler.ts",
            "apps/desktop/ui/src/features/updates/useUpdateScheduler.ts"]),
    ]
    impacts = []
    for name, old, paths in scopes:
        changes = git("diff", "--name-only", old, SOURCE, "--", *paths).decode().splitlines()
        assert not changes, (name, changes)
        impacts.append({"scope": name, "oldSource": old, "currentSource": SOURCE, "changedPaths": changes,
                        "paths": [{"path": path,
                                   "oldObject": git("rev-parse", old + ":" + path).decode().strip(),
                                   "currentObject": git("rev-parse", SOURCE + ":" + path).decode().strip()}
                                  for path in paths]})
    schedule = "apps/desktop/tauri/src/modules/update/schedule.rs"
    old = git("show", "be7395acfce0ec1e36b83300b576db408f8c35bb:" + schedule)
    current = git("show", SOURCE + ":" + schedule)
    def section(data, start, end):
        return data[data.index(start):data.index(end, data.index(start))]
    sections = [(b"pub const STATE_PATH:", b"#[derive(Debug, Clone, Copy"),
                (b"impl Target {", b"#[derive(Debug, Clone, Default"),
                (b"    pub fn reconcile_clock(", b"\npub fn cache_path(")]
    for start, end in sections:
        assert section(old, start, end) == section(current, start, end)
    impacts.append({"scope": "cycle-native-scheduler", "oldSource": "be7395acfce0ec1e36b83300b576db408f8c35bb",
                    "currentSource": SOURCE, "unchanged": ["cadence/startup constants", "Target interval/backoff",
                    "reconcile_clock/plan/reserve/complete"], "remainingChanges": "bounded storage safety and tests"})
    backup_diff = git("diff", "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff", SOURCE, "--",
                      "apps/desktop/tauri/src/modules/backup/manager.rs")
    impacts.append({"scope": "backup-manager", "oldSource": "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff",
                    "currentSource": SOURCE, "path": "apps/desktop/tauri/src/modules/backup/manager.rs",
                    "diffBytes": len(backup_diff), "diffSha256": sha(backup_diff),
                    "reviewedChanges": ["extract unchanged lock filename into a constant",
                                        "add nonblocking removal lock; normal backup lock semantics unchanged"],
                    "sourceCopiedToDocs": False})
    prefs = Path(r"C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop\manager-state\preferences.json")
    with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Software\com.gzers.opencodex.desktop") as key:
        pointer, kind = winreg.QueryValueEx(key, "StartupDataRootV1")
    assert kind == winreg.REG_SZ and isinstance(pointer, str)
    protect = {"preferencesSha256": sha(prefs.read_bytes()), "startupPointerUtf8Sha256": sha(pointer.encode()),
               "taskSha256": sha((REPO / TASK).read_bytes())}
    assert protect == {
        "preferencesSha256": "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d",
        "startupPointerUtf8Sha256": "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c",
        "taskSha256": "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"}
    record = {"observedAtUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
              "scope": "existing evidence reconciliation only; concrete claims and remaining items are in README and IMP-20",
              "taskRequiredChecks": task["required_checks"], "references": [witness(ROOT.parent / p) for p in reports],
              "manifestChecks": manifests, "offlineReplays": replays, "sourceImpact": impacts, "protection": protect,
              "productsStarted": 0, "foregroundAuthorizationCreated": False, "ciDispatched": False,
              "storageAlreadyCompleted": True, "cpuRssAccepted": True, "dpiCancelled": True,
              "fullWindowsGatePassed": False, "taskClosed": False, "releaseAccepted": False}
    write("reconciliation.json", record)
    paths = sorted(path for path in ROOT.rglob("*") if path.is_file() and path.name != "manifest.json")
    write("manifest.json", {"files": [dict(witness(path), path=path.relative_to(ROOT).as_posix()) for path in paths]})


def verify():
    manifest = read(ROOT / "manifest.json")
    names = {row["path"] for row in manifest["files"]}
    assert names == {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name != "manifest.json"}
    for row in manifest["files"]:
        path = ROOT / row["path"]
        assert path.resolve().is_relative_to(ROOT.resolve())
        assert path.stat().st_size == row["bytes"] and sha(path.read_bytes()) == row["sha256"], path
    record = read(ROOT / "reconciliation.json")
    assert record["source"] == SOURCE and len(record["taskRequiredChecks"]) == 8
    for row in record["references"]:
        assert witness(REPO / row["path"]) == row
    for replayed in record["offlineReplays"]:
        assert witness(REPO / replayed["method"]["path"]) == replayed["method"]
        assert replayed["exitCode"] == 0
    assert not record["productsStarted"] and not record["ciDispatched"] and not record["taskClosed"]
    print(json.dumps({"status": "verified-evidence-reconciliation", "files": len(names),
                      "originalReportsReferenced": len(record["references"]), "offlineReplays": len(record["offlineReplays"]),
                      "source": SOURCE, "productsStarted": 0, "fullWindowsGatePassed": False}))


if sys.argv[1:] == ["--build"]:
    build()
elif sys.argv[1:] != ["--verify"]:
    raise SystemExit("Expected --build or --verify")
verify()
