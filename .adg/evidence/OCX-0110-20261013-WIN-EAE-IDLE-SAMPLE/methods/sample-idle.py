"""Read-only ten-minute real CPU/working-set sample of the retained candidate.
No launch, terminate, UI input, native check or product file mutation.
"""
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
import os
import statistics
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

evidence = Path(__file__).resolve().parents[1]
base = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110")
data = base / "data"
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
image = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
main_hwnd = 1051698
required_ns = 600_000_000_000
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
psapi = ctypes.WinDLL("psapi", use_last_error=True)
user = ctypes.WinDLL("user32", use_last_error=True)

class FileTime(ctypes.Structure):
    _fields_ = [("low", wt.DWORD), ("high", wt.DWORD)]
    def integer(self):
        return (self.high << 32) | self.low
class ProcessEntry(ctypes.Structure):
    _fields_ = [("dwSize", wt.DWORD), ("cntUsage", wt.DWORD), ("th32ProcessID", wt.DWORD),
                ("th32DefaultHeapID", ctypes.c_size_t), ("th32ModuleID", wt.DWORD),
                ("cntThreads", wt.DWORD), ("th32ParentProcessID", wt.DWORD),
                ("pcPriClassBase", wt.LONG), ("dwFlags", wt.DWORD), ("szExeFile", wt.WCHAR * 260)]
class MemoryCounters(ctypes.Structure):
    _fields_ = [("cb", wt.DWORD), ("PageFaultCount", wt.DWORD),
                ("PeakWorkingSetSize", ctypes.c_size_t), ("WorkingSetSize", ctypes.c_size_t),
                ("QuotaPeakPagedPoolUsage", ctypes.c_size_t), ("QuotaPagedPoolUsage", ctypes.c_size_t),
                ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t), ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                ("PagefileUsage", ctypes.c_size_t), ("PeakPagefileUsage", ctypes.c_size_t)]
kernel.OpenProcess.argtypes = [wt.DWORD, wt.BOOL, wt.DWORD]
kernel.OpenProcess.restype = wt.HANDLE
kernel.CloseHandle.argtypes = [wt.HANDLE]
kernel.CloseHandle.restype = wt.BOOL
kernel.GetProcessTimes.argtypes = [wt.HANDLE, ctypes.POINTER(FileTime), ctypes.POINTER(FileTime),
                                  ctypes.POINTER(FileTime), ctypes.POINTER(FileTime)]
kernel.GetProcessTimes.restype = wt.BOOL
kernel.WaitForSingleObject.argtypes = [wt.HANDLE, wt.DWORD]
kernel.WaitForSingleObject.restype = wt.DWORD
kernel.QueryFullProcessImageNameW.argtypes = [wt.HANDLE, wt.DWORD, wt.LPWSTR, ctypes.POINTER(wt.DWORD)]
kernel.QueryFullProcessImageNameW.restype = wt.BOOL
kernel.GetActiveProcessorCount.argtypes = [wt.WORD]
kernel.GetActiveProcessorCount.restype = wt.DWORD
kernel.GetCurrentProcess.restype = wt.HANDLE
kernel.CreateToolhelp32Snapshot.argtypes = [wt.DWORD, wt.DWORD]
kernel.CreateToolhelp32Snapshot.restype = wt.HANDLE
kernel.Process32FirstW.argtypes = [wt.HANDLE, ctypes.POINTER(ProcessEntry)]
kernel.Process32NextW.argtypes = [wt.HANDLE, ctypes.POINTER(ProcessEntry)]
kernel.Process32FirstW.restype = kernel.Process32NextW.restype = wt.BOOL
psapi.GetProcessMemoryInfo.argtypes = [wt.HANDLE, ctypes.POINTER(MemoryCounters), wt.DWORD]
psapi.GetProcessMemoryInfo.restype = wt.BOOL
user.IsWindow.argtypes = [wt.HWND]
user.IsWindow.restype = wt.BOOL
user.IsWindowVisible.argtypes = [wt.HWND]
user.IsWindowVisible.restype = wt.BOOL
user.IsIconic.argtypes = [wt.HWND]
user.IsIconic.restype = wt.BOOL
user.GetDpiForWindow.argtypes = [wt.HWND]
user.GetDpiForWindow.restype = wt.UINT
user.GetWindowThreadProcessId.argtypes = [wt.HWND, ctypes.POINTER(wt.DWORD)]
user.GetWindowThreadProcessId.restype = wt.DWORD
user.GetForegroundWindow.restype = wt.HWND

