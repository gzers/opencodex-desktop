// Passive product-family process and network events. No process is launched,
// killed, suspended, or assigned a different token by this recorder.
using System;
using System.Collections.Generic;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using Microsoft.Diagnostics.Tracing;
using Microsoft.Diagnostics.Tracing.Parsers;
using Microsoft.Diagnostics.Tracing.Session;

namespace OpenCodex.Diagnostics
{
    public static class QueryCounterRules
    {
        public static string ChildKind(string imageBasename, string commandLine)
        {
            if (String.Equals(imageBasename, "msedgewebview2.exe", StringComparison.OrdinalIgnoreCase))
                return "browser-runtime";
            if (String.Equals(imageBasename, "conhost.exe", StringComparison.OrdinalIgnoreCase))
                return "console-host";
            bool command = new[] { "node.exe", "cmd.exe", "powershell.exe", "pwsh.exe" }
                .Contains(imageBasename, StringComparer.OrdinalIgnoreCase);
            if (!command) return "other-owned-process";
            string text = commandLine ?? "";
            if (text.IndexOf("npm", StringComparison.OrdinalIgnoreCase) >= 0 &&
                Regex.IsMatch(text, @"(?:^|[\s""'])view(?=[\s""']|$)", RegexOptions.IgnoreCase))
                return "remote-metadata-query";
            if (Regex.IsMatch(text, @"(?:^|[\s""'])(?:--version|-v)(?=[\s""']|$)", RegexOptions.IgnoreCase))
                return "local-version-probe";
            return "unclassified-command";
        }

        public static bool DebugTransport(IPAddress source, int sourcePort,
                                          IPAddress destination, int destinationPort, int debugPort) =>
            debugPort > 0 && debugPort <= 65535
            && IPAddress.IsLoopback(source) && IPAddress.IsLoopback(destination)
            && (sourcePort == debugPort || destinationPort == debugPort);

        public static bool Complete(long dropped, int decodeErrors, uint? kernelLost,
                                    uint? realtimeBuffersLost, bool drained, bool stopped,
                                    int roots, int maximumConcurrentRoots) =>
            dropped == 0 && decodeErrors == 0 && kernelLost == 0 && realtimeBuffersLost == 0
            && drained && stopped && roots > 0 && maximumConcurrentRoots == 1;

