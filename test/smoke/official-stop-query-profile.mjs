// Exact pinned runtime, read-only functions only. Never candidate acceptance.
import assert from 'node:assert/strict'
import {readFile,writeFile} from 'node:fs/promises'
import {resolve,join,relative,isAbsolute} from 'node:path'
import {pathToFileURL} from 'node:url'
import {createHash} from 'node:crypto'
assert(process.platform==='win32'&&process.env.GITHUB_ACTIONS==='true','Ephemeral Windows Actions only')
const [packageDirectory,outputFile]=process.argv.slice(2)
const packageRoot=resolve(packageDirectory),output=resolve(outputFile),taskRoot=resolve(process.env.RUNNER_TEMP??'')
const under=(root,path)=>{const rel=relative(root,path);return rel&&!rel.startsWith('..')&&!isAbsolute(rel)}
assert(under(taskRoot,packageRoot)&&under(taskRoot,output),'Owned runner TEMP paths only')
const declaration=JSON.parse(await readFile(join(packageRoot,'package.json'),'utf8'))
const expected=JSON.parse(await readFile(new URL('../../.github/maintained-runtime.json',import.meta.url),'utf8'))
assert.equal(declaration.version,expected.version)
const sources={}
for(const name of ['src/service.ts','src/service/windows-scheduler.ts','src/service/orchestration.ts','src/lib/winsw.ts']){
 const bytes=await readFile(join(packageRoot,name));sources[name]={bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')}
}
const report={schemaVersion:1,accepted:false,diagnosticOnly:true,source:process.env.GITHUB_SHA,runtimeVersion:expected.version,runtimeArchiveSha256:expected.sha256,sources,queries:[],productStarted:false,serviceOrTaskMutation:false,businessNetworkUsed:false,originalDesktopStopBudgetMilliseconds:10000,limitations:'Runner token; direct read-only queries, not ordinary-SID Desktop stop acceptance. No cached or injected absence.'}
async function measure(name,work,project){const began=performance.now();try{const result=await work();report.queries.push({name,elapsedMilliseconds:performance.now()-began,result:project(result),accepted:false});return result}catch(error){report.queries.push({name,elapsedMilliseconds:performance.now()-began,errorType:error?.name??'unknown',accepted:false});return null}}
try{
 const service=await measure('import-service',()=>import(pathToFileURL(join(packageRoot,'src/service.ts')).href),()=>({loaded:true}))
 const scheduler=await measure('import-scheduler',()=>import(pathToFileURL(join(packageRoot,'src/service/windows-scheduler.ts')).href),()=>({loaded:true}))
 const winsw=await measure('import-native-service',()=>import(pathToFileURL(join(packageRoot,'src/lib/winsw.ts')).href),()=>({loaded:true}))
 if(service)await measure('respawn-risk-default-real-query',()=>service.installedServiceRespawnRisk(),value=>({risk:['none','respawnable','unknown','self-unload'].includes(value)?value:'invalid'}))
 if(scheduler){for(const attempt of [1,2])await measure('scheduler-real-query-'+attempt,()=>scheduler.probeWindowsSchedulerTask(),value=>({status:['absent','present','unknown'].includes(value?.status)?value.status:'invalid',queryFailureDetailPresent:typeof value?.detail==='string',timeoutMentioned:typeof value?.detail==='string'&&/ETIMEDOUT|timed? ?out|timeout/i.test(value.detail)}))}
 if(winsw)await measure('native-service-real-query',()=>winsw.statusWinswRaw(),value=>({status:['started','stopped','nonexistent','unknown'].includes(value)?value:'invalid'}))
}finally{
 report.status=report.queries.length===7&&report.queries.every(row=>!row.errorType)?'complete-observation':'incomplete-observation';
 await writeFile(output,JSON.stringify(report,null,2)+'\n',{flag:'wx'})
 console.log(JSON.stringify({diagnosticOnly:true,accepted:false,queryTimings:report.queries,productStarted:false}))
}

if(report.status!=='complete-observation')process.exitCode=1
