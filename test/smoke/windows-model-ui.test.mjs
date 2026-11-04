import {test} from 'node:test'
import assert from 'node:assert/strict'
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
import {runInNewContext} from 'node:vm'
import {modelFileEvidence,assertModelFileWrite,assertModelSandbox,assertModelExportPath,modelConfigPath,nativeModelKeyboardChecksRequired,nativeModelUiContract,confirmNativeTemplateDefault,parseTaskContent,verifyWindowsModelUi} from './windows-model-ui.mjs'
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
test('020 and 021 require the native keyboard/menu checks while older admitted images retain baseline behavior',()=>{
 for(const version of ['0.2.20','0.2.21'])assert.equal(nativeModelKeyboardChecksRequired(version),true,version)
 for(const version of ['0.2.16','0.2.17','0.2.18','0.2.19'])assert.equal(nativeModelKeyboardChecksRequired(version),false,version)
 for(const version of ['0.2.22','0.2.210','0.3.0','1.0.0','v0.2.21','0.2.021','0.2.21-rc.1','0.2.21 ','',null,undefined,21,{}]){
  assert.throws(()=>nativeModelKeyboardChecksRequired(version),/Unsupported native model UI acceptance version/)
 }
})
test('native acceptance rejects unadmitted versions before IPC, while 020 and 021 enter the CDP driver',async()=>{
 const home=root+'\\opencodex-home'
 const bound={...facts,dataRoot:{...facts.dataRoot,opencodexHomeMode:'inside',currentOpencodexHome:home,opencodexHome:home}}
 for(const version of ['0.2.22','0.3.0','0.2.21-rc.1']){
  let calls=0
  await assert.rejects(verifyWindowsModelUi(async()=>{calls++},'.',{...bound,about:{...bound.about,version}},root),/Unsupported native model UI acceptance version/)
  assert.equal(calls,0,version)
 }
 for(const version of ['0.2.20','0.2.21']){
  const reached=new Error('admitted native CDP driver reached');let calls=0
  await assert.rejects(verifyWindowsModelUi(async method=>{calls++;assert.equal(method,'Runtime.evaluate');throw reached},'.',{...bound,about:{...bound.about,version}},root),error=>error===reached)
  assert.equal(calls,1,version)
 }
})
test('021 uses public native AppModal/header X; the 020 upgrade image keeps its own mask/dialog contract',()=>{
 assert.deepEqual(nativeModelUiContract('0.2.21'),{publicDialogs:true,confirmation:'dialog.app-modal-dialog[open]',close:'dialog[open] > .ui-dialog-header'})
 assert.deepEqual(nativeModelUiContract('0.2.20'),{publicDialogs:false,confirmation:'.modal-mask',close:'dialog[open]'})
 for(const version of ['0.2.19','0.2.18','0.2.17','0.2.16'])assert.equal(nativeModelUiContract(version).publicDialogs,false)
 assert.throws(()=>nativeModelUiContract('0.2.22'))
})
for(const version of ['0.2.20','0.2.21'])test(version+' default confirmation runs the actual driver transition without confusing the version manager',async()=>{
 let state='manager',committed=false;const clicks=[]
 const document={
  querySelectorAll:selector=>{assert.equal(selector,'dialog[open]');return state==='manager'||state==='native-confirmation'?[{}]:[]},
  querySelector:selector=>selector==='.modal-mask'&&state==='old-confirmation'?{}:selector==='dialog.app-modal-dialog[open] .ui-dialog-header h2'&&state==='native-confirmation'?{textContent:'设为默认版本'}:null,
 }
 await confirmNativeTemplateDefault({
  click:async(text,selector)=>{clicks.push({text,selector});assert.equal(text,'设为默认');if(state==='manager'){assert.equal(selector,'dialog[open]');state=version==='0.2.21'?'native-confirmation':'old-confirmation'}else{assert.equal(selector,nativeModelUiContract(version).confirmation);state='manager';committed=true}},
  wait:async expression=>assert.equal(runInNewContext(expression,{document}),true),
 },version)
 assert(committed);assert.equal(state,'manager');assert.equal(clicks.length,2)
})
test('021 refuses a leftover legacy mask or a different native dialog before confirming any default',async()=>{
 for(const scene of ['legacy-mask','wrong-title','two-native-dialogs']){
  let clicks=0
  const document={querySelectorAll:()=>Array(scene==='two-native-dialogs'?2:scene==='legacy-mask'?0:1).fill({}),querySelector:selector=>selector==='.modal-mask'?{}:scene==='wrong-title'?{textContent:'版本管理'}:null}
  await assert.rejects(confirmNativeTemplateDefault({click:async()=>{clicks++},wait:async expression=>assert.equal(runInNewContext(expression,{document}),true)},'0.2.21'))
  assert.equal(clicks,1,'wrong host must never receive the commit click')
 }
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
