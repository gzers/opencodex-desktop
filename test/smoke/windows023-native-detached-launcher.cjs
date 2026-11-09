// Match the original Node detached-spawn mechanics; no actual proxy is started.
const assert=require('node:assert/strict');
const {spawn}=require('node:child_process');
const {join,isAbsolute}=require('node:path');
assert(process.platform==='win32'&&process.env.GITHUB_ACTIONS==='true');
const bun=process.env.OCXD_RECORD_PROBE_BUN;
assert(isAbsolute(bun)&&process.env.OCXD_LEASE_PROBE_ROOT);
const child=spawn(bun,['--no-env-file',join(__dirname,'windows023-native-lifecycle-lease.mjs'),'service'],{
 detached:true,stdio:'ignore',windowsHide:true,env:process.env,
});
child.once('error',()=>{process.exitCode=1;});
child.unref();
