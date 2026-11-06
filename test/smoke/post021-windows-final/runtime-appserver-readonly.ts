import assert from 'node:assert/strict'
import {readFileSync,writeFileSync} from 'node:fs'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
import {execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
const [candidate,output]=process.argv.slice(2)
assert(candidate&&output)
const ps=join(process.env.SystemRoot!,'System32/WindowsPowerShell/v1.0/powershell.exe')
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
const module=await import(pathToFileURL(join(resolve(candidate),'src/codex/app-server-processes.ts')).href)
const report:any={kind:'read-only-native-query',owner,timeoutMs:8000,startedAtUtc:new Date().toISOString()}
const started=performance.now()
try{
 const rows=module.listWindowsSnapshots((script:string)=>{
  report.scriptSha256=createHash('sha256').update(script).digest('hex')
  return execFileSync(ps,['-NoProfile','-NoLogo','-NonInteractive','-Command',script],{encoding:'utf8',timeout:8000,windowsHide:true})
 })
 assert(rows.every((row:any)=>row.owner.toLowerCase()===owner.toLowerCase()))
 report.result='passed';report.totalCandidates=rows.length
 report.appServers=rows.filter((r:any)=>module.isCodexAppServerCommandLine(r.commandLine,r.executable)).map((r:any)=>({pid:r.pid,owner:r.owner,commandSha256:createHash('sha256').update(r.commandLine).digest('hex')}))
}catch(error:any){report.result='failed';report.error=error.message;process.exitCode=1}
report.elapsedMs=performance.now()-started
report.sourceSha256=createHash('sha256').update(readFileSync(join(resolve(candidate),'src/codex/app-server-processes.ts'))).digest('hex')
writeFileSync(output,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report))
