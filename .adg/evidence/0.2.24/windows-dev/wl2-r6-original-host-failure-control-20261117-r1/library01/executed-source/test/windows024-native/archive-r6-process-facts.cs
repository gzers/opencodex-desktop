using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;

// Read-only observation. No terminate, resume, Job handle, impersonation,
// privileges, Owner constructor or lifecycle-success receipt exists here.
public static class R6ArchiveProcessFacts {
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern IntPtr OpenProcess(uint access, bool inherit, uint pid);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool GetProcessTimes(IntPtr handle, out long creation, out long exit, out long kernel, out long user);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    private static extern bool QueryFullProcessImageName(IntPtr handle, uint flags, StringBuilder path, ref uint size);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool ProcessIdToSessionId(uint pid, out uint session);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern uint WaitForSingleObject(IntPtr handle, uint timeout);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool CloseHandle(IntPtr handle);
    public sealed class Facts {
        public uint pid;
        public string filetime;
        public string image;
        public uint session;
        public bool exited;
        public string source = "fresh query-only native handle; no Job/Owner/NormalExit or cleanup authority";
    }
    public static Facts Observe(uint pid) {
        IntPtr handle = OpenProcess(0x1000 | 0x100000, false, pid);
        if (handle == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        try {
            long creation, exit, kernel, user;
            if (!GetProcessTimes(handle, out creation, out exit, out kernel, out user)) throw new Win32Exception(Marshal.GetLastWin32Error());
            uint session;
            if (!ProcessIdToSessionId(pid, out session)) throw new Win32Exception(Marshal.GetLastWin32Error());
            StringBuilder image = new StringBuilder(32768); uint size = 32768;
            if (!QueryFullProcessImageName(handle, 0, image, ref size)) throw new Win32Exception(Marshal.GetLastWin32Error());
            uint wait = WaitForSingleObject(handle, 0);
            if (wait != 0 && wait != 258) throw new Win32Exception(Marshal.GetLastWin32Error());
            return new Facts {pid=pid, filetime=creation.ToString(System.Globalization.CultureInfo.InvariantCulture), image=image.ToString(), session=session, exited=wait==0};
        } finally {
            if (!CloseHandle(handle)) throw new Win32Exception(Marshal.GetLastWin32Error());
        }
    }
}
