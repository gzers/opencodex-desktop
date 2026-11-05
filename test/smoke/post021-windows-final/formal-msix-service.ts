import assert from 'node:assert/strict'
import {readFileSync,writeFileSync,appendFileSync,existsSync} from 'node:fs'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
import {execFileSync} from 'node:child_process'
import {randomBytes,createHash} from 'node:crypto'

const [planFile,runRoot,attemptRoot=runRoot]=process.argv.slice(2),plan=JSON.parse(readFileSync(planFile,'utf8'))
const root=resolve(runRoot),home=join(root,'sandbox','opencodex-home')
const artifacts=resolve(attemptRoot)
assert(root.startsWith(resolve('D:/Users/Administrator/WorkSpace/Test')+'\\'))
assert.equal(resolve(process.env.OPENCODEX_HOME??''),resolve(home))
assert.equal(resolve(process.env.CODEX_HOME??''),resolve('C:/Users/Administrator/.codex'))
const ps=join(process.env.SystemRoot!,'System32/WindowsPowerShell/v1.0/powershell.exe')
const actual=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(actual.toLowerCase(),plan.expectedOwner.toLowerCase())
const official=plan.officialPackage,load=async(path:string)=>import(pathToFileURL(join(official,path)).href)
assert.equal(JSON.parse(readFileSync(join(official,'package.json'),'utf8')).version,'2.84.0')
const adapterSha256=createHash('sha256').update(readFileSync(join(official,'src/codex/desktop-app/windows.ts'))).digest('hex')
if(plan.candidateKind){assert.equal(plan.candidateKind,'local-runtime-source-candidate-not-published2.84.0');assert.equal(adapterSha256,plan.candidateAdapterSha256)}
if(plan.candidateAppserverSha256)assert.equal(createHash('sha256').update(readFileSync(join(official,'src/codex/app-server-processes.ts'))).digest('hex'),plan.candidateAppserverSha256)
const {getDefaultConfig}=await load('src/config.ts')
const config=getDefaultConfig()
Object.assign(config,{port:plan.servicePort,bindHost:'127.0.0.1',hostname:'127.0.0.1',codexAutoStart:false,oauthOpenBrowser:false,syncOnStart:false,autoUpdate:false,claudeCodeEnabled:false,websockets:false,defaultProvider:'Post021AdminFixture',clientIntegrations:{codex:false,grok:false,claude:false},providers:{Post021AdminFixture:{adapter:'openai-chat',baseUrl:'http://127.0.0.1:'+plan.fixturePort+'/v1',authMode:'key',allowPrivateNetwork:true,models:['raw-model'],selectedModels:['raw-model'],contextWindow:65536}},modelDiscovery:{knownModels:{Post021AdminFixture:{ids:['raw-model'],removed:[],updatedAt:new Date().toISOString()}}}})
if(existsSync(join(home,'config.json'))){
 const existing=JSON.parse(readFileSync(join(home,'config.json'),'utf8'))
 assert.equal(existing.port,plan.servicePort)
 assert.equal(existing.defaultProvider,'Post021AdminFixture')
 assert(Object.keys(existing.providers).every(name=>name==='Post021AdminFixture'))
 assert.equal(existing.providers.Post021AdminFixture.baseUrl,'http://127.0.0.1:'+plan.fixturePort+'/v1')
}else writeFileSync(join(home,'config.json'),JSON.stringify(config,null,2))
// Only model-list reads are implemented; generation and every other operation
// fail locally. No actual provider, WebDAV or inference service is contacted.
const fixture=Bun.serve({hostname:'127.0.0.1',port:plan.fixturePort,fetch(request){
 const path=new URL(request.url).pathname
 appendFileSync(join(artifacts,'fixture-requests.jsonl'),JSON.stringify({at:new Date().toISOString(),method:request.method,path})+'\n')
 if(request.method==='GET'&&path==='/v1/models')return Response.json({object:'list',data:[{id:'raw-model',object:'model',owned_by:'synthetic'}]})
 return Response.json({error:{message:'Local formal-restart fixture does not implement generation'}},{status:405})
}})
const {startServer}=await load('src/server/index.ts'),{createReadinessGate,runStartupReadinessSync}=await load('src/server/readiness.ts')
const readinessGate=createReadinessGate(),secret=randomBytes(32).toString('base64url')
// Do not inject managementApi, restartDesktopApp, adapters, exec or process IO.
// The selected package's default restart service and Windows adapter own restart.
// A local source candidate is explicitly identified in the plan and evidence.
const server=startServer(plan.servicePort,{localAttestationSecret:secret,readinessGate})
writeFileSync(join(home,'runtime-port.json'),JSON.stringify({pid:process.pid,port:server.port,attestationSecret:secret}))
const {syncModelsToCodex}=await load('src/codex/sync.ts')
const startup=await runStartupReadinessSync(readinessGate,()=>syncModelsToCodex(server.port,undefined,null))
writeFileSync(join(artifacts,'service-ready.json'),JSON.stringify({owner:actual,pid:process.pid,port:server.port,package:'2.84.0',candidateKind:plan.candidateKind??null,home,readiness:readinessGate.getStatus(),startupStatus:startup?.status,restartImplementation:plan.candidateKind?'local source candidate default adapter; no injected adapter or IO overrides':'published defaults; no adapter or IO overrides',restartSourceSha256:adapterSha256},null,2))
const shutdown=()=>{fixture.stop(true);server.stop(true);process.exit(0)}
process.on('SIGTERM',shutdown)
