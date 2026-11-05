import assert from 'node:assert/strict'
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
import {execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
const [planFile]=process.argv.slice(2),plan=JSON.parse(readFileSync(planFile,'utf8')),root=resolve(plan.runRoot)
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'))
const ps=join(process.env.SystemRoot!,'System32/WindowsPowerShell/v1.0/powershell.exe')
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(owner.toLowerCase(),plan.expectedOwner.toLowerCase())
const output=join(root,'default-process-probe-'+new Date().toISOString().replace(/[^0-9]/g,''));mkdirSync(output)
const {windowsDesktopAppAdapter:adapter,windowsDefaultExec:originalExec}=await import(pathToFileURL(join(plan.officialPackage,'src/codex/desktop-app/windows.ts')).href)
const report:any={result:'running-readonly',acceptance:false,owner,startedAtUtc:new Date().toISOString(),candidateKind:plan.candidateKind??null,adapterSha256:createHash('sha256').update(readFileSync(join(plan.officialPackage,'src/codex/desktop-app/windows.ts'))).digest('hex'),calls:[],processActions:'none; discover/list only; no restart methods invoked'}
let phase='discovery'
const observer=(extended=false)=>(file:string,args:string[],options:any)=>{
 const start=performance.now(),entry:any={phase,extendedDiagnosticOnly:extended,file,timeout:extended?60000:options?.timeout,scriptSha256:createHash('sha256').update(args.join('\n')).digest('hex')}
 try{
  const out=originalExec(file,args,extended?{...options,timeout:60000}:options)
  entry.exit='success';entry.stdoutBytes=Buffer.byteLength(out);return out
 }catch(error:any){
  Object.assign(entry,{exit:'error',code:error.code,status:error.status,signal:error.signal,message:error.message?.split('\n')[0],stdoutBytes:error.stdout?.length??0,stderrBytes:error.stderr?.length??0})
  // Preserve exact subprocess diagnostics locally; sanitize before contribution.
  if(error.stdout)writeFileSync(join(output,phase+'.stdout.log'),error.stdout)
  if(error.stderr)writeFileSync(join(output,phase+'.stderr.log'),error.stderr)
  throw error
 }finally{entry.elapsedMs=performance.now()-start;report.calls.push(entry)}
}
try{
 report.install=adapter.discover(observer())
 assert(report.install,'Default package discovery failed')
 phase='default-list';report.defaultList=adapter.listProcesses(observer(),report.install)
 phase='extended-readonly-list';report.extendedReadonlyList=adapter.listProcesses(observer(true),report.install)
 report.result=report.defaultList===null?'default-process-probe-failed':'default-process-probe-observed'
}catch(error:any){report.result='failed';report.error=error.message}
finally{report.finishedAtUtc=new Date().toISOString();writeFileSync(join(output,'probe.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({result:report.result,output,calls:report.calls}))}
