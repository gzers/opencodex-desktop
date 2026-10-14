// A one-shot, real-time diagnostic. ETW is enabled only by RunOnce, never by
// construction or SelfTest. Kernel events are transiently decoded in memory;
// only the explicitly owned family is serialized, without other-party IDs.
using System;
using System.Collections.Generic;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Runtime.InteropServices;
using Microsoft.Diagnostics.Tracing;
using Microsoft.Diagnostics.Tracing.Parsers;
using Microsoft.Diagnostics.Tracing.Parsers.Kernel;
using Microsoft.Diagnostics.Tracing.Session;

namespace OpenCodex.Diagnostics
{
    // Generation intervals use original ETW QPC timestamps. Never use delayed
    // callback arrival or a bare PID/TID as process identity.
    public sealed class OwnedEtwScope
    {
        private sealed class Life
        {
            public int Id, Parent;
            public long Start, End = long.MaxValue;
            public string Role;
        }
        private readonly List<Life> processes = new List<Life>();
        private readonly List<Life> threads = new List<Life>();
        private readonly int root;
        private readonly long notBefore;
        private bool rootBound;
        public OwnedEtwScope(int rootPid, long minimumQpc)
        {
            root = rootPid;
            notBefore = minimumQpc;
        }
        private static Life At(List<Life> list, int id, long qpc) =>
            list.LastOrDefault(item => item.Id == id && item.Start <= qpc && qpc < item.End);
        private static void Replace(List<Life> list, int id, long qpc)
        {
            foreach (var item in list.Where(item => item.Id == id && item.Start <= qpc && qpc < item.End))
                item.End = qpc;
        }
        public bool ProcessStart(int pid, int parentPid, long qpc, bool isRootImage, string admittedRole)
        {
            Replace(processes, pid, qpc); // An unrelated reuse also expires ownership.
            if (qpc < notBefore) return false;
            if (pid == root)
            {
                if (!isRootImage || rootBound) return false;
                rootBound = true;
                processes.Add(new Life { Id = pid, Parent = parentPid, Start = qpc, Role = "test" });
                return true;
            }
            if (At(processes, parentPid, qpc) == null || admittedRole == null) return false;
            processes.Add(new Life { Id = pid, Parent = parentPid, Start = qpc, Role = admittedRole });
            return true;
        }
        public bool ProcessStop(int pid, long qpc)
        {
            var item = At(processes, pid, qpc);
            if (item == null) return false;
            item.End = qpc + 1;
            foreach (var thread in threads.Where(thread => thread.Parent == pid
                && thread.Start <= qpc && qpc < thread.End))
                thread.End = qpc + 1;
            return true;
        }
        public bool ProcessOwned(int pid, long qpc) => At(processes, pid, qpc) != null;
        public string Role(int pid, long qpc) => At(processes, pid, qpc)?.Role;
        public long? ProcessGeneration(int pid, long qpc) => At(processes, pid, qpc)?.Start;
        public bool ThreadStart(int pid, int tid, long qpc)
        {
            Replace(threads, tid, qpc);
            if (!ProcessOwned(pid, qpc)) return false;
            threads.Add(new Life { Id = tid, Parent = pid, Start = qpc });
            return true;
        }
        public bool ThreadStop(int tid, long qpc)
        {
            var item = At(threads, tid, qpc);
            if (item == null) return false;
            item.End = qpc + 1;
            return true;
        }
        public int? ThreadOwner(int tid, long qpc)
        {
            var item = At(threads, tid, qpc);
            return item != null && ProcessOwned(item.Parent, qpc) ? item.Parent : (int?)null;
        }
        public static int SelfTest()
        {
            int checks = 0;
            Action<bool> require = value => { if (!value) throw new InvalidOperationException("Owned ETW scope contract"); checks++; };
            var scope = new OwnedEtwScope(100, 1000);
            require(!scope.ProcessStart(999, 1, 900, false, "node"));
            require(!scope.ProcessStart(100, 1, 1000, false, null));
            require(scope.ProcessStart(100, 1, 1001, true, null));
            require(scope.ProcessStart(101, 100, 1002, false, "command"));
            require(scope.ProcessStart(102, 101, 1003, false, "node"));
            require(!scope.ProcessStart(103, 999, 1004, false, "node"));
            require(!scope.ProcessStart(104, 100, 1004, false, null));
            require(scope.ThreadStart(102, 200, 1005));
            require(!scope.ThreadStart(103, 201, 1005));
            require(scope.ThreadOwner(200, 1006) == 102);
            require(scope.ThreadOwner(201, 1006) == null);
            require(scope.ProcessStop(102, 1010));
            require(scope.ProcessOwned(102, 1006)); // Delayed ETW delivery is valid.
            require(!scope.ProcessOwned(102, 1011));
            require(scope.ThreadOwner(200, 1011) == null);
            require(!scope.ProcessStart(102, 999, 1020, false, "node"));
            require(!scope.ProcessOwned(102, 1021));
            require(scope.ProcessStart(105, 100, 1030, false, "node"));
            require(scope.ThreadStart(105, 200, 1031));
            require(scope.ThreadOwner(200, 1032) == 105);
            require(scope.ThreadOwner(200, 1006) == 102); // Old generation remains distinct.
            require(scope.ThreadStop(200, 1040));
            require(scope.ThreadOwner(200, 1041) == null);
            require(scope.ProcessStop(105, 1050));
            require(!scope.ProcessStop(999, 1050));
            require(scope.Role(100, 1100) == "test");
            require(scope.ProcessStart(106, 100, 1110, false, "node"));
            require(scope.ThreadStart(106, 205, 1111));
            require(scope.ProcessStop(106, 1120));
            require(scope.ProcessStart(106, 100, 1130, false, "node"));
            require(scope.ThreadOwner(205, 1131) == null);
            require(scope.ProcessGeneration(106, 1115) != scope.ProcessGeneration(106, 1131));
            require(scope.ThreadStart(106, 206, 1132));
            require(!scope.ThreadStart(999, 206, 1140));
            require(scope.ThreadOwner(206, 1141) == null);
            require(scope.ThreadOwner(206, 1135) == 106);
            require(scope.ProcessStop(100, 1150));
            require(!scope.ProcessStart(100, 1, 1160, true, null));
            return checks;
        }
    }

