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
function defaultConfirmationFixture(version,mutate=()=>{}){
 class Dialog {
  constructor(classes,title){this.classes=new Set(classes);this.title=title;this.open=true}
  querySelector(selector){assert.equal(selector,':scope > .ui-dialog-header h2');return this.title===null?null:{textContent:this.title}}
 }
 const parent=new Dialog(['model-version-manager'],'版本管理 · Native fixture'),confirmation=new Dialog(['app-modal-dialog'],'设为默认版本')
 const scene={dialogs:[parent],mask:false,commits:0,clicks:[],observed:[]}
 const document={
  querySelectorAll:selector=>{
   const match=/^dialog(?:[.]([a-z-]+))?\[open\]$/.exec(selector);assert(match,'Unsupported fixture selector: '+selector)
   return scene.dialogs.filter(d=>d.open&&(!match[1]||d.classes.has(match[1])))
  },
  querySelector:selector=>{assert.equal(selector,'.modal-mask');return scene.mask?{}:null},
 }
 const driver={
  click:async(text,selector)=>{
   scene.clicks.push({text,selector});assert.equal(text,'设为默认')
   if(scene.clicks.length===1){
    assert.equal(selector,'dialog[open]')
    if(version==='0.2.21')scene.dialogs.push(confirmation)
    else {parent.open=false;scene.mask=true}
    mutate(scene,{parent,confirmation,Dialog})
   }else{
    assert.equal(selector,version==='0.2.21'?'dialog.app-modal-dialog[open]':'.modal-mask')
    scene.commits++;confirmation.open=false;scene.mask=false;parent.open=true
   }
  },
  wait:async expression=>{scene.observed.push(scene.dialogs.filter(d=>d.open));assert.equal(runInNewContext(expression,{document,HTMLDialogElement:Dialog}),true)},
 }
 return {driver,scene,parent,confirmation,document,Dialog}
}
for(const version of ['0.2.20','0.2.21'])test(version+' executes the actual default-confirmation driver through its real layer transition',async()=>{
 const {driver,scene,parent,confirmation}=defaultConfirmationFixture(version)
 await confirmNativeTemplateDefault(driver,version)
 assert.equal(scene.commits,1);assert.equal(scene.clicks.length,2)
 assert.deepEqual(scene.observed[0],version==='0.2.21'?[parent,confirmation]:[],'021 keeps the exact parent while 020 suspends it')
 assert.deepEqual(scene.dialogs.filter(d=>d.open),[parent],'confirmation must return to the same single version-manager parent')
})
test('021 refuses a third layer, incorrect parent/child, duplicate AppModal or any legacy mask before committing',async()=>{
 const malformed=[
  ['third native layer',(s,{Dialog})=>s.dialogs.push(new Dialog(['other-dialog'],'额外层'))],
  ['wrong child title',(s,{confirmation})=>{confirmation.title='版本管理'}],
  ['title only contains expected substring',(s,{confirmation})=>{confirmation.title='其它 · 设为默认版本'}],
  ['wrong parent title',(s,{parent})=>{parent.title='其它查看窗口'}],
  ['missing parent header',(s,{parent})=>{parent.title=null}],
  ['wrong parent class',(s,{parent})=>{parent.classes.clear()}],
  ['parent was closed',(s,{parent})=>{parent.open=false}],
  ['two AppModals',(s,{parent})=>{parent.classes=new Set(['app-modal-dialog']);parent.title='设为默认版本'}],
  ['legacy mask under valid native layers',s=>{s.mask=true}],
 ]
 for(const [name,mutate] of malformed){
  const {driver,scene}=defaultConfirmationFixture('0.2.21',mutate)
  await assert.rejects(confirmNativeTemplateDefault(driver,'0.2.21'),undefined,name)
  assert.equal(scene.commits,0,name);assert.equal(scene.clicks.length,1,'wrong host must never receive the commit click: '+name)
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
