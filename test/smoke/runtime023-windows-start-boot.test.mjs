// Evaluate the exact bootstrap body with hermetic OS/argument seams only.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {win32} from 'node:path';
const source=readFileSync(new URL('./runtime023-windows-start-boot.ts',import.meta.url),'utf8');
const body=source.replace(/^import .*;\r?\n/gm,'');
const execute=new Function('process','win32','acquireWindowsRuntimeLifetime','windowsLifetimeAdmissionStage','console',body);
for(const argv of [['bun','cli','start'],['bun','cli','--ocx-internal-launch-proof=fixture','start','--port','55581']]){
 const calls=[];execute({platform:'win32',argv,env:{OPENCODEX_HOME:'C:\\Users\\fixture\\.codex-desktop\\opencodex\\home'}},win32,home=>calls.push(home));
 assert.deepEqual(calls,['C:\\Users\\fixture\\.codex-desktop\\opencodex\\home']);
}
for(const command of ['status','stop','version','ready','capabilities','update']){
 execute({platform:'win32',argv:['bun','cli',command],env:{}},win32,()=>assert.fail('Read-only or stop cannot create service lifetime'));
}
for(const home of [undefined,'relative','~/.opencodex','']){
 assert.throws(()=>execute({platform:'win32',argv:['bun','cli','start'],env:{OPENCODEX_HOME:home}},win32,()=>assert.fail('Invalid explicit H must refuse first')));
}
const expected=Object.assign(new Error('native-denied-secret'),{ocxdWin32ErrorCode:5});const markers=[];
assert.throws(()=>execute({platform:'win32',argv:['bun','cli','start'],env:{OPENCODEX_HOME:'C:\\fixture'}},win32,()=>{throw expected},()=> 'job-self-assign',{error:text=>markers.push(text)}),error=>error===expected);
assert.deepEqual(markers,['ocxd-lifetime-admission-failed stage=job-self-assign kind=error win32=5']);

execute({platform:'darwin',argv:['bun','cli','start'],env:{}},win32,()=>assert.fail('No Windows Job on macOS'));
console.log('Exact early boot body: start/proof args, no readonly mutation, explicit H refusal and native failure propagation pass; not native Windows proof');
