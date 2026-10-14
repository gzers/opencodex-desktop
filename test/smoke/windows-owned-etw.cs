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
using System.Threading.Tasks;
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
        public bool HotEventsOwned(int pid, long qpc)
        {
            // Full libtest root scheduling/I/O floods a bounded trace before
            // the query children start. Preserve root identity/lifecycle, but
            // target high-volume events to the admitted Node/cmd descendants.
            string role = Role(pid, qpc);
            return role == "node" || role == "command"
                || role == "node-basename" || role == "command-basename";
        }
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
        public static int SelfTestHotScope()
        {
            int checks = 0;
            Action<bool> require = value => {
                if (!value) throw new InvalidOperationException("Owned descendant hot-event scope");
                checks++;
            };
            var scope = new OwnedEtwScope(100, 1000);
            scope.ProcessStart(100, 1, 1001, true, null);
            require(scope.ProcessOwned(100, 1002));
            require(!scope.HotEventsOwned(100, 1002));
            scope.ProcessStart(101, 100, 1002, false, "command");
            require(scope.HotEventsOwned(101, 1002));
            scope.ProcessStart(102, 101, 1003, false, "node");
            require(scope.HotEventsOwned(102, 1004));
            require(!scope.HotEventsOwned(999, 1004));
            scope.ProcessStop(102, 1010);
            require(scope.HotEventsOwned(102, 1004)); // Delayed original event remains valid.
            require(!scope.HotEventsOwned(102, 1011));
            scope.ProcessStart(102, 999, 1020, false, "node");
            require(!scope.HotEventsOwned(102, 1021));
            scope.ProcessStart(103, 101, 1030, false, "node");
            scope.ThreadStart(103, 206, 1031);
            require(scope.HotEventsOwned(scope.ThreadOwner(206, 1032).Value, 1032));
            scope.ProcessStop(100, 1040);
            require(!scope.HotEventsOwned(100, 1041));
            return checks;
        }
    }

    public sealed class OwnedEtwLineBudget
    {
        private readonly long maximum;
        private readonly object gate = new object();
        private long used;
        public OwnedEtwLineBudget(long maximumBytes)
        {
            if (maximumBytes <= 0) throw new ArgumentOutOfRangeException(nameof(maximumBytes));
            maximum = maximumBytes;
        }
        public long QueuedBytes { get { lock (gate) return used; } }
        public bool TrySubmit(BlockingCollection<string> queue, string row)
        {
            // Count actual UTF-8 bytes including the exact LF written below.
            // Failed queue admission must not consume the remaining budget.
            long bytes = Encoding.UTF8.GetByteCount(row) + 1L;
            lock (gate) {
                if (bytes > maximum - used || !queue.TryAdd(row)) return false;
                used += bytes;
                return true;
            }
        }
        public static StreamWriter OpenWriter(Stream output)
        {
            var writer = new StreamWriter(output, new UTF8Encoding(false), 4096, true);
            writer.NewLine = "\n";
            return writer;
        }
        public static int SelfTest()
        {
            int checks = 0;
            Action<bool> require = value => {
                if (!value) throw new InvalidOperationException("Bounded UTF-8 JSONL output");
                checks++;
            };
            string first = "{\"message\":\"汉字\"}", second = "{\"message\":\"b\"}";
            long limit = Encoding.UTF8.GetByteCount(first) + Encoding.UTF8.GetByteCount(second) + 2;
            using var queue = new BlockingCollection<string>(2);
            var budget = new OwnedEtwLineBudget(limit);
            require(budget.TrySubmit(queue, first));
            require(budget.TrySubmit(queue, second));
            require(!budget.TrySubmit(queue, second));
            queue.CompleteAdding();
            using var output = new MemoryStream();
            using (var writer = OpenWriter(output))
                foreach (var row in queue.GetConsumingEnumerable()) writer.WriteLine(row);
            require(output.Length == limit && budget.QueuedBytes == limit);
            require(Encoding.UTF8.GetString(output.ToArray()) == first + "\n" + second + "\n");
            require(!output.ToArray().Contains((byte)'\r'));
            using var full = new BlockingCollection<string>(1);
            var queueBudget = new OwnedEtwLineBudget(2 * (Encoding.UTF8.GetByteCount(second) + 1));
            require(queueBudget.TrySubmit(full, second));
            require(!queueBudget.TrySubmit(full, second));
            require(queueBudget.QueuedBytes == Encoding.UTF8.GetByteCount(second) + 1);
            full.Take();
            require(queueBudget.TrySubmit(full, second));
            return checks;
        }
    }

    public static class OwnedEtwLifecycle
    {
        public static T KernelBeforeSource<T>(Action enableKernel, Func<T> getSource)
        {
            // TraceEvent's Source getter starts a session when it is inactive.
            // Kernel enablement must precede that getter, or the library refuses
            // to enable the kernel provider on the already-started session.
            enableKernel();
            return getSource();
        }
        public static List<string> FinishAll(IEnumerable<KeyValuePair<string, Action>> stages)
        {
            var errors = new List<string>();
            foreach (var stage in stages)
            {
                try { stage.Value(); }
                catch (Exception error) { errors.Add(stage.Key + ":" + error.GetType().Name); }
            }
            return errors;
        }
        public static int SelfTest()
        {
            int checks = 0;
            Action<bool> require = value => {
                if (!value) throw new InvalidOperationException("ETW lifecycle contract");
                checks++;
            };
            bool sessionStarted = false, kernelEnabled = false, sourceRead = false;
            Action enable = () => {
                if (sessionStarted) throw new InvalidOperationException("Kernel must be enabled first.");
                kernelEnabled = true;
                sessionStarted = true;
            };
            Func<int> getSource = () => {
                sourceRead = true;
                sessionStarted = true;
                return kernelEnabled ? 1 : 0;
            };
            // Independent red model reproduces the old getter-before-enable
            // order without touching Windows ETW or requiring administrator.
            bool oldOrderRejected = false;
            getSource();
            try { enable(); } catch (InvalidOperationException) { oldOrderRejected = true; }
            require(oldOrderRejected && !kernelEnabled);
            sessionStarted = kernelEnabled = sourceRead = false;
            require(KernelBeforeSource(enable, getSource) == 1);
            require(sessionStarted && kernelEnabled && sourceRead);
            sourceRead = false;
            bool failedEnablePreserved = false;
            try {
                KernelBeforeSource(() => { throw new UnauthorizedAccessException(); }, getSource);
            } catch (UnauthorizedAccessException) { failedEnablePreserved = true; }
            require(failedEnablePreserved && !sourceRead);
            var completed = new List<string>();
            var errors = FinishAll(new[] {
                new KeyValuePair<string, Action>("stop", () => { completed.Add("stop"); throw new IOException(); }),
                new KeyValuePair<string, Action>("drain", () => { completed.Add("drain"); }),
                new KeyValuePair<string, Action>("dispose", () => { completed.Add("dispose"); throw new InvalidOperationException(); }),
                new KeyValuePair<string, Action>("writer", () => { completed.Add("writer"); }),
            });
            require(completed.SequenceEqual(new[] { "stop", "drain", "dispose", "writer" }));
            require(errors.SequenceEqual(new[] { "stop:IOException", "dispose:InvalidOperationException" }));
            require(FinishAll(Array.Empty<KeyValuePair<string, Action>>()).Count == 0);
            return checks;
        }
    }

    public static class OwnedEtwNativeStop
    {
        [StructLayout(LayoutKind.Sequential)]
        private struct Wnode
        {
            public uint BufferSize, ProviderId;
            public ulong HistoricalContext;
            public long TimeStamp;
            public Guid Guid;
            public uint ClientContext, Flags;
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct Properties
        {
            public Wnode Wnode;
            public uint BufferSize, MinimumBuffers, MaximumBuffers, MaximumFileSize,
                        LogFileMode, FlushTimer, EnableFlags;
            public int AgeLimit;
            public uint NumberOfBuffers, FreeBuffers, EventsLost, BuffersWritten,
                        LogBuffersLost, RealTimeBuffersLost;
            public IntPtr LoggerThreadId;
            public uint LogFileNameOffset, LoggerNameOffset;
        }
        public sealed class Result
        {
            public uint Status { get; set; }
            public bool StoppedOrAlreadyAbsent { get; set; }
            public uint? FinalEventsLost { get; set; }
            public uint? FinalRealTimeBuffersLost { get; set; }
        }
        [DllImport("advapi32.dll", CharSet = CharSet.Unicode, EntryPoint = "ControlTraceW")]
        private static extern uint ControlTrace(ulong handle, string name, IntPtr properties, uint controlCode);
        private static void RequireOwnName(string name)
        {
            const string prefix = "OpenCodex-BD2-";
            if (name == null || !name.StartsWith(prefix, StringComparison.Ordinal)
                || !Guid.TryParseExact(name.Substring(prefix.Length), "N", out _))
                throw new ArgumentException("Only the unique task-created session can be stopped.");
        }
        private static IntPtr Buffer(string name)
        {
            RequireOwnName(name);
            int size = Marshal.SizeOf<Properties>();
            int capacity = size + 4096; // Two maximum 1024-character name buffers.
            IntPtr buffer = Marshal.AllocHGlobal(capacity);
            Marshal.Copy(new byte[capacity], 0, buffer, capacity);
            var properties = new Properties {
                Wnode = new Wnode { BufferSize = (uint)capacity, Flags = 0x00020000 },
                LoggerNameOffset = (uint)size, LogFileNameOffset = (uint)(size + 2048)
            };
            Marshal.StructureToPtr(properties, buffer, false);
            byte[] nameBytes = Encoding.Unicode.GetBytes(name + "\0");
            Marshal.Copy(nameBytes, 0, IntPtr.Add(buffer, size), nameBytes.Length);
            return buffer;
        }
        // TraceEventSession.Stop resets a machine-wide sampling interval. Avoid
        // it completely: this calls only ControlTrace STOP by our unique name,
        // and returns that owned session's final loss counters.
        public static Result Stop(string name)
        {
            IntPtr buffer = Buffer(name);
            try {
                uint status = ControlTrace(0, name, buffer, 1);
                var properties = Marshal.PtrToStructure<Properties>(buffer);
                return new Result {
                    Status = status, StoppedOrAlreadyAbsent = status == 0 || status == 4201,
                    FinalEventsLost = status == 0 ? properties.EventsLost : (uint?)null,
                    FinalRealTimeBuffersLost = status == 0 ? properties.RealTimeBuffersLost : (uint?)null
                };
            }
            finally { Marshal.FreeHGlobal(buffer); }
        }
        public static int SelfTest()
        {
            int checks = 0;
            Action<bool> require = value => {
                if (!value) throw new InvalidOperationException("Owned native-stop contract");
                checks++;
            };
            require(IntPtr.Size == 8);
            require(Marshal.SizeOf<Wnode>() == 48);
            require(Marshal.SizeOf<Properties>() == 120);
            require(Marshal.OffsetOf<Properties>(nameof(Properties.LoggerNameOffset)).ToInt32() == 116);
            foreach (string rejected in new[] { "NT Kernel Logger", "OpenCodex-BD2-not-a-guid" }) {
                bool denied = false;
                try { RequireOwnName(rejected); } catch (ArgumentException) { denied = true; }
                require(denied);
            }
            string name = "OpenCodex-BD2-" + Guid.NewGuid().ToString("N");
            IntPtr buffer = Buffer(name);
            try {
                var properties = Marshal.PtrToStructure<Properties>(buffer);
                require(properties.Wnode.BufferSize == 4216
                    && properties.LogFileNameOffset == 2168
                    && properties.Wnode.Guid == Guid.Empty);
                require(Marshal.PtrToStringUni(IntPtr.Add(buffer, (int)properties.LoggerNameOffset)) == name);
            }
            finally { Marshal.FreeHGlobal(buffer); }
            return checks; // No ControlTrace call or Windows session in these tests.
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
        private static string QueryProcessImage(IntPtr handle)
        {
            // MainModule enumerates loader modules, which need not be ready on
            // a just-created process. Query the already-held process handle;
            // never wait for initialization or relaunch the original test.
            var image = new StringBuilder(32768);
            uint size = 32768;
            if (!QueryFullProcessImageNameW(handle, 0, image, ref size))
                throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
            return image.ToString();
        }
        private static string RequireExpectedImage(string actual, string expected)
        {
            if (!SamePath(actual, expected)) throw new InvalidOperationException("Root image mismatch.");
            return actual;
        }
        public static int SelfTestProcessImage()
        {
            // Only this controller's retained handle is queried. No process,
            // Node, original library, ETW session, or startup wait is created.
            int checks = 0;
            Action<bool> require = value => {
                if (!value) throw new InvalidOperationException("Owned root image contract");
                checks++;
            };
            using var current = Process.GetCurrentProcess();
            string actual = QueryProcessImage(current.Handle);
            require(SamePath(actual, Environment.ProcessPath));
            require(RequireExpectedImage(actual, Environment.ProcessPath) == actual);
            require(RequireExpectedImage(@"\\?\" + actual, Environment.ProcessPath) == @"\\?\" + actual);
            bool mismatchRejected = false;
            try { RequireExpectedImage(actual, actual + ".unowned"); }
            catch (InvalidOperationException) { mismatchRejected = true; }
            require(mismatchRejected);
            bool invalidHandleRejected = false;
            try { QueryProcessImage(IntPtr.Zero); }
            catch (System.ComponentModel.Win32Exception) { invalidHandleRejected = true; }
            require(invalidHandleRejected);
            return checks;
        }
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
        private static string ExecutableRole(string eventImage, string commandLine, string node, string command)
        {
            // Kernel ProcessStart carries the actual image independently of
            // argv[0]. Rust/Windows command wrappers need not put a full image
            // path in argv[0]. Prefer the exact ETW image, without saving argv.
            if (SamePath(eventImage, node)) return "node";
            if (SamePath(eventImage, command)) return "command";
            // Older events can contain only an image basename. In that case a
            // full exact executable argument is still required. An incompatible
            // full event-image path is never overridden by a claimed argv[0].
            if (String.IsNullOrEmpty(eventImage) || Path.GetFileName(eventImage) != eventImage) return null;
            string argument = ExecutableArgument(commandLine);
            if (String.Equals(eventImage, Path.GetFileName(node), StringComparison.OrdinalIgnoreCase)
                && SamePath(argument, node)) return "node";
            if (String.Equals(eventImage, Path.GetFileName(command), StringComparison.OrdinalIgnoreCase)
                && SamePath(argument, command)) return "command";
            // Windows kernel events may expose only a basename while argv[0]
            // is the same basename. Admit only this direct child of an
            // already-owned process; RetainIdentity still checks the live
            // handle's full path below.
            if (String.Equals(eventImage, Path.GetFileName(node), StringComparison.OrdinalIgnoreCase)
                && String.Equals(argument, Path.GetFileName(node), StringComparison.OrdinalIgnoreCase))
                return "node-basename";
            if (String.Equals(eventImage, Path.GetFileName(command), StringComparison.OrdinalIgnoreCase)
                && String.Equals(argument, Path.GetFileName(command), StringComparison.OrdinalIgnoreCase))
                return "command-basename";
            return null;
        }
        public static int SelfTestExecutableAdmission()
        {
            int checks = 0;
            string node = @"C:\owned\node.exe", command = @"C:\Windows\System32\cmd.exe";
            Action<bool> require = value => {
                if (!value) throw new InvalidOperationException("Owned executable image admission check " + (checks + 1));
                checks++;
            };
            require(ExecutableRole(node, "node.exe fixture.cjs", node, command) == "node");
            require(ExecutableRole(command, "cmd.exe /c fixture.cmd", node, command) == "command");
            require(ExecutableRole(command, "", node, command) == "command");
            require(ExecutableRole("node.exe", "\"" + node + "\" fixture.cjs", node, command) == "node");
            require(ExecutableRole("cmd.exe", "\"" + command + "\" /c fixture.cmd", node, command) == "command");
            require(ExecutableRole("cmd.exe", "cmd.exe /c fixture.cmd", node, command) == "command-basename");
            require(ExecutableRole("", "\"" + command + "\" /c fixture.cmd", node, command) == null);
            require(ExecutableRole("other.exe", "\"" + node + "\" fixture.cjs", node, command) == null);
            require(ExecutableRole(@"C:\other\node.exe", "\"" + node + "\" fixture.cjs", node, command) == null);
            require(ExecutableRole(@"\\?\" + node, "node.exe", node, command) == "node");
            require(ExecutableRole(command, "\"" + node + "\"", node, command) == "command");
            require(ExecutableRole("node.exe", "node.exe", node, command) == "node-basename");
            require(ExecutableRole("cmd.exe", "cmd.exe", node, command) == "command-basename");
            return checks;
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
            // Stop/Dispose's implicit Stop in TraceEvent 3.1.21 would call a
            // machine-wide TraceSampledProfileIntervalInfo reset. We use our
            // narrowly named native stopper and only dispose decoding handles.
            session.StopOnDispose = false;
            var queue = new BlockingCollection<string>(32768);
            int dropped = 0, recorded = 0, callbackErrors = 0, rootStartSeen = 0;
            int knownImageChildrenNotAdmitted = 0;
            var lineBudget = new OwnedEtwLineBudget(64L * 1024 * 1024);
            long omittedRootScheduling = 0, omittedRootFileIo = 0;
            var pendingIo = new Dictionary<ulong, PendingIo>();
            var retainedHandles = new List<IntPtr>();
            Exception writerError = null;
            var writer = new Thread(() => {
                try {
                    using var output = new FileStream(
                        Path.Combine(outputDirectory, "owned-etw.events.jsonl"),
                        FileMode.CreateNew, FileAccess.Write, FileShare.Read);
                    using var stream = OwnedEtwLineBudget.OpenWriter(output);
                    foreach (var row in queue.GetConsumingEnumerable()) stream.WriteLine(row);
                }
                catch (Exception error) {
                    // Never let a background writer exception terminate the
                    // controller before its own ETW session is stopped.
                    writerError = error;
                    foreach (var row in queue.GetConsumingEnumerable())
                        Interlocked.Increment(ref dropped);
                }
            }) { IsBackground = true, Name = "OpenCodex-owned-etw-writer" };
            writer.Start();
            Action<object> emit = value => {
                string row = JsonSerializer.Serialize(value);
                if (!lineBudget.TrySubmit(queue, row)) Interlocked.Increment(ref dropped);
                else Interlocked.Increment(ref recorded);
            };
            OwnedEtwScope scope = null;
            Process process = null;
            Task<string> stdout = null, stderr = null;
            Exception captureError = null;
            Exception runError = null;
            var started = DateTime.UtcNow;
            long minimumQpc = 0;
            string commandExe = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), "cmd.exe");
            Thread consumer = null;
            bool outerCleanup = false, consumerJoined = false, consumerStarted = false, sessionStopped = false;
            OwnedEtwNativeStop.Result nativeStop = null;
            EventHandler processExitStop = (_, __) => {
                try { OwnedEtwNativeStop.Stop(sessionName); } catch { /* Capture already invalid if explicit stop failed. */ }
            };
            AppDomain.CurrentDomain.ProcessExit += processExitStop;
            int? exitCode = null, lost = null;
            int? exitCodeAfterCleanup = null;
            bool stdoutSaved = false, stderrSaved = false, ownedRootExited = false;
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
                var kernel = OwnedEtwLifecycle.KernelBeforeSource(
                    () => session.EnableKernelProvider(
                        KernelTraceEventParser.Keywords.Process | KernelTraceEventParser.Keywords.Thread
                        | KernelTraceEventParser.Keywords.ImageLoad | KernelTraceEventParser.Keywords.ContextSwitch
                        | KernelTraceEventParser.Keywords.Dispatcher | KernelTraceEventParser.Keywords.FileIO
                        | KernelTraceEventParser.Keywords.FileIOInit),
                    () => session.Source.Kernel);
                kernel.ProcessStart += data => guarded(data, () => {
                    if (scope == null) return;
                    bool root = process != null && data.ProcessID == process.Id
                        && String.Equals(Path.GetFileName(data.ImageFileName), Path.GetFileName(testExe), StringComparison.OrdinalIgnoreCase);
                    string role = null;
                    if (!root && scope.ProcessOwned(data.ParentID, OriginalQpc(data)))
                    {
                        // CommandLine is inspected only for an already-owned
                        // descendant and is never serialized.
                        role = ExecutableRole(data.ImageFileName, data.CommandLine, nodeExe, commandExe);
                        string basename = Path.GetFileName(data.ImageFileName);
                        string knownKind = String.Equals(basename, Path.GetFileName(nodeExe), StringComparison.OrdinalIgnoreCase)
                            ? "node" : String.Equals(basename, Path.GetFileName(commandExe), StringComparison.OrdinalIgnoreCase)
                            ? "command" : null;
                        if (role == null && knownKind != null) {
                            knownImageChildrenNotAdmitted++;
                            string argument = ExecutableArgument(data.CommandLine);
                            emit(new { kind = "known-child-image-not-admitted", original = Header(data),
                                pid = data.ProcessID, parentPid = data.ParentID, knownImageKind = knownKind,
                                eventImageIsExactKnownPath = SamePath(data.ImageFileName, nodeExe) || SamePath(data.ImageFileName, commandExe),
                                eventImageIsBasenameOnly = basename == data.ImageFileName,
                                firstArgumentIsExactKnownPath = SamePath(argument, nodeExe) || SamePath(argument, commandExe),
                                firstArgumentIsBareKnownName = String.Equals(argument, Path.GetFileName(nodeExe), StringComparison.OrdinalIgnoreCase)
                                    || String.Equals(argument, Path.GetFileName(commandExe), StringComparison.OrdinalIgnoreCase),
                                fullCommandLineSaved = false, sidSaved = false, descendantsNotAssumedCovered = true });
                        }
                    }
                    if (!scope.ProcessStart(data.ProcessID, data.ParentID, OriginalQpc(data), root, role)) return;
                    if (root) Interlocked.Exchange(ref rootStartSeen, 1);
                    string expectedImage = root ? testExe
                        : role.StartsWith("node", StringComparison.Ordinal) ? nodeExe : commandExe;
                    emit(new { kind = "process-start", original = Header(data), pid = data.ProcessID,
                        parentPid = root ? (int?)null : data.ParentID, role = root ? "test" : role,
                        uniqueProcessKey = data.UniqueProcessKey,
                        ownedImage = expectedImage,
                        exactEtwImageMatched = SamePath(data.ImageFileName, expectedImage),
                        exactExecutableArgumentMatched = SamePath(ExecutableArgument(data.CommandLine), expectedImage),
                        apiIdentityObservation = RetainIdentity(data.ProcessID, expectedImage,
                            data.TimeStamp.ToUniversalTime(), retainedHandles),
                        fullCommandLineSaved = false, sidSaved = false,
                        identityBasis = root ? "retained root handle/image/creation" :
                            role.EndsWith("-basename", StringComparison.Ordinal)
                            ? "ETW lifecycle + matching image/argv basenames; live handle path check retained"
                            : "ETW lifecycle + owned parent + exact event image or basename-consistent exact executable argument" });
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
                    if (incoming.HasValue && scope.HotEventsOwned(incoming.Value, OriginalQpc(data)))
                        emit(new { kind = "owned-thread-scheduled-in", original = Header(data), pid = incoming.Value,
                            tid = data.NewThreadID, priority = data.NewThreadPriority, waitTime = data.NewThreadWaitTime,
                            counterpartThreadIdSaved = false });
                    else if (incoming.HasValue) omittedRootScheduling++;
                    if (outgoing.HasValue && scope.HotEventsOwned(outgoing.Value, OriginalQpc(data)))
                        emit(new { kind = "owned-thread-scheduled-out", original = Header(data), pid = outgoing.Value,
                            tid = data.OldThreadID, priority = data.OldThreadPriority,
                            waitReason = data.OldThreadWaitReason.ToString(), waitMode = data.OldThreadWaitMode.ToString(),
                            state = data.OldThreadState.ToString(), counterpartThreadIdSaved = false });
                    else if (outgoing.HasValue) omittedRootScheduling++;
                });
                kernel.DispatcherReadyThread += data => guarded(data, () => {
                    int? owner = scope?.ThreadOwner(data.AwakenedThreadID, OriginalQpc(data));
                    if (!owner.HasValue) return;
                    if (!scope.HotEventsOwned(owner.Value, OriginalQpc(data))) {
                        omittedRootScheduling++;
                        return;
                    }
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
                    if (!scope.HotEventsOwned(data.ProcessID, OriginalQpc(data))) {
                        omittedRootFileIo++;
                        return;
                    }
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
                    if (!scope.HotEventsOwned(data.ProcessID, OriginalQpc(data))) {
                        omittedRootFileIo++;
                        return;
                    }
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
                // Drain immediately, including when identity validation fails.
                // Original libtest capture and CLI arguments remain unchanged.
                stdout = process.StandardOutput.ReadToEndAsync();
                stderr = process.StandardError.ReadToEndAsync();
                processCreatedUtc = process.StartTime.ToUniversalTime();
                string actualImage = QueryProcessImage(process.Handle);
                emit(new { kind = "root-handle-image-query", atUtc = DateTime.UtcNow, pid = process.Id,
                    createdUtc = processCreatedUtc, image = actualImage, expectedImageMatches = SamePath(actualImage, testExe),
                    query = "QueryFullProcessImageNameW on retained Process.Handle; no loader-module enumeration" });
                RequireExpectedImage(actualImage, testExe);
                scope = new OwnedEtwScope(process.Id, minimumQpc);
                emit(new { kind = "root-handle-binding", atUtc = DateTime.UtcNow, pid = process.Id,
                    createdUtc = processCreatedUtc, image = actualImage, minimumQpc,
                    createdFileTime100ns = process.StartTime.ToFileTimeUtc(), actualToken = ProcessToken(process),
                    fullLibraryArgumentCount = info.ArgumentList.Count, defaultConcurrency = true,
                    originalCapture = true, noPrewarm = true });
                consumer.Start();
                consumerStarted = true;
                if (!ready.Wait(5000)) throw new TimeoutException("ETW consumer did not schedule; stop owned session.");
                if (!process.WaitForExit(120000))
                {
                    outerCleanup = true;
                    process.Kill(true);
                    process.WaitForExit(10000);
                }
                exitCode = process.ExitCode;
                // Flush/stop only our unique session. ProcessTrace drains buffered
                // original events; it does not depend on Node still being alive.
                lost = session.EventsLost;
            }
            catch (Exception error)
            {
                // Preserve setup/root-identity errors as well as decoder errors.
                // A null decoder error alone must never suggest valid coverage.
                runError = error;
                throw;
            }
            finally
            {
                bool writerJoined = false;
                var cleanupErrors = OwnedEtwLifecycle.FinishAll(new[] {
                    new KeyValuePair<string, Action>("owned-process", () => {
                        if (process != null && !process.HasExited) {
                            outerCleanup = true;
                            process.Kill(true);
                            process.WaitForExit(10000);
                        }
                        if (process != null) {
                            ownedRootExited = process.HasExited;
                            if (!ownedRootExited) throw new TimeoutException("Owned root cleanup deadline exceeded.");
                            exitCodeAfterCleanup = process.ExitCode;
                        }
                    }),
                    new KeyValuePair<string, Action>("session-stop", () => {
                        nativeStop = OwnedEtwNativeStop.Stop(sessionName);
                        sessionStopped = nativeStop.StoppedOrAlreadyAbsent;
                    }),
                    new KeyValuePair<string, Action>("consumer-drain", () => {
                        if (consumerStarted) consumerJoined = consumer.Join(10000);
                        if (consumerStarted && !consumerJoined) {
                            session.Source.StopProcessing();
                            consumerJoined = consumer.Join(5000);
                        }
                    }),
                    new KeyValuePair<string, Action>("session-dispose", () => session.Dispose()),
                    new KeyValuePair<string, Action>("handles-close", () => {
                        // If a decoder is still running, do not race its handle
                        // list. The controller's process exit releases those
                        // handles; the explicit incomplete-drain result invalidates capture.
                        if (!consumerStarted || consumerJoined)
                            foreach (IntPtr handle in retainedHandles) CloseHandle(handle);
                    }),
                    new KeyValuePair<string, Action>("writer-complete", () => queue.CompleteAdding()),
                    new KeyValuePair<string, Action>("writer-drain", () => writerJoined = writer.Join(10000)),
                    new KeyValuePair<string, Action>("stdout-drain", () => {
                        if (stdout == null) return;
                        if (!stdout.Wait(5000)) throw new TimeoutException("Owned stdout drain deadline exceeded.");
                        File.WriteAllText(Path.Combine(outputDirectory, "test-stdout.original.log"), stdout.Result);
                        stdoutSaved = true;
                    }),
                    new KeyValuePair<string, Action>("stderr-drain", () => {
                        if (stderr == null) return;
                        if (!stderr.Wait(5000)) throw new TimeoutException("Owned stderr drain deadline exceeded.");
                        File.WriteAllText(Path.Combine(outputDirectory, "test-stderr.original.log"), stderr.Result);
                        stderrSaved = true;
                    }),
                });
                if (sessionStopped) AppDomain.CurrentDomain.ProcessExit -= processExitStop;
                var serializedFile = new FileInfo(Path.Combine(outputDirectory, "owned-etw.events.jsonl"));
                long? actualSerializedBytes = writerJoined && serializedFile.Exists
                    ? serializedFile.Length : (long?)null;
                var result = new {
                    startedAtUtc = started, finishedAtUtc = DateTime.UtcNow, sessionName,
                    testPid = process?.Id, rootCreationUtc = processCreatedUtc, testExitCode = exitCode,
                    testExitCodeAfterCleanup = exitCodeAfterCleanup, ownedRootExited,
                    stdoutSaved, stderrSaved, originalLibraryCompleted = exitCode.HasValue && !outerCleanup,
                    rootProcessStartSeen = rootStartSeen != 0, minimumQpc,
                    processTraceStarted = consumerStarted, processTraceJoined = consumerJoined,
                    ownedSessionStopSucceeded = sessionStopped,
                    writerJoined, droppedRecords = dropped,
                    recordedEvents = recorded, decoderCallbackErrors = callbackErrors,
                    knownImageChildrenNotAdmitted, knownImageCoverageGap = knownImageChildrenNotAdmitted != 0,
                    maximumSerializedBytes = 64L * 1024 * 1024,
                    actualSerializedBytes, queuedSerializedBytes = lineBudget.QueuedBytes,
                    serializedBytesWithinLimit = actualSerializedBytes.HasValue && actualSerializedBytes.Value <= 64L * 1024 * 1024,
                    jsonLinesNewline = "LF", hotEventsScope = "admitted Node/cmd descendants only",
                    rootIdentityAndProcessThreadImageLifecyclesRetained = true,
                    rootSchedulingEventsIntentionallyOmitted = omittedRootScheduling,
                    rootFileIoInitiationsIntentionallyOmitted = omittedRootFileIo,
                    omittedRootHotEventsAreNotClaimsOfNoRootActivity = true,
                    maximumQueueRecords = 32768, maximumTrackedIrps = 32768,
                    kernelEventsLost = lost, captureError = captureError?.GetType().Name,
                    runError = runError?.GetType().Name, runErrorMessage = runError?.Message,
                    writerError = writerError?.GetType().Name, cleanupErrors,
                    kernelBeforeSourceOrder = true,
                    nativeOwnedSessionStop = nativeStop, traceEventGenericStopInvoked = false,
                    globalCpuSampleIntervalWritten = false,
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
