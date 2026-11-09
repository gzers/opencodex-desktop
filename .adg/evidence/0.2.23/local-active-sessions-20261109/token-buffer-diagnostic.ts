import {dlopen,ptr,FFIType as T} from 'bun:ffi';
import {join} from 'node:path';
const system=process.env.OCXD_SYSTEM_DIR!;
const kernel=dlopen(join(system,'kernel32.dll'),{GetCurrentProcess:{args:[],returns:T.u64},CloseHandle:{args:[T.u64],returns:T.i32}});
const advapi=dlopen(join(system,'advapi32.dll'),{OpenProcessToken:{args:[T.u64,T.u32,T.ptr],returns:T.i32},GetTokenInformation:{args:[T.u64,T.u32,T.ptr,T.u32,T.ptr],returns:T.i32}});
const token=new BigUint64Array(1);
if(!advapi.symbols.OpenProcessToken(kernel.symbols.GetCurrentProcess(),8,ptr(token)))throw Error('token-open');
try{
 const needed=new Uint32Array(1);advapi.symbols.GetTokenInformation(token[0]!,1,null,0,ptr(needed));
 if(needed[0]!<16||needed[0]!>65536)throw Error('size');
 for(const kind of ['Uint8Array','BigUint64Array']){
  const aligned=kind==='Uint8Array'?new Uint8Array(needed[0]!):new Uint8Array(new BigUint64Array(Math.ceil(needed[0]!/8)).buffer);
  const before=ptr(aligned),ok=advapi.symbols.GetTokenInformation(token[0]!,1,ptr(aligned),aligned.length,ptr(needed)),after=ptr(aligned);
  const address=new DataView(aligned.buffer).getBigUint64(aligned.byteOffset,true),base=BigInt(after);
  const materializedBase=BigInt(ptr(aligned));
  console.log(JSON.stringify({kind,ok,bytes:aligned.length,returned:needed[0],pointerStable:before===after,pointerSafeInteger:Number.isSafeInteger(after),inside:address>=base&&address+8n<=base+BigInt(aligned.length),relativeOffset:String(address-base),byteOffset:aligned.byteOffset,materializedBaseMatchesNativeBase:materializedBase===base,insideAfterMaterialization:address>=materializedBase&&address+8n<=materializedBase+BigInt(aligned.length)}));
 }
}finally{kernel.symbols.CloseHandle(token[0]!);advapi.close();kernel.close()}
