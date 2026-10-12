"""Bounded read-only controller of six production commands with independent OS lock.
Does not launch another desktop, inject IPC responses or kill by process name.
"""
import ctypes
import ctypes.wintypes as wt
import json
import os
import subprocess
import sys
import time
from pathlib import Path

evidence = Path(__file__).resolve().parents[1]
methods = evidence / "methods"
control = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110\control\native-w2-lock-20261013")
node = Path(r"C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe")
query = Path(r"C:\Users\15119\.codex\tmp\ocx-0110-cdp-query.mjs")
flags = subprocess.CREATE_NO_WINDOW
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
class FileTime(ctypes.Structure):
    _fields_ = [("low", wt.DWORD), ("high", wt.DWORD)]
    def integer(self):
        return (self.high << 32) | self.low
kernel.GetProcessTimes.argtypes = [wt.HANDLE, ctypes.POINTER(FileTime), ctypes.POINTER(FileTime),
                                  ctypes.POINTER(FileTime), ctypes.POINTER(FileTime)]
kernel.WaitForSingleObject.argtypes = [wt.HANDLE, wt.DWORD]
kernel.WaitForSingleObject.restype = wt.DWORD
def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))
def write(name, value):
    (evidence / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
def native(script, output):
    destination = evidence / (output + ".json")
    assert not destination.exists()
    with destination.open("wb") as stdout, (evidence / (output + ".stderr.txt")).open("wb") as stderr:
        result = subprocess.run([str(node), str(query), str(methods / script)],
                                stdout=stdout, stderr=stderr, timeout=40, creationflags=flags)
    assert result.returncode == 0, f"Native call failed: {output}; original stderr retained"
    value = read(destination)
    assert value["sourceCommit"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc" and value["applicationPid"] == 37276
    return value["result"]
def filesystem(mode, output, phase=None):
    assert not (evidence / (output + ".stdout.txt")).exists()
    argv = [sys.executable, str(methods / "filesystem.py"), mode]
    if phase:
        argv.append(phase)
    with (evidence / (output + ".stdout.txt")).open("wb") as stdout, (evidence / (output + ".stderr.txt")).open("wb") as stderr:
        result = subprocess.run(argv, stdout=stdout, stderr=stderr, timeout=35, creationflags=flags)
    assert result.returncode == 0, f"Filesystem method failed: {output}; original stderr retained"
def process_times(process):
    created, ended, cpu_kernel, cpu_user = FileTime(), FileTime(), FileTime(), FileTime()
    handle = wt.HANDLE(int(process._handle))
    if not kernel.GetProcessTimes(handle, ctypes.byref(created), ctypes.byref(ended),
                                  ctypes.byref(cpu_kernel), ctypes.byref(cpu_user)):
        raise ctypes.WinError(ctypes.get_last_error())
    state = kernel.WaitForSingleObject(handle, 0)
    assert state in {0, 258}
    return {"pid": process.pid, "creationFileTime100ns": created.integer(),
            "exitFileTime100ns": ended.integer(), "liveHandleVerified": state == 258}

control.mkdir(exist_ok=True)
assert not list(control.glob("*")), "Never restart or overwrite an existing controller session"
report = {"sourceCommit": "eae85322ebca3b755c034db97cbebf2ea5bf71fc", "applicationPid": 37276,
          "controllerPid": os.getpid(), "phases": [], "status": "preparing",
          "uiInputPerformed": False, "productRestarted": False, "secondDesktopStarted": False,
          "fullNativeGateAccepted": False, "pairedPerformanceAccepted": False,
          "realCyclesAccepted": False, "releaseAccepted": False}
setup_attempted = False
try:
    filesystem("before", "before")
    setup_attempted = True
    native("setup.js", "setup")
    for phase in ["create", "list", "pin", "policy", "restore", "cleanup"]:
        prepared = native("prepare-phase.js", phase + "-prepared")
        assert prepared["phase"] == phase
        if phase == "cleanup":
            filesystem("age-cleanup", "cleanup-age")
            native("cleanup-preview.js", "cleanup-preview")
        ready = control / (phase + "-ready.json")
        release = control / (phase + "-release.request")
        assert not ready.exists() and not release.exists()
        phase_report = {"phase": phase, "status": "starting-independent-lock-holder"}
        report["phases"].append(phase_report)
        report["status"] = "running-real-native-lock-scenes"
        write("controller-state.json", report)
        with (evidence / (phase + "-holder.stdout.txt")).open("wb") as holder_out, \
             (evidence / (phase + "-holder.stderr.txt")).open("wb") as holder_err, \
             (evidence / (phase + "-result.json")).open("wb") as command_out, \
             (evidence / (phase + "-result.stderr.txt")).open("wb") as command_err:
            holder = subprocess.Popen([sys.executable, str(methods / "filesystem.py"), "hold", phase],
                                      stdout=holder_out, stderr=holder_err, creationflags=flags)
            command = None
            try:
                deadline = time.monotonic() + 8
                while not ready.exists():
                    assert holder.poll() is None, "Independent holder exited before acquiring lock"
                    assert time.monotonic() < deadline, "Lock acquisition witness not ready"
                    time.sleep(0.05)
                identity = read(ready)
                assert identity["realWindowsExclusiveLockHeld"] and identity["phase"] == phase
                actual = process_times(holder)
                assert actual["liveHandleVerified"] and actual["pid"] == identity["helperPid"]
                assert actual["creationFileTime100ns"] == identity["helperCreationFileTime100ns"]
                phase_report["holderInitialLiveHandle"] = actual
                phase_report["holderToken"] = identity["helperToken"]
                command = subprocess.Popen([str(node), str(query), str(methods / "begin.js")],
                                           stdout=command_out, stderr=command_err, creationflags=flags)
                phase_report["nativeCallerInitialLiveHandle"] = process_times(command)
                deadline = time.monotonic() + 8
                while True:
                    first = native("observe.js", phase + "-pending-" + str(phase_report.get("pendingObservations", 0)))
                    phase_report["pendingObservations"] = phase_report.get("pendingObservations", 0) + 1
                    assert holder.poll() is None and command.poll() is None, "Command or holder terminal before lock release"
                    if first["operation"] is not None:
                        assert first["operation"]["phase"] == phase and first["operation"]["status"] == "pending"
                        break
                    assert time.monotonic() < deadline, "Native command never entered pending state"
                    time.sleep(0.05)
                time.sleep(1)
                second = native("observe.js", phase + "-pending-stable")
                assert second["operation"]["status"] == "pending" and second["operation"]["phase"] == phase
                assert second["atMs"] - first["atMs"] >= 1000
                assert second["operation"]["startedAtMs"] == first["operation"]["startedAtMs"]
                phase_report["actualWebviewPendingMs"] = second["atMs"] - first["atMs"]
                filesystem("during", phase + "-during", phase)
                during = read(evidence / (phase + "-while-held-files.json"))
                before = identity["filesAtLockAcquisition"]
                assert during["backupFiles"] == before["backupFiles"]
                assert during["stateHashes"] == before["stateHashes"]
                phase_report["backupPreferenceRootPolicyScheduleUnchangedWhileHeld"] = True
                phase_report["holderBeforeReleaseLiveHandle"] = process_times(holder)
                phase_report["nativeCallerBeforeReleaseLiveHandle"] = process_times(command)
                assert phase_report["holderBeforeReleaseLiveHandle"]["liveHandleVerified"]
                assert phase_report["nativeCallerBeforeReleaseLiveHandle"]["liveHandleVerified"]
            finally:
                # Natural unlock always happens before trying to observe the command terminal.
                # No PID/name termination is used to manufacture successful completion.
                if not release.exists():
                    release.write_bytes(b"release-owned-cooperative-lock\n")
                phase_report["holderExitCode"] = holder.wait(timeout=35)
            if command is not None:
                phase_report["nativeCallerExitCode"] = command.wait(timeout=40)
                phase_report["holderTerminalHandle"] = process_times(holder)
                phase_report["nativeCallerTerminalHandle"] = process_times(command)
                assert phase_report["holderExitCode"] == phase_report["nativeCallerExitCode"] == 0
                assert not phase_report["holderTerminalHandle"]["liveHandleVerified"]
                assert not phase_report["nativeCallerTerminalHandle"]["liveHandleVerified"]
        result = read(evidence / (phase + "-result.json"))
        assert result["sourceCommit"] == report["sourceCommit"] and result["applicationPid"] == 37276
        assert result["result"]["operation"]["phase"] == phase
        assert result["result"]["operation"]["status"] == "completed"
        assert result["result"]["nativeBridgeUnchanged"]
        phase_report["status"] = "actual-native-command-completed-after-independent-release"
        phase_report["actualNativeElapsedMs"] = result["result"]["operation"]["finishedAtMs"] - result["result"]["operation"]["startedAtMs"]
        assert phase_report["actualNativeElapsedMs"] >= phase_report["actualWebviewPendingMs"]
        write("controller-state.json", report)
        print(json.dumps({"phase": phase, "status": phase_report["status"],
                          "actualPendingMs": phase_report["actualWebviewPendingMs"]}), flush=True)
    report["status"] = "six-real-native-lock-phases-completed"
except BaseException as error:
    report["status"] = "failed-or-interrupted"
    report["error"] = repr(error)
    raise
finally:
    try:
        if (evidence / "cleanup-date-journal.json").exists():
            filesystem("restore-cleanup", "cleanup-dates-restore")
    finally:
        if setup_attempted:
            native("finish.js", "native-finish")
        filesystem("after", "after")
        write("controller-state.json", report)
print(json.dumps({"status": report["status"], "actualPhases": len(report["phases"])}), flush=True)
