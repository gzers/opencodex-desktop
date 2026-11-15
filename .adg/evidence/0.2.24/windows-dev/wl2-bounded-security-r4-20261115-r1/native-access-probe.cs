using System;
using System.Runtime.InteropServices;
public class OcxR4Masks {
 [DllImport("advapi32.dll")] public static extern void QuerySecurityAccessMask(uint flags,out uint desired);
}
