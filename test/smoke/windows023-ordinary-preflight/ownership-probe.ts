// Read the adopted implementation with its real OS runner, in a fresh actual
// ordinary SID Profile. No injected service verdict, no service start/stop or
// client/config writes. Output fixed classifications + status/length/hash only.
import {createHash} from 'node:crypto';
import {readFileSync,existsSync} from 'node:fs';
import {join,basename} from 'node:path';
import {pathToFileURL} from 'node:url';
const root=process.env.OCXD_OWNERSHIP_DIAG_PACKAGE!;
const meta=JSON.parse(readFileSync(join(root,'package.json'),'utf8'));
if(meta.name!=='@bitkyc08/opencodex'||meta.version!=='2.97.0-desktop023.names.2')throw Error('Exact adopted runtime required');
const probe=await import(pathToFileURL(join(root,'src/service-manager-probe.ts')).href);
const native=await import(pathToFileURL(join(root,'src/integrations/native/ownership-preflight.ts')).href);
const commands:any[]=[];
const digest=(v:any)=>createHash('sha256').update(v).digest('hex');
const reason=(text:string)=>/Task Scheduler/.test(text)?'task_scheduler_unavailable':/WinSW/.test(text)?'winsw_unavailable':/state.*(unreadable|could not|malformed)/.test(text)?'service_state_unreadable':/different|disagree|another/.test(text)?'service_home_disagreement':/no service state and no service manager claim/.test(text)?'no_service_claim':'other';
const runRaw=(file:string,args:string[],options:any)=>{
 const started=Date.now();const result=probe.defaultRawProbeRunner(file,args,options);
 const decoder=new TextDecoder('utf-8');const text=decoder.decode(result.stdout)+'\n'+decoder.decode(result.stderr);
 commands.push({image:basename(file),operation:args.includes('/xml')?'task-xml':args.includes('CSV')?'task-list':args.includes('query')?'service-query':'other',elapsedMilliseconds:Date.now()-started,status:result.status,timedOut:result.timedOut,spawnFailed:result.spawnFailed,stdoutBytes:result.stdout.length,stderrBytes:result.stderr.length,stdoutSha256:digest(result.stdout),stderrSha256:digest(result.stderr),observedError:/access is denied/i.test(text)?'access_denied':/cannot find the file specified/i.test(text)?'not_found_english':/1060/.test(text)?'scm_1060_no_service':'other',accepted:false});
 return result;
};
const start=Date.now();
const ownership=native.inspectNativeCodexOwnership({runRaw});
console.log(JSON.stringify({probe:'actual-adopted-ordinary-native-ownership',version:meta.version,elapsedMilliseconds:Date.now()-start,ownership:ownership.ownership,reason:reason(ownership.reason),commands,clientHomeExists:existsSync(process.env.CODEX_HOME!),configExists:existsSync(join(process.env.OPENCODEX_HOME!,'config.json')),accepted:false,fullSoftwarePassed:false}));
