/** First CLI dependency: admit the service before the rest of its module graph. */
import {win32} from 'node:path';
import {acquireWindowsRuntimeLifetime,windowsLifetimeAdmissionStage} from './windows-runtime-lifetime';

// The integrated Desktop always supplies an explicit own-H. Read-only/status
// invocations never join the service Job, create a lease or gain Stop authority.
const args=process.argv.slice(2).filter(arg=>!arg.startsWith('--ocx-internal-launch-proof='));
if(process.platform==='win32'&&args[0]==='start'){
  const home=process.env.OPENCODEX_HOME?.trim();
  if(!home||!win32.isAbsolute(home))throw new Error('Desktop start requires explicit absolute runtime HOME');
  try{acquireWindowsRuntimeLifetime(home);}catch(error){
    // Fixed diagnosis only. Never serialize the exception's path, native text,
    // stack, user config, SID, secret or server response.
    const stage=windowsLifetimeAdmissionStage();
    const allowed=/^(?:entry|load-native|canonical-home|ancestor-[0-9]{1,2}|lease-create|lease-identity-acl|lease-lock|job-create|job-identity-acl|job-limits|job-self-assign|job-membership|process-times|committed)$/;
    const kind=error instanceof ReferenceError?'reference':error instanceof TypeError?'type':error instanceof RangeError?'range':'error';
    const code=error&&typeof error==='object'&&'ocxdWin32ErrorCode' in error?error.ocxdWin32ErrorCode:undefined;const win32=typeof code==='number'&&Number.isInteger(code)&&code>=0&&code<=0xffffffff?code:0;
    console.error('ocxd-lifetime-admission-failed stage='+(allowed.test(stage)?stage:'unknown')+' kind='+kind+' win32='+win32);
    throw error;
  }
}