def utc():
    return datetime.now(timezone.utc).isoformat()
def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))
def write(name, value):
    path = evidence / name
    pending = path.with_suffix(path.suffix + ".pending")
    pending.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
    os.replace(pending, path)
def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def state_hashes():
    return {name: digest(data / "manager-state" / name)
            for name in ["preferences.json", "data-root.json", "update-schedule.json", "backup-cleanup-policy.json"]}
def raw_times(handle):
    created, exited, cpu_kernel, cpu_user = FileTime(), FileTime(), FileTime(), FileTime()
    if not kernel.GetProcessTimes(handle, ctypes.byref(created), ctypes.byref(exited),
                                  ctypes.byref(cpu_kernel), ctypes.byref(cpu_user)):
        raise ctypes.WinError(ctypes.get_last_error())
    return {"creation100ns": created.integer(), "exit100ns": exited.integer(),
            "kernel100ns": cpu_kernel.integer(), "user100ns": cpu_user.integer(),
            "totalCpu100ns": cpu_kernel.integer() + cpu_user.integer()}
def creation_utc(ticks):
    return datetime.fromtimestamp((ticks - 116444736000000000) / 10_000_000, timezone.utc).isoformat()
def image_path(handle):
    chars = wt.DWORD(32768)
    buffer = ctypes.create_unicode_buffer(chars.value)
    if not kernel.QueryFullProcessImageNameW(handle, 0, buffer, ctypes.byref(chars)):
        raise ctypes.WinError(ctypes.get_last_error())
    return buffer.value
def snapshot():
    handle = kernel.CreateToolhelp32Snapshot(2, 0)
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    entries = []
    try:
        item = ProcessEntry()
        item.dwSize = ctypes.sizeof(item)
        success = kernel.Process32FirstW(handle, ctypes.byref(item))
        while success:
            entries.append({"pid": item.th32ProcessID, "parentPid": item.th32ParentProcessID,
                            "exeName": str(item.szExeFile)})
            success = kernel.Process32NextW(handle, ctypes.byref(item))
        if ctypes.get_last_error() not in {0, 18}:
            raise ctypes.WinError(ctypes.get_last_error())
        return entries
    finally:
        kernel.CloseHandle(handle)
def live(handle):
    result = kernel.WaitForSingleObject(handle, 0)
    if result == 0:
        return False
    if result == 258:
        return True
    raise ctypes.WinError(ctypes.get_last_error())
def memory(handle):
    value = MemoryCounters()
    value.cb = ctypes.sizeof(value)
    if not psapi.GetProcessMemoryInfo(handle, ctypes.byref(value), value.cb):
        raise ctypes.WinError(ctypes.get_last_error())
    return {"workingSetBytes": value.WorkingSetSize, "peakWorkingSetBytes": value.PeakWorkingSetSize,
            "pagefileUsageBytes": value.PagefileUsage, "pageFaultCount": value.PageFaultCount}
def window():
    owner = wt.DWORD()
    assert user.IsWindow(main_hwnd)
    assert user.GetWindowThreadProcessId(main_hwnd, ctypes.byref(owner)) and owner.value == 37276
    return {"hwnd": main_hwnd, "ownerPid": owner.value, "dpi": user.GetDpiForWindow(main_hwnd),
            "visible": bool(user.IsWindowVisible(main_hwnd)), "minimized": bool(user.IsIconic(main_hwnd)),
            "foreground": user.GetForegroundWindow() == main_hwnd}

session = read(base / "control/current-session.json")
assert session["status"] == "running" and session["sourceCommit"] == source and session["applicationPid"] == 37276
assert session["executableSha256"] == image and digest(Path(session["executable"])) == image
assert session["applicationToken"]["elevated"] is False and session["applicationToken"]["elevationType"] == 3
expected_creation = int(datetime(2026, 10, 12, 14, 53, 12, tzinfo=timezone.utc).timestamp()) * 10_000_000 + 88010 + 116444736000000000
logical_count = kernel.GetActiveProcessorCount(0xffff)
assert logical_count > 0
nodes = {}
attach_events = []
candidate_open_races = []
start_ns = None
start_wall100ns = None
cpu_baselines = {}
seen_samples = []

