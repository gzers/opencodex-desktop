import {test} from 'node:test'
import assert from 'node:assert/strict'
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
import {createContext,runInContext,runInNewContext} from 'node:vm'
import {modelFileEvidence,assertModelFileWrite,assertModelSandbox,assertModelExportPath,modelConfigPath,nativeModelKeyboardChecksRequired,nativeModelUiContract,confirmNativeTemplateDefault,clickNativeModelMenu,parseTaskContent,verifyWindowsModelUi} from './windows-model-ui.mjs'
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

function nativeMenuFixture({tops=[535.25,499.25,356.25,356.25,356.25],afterMove=()=>{},opens=true}={}){
 const scene={frame:0,top:tops[0],occluded:false,open:false,scrollCalls:0},commands=[],listeners=new Map(),frames=new Map()
 let nextFrame=0
 const icon={tagName:'svg',getAttribute:()=>null},overlay={tagName:'DIV',getAttribute:()=>null},scroller={scrollTop:0,scrollLeft:0,parentElement:null}
 const button={tagName:'BUTTON',disabled:false,parentElement:scroller,getAttribute:()=> 'Fast支持',contains:e=>e===button||e===icon,scrollIntoView:()=>{scene.scrollCalls++},getBoundingClientRect:()=>({x:515.5,y:scene.top,left:515.5,right:903,top:scene.top,bottom:scene.top+34,width:387.5,height:34})}
 const document={hidden:false,activeElement:button,hasFocus:()=>true,querySelector:selector=>selector.includes('[data-ui-escape-owner]')?(scene.open?{}:null):button,
  elementFromPoint:(x,y)=>scene.occluded?overlay:x>=515.5&&x<=903&&y>=scene.top&&y<=scene.top+34?icon:null,
  addEventListener:(type,fn)=>listeners.set(type,fn),removeEventListener:type=>listeners.delete(type)}
 const context=createContext({document,window:{screenX:0,screenY:0,visualViewport:{width:1028,height:749,offsetLeft:0,offsetTop:0,pageLeft:0,pageTop:0,scale:1}},innerWidth:1028,innerHeight:749,outerWidth:1044,outerHeight:788,devicePixelRatio:1,performance:{now:()=>scene.frame*16},requestAnimationFrame:fn=>{frames.set(++nextFrame,fn);return nextFrame},cancelAnimationFrame:id=>frames.delete(id)})
 const evaluate=async expression=>runInContext(expression,context)
 function advance(){scene.top=tops[Math.min(scene.frame++,tops.length-1)];const current=[...frames.values()];frames.clear();for(const fn of current)fn(scene.frame*16)}
 const driver={evaluate,wait:async(expression,label)=>{for(let n=0;n<12;n++){advance();const value=await evaluate(expression);if(value)return value}throw Error('Native model UI timeout: '+label)},
  call:async(method,args)=>{
   assert.equal(method,'Input.dispatchMouseEvent');commands.push(args)
   if(args.type==='mouseMoved'){afterMove(scene);return}
   const types=args.type==='mousePressed'?['pointerdown','mousedown']:['pointerup','mouseup','click']
   for(const type of types)listeners.get(type)?.({type,isTrusted:true,target:document.elementFromPoint(args.x,args.y),clientX:args.x,clientY:args.y})
   if(args.type==='mouseReleased'&&opens&&!scene.occluded&&button.contains(document.elementFromPoint(args.x,args.y)))scene.open=true
  }}
 return {driver,scene,commands,context,frames,listeners}
}
test('actual native menu driver waits through moving frames, rechecks a composite hit, then sends exactly one pointer click',async t=>{
 const output=await mkdtemp(join(tmpdir(),'ocx-menu-frames-'));t.after(()=>rm(output,{recursive:true,force:true}))
 const f=nativeMenuFixture();await clickNativeModelMenu(f.driver,output)
 assert.equal(f.scene.scrollCalls,1);assert.deepEqual(f.commands.map(c=>c.type),['mouseMoved','mousePressed','mouseReleased'])
 assert(f.commands.every(c=>c.x===709.25&&c.y===373.25),'must not click the earlier y552.25 or y516.25 samples')
 const trace=JSON.parse(await readFile(join(output,'model-menu-input-native.json'),'utf8'))
 assert.equal(trace.result,'pass');assert.deepEqual(trace.point.stableFrames,[3,4,5]);assert.deepEqual(trace.changes.map(f=>f.y),[552.25,516.25,373.25])
 assert.equal(trace.point.viewport.dpr,1);assert.equal(trace.point.viewport.visual.scale,1)
 assert(trace.stages[0].sameGeometry&&trace.stages[0].sameHit);assert.equal(trace.stages[0].current.hitTarget.tag,'svg')
 assert.deepEqual(trace.events.map(e=>e.type),['pointerdown','mousedown','pointerup','mouseup','click']);assert(trace.events.every(e=>e.trusted&&e.targetWithinButton))
 assert.equal(f.context.__ocxModelMenuInput,undefined);assert.equal(f.frames.size,0);assert.equal(f.listeners.size,0)
})
test('native menu driver refuses movement or occlusion after stable frames before pressing, retaining the failed guard',async t=>{
 const output=await mkdtemp(join(tmpdir(),'ocx-menu-guard-'));t.after(()=>rm(output,{recursive:true,force:true}))
 for(const afterMove of [s=>{s.top-=36},s=>{s.occluded=true}]){
  const f=nativeMenuFixture({afterMove})
  await assert.rejects(clickNativeModelMenu(f.driver,output),/moved or became occluded/)
  assert.deepEqual(f.commands.map(c=>c.type),['mouseMoved'])
  const trace=JSON.parse(await readFile(join(output,'model-menu-input-native.json'),'utf8'))
  assert.equal(trace.result,'fail');assert(!trace.stages[0].sameGeometry||!trace.stages[0].sameHit);assert.deepEqual(trace.events,[])
  assert.equal(f.frames.size,0);assert.equal(f.listeners.size,0)
 }
})
test('continually moving menu geometry never receives pointer input',async t=>{
 const output=await mkdtemp(join(tmpdir(),'ocx-menu-moving-'));t.after(()=>rm(output,{recursive:true,force:true}))
 const f=nativeMenuFixture({tops:Array.from({length:12},(_,n)=>535.25-n)})
 await assert.rejects(clickNativeModelMenu(f.driver,output),/stable across frames/)
 assert.deepEqual(f.commands,[]);assert.equal(f.context.__ocxModelMenuInput,undefined);assert.equal(f.frames.size,0)
 const trace=JSON.parse(await readFile(join(output,'model-menu-input-native.json'),'utf8'));assert.equal(trace.result,'fail');assert.equal(trace.frames.length,12)
})
test('a real pointer click that does not open the inner menu retains the original failure and never retries',async t=>{
 const output=await mkdtemp(join(tmpdir(),'ocx-menu-unopened-'));t.after(()=>rm(output,{recursive:true,force:true}))
 const f=nativeMenuFixture({opens:false})
 await assert.rejects(clickNativeModelMenu(f.driver,output),{message:'Native model UI timeout: native inner select open'})
 assert.deepEqual(f.commands.map(c=>c.type),['mouseMoved','mousePressed','mouseReleased'])
 const trace=JSON.parse(await readFile(join(output,'model-menu-input-native.json'),'utf8'))
 assert.equal(trace.error,'Native model UI timeout: native inner select open');assert(trace.events.some(e=>e.type==='pointerdown'&&e.targetWithinButton))
 assert.equal(f.context.__ocxModelMenuInput,undefined);assert.equal(f.frames.size,0);assert.equal(f.listeners.size,0)
})
