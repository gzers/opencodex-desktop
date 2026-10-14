// Bounded diagnostic observer; no input, security changes, or named-process kill.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Threading;

public sealed class OwnedProcessObserver : IDisposable
{
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct ProcessEntry
    {
        public uint Size, Usage, Pid;
        public UIntPtr Heap;
        public uint Module, Threads, ParentPid;
        public int BasePriority;
        public uint Flags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)] public string Exe;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct FileTime { public uint Low, High; public ulong Ticks => ((ulong)High << 32) | Low; }
    [StructLayout(LayoutKind.Sequential)]
    private struct IoCounters
    {
        public ulong ReadOperations, WriteOperations, OtherOperations;
        public ulong ReadBytes, WriteBytes, OtherBytes;
    }
    [DllImport("kernel32.dll", SetLastError = true)] private static extern IntPtr CreateToolhelp32Snapshot(uint flags, uint pid);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern bool Process32FirstW(IntPtr snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern bool Process32NextW(IntPtr snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern IntPtr OpenProcess(uint access, bool inherit, uint pid);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool GetProcessTimes(IntPtr process, out FileTime create, out FileTime exit, out FileTime kernel, out FileTime user);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool GetProcessIoCounters(IntPtr process, out IoCounters counters);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool GetExitCodeProcess(IntPtr process, out uint code);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern bool QueryFullProcessImageNameW(IntPtr process, uint flags, StringBuilder path, ref uint size);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern IntPtr OpenThread(uint access, bool inherit, uint id);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint GetProcessIdOfThread(IntPtr thread);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool GetThreadTimes(IntPtr thread, out FileTime create, out FileTime exit, out FileTime kernel, out FileTime user);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool QueryThreadCycleTime(IntPtr thread, out ulong cycles);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool CloseHandle(IntPtr handle);
    [DllImport("advapi32.dll", SetLastError = true)] private static extern IntPtr OpenThreadWaitChainSession(uint flags, IntPtr callback);
    [DllImport("advapi32.dll", SetLastError = true)] private static extern void CloseThreadWaitChainSession(IntPtr session);
    [DllImport("advapi32.dll", SetLastError = true)] private static extern bool GetThreadWaitChain(IntPtr session, UIntPtr context, uint flags, uint threadId, ref uint count, IntPtr nodes, out bool cycle);

    private sealed class Owned
    {
        public uint Pid, Parent;
        public IntPtr Handle;
        public ulong Created;
        public string Image;
    }
    private readonly Dictionary<uint, Owned> owned = new Dictionary<uint, Owned>();
    private readonly HashSet<uint> attemptedWaitChain = new HashSet<uint>();
    private readonly string nodePath;
    private readonly ulong boundary;
    private readonly uint rootPid;
    private readonly StreamWriter writer;
    private readonly object writeLock = new object();
    private readonly FileSystemWatcher files;
    private readonly Stopwatch clock = Stopwatch.StartNew();
    private readonly IntPtr wct;
    public int Samples { get; private set; }
    public int ProcessesObserved => owned.Count;
    public bool FileBufferOverflow { get; private set; }
    public double MaximumSampleMs { get; private set; }

    public OwnedProcessObserver(int pid, string expectedImage, string expectedNode, string temporaryRoot, string output)
    {
        rootPid = (uint)pid;
        nodePath = Path.GetFullPath(expectedNode);
        writer = new StreamWriter(new FileStream(output, FileMode.CreateNew, FileAccess.Write, FileShare.Read),
                                  new UTF8Encoding(false)) { AutoFlush = true };
        var process = OpenOwned(rootPid, 0, expectedImage, 0);
        if (process == null) throw new InvalidOperationException("Owned root identity unavailable");
        owned.Add(rootPid, process);
        boundary = process.Created;
        wct = OpenThreadWaitChainSession(0, IntPtr.Zero);
        Write(new { kind = "observer-start", atUtc = DateTime.UtcNow, rootPid, nodePath,
                    rootCreationFileTime100ns = boundary, waitChainSessionOpened = wct != IntPtr.Zero,
                    waitChainSessionError = wct == IntPtr.Zero ? Marshal.GetLastWin32Error() : 0,
                    processScope = "retained root and new descendants matching exact Node image",
                    fileScope = temporaryRoot, schedulingEtwRecorded = false });
        files = new FileSystemWatcher(temporaryRoot)
        {
            IncludeSubdirectories = true,
            NotifyFilter = NotifyFilters.FileName | NotifyFilters.LastWrite | NotifyFilters.CreationTime,
            InternalBufferSize = 65536
        };
        files.Created += (_, e) => FileEvent(e.FullPath, "created", null);
        files.Changed += (_, e) => FileEvent(e.FullPath, "changed", null);
        files.Renamed += (_, e) => FileEvent(e.FullPath, "renamed", e.OldFullPath);
        files.Deleted += (_, e) => FileEvent(e.FullPath, "deleted", null);
        files.Error += (_, e) => { FileBufferOverflow = true; Write(new { kind = "file-watcher-error", elapsedMs = clock.Elapsed.TotalMilliseconds, error = e.GetException().GetType().Name }); };
        files.EnableRaisingEvents = true;
    }
    private void FileEvent(string path, string change, string previous)
    {
        string name = Path.GetFileName(path);
        string[] names = { "mode", "success-output", "metadata-fixture.cjs", "npm.cmd",
            "control-started.json", "control-started.json.pending", "parent-started.json",
            "parent-started.json.pending", "parent-pid", "parent-pid.pending",
            "child-pid", "child-pid.pending", "descendant-started.json",
            "descendant-started.json.pending", "before-child-spawn.json",
            "after-child-spawn.json", "ownership-ready", "ownership-ready-seen.json",
            "before-nonzero-exit.json" };
        if (!names.Contains(name)) return;
        Write(new { kind = "fixture-file-notification", elapsedMs = clock.Elapsed.TotalMilliseconds,
                    atUtc = DateTime.UtcNow, path, change, previous,
                    scope = "notification delivery time; not the file operation start/duration" });
    }
    private void Write(object value)
    {
        lock (writeLock) writer.WriteLine(JsonSerializer.Serialize(value));
    }
    private static Owned OpenOwned(uint pid, uint parent, string expected, ulong notBefore)
    {
        IntPtr handle = OpenProcess(0x1000 | 0x00100000, false, pid);
        if (handle == IntPtr.Zero) return null;
        bool retained = false;
        try
        {
            var image = new StringBuilder(32768); uint size = 32768;
            if (!QueryFullProcessImageNameW(handle, 0, image, ref size)
                || !GetProcessTimes(handle, out var created, out _, out _, out _)
                || created.Ticks < notBefore
                || !String.Equals(Path.GetFullPath(image.ToString()), Path.GetFullPath(expected), StringComparison.OrdinalIgnoreCase))
                return null;
            retained = true;
            return new Owned { Pid = pid, Parent = parent, Handle = handle, Created = created.Ticks, Image = image.ToString() };
        }
        finally { if (!retained) CloseHandle(handle); }
    }
    private void Discover()
    {
        IntPtr snapshot = CreateToolhelp32Snapshot(2, 0);
        if (snapshot == new IntPtr(-1)) { Write(new { kind = "process-enumeration-error", error = Marshal.GetLastWin32Error() }); return; }
        var rows = new List<ProcessEntry>();
        try
        {
            var entry = new ProcessEntry { Size = (uint)Marshal.SizeOf<ProcessEntry>() };
            if (Process32FirstW(snapshot, ref entry))
                do { rows.Add(entry); entry.Size = (uint)Marshal.SizeOf<ProcessEntry>(); } while (Process32NextW(snapshot, ref entry));
        }
        finally { CloseHandle(snapshot); }
        // Some production commands use cmd.exe before Node. Build ancestry only
        // in memory; save identities only for exact owned Node images.
        var descendants = new HashSet<uint> { rootPid };
        for (int depth = 0; depth < 8; depth++)
        {
            int previous = descendants.Count;
            foreach (var row in rows)
                if (descendants.Contains(row.ParentPid)) descendants.Add(row.Pid);
            if (previous == descendants.Count) break;
        }
        foreach (var row in rows)
        {
            if (row.Pid == rootPid || owned.ContainsKey(row.Pid) || !descendants.Contains(row.Pid)) continue;
            if (!String.Equals(row.Exe, "node.exe", StringComparison.OrdinalIgnoreCase)) continue;
            var child = OpenOwned(row.Pid, row.ParentPid, nodePath, boundary);
            if (child == null) continue;
            owned.Add(row.Pid, child);
            Write(new { kind = "owned-node-discovered", elapsedMs = clock.Elapsed.TotalMilliseconds,
                        pid = child.Pid, parentPid = child.Parent, creationFileTime100ns = child.Created,
                        executable = child.Image, imageAndCreationVerified = true });
        }
    }
    private object InspectThreads(Owned item)
    {
        var result = new List<object>();
        try
        {
            using var process = Process.GetProcessById((int)item.Pid);
            foreach (ProcessThread thread in process.Threads)
            {
                try
                {
                    int id = thread.Id;
                    IntPtr handle = OpenThread(0x0800, false, (uint)id);
                    if (handle == IntPtr.Zero) { result.Add(new { id, error = Marshal.GetLastWin32Error() }); continue; }
                    try
                    {
                        if (GetProcessIdOfThread(handle) != item.Pid) { result.Add(new { id, error = "owner-changed" }); continue; }
                        bool times = GetThreadTimes(handle, out var create, out var exit, out var kernel, out var user);
                        int timeError = times ? 0 : Marshal.GetLastWin32Error();
                        bool cyclesOk = QueryThreadCycleTime(handle, out var cycles);
                        int cyclesError = cyclesOk ? 0 : Marshal.GetLastWin32Error();
                        var state = thread.ThreadState;
                        string reason = state == System.Diagnostics.ThreadState.Wait ? thread.WaitReason.ToString() : null;
                        object chain = null;
                        // WCT may require SeDebugPrivilege. Query each observed
                        // thread once, preserve denial, and never enable privileges.
                        if (wct != IntPtr.Zero && attemptedWaitChain.Add((uint)id))
                        {
                            // WAITCHAIN_NODE_INFO: two enums + 528-byte union.
                            int nodeSize = 544;
                            IntPtr buffer = Marshal.AllocHGlobal(16 * nodeSize);
                            try
                            {
                                uint count = 16;
                                bool ok = GetThreadWaitChain(wct, UIntPtr.Zero, 0, (uint)id, ref count, buffer, out var cycle);
                                int error = ok ? 0 : Marshal.GetLastWin32Error();
                                // Keep the first in-process thread node only;
                                // never persist other-process names/identities.
                                int type = ok && count > 0 ? Marshal.ReadInt32(buffer, 0) : 0;
                                int status = ok && count > 0 ? Marshal.ReadInt32(buffer, 4) : 0;
                                chain = new { ok, error, count, cycle, firstObjectType = type, firstObjectStatus = status,
                                              threadWaitTime = type == 8 ? Marshal.ReadInt32(buffer, 16) : (int?)null,
                                              threadContextSwitches = type == 8 ? Marshal.ReadInt32(buffer, 20) : (int?)null,
                                              beyondFirstNodeSaved = false };
                            }
                            finally { Marshal.FreeHGlobal(buffer); }
                        }
                        result.Add(new { id, state = state.ToString(), waitReason = reason,
                                         basePriority = thread.BasePriority, currentPriority = thread.CurrentPriority,
                                         timesOk = times, timesError = timeError,
                                         creationFileTime100ns = times ? create.Ticks : (ulong?)null,
                                         kernelCpu100ns = times ? kernel.Ticks : (ulong?)null,
                                         userCpu100ns = times ? user.Ticks : (ulong?)null,
                                         cyclesOk, cyclesError, cycles = cyclesOk ? cycles : (ulong?)null,
                                         waitChain = chain });
                    }
                    finally { CloseHandle(handle); }
                }
                catch (Exception error) { result.Add(new { threadError = error.GetType().Name }); }
                finally { thread.Dispose(); }
            }
        }
        catch (Exception error) { return new { error = error.GetType().Name }; }
        return result;
    }
    public void Sample()
    {
        var elapsed = Stopwatch.StartNew();
        Discover();
        foreach (var item in owned.Values)
        {
            uint wait = WaitForSingleObject(item.Handle, 0);
            bool times = GetProcessTimes(item.Handle, out var create, out var exit, out var kernel, out var user);
            int timeError = times ? 0 : Marshal.GetLastWin32Error();
            if (times && create.Ticks != item.Created) throw new InvalidOperationException("Retained identity changed");
            bool ioOk = GetProcessIoCounters(item.Handle, out var io);
            int ioError = ioOk ? 0 : Marshal.GetLastWin32Error();
            bool exitOk = GetExitCodeProcess(item.Handle, out var exitCode);
            int exitError = exitOk ? 0 : Marshal.GetLastWin32Error();
            Write(new { kind = "owned-process-sample", sample = Samples, elapsedMs = clock.Elapsed.TotalMilliseconds,
                        atUtc = DateTime.UtcNow, pid = item.Pid, parentPid = item.Parent,
                        creationFileTime100ns = item.Created, retainedHandle = true, waitStatus = wait,
                        timesOk = times, timesError = timeError,
                        exitFileTime100ns = times ? exit.Ticks : (ulong?)null,
                        kernelCpu100ns = times ? kernel.Ticks : (ulong?)null,
                        userCpu100ns = times ? user.Ticks : (ulong?)null,
                        ioOk, ioError, io = ioOk ? new {
                            readOperations = io.ReadOperations, writeOperations = io.WriteOperations, otherOperations = io.OtherOperations,
                            readBytes = io.ReadBytes, writeBytes = io.WriteBytes, otherBytes = io.OtherBytes } : null,
                        exitCodeOk = exitOk, exitError, exitCode = exitOk ? exitCode : (uint?)null,
                        threads = wait == 258 ? InspectThreads(item) : null });
        }
        Samples++;
        MaximumSampleMs = Math.Max(MaximumSampleMs, elapsed.Elapsed.TotalMilliseconds);
    }
    public void Dispose()
    {
        files.EnableRaisingEvents = false;
        files.Dispose();
        if (wct != IntPtr.Zero) CloseThreadWaitChainSession(wct);
        foreach (var item in owned.Values) CloseHandle(item.Handle);
        Write(new { kind = "observer-finished", atUtc = DateTime.UtcNow, elapsedMs = clock.Elapsed.TotalMilliseconds,
                    samples = Samples, ownedProcesses = owned.Count, fileBufferOverflow = FileBufferOverflow,
                    maximumSampleMs = MaximumSampleMs, schedulingEtwRecorded = false,
                    unrelatedProcessDataPersisted = false, processTerminatedByObserver = false });
        writer.Dispose();
    }
}
