// New synthetic CREATE_NEW files only. No security mutation, privilege change,
// impersonation, original client access or Full/SACL query is exposed.
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using Microsoft.Win32.SafeHandles;

public static class PopulatedSecurityProbe024 {
    [StructLayout(LayoutKind.Sequential)]
    struct SA { public uint Length; public IntPtr Descriptor; public int Inherit; }
    [DllImport("advapi32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool ConvertStringSecurityDescriptorToSecurityDescriptorW(string value,uint revision,out IntPtr sd,out uint size);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern SafeFileHandle CreateFileW(string path,uint access,uint share,ref SA sa,uint creation,uint flags,IntPtr template);
    [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr p);
    [DllImport("advapi32.dll")] static extern void QuerySecurityAccessMask(uint mask,out uint access);
    [DllImport("advapi32.dll")] static extern uint GetSecurityInfo(SafeFileHandle h,int kind,uint mask,out IntPtr owner,out IntPtr group,out IntPtr dacl,out IntPtr sacl,out IntPtr sd);
    [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetAce(IntPtr acl,uint index,out IntPtr ace);

    public sealed class Query {
        public uint Mask {get;set;}
        public uint RequiredAccess {get;set;}
        public uint RawStatus {get;set;}
        public List<object> Aces {get;set;} = new List<object>();
        public bool? AclPointerPresent {get;set;}
        public string StructureError {get;set;}
    }
    public sealed class Result {
        public string Name {get;set;}
        public string Path {get;set;}
        public string Stage {get;set;}
        public int RawStatus {get;set;}
        public bool Created {get;set;}
        public uint TargetMask {get;set;}
        public byte TargetAce {get;set;}
        public List<Query> Queries {get;set;} = new List<Query>();
    }
    static string Hash(byte[] bytes) {
        using(var sha=SHA256.Create()) return BitConverter.ToString(sha.ComputeHash(bytes)).Replace("-", "").ToLowerInvariant();
    }
    public static Result Trial(string name,string path,string sddl,uint targetMask,byte targetAce) {
        var result=new Result{Name=name,Path=path,TargetMask=targetMask,TargetAce=targetAce,Stage="ConvertStringSecurityDescriptorToSecurityDescriptorW"};
        IntPtr descriptor; uint size;
        if(!ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl,1,out descriptor,out size)) {
            result.RawStatus=Marshal.GetLastWin32Error(); return result;
        }
        try {
            var sa=new SA{Length=(uint)Marshal.SizeOf(typeof(SA)),Descriptor=descriptor,Inherit=0};
            result.Stage="CreateFileW-CREATE_NEW-READ_CONTROL";
            using(var handle=CreateFileW(path,0x20000,1,ref sa,1,0x200000,IntPtr.Zero)) {
                if(handle.IsInvalid) {result.RawStatus=Marshal.GetLastWin32Error(); return result;}
                result.Created=true;
                foreach(uint mask in new uint[]{1,2,4,16,32,64,128,256,503}) {
                    uint access; QuerySecurityAccessMask(mask,out access);
                    var query=new Query{Mask=mask,RequiredAccess=access};
                    result.Queries.Add(query);
                    IntPtr owner,group,dacl,sacl,sd;
                    query.RawStatus=GetSecurityInfo(handle,1,mask,out owner,out group,out dacl,out sacl,out sd);
                    if(query.RawStatus!=0) {if(sd!=IntPtr.Zero) LocalFree(sd); continue;}
                    try {
                        IntPtr acl=mask==4 ? dacl : mask>=16 ? sacl : IntPtr.Zero;
                        query.AclPointerPresent=acl!=IntPtr.Zero;
                        if(acl==IntPtr.Zero) continue;
                        int aclSize=(ushort)Marshal.ReadInt16(acl,2), count=(ushort)Marshal.ReadInt16(acl,4);
                        if(aclSize<8 || count>4096) throw new InvalidOperationException("ACL bounds");
                        for(uint i=0;i<count;i++) {
                            IntPtr ace;
                            if(!GetAce(acl,i,out ace)) throw new InvalidOperationException("GetAce raw="+Marshal.GetLastWin32Error());
                            int length=(ushort)Marshal.ReadInt16(ace,2);
                            long offset=ace.ToInt64()-acl.ToInt64();
                            if(length<8 || length%4!=0 || offset<8 || offset+length>aclSize) throw new InvalidOperationException("ACE bounds");
                            byte[] bytes=new byte[length]; Marshal.Copy(ace,bytes,0,length);
                            query.Aces.Add(new {Type=bytes[0],Flags=bytes[1],Bytes=length,Sha256=Hash(bytes)});
                        }
                    } catch(Exception ex) {query.StructureError=ex.Message;}
                    finally {if(sd!=IntPtr.Zero) LocalFree(sd);}
                }
                result.Stage="CreatedAndQueried";
                return result;
            }
        } finally {LocalFree(descriptor);}
    }
}
