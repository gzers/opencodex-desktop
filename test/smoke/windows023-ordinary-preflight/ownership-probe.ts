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

if(process.env.OCXD_READONLY_STOP_COSTS==='1'){
 const {spawnSync}=await import('node:child_process');
 const wrappers=await import(pathToFileURL(join(root,'src/lib/windows-service-wrappers.ts')).href);
 const elevation=await import(pathToFileURL(join(root,'src/lib/windows-elevation.ts')).href);
 const state=await import(pathToFileURL(join(root,'src/service/state.ts')).href);
 const service=await import(pathToFileURL(join(root,'src/service.ts')).href);
 const scheduler=await import(pathToFileURL(join(root,'src/service/windows-scheduler.ts')).href);
 const winsw=await import(pathToFileURL(join(root,'src/lib/winsw.ts')).href);
 const phases:any[]=[];
 const measure=(name:string,work:()=>unknown)=>{const t=performance.now();try{const value=work();phases.push({name,milliseconds:performance.now()-t,value,accepted:false});}catch(error){phases.push({name,milliseconds:performance.now()-t,errorType:error?.constructor?.name??'unknown',accepted:false});}};
 measure('official-respawn-risk',()=>({risk:service.installedServiceRespawnRisk()}));
 measure('official-scheduler-presence',()=>({status:scheduler.probeWindowsSchedulerTask().status}));
 measure('official-native-service-presence',()=>({status:winsw.statusWinswRaw()}));
 const original=wrappers.windowsWrapperKillScript([state.windowsServiceScriptPath(),state.windowsLauncherVbsPath()]);
 const sink='ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }';
 if(original.split(sink).length!==2||!original.endsWith(sink))throw Error('Exact original wrapper sink changed');
 // Keep the original token/path matcher; replace its sole mutation sink with
 // sorted IDs. Never invoke the wrapper killer or a service stop function.
 const readonly=original.slice(0,-sink.length)+'ForEach-Object { $_.ProcessId } | Sort-Object';
 const fullQuery='Get-CimInstance Win32_Process';
 if(readonly.split(fullQuery).length!==2||/Stop-Process|taskkill|Invoke-CimMethod|Remove-CimInstance/.test(readonly))throw Error('Read-only projection admission failed');
 const filter=String.fromCharCode(34)+"Name='cmd.exe' OR Name='wscript.exe' OR Name='cscript.exe'"+String.fromCharCode(34);
 const filtered=readonly.replace(fullQuery,fullQuery+' -Filter '+filter);
 const powershell=elevation.resolveTrustedWindowsPowerShellExe();
 const samples:any[]=[];
 // Counterbalance fresh processes; retain the original 5s child budget.
 for(const variant of ['original-readonly','filtered-readonly','filtered-readonly','original-readonly']){
  const command=variant==='original-readonly'?readonly:filtered,t=performance.now();
  const value=spawnSync(powershell,['-NoProfile','-NoLogo','-NonInteractive','-Command',command],{stdio:['ignore','pipe','pipe'],timeout:5000,windowsHide:true});
  const stdout=Buffer.from(value.stdout??[]),stderr=Buffer.from(value.stderr??[]);
  samples.push({variant,milliseconds:performance.now()-t,status:value.status,signal:value.signal,errorCode:value.error?.code??null,stdoutBytes:stdout.length,stdoutSha256:digest(stdout),stderrBytes:stderr.length,stderrSha256:digest(stderr),accepted:false});
 }
 const sourceFiles:any={};for(const relative of ['src/service/windows-ops.ts','src/lib/windows-service-wrappers.ts']){const b=readFileSync(join(root,relative));sourceFiles[relative]={bytes:b.length,sha256:digest(b)};}
 console.log(JSON.stringify({probe:'ordinary-readonly-stop-costs',accepted:false,diagnosticOnly:true,fullSoftwarePassed:false,phases,samples,sourceFiles,matcherBytesUnchanged:true,mutationSinkReplaced:true,serviceOrTaskMutation:false,productStarted:false,originalChildBudgetMilliseconds:5000,limitations:'Read-only projection under one fresh ordinary SID. Filtered query is only an experiment; no runtime patch or real stop acceptance. Zero fixture matches do not prove all matching counterexamples.'}));
}
