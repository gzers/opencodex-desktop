import assert from 'node:assert/strict'
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs'
import {join,resolve} from 'node:path'
import {execFileSync} from 'node:child_process'
import {pathToFileURL} from 'node:url'
import {createHash} from 'node:crypto'
const [planFile]=process.argv.slice(2),plan=JSON.parse(readFileSync(planFile,'utf8')),root=resolve(plan.runRoot)
const ps=join(process.env.SystemRoot!,'System32/WindowsPowerShell/v1.0/powershell.exe')
assert.equal(execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',timeout:15000,windowsHide:true}).trim().toLowerCase(),plan.expectedOwner.toLowerCase())
process.env.CODEX_HOME='C:/Users/Administrator/.codex';process.env.OPENCODEX_HOME=join(root,'sandbox','opencodex-home')
const output=join(root,'appserver-probe-'+new Date().toISOString().replace(/[^0-9]/g,''));mkdirSync(output)
const module=await import(pathToFileURL(join(plan.officialPackage,'src/codex/app-server-processes.ts')).href)
const sourceSha256=createHash('sha256').update(readFileSync(join(plan.officialPackage,'src/codex/app-server-processes.ts'))).digest('hex')
if(plan.candidateAppserverSha256)assert.equal(sourceSha256,plan.candidateAppserverSha256)
const report:any={result:'read-only',acceptance:false,startedAtUtc:new Date().toISOString(),candidateKind:plan.candidateKind??null,sourceSha256,queries:[],actions:'No restart/kill/config write; only snapshot/catalog-state queries'}
for(const timeout of [8000,60000]){
 const item:any={timeoutMs:timeout,diagnosticExtended:timeout>8000},started=performance.now()
 try{
  const rows=module.listWindowsSnapshots((script:string)=>{
   item.scriptSha256=createHash('sha256').update(script).digest('hex')
   try{return execFileSync(ps,['-NoProfile','-NoLogo','-NonInteractive','-Command',script],{encoding:'utf8',timeout,windowsHide:true})}
   catch(e:any){Object.assign(item,{code:e.code,status:e.status,signal:e.signal,stderrBytes:e.stderr?.length??0,stdoutBytes:e.stdout?.length??0});if(e.stdout)writeFileSync(join(output,'snapshot-'+timeout+'.stdout.local.log'),e.stdout);if(e.stderr)writeFileSync(join(output,'snapshot-'+timeout+'.stderr.local.log'),e.stderr);throw e}
  })
  item.result='success';item.totalCandidates=rows.length;item.appServers=rows.filter((r:any)=>module.isCodexAppServerCommandLine(r.commandLine,r.executable)).map((r:any)=>({pid:r.pid,owner:r.owner,commandSha256:createHash('sha256').update(r.commandLine).digest('hex')}))
 }catch(e:any){item.result='failed';item.message=String(e.message).split('\n')[0]}
 item.elapsedMs=performance.now()-started;report.queries.push(item)
}
const stateStart=performance.now()
try{module.resetCodexAppServerCatalogStateCache();report.defaultCatalogState=module.collectCodexAppServerCatalogState();report.defaultCatalogElapsedMs=performance.now()-stateStart}catch(e:any){report.catalogError=e.message}
report.finishedAtUtc=new Date().toISOString();writeFileSync(join(output,'probe.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({output,queries:report.queries,state:report.defaultCatalogState,elapsedMs:report.defaultCatalogElapsedMs}))
