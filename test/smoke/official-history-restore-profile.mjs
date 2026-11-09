// Writes ONLY fresh synthetic fixture homes; no server/action/service mutation.
import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,stat} from 'node:fs/promises'
import {resolve,join,relative,isAbsolute} from 'node:path'
import {pathToFileURL} from 'node:url'
import {createHash} from 'node:crypto'
assert(process.platform==='win32'&&process.env.GITHUB_ACTIONS==='true','Ephemeral Windows Actions required')
const [packageDirectory,outputFile,scenario]=process.argv.slice(2)
assert(['worker','restore'].includes(scenario))
const boundary=resolve(process.env.RUNNER_TEMP??''),root=resolve(packageDirectory),output=resolve(outputFile)
const below=path=>{const r=relative(boundary,path);return r&&!r.startsWith('..')&&!isAbsolute(r)}
const codeHome=resolve(process.env.CODEX_HOME??''),runtimeHome=resolve(process.env.OPENCODEX_HOME??'')
for(const p of [root,output,codeHome,runtimeHome])assert(below(p),'Only owned ephemeral paths')
const pin=JSON.parse(await readFile(new URL('../../.github/maintained-runtime.json',import.meta.url),'utf8'))
assert.equal(JSON.parse(await readFile(join(root,'package.json'),'utf8')).version,pin.version)
for(const home of [codeHome,runtimeHome]){try{await stat(home);assert.fail('Fresh nonexistent synthetic home required')}catch(e){if(e.code!=='ENOENT')throw e}await mkdir(home,{recursive:true})}
await writeFile(join(codeHome,'config.toml'),'model_provider = "openai"\n',{flag:'wx'})
await writeFile(join(runtimeHome,'config.json'),JSON.stringify({hostname:'127.0.0.1',openaiProviderTierVersion:2,defaultProvider:'profile-fixture',defaultModelAliases:false,providers:{'profile-fixture':{adapter:'openai-chat',baseUrl:'http://127.0.0.1:1/v1',apiKey:'synthetic-unused',models:['profile-fixture-model'],selectedModels:['profile-fixture-model'],liveModels:false,defaultAliases:false}}})+'\n',{flag:'wx'})
const sources={}
for(const p of ['src/codex/history-job.ts','src/codex/history-worker.ts','src/codex/inject/restore.ts']){const b=await readFile(join(root,p));sources[p]={bytes:b.length,sha256:createHash('sha256').update(b).digest('hex')}}
const report={accepted:false,diagnosticOnly:true,scenario,source:process.env.GITHUB_SHA,runtimeVersion:pin.version,runtimeArchiveSha256:pin.sha256,sources,productStarted:false,businessNetworkUsed:false,serviceOrTaskMutation:false,syntheticFixtureWrites:true,noRealClientData:true,measurements:[],limitations:'Runner token and fresh empty synthetic homes only. No active proxy, injected routing, history rows or official POST; not stop acceptance.'}
const began=performance.now()
try{
 if(scenario==='worker'){
  const start=performance.now(),module=await import(pathToFileURL(join(root,'src/codex/history-job.ts')).href)
  report.measurements.push({phase:'import-history-job',milliseconds:performance.now()-start})
  const target=module.resolveCodexHistoryJobTarget()
  for(const key of ['canonicalCodexHome','canonicalStateDbPath','canonicalBackupPath'])assert(below(resolve(target[key])),'Worker target outside own fixture')
  const work=performance.now(),value=await module.runCodexHistoryJob({...target,operation:'restore-openai'})
  report.measurements.push({phase:'actual-history-restore-worker-and-join',milliseconds:performance.now()-work,result:{kind:value.kind,reason:typeof value.reason==='string'?value.reason:null,rows:Number.isInteger(value.rows)?value.rows:null,files:Number.isInteger(value.files)?value.files:null}})
 }else{
  const start=performance.now(),module=await import(pathToFileURL(join(root,'src/codex/inject/restore.ts')).href)
  report.measurements.push({phase:'import-config-restore',milliseconds:performance.now()-start})
  const work=performance.now(),value=await module.restoreNativeCodexAsync()
  report.measurements.push({phase:'actual-empty-client-native-restore',milliseconds:performance.now()-work,result:{success:value.success===true,configState:value.config?.state??null,historyState:value.history?.state??null,catalogState:value.catalog?.state??null}})
 }
 report.status='complete-observation'
}catch(error){report.status='incomplete-observation';report.errorType=error?.name??'unknown';process.exitCode=1}
finally{report.totalMilliseconds=performance.now()-began;await writeFile(output,JSON.stringify(report,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify(report))}
