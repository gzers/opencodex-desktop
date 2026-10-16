// Controlled self-process proof or an explicitly bound existing GPU. Never
// launches a process, replays query fixtures, or writes an external target.
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using Microsoft.Diagnostics.Tracing;
using Microsoft.Diagnostics.Tracing.Parsers;
using Microsoft.Diagnostics.Tracing.Session;

namespace OpenCodex.Diagnostics
{
    public static class ControlledMemoryRecorder
    {
        private const int ProbeBytes = 8 * 1024 * 1024;
        private const int MaximumRecords = 4096;
        private const int MaximumSerializedBytes = 4 * 1024 * 1024;
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr VirtualAlloc(IntPtr address, UIntPtr size, uint allocationType, uint protection);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool VirtualFree(IntPtr address, UIntPtr size, uint freeType);

        private static IntPtr AllocateProbe()
        {
            IntPtr address = VirtualAlloc(IntPtr.Zero, (UIntPtr)ProbeBytes, 0x3000, 4);
            if (address == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
            for (int offset = 0; offset < ProbeBytes; offset += 4096)
                Marshal.WriteByte(address, offset, 1);
            return address;
        }
        private static void FreeProbe(IntPtr address)
        {
            if (!VirtualFree(address, UIntPtr.Zero, 0x8000))
                throw new Win32Exception(Marshal.GetLastWin32Error());
        }
        public static int SelfTest()
        {
            // Validate the real controlled allocation API without opening ETW.
            if (IntPtr.Size != 8 || Environment.SystemPageSize != 4096)
                throw new PlatformNotSupportedException("Windows x64/4KiB pages required.");
            IntPtr address = AllocateProbe();
            try {
                if (Marshal.ReadByte(address) != 1 || Marshal.ReadByte(address, ProbeBytes - 4096) != 1)
                    throw new InvalidOperationException("Controlled pages were not committed and touched.");
            } finally { FreeProbe(address); }
            return 3;
        }
        public static string RunOnce(string outputDirectory)
        {
            using var self = Process.GetCurrentProcess();
            if (!String.Equals(Path.GetFileName(self.MainModule.FileName), "pwsh.exe", StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("This diagnostic reads its PowerShell controller only.");
            return Capture(outputDirectory, self, null, true, 0);
        }
        private static bool NativeSetMatches(Process desktopRoot)
        {
            Process[] native = Process.GetProcessesByName("opencodex-desktop");
            Process[] cli = Process.GetProcessesByName("ocxd");
            try {
                return cli.Length == 0 && (desktopRoot == null
                    ? native.Length == 0
                    : native.Length == 1 && native[0].Id == desktopRoot.Id);
            } finally {
                foreach (var process in native) process.Dispose();
                foreach (var process in cli) process.Dispose();
            }
        }
        private static void RequireImage(Process process, long created, string image, string digest, string leaf)
        {
            if (created <= 0 || !Path.IsPathFullyQualified(image) || digest == null ||
                digest.Length != 64 || digest.Any(c => !"0123456789abcdef".Contains(c)) ||
                !String.Equals(Path.GetFileName(image), leaf, StringComparison.OrdinalIgnoreCase))
                throw new ArgumentException("Exact bound image, digest and generation required.");
            if (process.HasExited || process.StartTime.ToUniversalTime().ToFileTimeUtc() != created ||
                !String.Equals(process.MainModule.FileName, image, StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("Retained bound process identity changed.");
            using var stream = File.OpenRead(image);
            string actual = Convert.ToHexString(SHA256.HashData(stream)).ToLowerInvariant();
            if (actual != digest) throw new InvalidOperationException("Bound image digest changed.");
        }
        // The caller retains root, GPU and ancestor leases, verifies actual
        // TokenUser/elevation/ancestry/GPU argv, and revalidates them after this
        // call. This method never creates a process or writes target memory.
        public static string RunBoundGpu(string outputDirectory, Process root, Process gpu,
            long rootCreated, long gpuCreated, string rootImage, string gpuImage,
            string rootDigest, string gpuDigest, int seconds)
        {
            if (root == null || gpu == null || root.Id == gpu.Id || seconds < 1 || seconds > 45)
                throw new ArgumentException("Separate bound desktop and GPU with a 1-45 second diagnostic window required.");
            _ = root.SafeHandle;
            _ = gpu.SafeHandle;
            RequireImage(root, rootCreated, rootImage, rootDigest, "opencodex-desktop.exe");
            RequireImage(gpu, gpuCreated, gpuImage, gpuDigest, "msedgewebview2.exe");
            if (!NativeSetMatches(root)) throw new InvalidOperationException("One exact native desktop required.");
            return Capture(outputDirectory, gpu, root, false, seconds);
        }
        private static string Capture(string outputDirectory, Process self, Process desktopRoot,
            bool controlledProbe, int seconds)
        {
            if (TraceEventSession.IsElevated() != true)
                throw new UnauthorizedAccessException("Memory ETW recording requires an elevated token.");
            if (!NativeSetMatches(desktopRoot))
                throw new InvalidOperationException("Native set does not match the admitted diagnostic mode.");
            if (Directory.Exists(outputDirectory))
                throw new IOException("New controlled output required; no repeat or overwrite.");
            var selfHandle = self.SafeHandle;
            int selfPid = self.Id;
            long created = self.StartTime.ToUniversalTime().ToFileTimeUtc();
            long? rootCreated = desktopRoot?.StartTime.ToUniversalTime().ToFileTimeUtc();
            string name = "OpenCodex-Memory-" + Guid.NewGuid().ToString("N");
            if (TraceEventSession.GetActiveSessionNames().Contains(name))
                throw new InvalidOperationException("Never attach or restart an existing session.");
            Directory.CreateDirectory(outputDirectory);
            var rows = new List<string>();
            var allocations = new List<Tuple<ulong, long, int, double>>();
            var stacks = new List<Tuple<int, double, int>>();
            object gate = new object();
            int omittedRecords = 0, callbackErrors = 0, serializedBytes = 0;
            Exception captureError = null, consumerError = null;
            var cleanupErrors = new List<string>();
            var started = DateTime.UtcNow;
            ulong probeAddress = 0;
            bool probeFreed = false, enabled = false, joined = false, stopProcessingUsed = false;
            Thread consumer = null;
            ETWTraceEventSource source = null;
            OwnedEtwNativeStop.Result stop = null;
            var session = new TraceEventSession(name,
                TraceEventSessionOptions.Create | TraceEventSessionOptions.NoRestartOnCreate);
            session.BufferSizeMB = 32;
            session.StopOnDispose = false;
            Action<object> emit = value => {
                string line = JsonSerializer.Serialize(value);
                int bytes = Encoding.UTF8.GetByteCount(line) + 1;
                lock (gate) {
                    if (rows.Count >= MaximumRecords || serializedBytes + bytes > MaximumSerializedBytes) {
                        omittedRecords++;
                        return;
                    }
                    rows.Add(line);
                    serializedBytes += bytes;
                }
            };
            Action<Action> guard = action => {
                try { action(); } catch { Interlocked.Increment(ref callbackErrors); }
            };
            try {
                OwnedEtwLifecycle.KernelBeforeSource(
                    () => session.EnableKernelProvider(
                        KernelTraceEventParser.Keywords.Process | KernelTraceEventParser.Keywords.Thread
                        | KernelTraceEventParser.Keywords.ImageLoad | KernelTraceEventParser.Keywords.VirtualAlloc,
                        KernelTraceEventParser.Keywords.VirtualAlloc),
                    () => { source = session.Source; return source; });
                enabled = true;
                var kernel = source.Kernel;
                kernel.VirtualMemAlloc += data => {
                    if (data.ProcessID != selfPid) return;
                    guard(() => {
                        lock (gate) {
                            if (allocations.Count >= MaximumRecords) { omittedRecords++; return; }
                            allocations.Add(Tuple.Create(
                                data.BaseAddr, Convert.ToInt64(data.Length), data.ThreadID, data.TimeStampRelativeMSec));
                        }
                        emit(new { kind = "virtual-allocation", pid = data.ProcessID, tid = data.ThreadID,
                            atUtc = data.TimeStamp.ToUniversalTime().ToString("o"),
                            relativeMilliseconds = data.TimeStampRelativeMSec,
                            length = Convert.ToInt64(data.Length), flags = Convert.ToUInt64(data.Flags),
                            rawAllocationAddressSaved = false });
                    });
                };
                kernel.VirtualMemFree += data => {
                    if (data.ProcessID != selfPid) return;
                    guard(() => emit(new { kind = "virtual-free", pid = data.ProcessID, tid = data.ThreadID,
                        atUtc = data.TimeStamp.ToUniversalTime().ToString("o"),
                        length = Convert.ToInt64(data.Length), flags = Convert.ToUInt64(data.Flags),
                        rawAllocationAddressSaved = false }));
                };
                kernel.StackWalkStack += data => {
                    if (data.ProcessID != selfPid) return;
                    guard(() => {
                        if (data.FrameCount > 128) { Interlocked.Increment(ref callbackErrors); return; }
                        lock (gate) {
                            if (stacks.Count >= MaximumRecords) { omittedRecords++; return; }
                            stacks.Add(Tuple.Create(data.ThreadID, data.EventTimeStampRelativeMSec, data.FrameCount));
                        }
                        var frames = new string[data.FrameCount];
                        for (int i = 0; i < frames.Length; i++)
                            frames[i] = data.InstructionPointer(i).ToString("x");
                        emit(new { kind = "allocation-stack", pid = data.ProcessID, tid = data.ThreadID,
                            eventRelativeMilliseconds = data.EventTimeStampRelativeMSec,
                            frameCount = data.FrameCount, instructionPointersHex = frames,
                            ownedControllerFramesOnly = controlledProbe, boundGpuFramesOnly = !controlledProbe });
                    });
                };
                var ready = new ManualResetEventSlim(false);
                consumer = new Thread(() => {
                    try { ready.Set(); source.Process(); }
                    catch (Exception error) { consumerError = error; }
                }) { IsBackground = true, Name = "OpenCodex-controlled-memory" };
                consumer.Start();
                if (!ready.Wait(2000)) throw new TimeoutException("Controlled trace reader not ready.");
                if (controlledProbe) {
                    IntPtr address = AllocateProbe();
                    probeAddress = unchecked((ulong)address.ToInt64());
                    try { Thread.Sleep(250); }
                    finally { FreeProbe(address); probeFreed = true; }
                    // ETW's first real-time buffer may take up to a second to flush.
                    Thread.Sleep(1500);
                } else {
                    using (var output = new FileStream(Path.Combine(outputDirectory, "recording-ready.json"),
                        FileMode.CreateNew, FileAccess.Write, FileShare.Read))
                    using (var writer = new StreamWriter(output, new UTF8Encoding(false)))
                        writer.Write(JsonSerializer.Serialize(new {
                            atUtc = DateTime.UtcNow, targetPid = selfPid,
                            targetCreationFileTime = created.ToString(), rootPid = desktopRoot.Id,
                            maximumObservationSeconds = seconds, nativeDesktopStartedByRecorder = false
                        }) + "\n");
                    var timer = Stopwatch.StartNew();
                    while (timer.ElapsedMilliseconds < seconds * 1000L) {
                        if (self.HasExited || desktopRoot.HasExited)
                            throw new InvalidOperationException("Bound target or root exited during observation.");
                        Thread.Sleep(100);
                    }
                }
            } catch (Exception error) { captureError = error; }
            finally {
                try { stop = OwnedEtwNativeStop.Stop(name); }
                catch (Exception error) { cleanupErrors.Add("stop:" + error.GetType().Name); }
                if (consumer != null) {
                    joined = consumer.Join(10000);
                    if (!joined) {
                        stopProcessingUsed = true;
                        try { source?.StopProcessing(); joined = consumer.Join(2000); }
                        catch (Exception error) { cleanupErrors.Add("reader:" + error.GetType().Name); }
                    }
                }
                try { session.Dispose(); }
                catch (Exception error) { cleanupErrors.Add("dispose:" + error.GetType().Name); }
            }
            List<Tuple<ulong, long, int, double>> matching;
            int matchingStackCount;
            lock (gate) {
                matching = controlledProbe
                    ? allocations.Where(row => row.Item1 == probeAddress && row.Item2 == ProbeBytes).ToList()
                    : allocations.ToList();
                matchingStackCount = matching.Count(row =>
                    stacks.Any(stack => stack.Item1 == row.Item3 && stack.Item2 == row.Item4 && stack.Item3 > 0));
                using var output = new FileStream(Path.Combine(outputDirectory, "owned-memory.events.jsonl"),
                    FileMode.CreateNew, FileAccess.Write, FileShare.Read);
                using var writer = new StreamWriter(output, new UTF8Encoding(false));
                foreach (string row in rows) { writer.Write(row); writer.Write('\n'); }
            }
            bool sessionAbsent = !TraceEventSession.GetActiveSessionNames().Contains(name);
            bool sameSelf = !self.HasExited && self.StartTime.ToUniversalTime().ToFileTimeUtc() == created;
            bool sameRoot = desktopRoot == null || (!desktopRoot.HasExited &&
                desktopRoot.StartTime.ToUniversalTime().ToFileTimeUtc() == rootCreated);
            bool passed = enabled && captureError == null && consumerError == null && callbackErrors == 0
                && omittedRecords == 0 && (!controlledProbe || (probeFreed && matching.Count > 0 && matchingStackCount > 0))
                && stop != null && stop.Status == 0 && stop.FinalEventsLost == 0
                && stop.FinalRealTimeBuffersLost == 0 && joined && !stopProcessingUsed
                && cleanupErrors.Count == 0 && sessionAbsent && sameSelf && sameRoot && NativeSetMatches(desktopRoot);
            string result = JsonSerializer.Serialize(new {
                passed, startedAtUtc = started, finishedAtUtc = DateTime.UtcNow,
                selfPid = controlledProbe ? (int?)selfPid : null,
                selfCreationFileTime = controlledProbe ? created.ToString() : null,
                targetPid = selfPid, targetCreationFileTime = created.ToString(),
                rootPid = desktopRoot?.Id, rootCreationFileTime = rootCreated?.ToString(),
                controlledSelf = controlledProbe, boundGpuObservation = !controlledProbe,
                sessionName = name, configuredBufferMiB = 32,
                probeBytes = controlledProbe ? (int?)ProbeBytes : null, probeFreed,
                matchingProbeAllocationEvents = controlledProbe ? matching.Count : 0,
                matchingAllocationStacks = matchingStackCount, observedAllocationEvents = matching.Count,
                allocationEvidenceObserved = matching.Count > 0 && matchingStackCount > 0,
                records = rows.Count, serializedBytes, maximumSerializedBytes = MaximumSerializedBytes,
                omittedRecords, callbackErrors, nativeStop = stop, readerJoined = joined, stopProcessingUsed,
                ownedSessionAbsent = sessionAbsent,
                retainedSelfGenerationUnchanged = controlledProbe ? (bool?)sameSelf : null,
                retainedTargetGenerationUnchanged = sameSelf,
                retainedRootGenerationUnchanged = desktopRoot != null ? (bool?)sameRoot : null,
                retainedSelfProcessHandle = controlledProbe ? (bool?)(!selfHandle.IsInvalid && !selfHandle.IsClosed) : null,
                retainedTargetProcessHandle = !selfHandle.IsInvalid && !selfHandle.IsClosed,
                captureError = captureError?.GetType().Name, consumerError = consumerError?.GetType().Name,
                cleanupErrors, globalSampleIntervalExplicitlyWritten = false,
                globalSampleIntervalBeforeAfterMeasured = false, genericSessionStopInvoked = false,
                systemWideEtlWritten = false, unrelatedEventPayloadsSerialized = false,
                rawAllocationAddressSaved = false, ownStackInstructionPointersSaved = controlledProbe,
                boundGpuStackInstructionPointersSaved = !controlledProbe,
                targetMemoryContentsSaved = false, nativeDesktopStarted = false,
                originalTestLibraryExecuted = false, officialQueryNodeExecuted = false,
                fullProductGatePassed = false
            });
            using (var output = new FileStream(Path.Combine(outputDirectory, "result.json"),
                FileMode.CreateNew, FileAccess.Write, FileShare.Read))
            using (var writer = new StreamWriter(output, new UTF8Encoding(false)))
                writer.Write(result + "\n");
            return result;
        }
    }
}