def attach(pid, parent=None, initial=False):
    handle = kernel.OpenProcess(0x101010, False, pid)
    if not handle:
        error = ctypes.get_last_error()
        if error in {87, 1168}:
            candidate_open_races.append({"atUtc": utc(), "pid": pid, "code": error, "scope": "candidate exited before handle acquisition"})
            return False
        raise ctypes.WinError(error)
    try:
        times = raw_times(handle)
        if parent:
            parent_times = raw_times(parent["handle"])
            if times["creation100ns"] < parent_times["creation100ns"] or (
                    parent_times["exit100ns"] and times["creation100ns"] > parent_times["exit100ns"]):
                candidate_open_races.append({"atUtc": utc(), "pid": pid, "scope": "parent creation/exit boundary mismatch"})
                return False
        path = image_path(handle)
        key = (pid, times["creation100ns"])
        if key in nodes:
            return False
        node = {"pid": pid, "parentPid": parent["pid"] if parent else None, "creation100ns": times["creation100ns"],
                "creationUtc": creation_utc(times["creation100ns"]), "path": path, "handle": handle}
        if pid == 37276:
            assert times["creation100ns"] == expected_creation
            assert Path(path) == Path(session["executable"])
        else:
            assert parent
        nodes[key] = node
        # Initial population baseline: observe CPU after attachment, then establish
        # every initial counter together immediately before the first sample.
        late_preexisting = not initial and start_wall100ns is not None and times["creation100ns"] < start_wall100ns
        cpu_baselines[key] = times["totalCpu100ns"] if initial or late_preexisting else 0
        attach_events.append({"atUtc": utc(), "pid": pid, "parentPid": node["parentPid"],
                              "creationUtc": node["creationUtc"], "path": path,
                              "initialPopulation": initial, "lateDiscoveredPreexisting": late_preexisting,
                              "liveHandleAcquired": True})
        handle = None
        return True
    finally:
        if handle:
            kernel.CloseHandle(handle)

def discover(initial=False):
    entries = snapshot()
    changed = True
    while changed:
        changed = False
        for entry in entries:
            if entry["pid"] == 37276:
                continue
            if any(node["pid"] == entry["pid"] for node in nodes.values()):
                continue
            parents = [node for node in nodes.values() if node["pid"] == entry["parentPid"]]
            if len(parents) != 1:
                continue
            if attach(entry["pid"], parents[0], initial):
                changed = True

def collect():
    discover()
    active, exited, total_cpu = [], [], 0
    for key, node in nodes.items():
        times = raw_times(node["handle"])
        assert times["creation100ns"] == node["creation100ns"]
        delta = times["totalCpu100ns"] - cpu_baselines[key]
        assert delta >= 0
        total_cpu += delta
        record = {"pid": node["pid"], "parentPid": node["parentPid"], "creationUtc": node["creationUtc"],
                  "image": Path(node["path"]).name, "cpuTotal100ns": times["totalCpu100ns"],
                  "cpuSinceSampleStart100ns": delta}
        if live(node["handle"]):
            record.update(memory(node["handle"]))
            active.append(record)
        else:
            record["exit100ns"] = times["exit100ns"]
            exited.append(record)
    assert any(item["pid"] == 37276 for item in active), "Original application exited; do not restart"
    return {"atUtc": utc(), "elapsedNs": time.perf_counter_ns() - start_ns, "familyCpuSinceStart100ns": total_cpu,
            "workingSetFamilySumBytes": sum(item["workingSetBytes"] for item in active),
            "rootWorkingSetBytes": next(item["workingSetBytes"] for item in active if item["pid"] == 37276),
            "activeObservedProcesses": active, "exitedObservedProcesses": exited, "window": window()}

report = {"sourceCommit": source, "applicationPid": 37276, "executableSha256": image,
          "requiredElapsedSeconds": 600, "sampleIntervalSeconds": 1, "status": "preparing",
          "samplerPid": os.getpid(), "samplerCreationUtc": creation_utc(raw_times(kernel.GetCurrentProcess())["creation100ns"]),
          "logicalProcessorCountAllGroups": logical_count,
          "allProcessLifetimeCountAccepted": False, "queryOrNetworkCountsAccepted": False,
          "pairedPerformanceAccepted": False, "completeIdleGateAccepted": False,
          "fullNativeGateAccepted": False, "realCyclesAccepted": False, "releaseAccepted": False}
