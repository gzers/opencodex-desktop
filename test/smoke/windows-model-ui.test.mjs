import {test} from 'node:test'
import assert from 'node:assert/strict'
import {assertModelSandbox,assertModelExportPath,parseTaskContent,verifyWindowsModelUi} from './windows-model-ui.mjs'
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
