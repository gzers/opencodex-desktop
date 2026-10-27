import {test} from 'node:test'
import assert from 'node:assert/strict'
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
import {modelFileEvidence,assertModelFileWrite,assertModelSandbox,assertModelExportPath,modelConfigPath,parseTaskContent,verifyWindowsModelUi} from './windows-model-ui.mjs'
import {overviewScene} from './windows-ui-alignment.mjs'
test('overview accepts either complete cards or the real settled setup gate, never an empty or partial page',()=>{
 assert.equal(overviewScene(Array(6).fill({}),{}),'ready')
 assert.equal(overviewScene([],{present:true,settled:true,checks:3}),'environment-gated')
 for(const gate of [{},{present:true,settled:false,checks:3},{present:true,settled:true,checks:0}])assert.throws(()=>overviewScene([],gate))
 assert.throws(()=>overviewScene([{}],{present:true,settled:true,checks:3}))
})
const root='C:\\runner\\owned-sandbox',facts={about:{platform:'Windows'},dataRoot:{currentDataRoot:root,activeDataRoot:root,runtimeActive:true}}
test('writes require the exact admitted immutable and saved sandbox, never a relative root or another account directory',()=>{
 assert.doesNotThrow(()=>assertModelSandbox(facts,root.toUpperCase()))
 assert.doesNotThrow(()=>assertModelSandbox({...facts,dataRoot:{...facts.dataRoot,currentDataRoot:'\\\\?\\'+root,activeDataRoot:'\\\\?\\'+root}},root))
 for(const bad of [{...facts,about:{platform:'macOS'}},{...facts,dataRoot:{...facts.dataRoot,currentDataRoot:'C:\\Users\\daily'}},{...facts,dataRoot:{...facts.dataRoot,activeDataRoot:'C:\\other'}},{...facts,dataRoot:{...facts.dataRoot,runtimeActive:false}}])assert.throws(()=>assertModelSandbox(bad,root))
 const slash=String.fromCharCode(92),unc=slash+slash+'server'+slash+'share'+slash+'owned',extended=slash+slash+'?'+slash+'UNC'+slash+'server'+slash+'share'+slash+'owned'
 assert.doesNotThrow(()=>assertModelSandbox({...facts,dataRoot:{...facts.dataRoot,currentDataRoot:extended,activeDataRoot:extended}},unc))
 assert.throws(()=>assertModelSandbox(facts,'relative'))
})
test('invalid native roots are rejected before invoking any application command',async()=>{
 let invoked=false;await assert.rejects(verifyWindowsModelUi(async()=>{invoked=true},'.',facts,'C:\\other'));assert.equal(invoked,false)
})
test('native file writes require both saved and immutable model homes inside the admitted sandbox',()=>{
 const home=root+'\\opencodex-home'
 const bound={...facts,dataRoot:{...facts.dataRoot,opencodexHomeMode:'inside',currentOpencodexHome:home,opencodexHome:home}}
 assert.equal(modelConfigPath(bound,root),home+'\\config.json')
 for(const change of [{opencodexHomeMode:'external'},{currentOpencodexHome:'C:\\Users\\daily'},{opencodexHome:home+'-other'},{currentOpencodexHome:'relative'}]){
  assert.throws(()=>modelConfigPath({...bound,dataRoot:{...bound.dataRoot,...change}},root))
 }
})

test('export containment handles Windows canonical prefixes while rejecting siblings, root files and traversal',()=>{
 const slash=String.fromCharCode(92),base=['C:','runner','owned'].join(slash),exports=base+slash+'exports'+slash+'file.ocxdconf'
 assert.doesNotThrow(()=>assertModelExportPath(exports,base));assert.doesNotThrow(()=>assertModelExportPath(slash+slash+'?'+slash+exports,base.toUpperCase()))
 const unc=slash+slash+'server'+slash+'share'+slash+'owned',extended=slash+slash+'?'+slash+'UNC'+slash+'server'+slash+'share'+slash+'owned'+slash+'exports'+slash+'file.ocxdconf'
 assert.doesNotThrow(()=>assertModelExportPath(extended,unc))
 for(const outside of [base+slash+'file.ocxdconf',base+slash+'exports-other'+slash+'file',base+slash+'exports'+slash+'..'+slash+'file',base+'-other'+slash+'exports'+slash+'file','relative'])assert.throws(()=>assertModelExportPath(outside,base))
})

test('task preview JSON begins after the instruction section even when receipt instructions contain braces',()=>{
 const pack={contract:{required:['submissionId']},task:{id:'fixture'},transport:{headers:{Authorization:'fixture-only'}}}
 const text='断连先GET /receipts/{submissionId}再决定重试。\n\n'+JSON.stringify(pack,null,2)
 assert.deepEqual(parseTaskContent(text),pack);assert.throws(()=>parseTaskContent('only {instructions}'))
})

test('native Save evidence requires changed file bytes and one exact backup without losing prior backups',async t=>{
 const directory=await mkdtemp(join(tmpdir(),'ocx-native-file-evidence-'))
 t.after(()=>rm(directory,{recursive:true,force:true}))
 const config=join(directory,'config.json'),backupRoot=join(directory,'backups','model-config')
 const original=Buffer.from('{"providers":{"fixture":{"apiKey":"synthetic-key"}}}\n')
 await writeFile(config,original)
 const before=await modelFileEvidence(config,directory)
 assert.deepEqual(before.backups,[])
 await mkdir(join(backupRoot,'first'),{recursive:true});await writeFile(join(backupRoot,'first','config.json'),original)
 await writeFile(config,'{"providers":{"fixture":{"apiKey":"synthetic-replacement"}}}\n')
 const saved=await modelFileEvidence(config,directory)
 await assertModelFileWrite(before,saved,directory)
 await assert.rejects(assertModelFileWrite(saved,saved,directory),/change actual config/)
 await assert.rejects(assertModelFileWrite(before,{...saved,backups:[]},directory),/exactly one/)
 await assert.rejects(assertModelFileWrite(before,{...saved,backups:['first','extra']},directory),/exactly one/)
 await writeFile(join(backupRoot,'first','config.json'),'wrong preimage')
 await assert.rejects(assertModelFileWrite(before,saved,directory),/exact prewrite/)
 await mkdir(join(backupRoot,'second'));await writeFile(join(backupRoot,'second','config.json'),saved.bytes)
 await writeFile(config,'{"providers":{"fixture":{"apiKey":"synthetic-third"}}}\n')
 const final=await modelFileEvidence(config,directory)
 await assertModelFileWrite(saved,final,directory)
 await assert.rejects(assertModelFileWrite(saved,{...final,backups:['second']},directory),/preserve earlier/)
 // Cancel evidence compares actual bytes and backup entries, without an IPC counter.
 const canceled=await modelFileEvidence(config,directory)
 assert(canceled.bytes.equals(final.bytes));assert.deepEqual(canceled.backups,final.backups)
 assert((await readFile(config)).equals(final.bytes))
})