        public static int SelfTest()
        {
            int count = 0;
            Action<bool> check = value => {
                if (!value) throw new InvalidOperationException("Query-counter completeness/filter contract");
                count++;
            };
            var loop = IPAddress.Loopback;
            var loop6 = IPAddress.IPv6Loopback;
            var external = IPAddress.Parse("192.0.2.1");
            check(DebugTransport(loop, 10001, loop, 9222, 9222));
            check(DebugTransport(loop6, 9222, loop6, 10001, 9222));
            check(!DebugTransport(loop, 10001, loop, 7890, 9222)); // Keep a local proxy.
            check(!DebugTransport(external, 9222, loop, 10001, 9222));
            check(!DebugTransport(loop, 10001, external, 9222, 9222));
            check(!DebugTransport(loop, 1, loop, 2, 0));
            check(Complete(0, 0, 0, 0, true, true, 10, 1));
            check(!Complete(1, 0, 0, 0, true, true, 10, 1));
            check(!Complete(0, 1, 0, 0, true, true, 10, 1));
            check(!Complete(0, 0, null, 0, true, true, 10, 1));
            check(!Complete(0, 0, 0, null, true, true, 10, 1));
            check(!Complete(0, 0, 1, 0, true, true, 10, 1));
            check(!Complete(0, 0, 0, 1, true, true, 10, 1));
            check(!Complete(0, 0, 0, 0, false, true, 10, 1));
            check(!Complete(0, 0, 0, 0, true, false, 10, 1));
            check(!Complete(0, 0, 0, 0, true, true, 0, 0));
            check(!Complete(0, 0, 0, 0, true, true, 10, 2));
            // Event lifetimes retain children that a one-second poll would miss.
            var scope = new OwnedEtwScope(1, 100);
            check(scope.ProcessStart(1, 99, 100, true, null));
            check(scope.ProcessStart(2, 1, 101, false, "descendant"));
            check(scope.ProcessStop(2, 102));
            check(scope.ProcessOwned(2, 101));
            check(!scope.ProcessOwned(2, 103));
            check(!scope.ProcessStart(2, 99, 110, false, null));
            check(!scope.ProcessOwned(2, 111));
            check(ChildKind("cmd.exe", @"""C:\Windows\cmd.exe"" /c npm.cmd view package version") == "remote-metadata-query");
            check(ChildKind("node.exe", @"""C:\node.exe"" ""C:\npm-cli.js"" ""view"" ""package""") == "remote-metadata-query");
            check(ChildKind("node.exe", @"""C:\view\node.exe"" --version") == "local-version-probe");
            check(ChildKind("cmd.exe", @"""C:\Windows\cmd.exe"" /d /c exit 0") == "unclassified-command");
            check(ChildKind("msedgewebview2.exe", "browser command line not serialized") == "browser-runtime");
            check(ChildKind("other.exe", "--version") == "other-owned-process");
            return count;
        }
    }

    public static class ProductQueryCounters
    {
        private sealed class Root
        {
            public int Pid;
            public OwnedEtwScope Scope;
        }
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr OpenProcess(uint access, bool inherit, int pid);
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern bool QueryFullProcessImageNameW(IntPtr process, uint flags, StringBuilder path, ref uint size);
        [DllImport("kernel32.dll")]
        private static extern bool CloseHandle(IntPtr handle);

        private static bool ExactLiveImage(int pid, string expected)
        {
            IntPtr handle = OpenProcess(0x1000, false, pid);
            if (handle == IntPtr.Zero) return false;
            try {
                var path = new StringBuilder(32768);
                uint size = 32768;
                return QueryFullProcessImageNameW(handle, 0, path, ref size)
                    && String.Equals(path.ToString(), expected, StringComparison.OrdinalIgnoreCase);
            } finally { CloseHandle(handle); }
        }
        private static long Qpc(TraceEvent data)
        {
#pragma warning disable CS0618
            return data.TimeStampQPC;
#pragma warning restore CS0618
        }
        private static object Header(TraceEvent data) => new {
            qpc = Qpc(data), atUtc = data.TimeStamp.ToUniversalTime(), provider = data.ProviderGuid,
            eventId = (int)data.ID, opcode = (int)data.Opcode, version = data.Version,
            eventName = data.EventName
        };

        public static string Observe(string executable, string outputDirectory, string stopFile,
                                     DateTime deadlineUtc, int debugPort)
        {
            if (TraceEventSession.IsElevated() != true)
                throw new UnauthorizedAccessException("Approved elevated ETW controller required.");
            if (deadlineUtc <= DateTime.UtcNow || deadlineUtc > DateTime.UtcNow.AddMinutes(25))
                throw new ArgumentException("A current bounded observation deadline is required.");
            if (Directory.Exists(outputDirectory))
                throw new IOException("Never overwrite or repeat an observation directory.");
            executable = Path.GetFullPath(executable);
            if (!File.Exists(executable)) throw new FileNotFoundException("Bound product executable required.");
            string sessionName = "OpenCodex-BD2-" + Guid.NewGuid().ToString("N");
            // Reuse the existing strictly named native stopper. It avoids the
            // generic TraceEvent Stop API's machine-wide sample-interval write.
            Directory.CreateDirectory(outputDirectory);
            var session = new TraceEventSession(sessionName,
                TraceEventSessionOptions.Create | TraceEventSessionOptions.NoRestartOnCreate);
            session.StopOnDispose = false;
            session.BufferSizeMB = 64;
            var queue = new BlockingCollection<string>(32768);
            var budget = new OwnedEtwLineBudget(64L * 1024 * 1024);
            long dropped = 0, recorded = 0, networkEvents = 0, networkBytes = 0;
            long debugEvents = 0, debugBytes = 0, processStarts = 0, processStops = 0;
            int decoderErrors = 0, ambiguousRoots = 0, maximumConcurrentRoots = 0;
            Exception writerError = null, consumerError = null, runError = null;
            var roots = new List<Root>();
            var writer = new Thread(() => {
                try {
                    using var file = new FileStream(Path.Combine(outputDirectory, "owned-events.original.jsonl"),
                        FileMode.CreateNew, FileAccess.Write, FileShare.Read);
                    using var text = OwnedEtwLineBudget.OpenWriter(file);
                    foreach (string row in queue.GetConsumingEnumerable()) text.WriteLine(row);
                } catch (Exception error) {
                    writerError = error;
                    foreach (string row in queue.GetConsumingEnumerable()) Interlocked.Increment(ref dropped);
                }
            }) { IsBackground = true, Name = "OpenCodex-query-counter-writer" };
            writer.Start();
            Action<object> emit = row => {
                if (!budget.TrySubmit(queue, JsonSerializer.Serialize(row))) Interlocked.Increment(ref dropped);
                else Interlocked.Increment(ref recorded);
            };
            Action<TraceEvent, Action> guard = (data, action) => {
                try { action(); } catch (Exception error) {
                    Interlocked.Increment(ref decoderErrors);
                    if (decoderErrors <= 8) emit(new { kind = "decoder-error", error = error.GetType().Name });
                }
            };
            Func<int, long, Root> owner = (pid, qpc) =>
                roots.LastOrDefault(root => root.Scope.ProcessOwned(pid, qpc));
            Action<TraceEvent, int, IPAddress, int, IPAddress, int> network =
                (data, bytes, source, sourcePort, destination, destinationPort) => guard(data, () => {
                    long qpc = Qpc(data);
                    Root root = owner(data.ProcessID, qpc);
                    if (root == null) return;
                    if (bytes < 0) throw new InvalidDataException("Negative kernel network size.");
                    bool debug = QueryCounterRules.DebugTransport(source, sourcePort, destination, destinationPort, debugPort);
                    if (debug) { debugEvents++; debugBytes += bytes; }
                    else { networkEvents++; networkBytes += bytes; }
                    emit(new { kind = "network", original = Header(data), pid = data.ProcessID,
                        generation = root.Scope.ProcessGeneration(data.ProcessID, qpc), rootPid = root.Pid,
                        bytes, observerLoopbackTransport = debug,
                        peerAddressesPortsOrPayloadSaved = false, httpRequestClaimed = false });
                });
            Thread consumer = null;
            bool consumerStarted = false, drained = false, writerJoined = false;
            OwnedEtwNativeStop.Result stopped = null;
            var cleanupErrors = new List<string>();
            EventHandler emergency = (_, __) => { try { OwnedEtwNativeStop.Stop(sessionName); } catch {} };
            AppDomain.CurrentDomain.ProcessExit += emergency;
            var startedUtc = DateTime.UtcNow;
            long minimumQpc = Stopwatch.GetTimestamp();
            try {
                var kernel = OwnedEtwLifecycle.KernelBeforeSource(
                    () => session.EnableKernelProvider(KernelTraceEventParser.Keywords.Process |
                        KernelTraceEventParser.Keywords.NetworkTCPIP),
                    () => session.Source.Kernel);
                kernel.ProcessStart += data => guard(data, () => {
                    long qpc = Qpc(data);
                    bool looksLikeRoot = String.Equals(Path.GetFileName(data.ImageFileName),
                        Path.GetFileName(executable), StringComparison.OrdinalIgnoreCase);
                    bool exactRoot = looksLikeRoot && ExactLiveImage(data.ProcessID, executable);
                    if (looksLikeRoot && !exactRoot) ambiguousRoots++;
                    // Every start also expires an unrelated reuse of an owned PID.
                    foreach (Root existing in roots) {
                        bool parentOwned = existing.Scope.ProcessOwned(data.ParentID, qpc);
                        if (!existing.Scope.ProcessStart(data.ProcessID, data.ParentID, qpc,
                            false, parentOwned ? "descendant" : null)) continue;
                        processStarts++;
                        emit(new { kind = "process-start", original = Header(data), pid = data.ProcessID,
                            parentPid = data.ParentID, rootPid = existing.Pid,
                            generation = existing.Scope.ProcessGeneration(data.ProcessID, qpc),
                            imageBasename = Path.GetFileName(data.ImageFileName), uniqueProcessKey = data.UniqueProcessKey,
                            invocationKind = QueryCounterRules.ChildKind(Path.GetFileName(data.ImageFileName), data.CommandLine),
                            fullCommandLineOrSidSaved = false, identity = "ETW owned-parent lifetime" });
                    }
                    if (!exactRoot) return;
                    var root = new Root { Pid = data.ProcessID,
                        Scope = new OwnedEtwScope(data.ProcessID, minimumQpc) };
                    if (!root.Scope.ProcessStart(data.ProcessID, data.ParentID, qpc, true, null)) {
                        ambiguousRoots++; return;
                    }
                    roots.Add(root);
                    int concurrent = roots.Count(item => item.Scope.ProcessOwned(item.Pid, qpc));
                    maximumConcurrentRoots = Math.Max(maximumConcurrentRoots, concurrent);
                    processStarts++;
                    emit(new { kind = "process-start", original = Header(data), pid = data.ProcessID,
                        rootPid = root.Pid, generation = qpc, imageBasename = Path.GetFileName(executable),
                        exactLiveImageVerified = true, uniqueProcessKey = data.UniqueProcessKey,
                        fullCommandLineOrSidSaved = false, identity = "ETW start plus exact live image" });
                });
                kernel.ProcessStop += data => guard(data, () => {
                    long qpc = Qpc(data);
                    Root root = owner(data.ProcessID, qpc);
                    if (root == null) return;
                    long? generation = root.Scope.ProcessGeneration(data.ProcessID, qpc);
                    if (!root.Scope.ProcessStop(data.ProcessID, qpc)) return;
                    processStops++;
                    emit(new { kind = "process-stop", original = Header(data), pid = data.ProcessID,
                        rootPid = root.Pid, generation, uniqueProcessKey = data.UniqueProcessKey,
                        exitStatus = data.ExitStatus });
                });
                kernel.TcpIpSend += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                kernel.TcpIpRecv += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                kernel.TcpIpSendIPV6 += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                kernel.TcpIpRecvIPV6 += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                kernel.UdpIpSend += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                kernel.UdpIpRecv += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                kernel.UdpIpSendIPV6 += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                kernel.UdpIpRecvIPV6 += data => network(data, data.size, data.saddr, data.sport, data.daddr, data.dport);
                consumer = new Thread(() => {
                    try { session.Source.Process(); } catch (Exception error) { consumerError = error; }
                }) { IsBackground = true, Name = "OpenCodex-query-counter-consumer" };
                consumer.Start();
                consumerStarted = true;
                File.WriteAllText(Path.Combine(outputDirectory, "ready.json"), JsonSerializer.Serialize(new {
                    sessionName, atUtc = DateTime.UtcNow, minimumQpc,
                    keywords = "Process | NetworkTCPIP", productStarted = false, foregroundUsed = false }));
                while (DateTime.UtcNow < deadlineUtc && !File.Exists(stopFile)) {
                    if (!consumer.IsAlive || consumerError != null || writerError != null || decoderErrors != 0 || dropped != 0)
                        throw new InvalidOperationException("Event capture incomplete; preserve failure.");
                    Thread.Sleep(100);
                }
            } catch (Exception error) { runError = error; }
            finally {
                try { stopped = OwnedEtwNativeStop.Stop(sessionName); }
                catch (Exception error) { cleanupErrors.Add("stop:" + error.GetType().Name); }
                if (stopped?.StoppedOrAlreadyAbsent == true) AppDomain.CurrentDomain.ProcessExit -= emergency;
                try { drained = !consumerStarted || consumer.Join(10000); }
                catch (Exception error) { cleanupErrors.Add("drain:" + error.GetType().Name); }
                // Incomplete drain is evidence failure. Avoid disposing a live decoder.
                if (drained) {
                    try { session.Dispose(); } catch (Exception error) { cleanupErrors.Add("dispose:" + error.GetType().Name); }
                    queue.CompleteAdding();
                    writerJoined = writer.Join(10000);
                }
                bool complete = QueryCounterRules.Complete(dropped, decoderErrors,
                    stopped?.FinalEventsLost, stopped?.FinalRealTimeBuffersLost,
                    drained, stopped?.Status == 0, roots.Count, maximumConcurrentRoots)
                    && ambiguousRoots == 0 && writerJoined && writerError == null
                    && consumerError == null && runError == null && cleanupErrors.Count == 0
                    && processStarts == processStops;
                File.WriteAllText(Path.Combine(outputDirectory, "terminal.json"), JsonSerializer.Serialize(new {
                    startedAtUtc = startedUtc, finishedAtUtc = DateTime.UtcNow, deadlineUtc,
                    sessionName, minimumQpc, rootStarts = roots.Count, maximumConcurrentRoots,
                    processStarts, processStops, networkEvents, networkBytes, debugEvents, debugBytes,
                    allObservedProcessLifetimesClosed = processStarts == processStops,
                    droppedRecords = dropped, recordedEvents = recorded, decoderErrors, ambiguousRoots,
                    nativeStop = stopped, consumerDrained = drained, writerJoined,
                    writerError = writerError?.ToString(), consumerError = consumerError?.ToString(),
                    runError = runError?.ToString(), cleanupErrors, complete,
                    processCountsIncludeBrowsersAndLocalProbes = true,
                    networkBytesAreKernelTransportSizes = true, logicalHttpRequestsCounted = false,
                    unownedEventsSaved = false, processLaunchedOrKilled = false,
                    globalSampleIntervalWritten = false, fullPerformanceGatePassed = false
                }, new JsonSerializerOptions { WriteIndented = true }));
            }
            if (runError != null) throw new InvalidOperationException("Query counter failed; see preserved terminal.", runError);
            return Path.Combine(outputDirectory, "terminal.json");
        }
    }
}