    public static class OwnedEtwRecorder
    {
        // Preserve the original ETW QPC domain for lifecycle and PID-reuse
        // correlation. TraceEvent marks this API discouraged, not unsupported.
        private static long OriginalQpc(TraceEvent data)
        {
#pragma warning disable CS0618
            return data.TimeStampQPC;
#pragma warning restore CS0618
        }
        private sealed class PendingIo
        {
            public int Pid, Tid;
            public long Qpc, Generation;
        }
        [DllImport("advapi32.dll", SetLastError = true)]
        private static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
        [DllImport("advapi32.dll", SetLastError = true)]
        private static extern bool GetTokenInformation(IntPtr token, int type, out int value, int size, out int returned);
        [DllImport("kernel32.dll")]
        private static extern bool CloseHandle(IntPtr handle);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr OpenProcess(uint access, bool inherit, int pid);
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern bool QueryFullProcessImageNameW(IntPtr process, uint flags, StringBuilder path, ref uint size);
        [StructLayout(LayoutKind.Sequential)]
        private struct NativeTime
        {
            public uint Low, High;
            public ulong Ticks => ((ulong)High << 32) | Low;
        }
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetProcessTimes(IntPtr process, out NativeTime created, out NativeTime exit,
                                                  out NativeTime kernel, out NativeTime user);
        private static object RetainIdentity(int pid, string expectedImage, DateTime originalEventUtc,
                                             List<IntPtr> handles)
        {
            IntPtr handle = OpenProcess(0x1000 | 0x00100000, false, pid);
            if (handle == IntPtr.Zero)
                return new { retained = false, error = Marshal.GetLastWin32Error(),
                    identity = "ETW lifecycle only; no live API handle observation" };
            var path = new StringBuilder(32768);
            uint size = 32768;
            bool ok = QueryFullProcessImageNameW(handle, 0, path, ref size)
                && GetProcessTimes(handle, out _, out _, out _, out _);
            if (!ok || !SamePath(path.ToString(), expectedImage)
                || !GetProcessTimes(handle, out var creation, out var exit, out var kernel, out var user))
            {
                int error = Marshal.GetLastWin32Error();
                CloseHandle(handle);
                return new { retained = false, error, imageMatches = false,
                    identity = "API identity unavailable or changed; ETW record is not relabelled as a live handle" };
            }
            handles.Add(handle);
            return new { retained = true, fullImageMatches = true, creationFileTime100ns = creation.Ticks,
                exitFileTime100ns = exit.Ticks, kernelCpu100ns = kernel.Ticks, userCpu100ns = user.Ticks,
                originalEventUtc, apiObservedUtc = DateTime.UtcNow,
                creationVsOriginalEventDeltaMs =
                    (DateTime.FromFileTimeUtc((long)creation.Ticks) - originalEventUtc).TotalMilliseconds,
                sameEventLifetimeRequiresTimestampReview = true };
        }
        private static object ProcessToken(Process process)
        {
            if (!OpenProcessToken(process.Handle, 8, out var token))
                throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
            try
            {
                if (!GetTokenInformation(token, 20, out int elevated, 4, out _)
                    || !GetTokenInformation(token, 18, out int type, 4, out _))
                    throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
                return new { elevated = elevated != 0, elevationType = type, pid = process.Id };
            }
            finally { CloseHandle(token); }
        }
        private static object Header(TraceEvent data) => new {
            qpc = OriginalQpc(data), atUtc = data.TimeStamp.ToUniversalTime(),
            provider = data.ProviderGuid, eventId = (int)data.ID,
            version = data.Version, opcode = (int)data.Opcode,
            eventName = data.EventName
        };
        private static string ExecutableArgument(string commandLine)
        {
            if (String.IsNullOrWhiteSpace(commandLine)) return "";
            commandLine = commandLine.TrimStart();
            if (commandLine[0] == '"')
            {
                int end = commandLine.IndexOf('"', 1);
                return end < 0 ? "" : commandLine.Substring(1, end - 1);
            }
            int space = commandLine.IndexOf(' ');
            return space < 0 ? commandLine : commandLine.Substring(0, space);
        }
        private static bool SamePath(string first, string second)
        {
            try {
                first = first.StartsWith(@"\\?\", StringComparison.Ordinal) ? first.Substring(4) : first;
                second = second.StartsWith(@"\\?\", StringComparison.Ordinal) ? second.Substring(4) : second;
                return String.Equals(Path.GetFullPath(first), Path.GetFullPath(second), StringComparison.OrdinalIgnoreCase);
            }
            catch { return false; }
        }
        private static string SafeFile(string path, string node, string temp)
        {
            // Omit filenames outside the fixture/known binary/Windows loader
            // paths even when the I/O was issued by our test process.
            if (String.IsNullOrEmpty(path)) return null;
            string windows = Environment.GetFolderPath(Environment.SpecialFolder.Windows);
            if (path.StartsWith(temp.TrimEnd('\\') + "\\", StringComparison.OrdinalIgnoreCase)
                || SamePath(path, node)
                || path.StartsWith(windows.TrimEnd('\\') + "\\System32\\", StringComparison.OrdinalIgnoreCase))
                return path;
            return null;
        }
        private static void Configure(ProcessStartInfo info, string node, string temp, string home)
        {
            info.UseShellExecute = false;
            info.CreateNoWindow = true;
            info.RedirectStandardOutput = true;
            info.RedirectStandardError = true;
            info.Environment["OPENCODEX_TEST_NODE"] = node;
            info.Environment["TEMP"] = temp;
            info.Environment["TMP"] = temp;
            info.Environment["HOME"] = home;
            info.Environment["USERPROFILE"] = home;
            foreach (string name in new[] { "RUST_TEST_THREADS", "OPENCODEX_SANDBOX_ROOT",
                "OPENCODEX_SANDBOX_BOUNDARY", "OPENCODEX_WINDOWS_SMOKE_CDP_PORT" })
                info.Environment.Remove(name);
        }
        // This entry point must be explicitly authorized by the caller. It
        // always runs the original binary once, with no filter or CLI options.
        public static string RunOnce(string testExe, string nodeExe, string workingDirectory,
                                     string outputDirectory, string temporaryDirectory,
                                     string homeDirectory, string sessionName)
        {
            if (TraceEventSession.IsElevated() != true)
                throw new UnauthorizedAccessException("Kernel ETW capture requires an approved elevated controller.");
            if (Environment.OSVersion.Version.Major < 10)
                throw new NotSupportedException("Use a modern dedicated system logger, never the shared NT Kernel Logger.");
            if (!sessionName.StartsWith("OpenCodex-BD2-", StringComparison.Ordinal))
                throw new ArgumentException("A task-specific unique session name is required.");
            if (TraceEventSession.GetActiveSessionNames().Contains(sessionName))
                throw new InvalidOperationException("Never restart or attach another session.");
            if (Directory.Exists(outputDirectory))
                throw new IOException("Do not reuse evidence or repeat this run.");
            Directory.CreateDirectory(outputDirectory);
            Directory.CreateDirectory(temporaryDirectory);
            Directory.CreateDirectory(homeDirectory);
            var session = new TraceEventSession(sessionName,
                TraceEventSessionOptions.Create | TraceEventSessionOptions.NoRestartOnCreate);
            session.BufferSizeMB = 64;
            session.StopOnDispose = true;
            var queue = new BlockingCollection<string>(32768);
            int dropped = 0, recorded = 0, callbackErrors = 0, rootStartSeen = 0;
            long submittedBytes = 0;
            var pendingIo = new Dictionary<ulong, PendingIo>();
            var retainedHandles = new List<IntPtr>();
            var writer = new Thread(() => {
                using var stream = new StreamWriter(new FileStream(
                    Path.Combine(outputDirectory, "owned-etw.events.jsonl"),
                    FileMode.CreateNew, FileAccess.Write, FileShare.Read), new UTF8Encoding(false));
                foreach (var row in queue.GetConsumingEnumerable()) stream.WriteLine(row);
            }) { IsBackground = true, Name = "OpenCodex-owned-etw-writer" };
            writer.Start();
            Action<object> emit = value => {
                string row = JsonSerializer.Serialize(value);
                if (Interlocked.Add(ref submittedBytes, Encoding.UTF8.GetByteCount(row) + 1) > 64L * 1024 * 1024
                    || !queue.TryAdd(row)) Interlocked.Increment(ref dropped);
                else Interlocked.Increment(ref recorded);
            };
            OwnedEtwScope scope = null;
            Process process = null;
            Exception captureError = null;
            var started = DateTime.UtcNow;
            long minimumQpc = 0;
            string commandExe = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), "cmd.exe");
            Thread consumer = null;
            bool outerCleanup = false, consumerJoined = false, consumerStarted = false, sessionStopped = false;
            int? exitCode = null, lost = null;
            DateTime? processCreatedUtc = null;
            Action<TraceEvent, Action> guarded = (data, action) => {
                try { action(); }
                catch (Exception error) {
                    Interlocked.Increment(ref callbackErrors);
                    if (callbackErrors <= 8)
                        emit(new { kind = "decoder-error", original = Header(data), error = error.GetType().Name });
                }
            };
            try
            {
                var kernel = session.Source.Kernel;
                kernel.ProcessStart += data => guarded(data, () => {
                    if (scope == null) return;
                    bool root = process != null && data.ProcessID == process.Id
                        && String.Equals(Path.GetFileName(data.ImageFileName), Path.GetFileName(testExe), StringComparison.OrdinalIgnoreCase);
                    string role = null;
                    if (!root && scope.ProcessOwned(data.ParentID, OriginalQpc(data)))
                    {
                        // CommandLine is inspected only for an already-owned
                        // descendant and is never serialized.
                        string image = ExecutableArgument(data.CommandLine);
                        if (SamePath(image, nodeExe)) role = "node";
                        else if (SamePath(image, commandExe)) role = "command";
                    }
                    if (!scope.ProcessStart(data.ProcessID, data.ParentID, OriginalQpc(data), root, role)) return;
                    if (root) Interlocked.Exchange(ref rootStartSeen, 1);
                    string expectedImage = root ? testExe : role == "node" ? nodeExe : commandExe;
                    emit(new { kind = "process-start", original = Header(data), pid = data.ProcessID,
                        parentPid = root ? (int?)null : data.ParentID, role = root ? "test" : role,
                        uniqueProcessKey = data.UniqueProcessKey,
                        ownedImage = expectedImage,
                        apiIdentityObservation = RetainIdentity(data.ProcessID, expectedImage,
                            data.TimeStamp.ToUniversalTime(), retainedHandles),
                        fullCommandLineSaved = false, sidSaved = false,
                        identityBasis = root ? "retained root handle/image/creation" : "ETW lifecycle + owned parent + exact executable argument" });
                });
                kernel.ProcessStop += data => guarded(data, () => {
                    if (scope?.ProcessStop(data.ProcessID, OriginalQpc(data)) != true) return;
                    emit(new { kind = "process-stop", original = Header(data),
                        pid = data.ProcessID, exitStatus = data.ExitStatus, uniqueProcessKey = data.UniqueProcessKey });
                });
                kernel.ThreadStart += data => guarded(data, () => {
                    if (scope?.ThreadStart(data.ProcessID, data.ThreadID, OriginalQpc(data)) != true) return;
                    emit(new { kind = "thread-start", original = Header(data), pid = data.ProcessID, tid = data.ThreadID });
                });
                kernel.ThreadStop += data => guarded(data, () => {
                    int? owner = scope?.ThreadOwner(data.ThreadID, OriginalQpc(data));
                    if (!owner.HasValue || scope.ThreadStop(data.ThreadID, OriginalQpc(data)) != true) return;
                    emit(new { kind = "thread-stop", original = Header(data), pid = owner.Value, tid = data.ThreadID });
                });
                kernel.ThreadCSwitch += data => guarded(data, () => {
                    int? incoming = scope?.ThreadOwner(data.NewThreadID, OriginalQpc(data));
                    int? outgoing = scope?.ThreadOwner(data.OldThreadID, OriginalQpc(data));
                    if (incoming.HasValue)
                        emit(new { kind = "owned-thread-scheduled-in", original = Header(data), pid = incoming.Value,
                            tid = data.NewThreadID, priority = data.NewThreadPriority, waitTime = data.NewThreadWaitTime,
                            counterpartThreadIdSaved = false });
                    if (outgoing.HasValue)
                        emit(new { kind = "owned-thread-scheduled-out", original = Header(data), pid = outgoing.Value,
                            tid = data.OldThreadID, priority = data.OldThreadPriority,
                            waitReason = data.OldThreadWaitReason.ToString(), waitMode = data.OldThreadWaitMode.ToString(),
                            state = data.OldThreadState.ToString(), counterpartThreadIdSaved = false });
                });
                kernel.DispatcherReadyThread += data => guarded(data, () => {
                    int? owner = scope?.ThreadOwner(data.AwakenedThreadID, OriginalQpc(data));
                    if (!owner.HasValue) return;
                    emit(new { kind = "owned-thread-ready", original = Header(data), pid = owner.Value,
                        tid = data.AwakenedThreadID, adjustReason = data.AdjustReason.ToString(),
                        adjustIncrement = data.AdjustIncrement, flags = data.Flags.ToString(),
                        readyingThreadIdSaved = false });
                });
                Action<ImageLoadTraceData> image = data => guarded(data, () => {
                    if (scope?.ProcessOwned(data.ProcessID, OriginalQpc(data)) != true) return;
                    string file = SafeFile(data.FileName, nodeExe, temporaryDirectory);
                    emit(new { kind = "owned-image", original = Header(data), pid = data.ProcessID,
                        image = file, pathOmitted = file == null, imageBase = data.ImageBase, imageSize = data.ImageSize });
                });
                kernel.ImageLoad += image;
                kernel.ImageUnload += image;
                Action<FileIOReadWriteTraceData> io = data => guarded(data, () => {
                    // Invalidate a recycled IRP even when its new issuer is not
                    // owned. No unrelated fields are saved.
                    pendingIo.Remove(data.IrpPtr);
                    if (scope?.ProcessOwned(data.ProcessID, OriginalQpc(data)) != true) return;
                    if (pendingIo.Count < 32768)
                        pendingIo[data.IrpPtr] = new PendingIo { Pid = data.ProcessID, Tid = data.ThreadID,
                            Qpc = OriginalQpc(data), Generation = scope.ProcessGeneration(data.ProcessID, OriginalQpc(data)).Value };
                    else Interlocked.Increment(ref dropped);
                    string file = SafeFile(data.FileName, nodeExe, temporaryDirectory);
                    emit(new { kind = "owned-file-read-write", original = Header(data),
                        pid = data.ProcessID, tid = data.ThreadID, file, pathOmitted = file == null,
                        fileObject = data.FileObject, irp = data.IrpPtr, offset = data.Offset, bytes = data.IoSize });
                });
                kernel.FileIORead += io;
                kernel.FileIOWrite += io;
                kernel.FileIOCreate += data => guarded(data, () => {
                    pendingIo.Remove(data.IrpPtr);
                    if (scope?.ProcessOwned(data.ProcessID, OriginalQpc(data)) != true) return;
                    if (pendingIo.Count < 32768)
                        pendingIo[data.IrpPtr] = new PendingIo { Pid = data.ProcessID, Tid = data.ThreadID,
                            Qpc = OriginalQpc(data), Generation = scope.ProcessGeneration(data.ProcessID, OriginalQpc(data)).Value };
                    else Interlocked.Increment(ref dropped);
                    string file = SafeFile(data.FileName, nodeExe, temporaryDirectory);
                    emit(new { kind = "owned-file-create", original = Header(data),
                        pid = data.ProcessID, tid = data.ThreadID, file, pathOmitted = file == null,
                        fileObject = data.FileObject, irp = data.IrpPtr });
                });
                kernel.FileIOOperationEnd += data => guarded(data, () => {
                    if (!pendingIo.TryGetValue(data.IrpPtr, out var operation)) return;
                    pendingIo.Remove(data.IrpPtr);
                    if (OriginalQpc(data) < operation.Qpc
                        || scope?.ProcessGeneration(operation.Pid, OriginalQpc(data)) != operation.Generation) return;
                    emit(new { kind = "owned-file-operation-end", original = Header(data),
                        pid = operation.Pid, tid = operation.Tid, irp = data.IrpPtr,
                        initiationQpc = operation.Qpc, status = data.NtStatus,
                        correlation = "owned initiating IRP; original completion-header process not stored",
                        completionsAfterOwnerExitOmitted = true });
                });
                // No filename, ETL merge, stack collection, profiler sampling,
                // network, registry, or other-party event serialization.
                session.EnableKernelProvider(
                    KernelTraceEventParser.Keywords.Process | KernelTraceEventParser.Keywords.Thread
                    | KernelTraceEventParser.Keywords.ImageLoad | KernelTraceEventParser.Keywords.ContextSwitch
                    | KernelTraceEventParser.Keywords.Dispatcher | KernelTraceEventParser.Keywords.FileIO
                    | KernelTraceEventParser.Keywords.FileIOInit);
                var ready = new ManualResetEventSlim(false);
                consumer = new Thread(() => {
                    ready.Set();
                    try { session.Source.Process(); }
                    catch (Exception error) { captureError = error; }
                }) { IsBackground = true, Name = "OpenCodex-owned-etw-consumer" };
                // The source handle and kernel provider already exist. Bind the
                // newly launched root before dispatching buffered callbacks, so
                // an early ProcessStart cannot race a null ownership scope.
                minimumQpc = Stopwatch.GetTimestamp();
                var info = new ProcessStartInfo(testExe) { WorkingDirectory = workingDirectory };
                Configure(info, nodeExe, temporaryDirectory, homeDirectory);
                process = Process.Start(info);
                if (process == null) throw new InvalidOperationException("Test did not start.");
                processCreatedUtc = process.StartTime.ToUniversalTime();
                string actualImage = process.MainModule.FileName;
                if (!SamePath(actualImage, testExe)) throw new InvalidOperationException("Root image mismatch.");
                scope = new OwnedEtwScope(process.Id, minimumQpc);
                emit(new { kind = "root-handle-binding", atUtc = DateTime.UtcNow, pid = process.Id,
                    createdUtc = processCreatedUtc, image = actualImage, minimumQpc,
                    createdFileTime100ns = process.StartTime.ToFileTimeUtc(), actualToken = ProcessToken(process),
                    fullLibraryArgumentCount = info.ArgumentList.Count, defaultConcurrency = true,
                    originalCapture = true, noPrewarm = true });
                consumer.Start();
                consumerStarted = true;
                if (!ready.Wait(5000)) throw new TimeoutException("ETW consumer did not schedule; stop owned session.");
                var stdout = process.StandardOutput.ReadToEndAsync();
                var stderr = process.StandardError.ReadToEndAsync();
                if (!process.WaitForExit(120000))
                {
                    outerCleanup = true;
                    process.Kill(true);
                    process.WaitForExit(10000);
                }
                exitCode = process.ExitCode;
                if (stdout.Wait(5000)) File.WriteAllText(Path.Combine(outputDirectory, "test-stdout.original.log"), stdout.Result);
                if (stderr.Wait(5000)) File.WriteAllText(Path.Combine(outputDirectory, "test-stderr.original.log"), stderr.Result);
                // Flush/stop only our unique session. ProcessTrace drains buffered
                // original events; it does not depend on Node still being alive.
                lost = session.EventsLost;
            }
            finally
            {
                try
                {
                    if (process != null && !process.HasExited)
                    {
                        outerCleanup = true;
                        process.Kill(true);
                        process.WaitForExit(10000);
                    }
                }
                catch (Exception error) { captureError = captureError ?? error; }
                sessionStopped = session.Stop(true);
                if (consumerStarted) consumerJoined = consumer.Join(10000);
                if (consumerStarted && !consumerJoined)
                {
                    session.Source.StopProcessing();
                    consumerJoined = consumer.Join(5000);
                }
                session.Dispose();
                foreach (IntPtr handle in retainedHandles) CloseHandle(handle);
                queue.CompleteAdding();
                bool writerJoined = writer.Join(10000);
                var result = new {
                    startedAtUtc = started, finishedAtUtc = DateTime.UtcNow, sessionName,
                    testPid = process?.Id, rootCreationUtc = processCreatedUtc, testExitCode = exitCode,
                    rootProcessStartSeen = rootStartSeen != 0, minimumQpc,
                    processTraceStarted = consumerStarted, processTraceJoined = consumerJoined,
                    ownedSessionStopSucceeded = sessionStopped,
                    writerJoined, droppedRecords = dropped,
                    recordedEvents = recorded, decoderCallbackErrors = callbackErrors,
                    maximumSerializedBytes = 64L * 1024 * 1024,
                    maximumQueueRecords = 32768, maximumTrackedIrps = 32768,
                    kernelEventsLost = lost, captureError = captureError?.GetType().Name,
                    forcedOuterCleanup = outerCleanup, systemWideEtlWritten = false,
                    unrelatedEventPayloadsSerialized = false, boundedOriginalRunCount = process == null ? 0 : 1,
                    defaultConcurrencyAndCapturePreserved = true, observerWctQueries = 0,
                    nodePrewarmed = false, performanceAcceptanceClaimed = false,
                    actualFailureOrRootCauseMustBeDeterminedFromEvidence = true
                };
                File.WriteAllText(Path.Combine(outputDirectory, "capture-result.json"), JsonSerializer.Serialize(result));
                process?.Dispose();
            }
            return File.ReadAllText(Path.Combine(outputDirectory, "capture-result.json"));
        }
    }
}
