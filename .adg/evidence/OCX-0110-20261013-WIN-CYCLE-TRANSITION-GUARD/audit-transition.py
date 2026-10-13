"""Snapshot existing raw cycle/owner state without native input, IPC or process control."""
import argparse
import hashlib
import importlib.util
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
from transition_contract import evaluate_transition

root = Path(__file__).resolve().parent
evidence = root.parent
cycle_source = evidence / "OCX-0110-20261012-WIN-EAE-REAL-STABLE-PANEL-CYCLE"
runtime = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110")
cycle = runtime / "cycles/eae-stable-panel-20261012"
supervisor_session = runtime / "evidence/run-20261012T145310Z/session.json"
existing_auditor = evidence / "OCX-0110-20261013-WIN-REAL-CYCLE-AUDITOR/methods/audit-cycle.py"
dependency = json.loads((root / "method-dependencies.json").read_text(encoding="utf-8-sig"))
if hashlib.sha256(existing_auditor.read_bytes()).hexdigest() != dependency["existingCycleAuditorSha256"]:
    raise ValueError("Existing frozen cycle auditor differs from its dependency witness")
spec = importlib.util.spec_from_file_location("existing_real_cycle_auditor", existing_auditor)
auditor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(auditor)

def encode(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()

def checked_raw(path):
    for candidate in [path, *path.parents]:
        if candidate.is_symlink() or candidate.stat().st_file_attributes & 0x400:
            raise ValueError("Reparse source rejected")
    return path.read_bytes()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-name", required=True)
    parser.add_argument("--external-snapshot", action="store_true")
    args = parser.parse_args()
    if not args.output_name.replace("-", "").isalnum():
        raise ValueError("Output name must be a single owned directory")
    if (root / "manifest.json").exists() and not args.external_snapshot:
        raise ValueError("Source archive is frozen; use --external-snapshot for a new independent evidence directory")
    for path in [root, evidence, runtime, cycle]:
        if path.is_symlink() or path.stat().st_file_attributes & 0x400:
            raise ValueError("Reparse source/output root rejected")
    output = evidence / ("OCX-0110-TRANSITION-" + args.output_name) if args.external_snapshot else root / args.output_name
    if output.exists():
        raise ValueError("Do not overwrite any earlier transition audit")
    originals = {
        "native-initialization.json": cycle_source / "native-initialization.json",
        "schedule-baseline.json": cycle_source / "schedule-baseline.json",
        "monitor-state.json": cycle / "monitor-state.json",
        "supervisor-session.json": supervisor_session,
        "disk-preferences.json": runtime / "data/manager-state/preferences.json",
    }
    raw_files = {name: checked_raw(path) for name, path in originals.items()}
    raw_observations = checked_raw(cycle / "observations.ndjson")
    last_lf = raw_observations.rfind(b"\n")
    if last_lf < 0:
        raise ValueError("A complete real observation prefix is required")
    raw_files["observations-prefix.ndjson"] = raw_observations[:last_lf + 1]
    observations = [json.loads(line) for line in raw_files["observations-prefix.ndjson"].splitlines() if line.strip()]
    read_raw = lambda name: json.loads(raw_files[name].decode("utf-8-sig"))
    initial = read_raw("native-initialization.json")
    baseline = read_raw("schedule-baseline.json")
    monitor = read_raw("monitor-state.json")
    session = read_raw("supervisor-session.json")
    preferences = read_raw("disk-preferences.json")["updates"]
    cleanup = None
    if (cycle / "native-cleanup.json").exists():
        raw_files["native-cleanup.json"] = checked_raw(cycle / "native-cleanup.json")
        cleanup = read_raw("native-cleanup.json")
    cycle_audit = auditor.evaluate(initial, baseline, observations)
    processes = auditor.actual_processes()
    # A whole-host read-only inventory supplements the exact original process
    # handles. An inaccessible query is an error, never an assumed zero count.
    shell = Path(r"C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe")
    command = (
        "$ErrorActionPreference='Stop'; "
        "$taskApps=@(Get-CimInstance Win32_Process -Filter \"Name='opencodex-desktop.exe'\"); "
        "[ordered]@{count=$taskApps.Count;items=@($taskApps | Select-Object ProcessId,CreationDate,ExecutablePath)} "
        "| ConvertTo-Json -Depth 5 -Compress"
    )
    value = subprocess.check_output(
        [str(shell), "-NoProfile", "-NonInteractive", "-Command", command],
        encoding="utf-8", timeout=30, creationflags=subprocess.CREATE_NO_WINDOW,
    )
    desktops = json.loads(value)
    result = evaluate_transition(
        initial, cycle_audit, monitor, cleanup, session, processes, desktops,
        {"channel": preferences["app_update_channel"], "automaticCheck": preferences["app_update_auto_check"]},
    )
    result.update({
        "auditAtUtc": datetime.now(timezone.utc).isoformat(),
        "observationsAudited": len(observations), "actualOriginalProcessHandles": processes,
        "independentWholeHostDesktopInventory": desktops, "monitorStatus": monitor["status"],
        "monitorFinishedAtUtc": monitor.get("finishedAtUtc"), "supervisorStatus": session["status"],
        "supervisorFinishedAtUtc": session.get("finishedAtUtc"),
        "scope": "read-only prerequisite audit; no stop request, command invocation, focus, timers, routing or app start",
    })
    output.mkdir()
    for name, raw in raw_files.items():
        (output / name).write_bytes(raw)
    (output / "cycle-prefix-audit.json").write_bytes(encode(cycle_audit))
    (output / "actual-processes-and-desktops.json").write_bytes(encode({"originalProcesses": processes, "desktops": desktops}))
    (output / "transition-audit.json").write_bytes(encode(result))
    print(json.dumps({
        "status": result["status"], "blockingConditions": result["blockingConditions"],
        "observationsAudited": len(observations), "desktopCount": desktops["count"],
        "transitionExecuted": False, "fullNativeAccepted": False,
    }))

if __name__ == "__main__":
    main()