mode = sys.argv[1] if len(sys.argv) == 2 else "sample"
assert mode in {"preflight", "sample"}
initial_hashes = state_hashes()
initial_daily = digest(Path(r"D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe"))
assert initial_daily == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
try:
    assert attach(37276, initial=True)
    discover(initial=True)
    for key, node in nodes.items():
        cpu_baselines[key] = raw_times(node["handle"])["totalCpu100ns"]
    start_ns = time.perf_counter_ns()
    start_wall100ns = time.time_ns() // 100 + 116444736000000000
    if mode == "preflight":
        first = collect()
        write("sampler-preflight.json", {**report, "status": "real-readonly-apis-preflight", "sample": first,
                                       "attachedActualProcesses": attach_events, "stateHashes": initial_hashes})
        print(json.dumps({"preflight": "passed", "ownedProcesses": len(nodes), "logicalProcessors": logical_count}))
    else:
        assert not (evidence / "resource-samples.ndjson").exists(), "Do not overwrite a prior window"
        report["startedAtUtc"] = utc()
        report["stateHashesBefore"] = initial_hashes
        report["status"] = "sampling-real-ten-minute-window"
        write("sampler-state.json", report)
        print(json.dumps({"status": report["status"], "samplerPid": os.getpid(), "requiredSeconds": 600}), flush=True)
        with (evidence / "resource-samples.ndjson").open("x", encoding="utf-8", newline="\n") as output:
            while True:
                tick_start = time.perf_counter_ns()
                sample = collect()
                sample["sampleIndex"] = len(seen_samples)
                output.write(json.dumps(sample, ensure_ascii=False) + "\n")
                output.flush()
                seen_samples.append(sample)
                report["samples"] = len(seen_samples)
                report["lastElapsedNs"] = sample["elapsedNs"]
                report["lastSampleAtUtc"] = sample["atUtc"]
                if len(seen_samples) % 10 == 0:
                    write("sampler-state.json", report)
                if sample["elapsedNs"] - seen_samples[0]["elapsedNs"] >= required_ns:
                    break
                elapsed = time.perf_counter_ns() - tick_start
                time.sleep(max(0.01, (1_000_000_000 - elapsed) / 1_000_000_000))
        first, last = seen_samples[0], seen_samples[-1]
        elapsed_seconds = (last["elapsedNs"] - first["elapsedNs"]) / 1_000_000_000
        cpu_seconds = (last["familyCpuSinceStart100ns"] - first["familyCpuSinceStart100ns"]) / 10_000_000
        assert elapsed_seconds >= 600 and cpu_seconds >= 0
        windows = sorted({json.dumps(item["window"], sort_keys=True) for item in seen_samples})
        report.update({"status": "sampled-real-ten-minute-window", "finishedAtUtc": utc(),
                       "actualObservedElapsedSeconds": elapsed_seconds,
                       "observedFamilyCpuSeconds": cpu_seconds,
                       "observedFamilyCpuSingleLogicalProcessorPercent": cpu_seconds / elapsed_seconds * 100,
                       "observedFamilyCpuAllLogicalProcessorsPercent": cpu_seconds / elapsed_seconds * 100 / logical_count,
                       "familySummedWorkingSetBytesMean": statistics.fmean(item["workingSetFamilySumBytes"] for item in seen_samples),
                       "familySummedWorkingSetBytesMax": max(item["workingSetFamilySumBytes"] for item in seen_samples),
                       "rootWorkingSetBytesMean": statistics.fmean(item["rootWorkingSetBytes"] for item in seen_samples),
                       "rootWorkingSetBytesMax": max(item["rootWorkingSetBytes"] for item in seen_samples),
                       "actualWindowStates": [json.loads(item) for item in windows],
                       "observedDistinctProcessHandles": len(nodes),
                       "candidateOpenRaces": candidate_open_races,
                       "attachedActualProcesses": attach_events,
                       "stateHashesAfter": state_hashes(),
                       "dailyExecutableSha256After": digest(Path(r"D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe")),
                       "transientProcessesBetweenSnapshots": "not proven absent; no complete lifetime-count claim",
                       "workingSetSumSharedPageCaveat": "arithmetic sum, not unique physical resident memory"})
        assert report["stateHashesAfter"] == initial_hashes
        assert report["dailyExecutableSha256After"] == initial_daily
        write("sampler-state.json", report)
        print(json.dumps({key: report[key] for key in ["status", "samples", "actualObservedElapsedSeconds",
                                                      "observedFamilyCpuSeconds", "observedDistinctProcessHandles"]}), flush=True)
except BaseException as error:
    report.update({"status": "failed-or-interrupted-sample", "error": repr(error), "finishedAtUtc": utc()})
    write("sampler-state.json", report)
    raise
finally:
    for node in nodes.values():
        kernel.CloseHandle(node["handle"])
