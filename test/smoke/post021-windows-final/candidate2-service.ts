import assert from 'node:assert/strict'
import {readFileSync,writeFileSync,appendFileSync,existsSync} from 'node:fs'
import {resolve,join} from 'node:path'
import {pathToFileURL} from 'node:url'
import {execFileSync} from 'node:child_process'
import {createHash,randomBytes} from 'node:crypto'
const [planFile,attempt]=process.argv.slice(2),plan=JSON.parse(readFileSync(planFile,'utf8')),root=resolve(plan.runRoot),home=join(root,'sandbox/opencodex-home')
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-candidate2-targeted-20261105'))
assert.equal(resolve(process.env.OPENCODEX_HOME??''),home)
assert.equal(resolve(process.env.CODEX_HOME??''),resolve('C:/Users/Administrator/.codex'))
const actual=execFileSync(join(process.env.SystemRoot!,'System32/WindowsPowerShell/v1.0/powershell.exe'),['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',timeout:15000,windowsHide:true}).trim()
assert.equal(actual.toLowerCase(),plan.expectedOwner.toLowerCase())
const official=join(root,'sandbox/runtime/opencodex/node_modules/@bitkyc08/opencodex'),manifest=JSON.parse(readFileSync(plan.artifactManifest,'utf8'))
assert.equal(JSON.parse(readFileSync(join(official,'package.json'),'utf8')).version,'2.97.0-post021.ownerquery.2')
for(const row of manifest.files){const b=readFileSync(join(official,row.path));assert.equal(b.length,row.bytes);assert.equal(createHash('sha256').update(b).digest('hex'),row.sha256,row.path)}
const load=(path:string)=>import(pathToFileURL(join(official,path)).href)
if(!existsSync(join(home,'config.json'))){
 const {getDefaultConfig}=await load('src/config.ts'),config=getDefaultConfig()
 Object.assign(config,{port:plan.servicePort,bindHost:'127.0.0.1',hostname:'127.0.0.1',codexAutoStart:false,oauthOpenBrowser:false,syncOnStart:false,autoUpdate:false,claudeCodeEnabled:false,websockets:false,defaultProvider:plan.fromProvider,clientIntegrations:{codex:false,grok:false,claude:false},providers:{[plan.fromProvider]:{name:plan.fromProvider,adapter:'openai-chat',baseUrl:'http://127.0.0.1:'+plan.fixturePort+'/v1',authMode:'key',allowPrivateNetwork:true,models:['raw-model'],selectedModels:['raw-model'],contextWindow:65569}},modelDiscovery:{knownModels:{[plan.fromProvider]:{ids:['raw-model'],removed:[],updatedAt:'2026-11-05T00:00:00.000Z'}},recentArrivals:{[plan.fromProvider]:[]}}})
 writeFileSync(join(home,'config.json'),JSON.stringify(config,null,2))
}
const saved=JSON.parse(readFileSync(join(home,'config.json'),'utf8'))
assert.equal(saved.port,plan.servicePort);assert.equal(Object.keys(saved.providers).length,1)
assert([plan.fromProvider,plan.toProvider].includes(saved.defaultProvider))
assert.equal(saved.providers[saved.defaultProvider].baseUrl,'http://127.0.0.1:'+plan.fixturePort+'/v1')
const fixture=Bun.serve({hostname:'127.0.0.1',port:plan.fixturePort,fetch(request){
 const path=new URL(request.url).pathname;appendFileSync(join(attempt,'fixture-requests.jsonl'),JSON.stringify({at:new Date().toISOString(),method:request.method,path})+'\n')
 if(request.method==='GET'&&path==='/v1/models')return Response.json({object:'list',data:[{id:'raw-model',object:'model',owned_by:'synthetic'}]})
 return Response.json({error:'No inference or other operation is implemented in this test fixture'},{status:405})
}})
const {startServer}=await load('src/server/index.ts'),{createReadinessGate,runStartupReadinessSync}=await load('src/server/readiness.ts')
const readinessGate=createReadinessGate(),secret=randomBytes(32).toString('base64url'),server=startServer(plan.servicePort,{localAttestationSecret:secret,readinessGate})
writeFileSync(join(home,'runtime-port.json'),JSON.stringify({pid:process.pid,port:server.port,attestationSecret:secret}))
const {syncModelsToCodex}=await load('src/codex/sync.ts')
const startup=await runStartupReadinessSync(readinessGate,()=>syncModelsToCodex(server.port,undefined,null))
writeFileSync(join(attempt,'service-ready.json'),JSON.stringify({pid:process.pid,owner:actual,version:manifest.version,tarballSha256:manifest.sha256,installedFilesVerified:manifest.files.length,officialPackage:official,port:server.port,readiness:readinessGate.getStatus(),startupStatus:startup?.status,defaultRestart:'No management/restart/adapter/process IO overrides'},null,2))
process.on('SIGTERM',()=>{fixture.stop(true);server.stop(true);process.exit(0)})
