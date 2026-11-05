import assert from 'node:assert/strict'
import {readFileSync,writeFileSync,existsSync} from 'node:fs'
import {join,resolve} from 'node:path'
import {execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
const [planFile]=process.argv.slice(2),plan=JSON.parse(readFileSync(planFile,'utf8')),root=resolve(plan.runRoot)
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'))
const ps=join(process.env.SystemRoot!,'System32/WindowsPowerShell/v1.0/powershell.exe')
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(owner.toLowerCase(),'ezio-mini\\administrator')
const token=readFileSync(join(root,'sandbox/opencodex-home/admin-api-token'),'utf8').trim()
const report:any={kind:'read-only-current-client-binding',owner,atUtc:new Date().toISOString(),mutations:'none; GET health/settings/native-integrations plus exact CLI --version only',runtimePackage:JSON.parse(readFileSync(join(plan.officialPackage,'package.json'),'utf8')).version}
const health=await(await fetch('http://127.0.0.1:'+plan.servicePort+'/healthz',{signal:AbortSignal.timeout(10000)})).json()
assert.equal(health.pid,69172);report.service={pid:health.pid,version:health.version}
for(const path of ['/api/settings','/api/native-integrations']){
 const response=await fetch('http://127.0.0.1:'+plan.servicePort+path,{headers:{Authorization:'Bearer '+token},signal:AbortSignal.timeout(30000)})
 assert(response.ok);const value=await response.json()
 if(path==='/api/settings')report.codexRuntime=value.codexRuntime
 else report.codexIntegration=value.clients.find((c:any)=>c.clientId==='codex')
}
report.planClient={path:plan.codexExe,exists:existsSync(plan.codexExe)}
if(report.planClient.exists){
 report.planClient.sha256=createHash('sha256').update(readFileSync(plan.codexExe)).digest('hex')
 try{report.planClient.version=execFileSync(plan.codexExe,['--version'],{encoding:'utf8',windowsHide:true,timeout:10000}).trim()}
 catch(e:any){report.planClient.versionError={code:e.code,status:e.status}}
}
const name=join(root,'runtime-binding-readonly-'+new Date().toISOString().replace(/[^0-9]/g,'')+'.json')
writeFileSync(name,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({output:name,codexRuntime:report.codexRuntime,codexIntegration:report.codexIntegration,planClient:report.planClient}))
