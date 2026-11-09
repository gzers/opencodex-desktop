/** First CLI dependency: admit the service before the rest of its module graph. */
import {win32} from 'node:path';
import {acquireWindowsRuntimeLifetime} from './windows-runtime-lifetime';

// The integrated Desktop always supplies an explicit own-H. Read-only/status
// invocations never join the service Job, create a lease or gain Stop authority.
const args=process.argv.slice(2).filter(arg=>!arg.startsWith('--ocx-internal-launch-proof='));
if(process.platform==='win32'&&args[0]==='start'){
  const home=process.env.OPENCODEX_HOME?.trim();
  if(!home||!win32.isAbsolute(home))throw new Error('Desktop start requires explicit absolute runtime HOME');
  acquireWindowsRuntimeLifetime(home);
}
