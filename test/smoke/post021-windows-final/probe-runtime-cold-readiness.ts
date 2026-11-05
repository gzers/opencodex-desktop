import assert from 'node:assert/strict'
import {mkdirSync,writeFileSync} from 'node:fs'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
const [packagePath,client,outputPath]=process.argv.slice(2)
assert(packagePath&&client&&outputPath)
const root=resolve(outputPath);mkdirSync(root,{recursive:false})
for(const path of ['home','opencodex-home','codex-home','temp'])mkdirSync(join(root,path))
Object.assign(process.env,{OPENCODEX_HOME:join(root,'opencodex-home'),CODEX_HOME:join(root,'codex-home'),HOME:join(root,'home'),USERPROFILE:join(root,'home'),CODEX_CLI_PATH:resolve(client),DO_NOT_TRACK:'1',TEMP:join(root,'temp'),TMP:join(root,'temp')})
const {getCodexRuntimeSnapshot}=await import(pathToFileURL(join(resolve(packagePath),'src/codex/runtime.ts')).href)
const rows:any[]=[],started=performance.now(),deadline=Date.now()+15000
while(Date.now()<deadline){
 const value=getCodexRuntimeSnapshot();rows.push({elapsedMs:performance.now()-started,path:value.runtime.command,version:value.runtime.version,source:value.runtime.source})
 if(value.runtime.version)break
 await new Promise(r=>setTimeout(r,100))
}
const report={kind:'actual-runtime-cold-snapshot-readonly',result:rows.at(-1)?.version?'warm-after-cold':'not-ready',mutations:'Only isolated runtime discovery/cache; no daily config, service or client control',rows}
writeFileSync(join(root,'review.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report))
assert.equal(rows[0]?.version,null,'Expected the real cold snapshot contract')
assert(rows.at(-1)?.version,'Validated runtime must eventually settle')
