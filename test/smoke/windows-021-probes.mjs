// Acceptance tooling for the installed EXE; no DEV renderer or mock bridge.
import assert from 'node:assert/strict'
import {randomUUID,createHash} from 'node:crypto'
import {readFile,writeFile,readdir,lstat,realpath} from 'node:fs/promises'
import {join,win32} from 'node:path'
import {assertModelSandbox,modelConfigPath,modelFileEvidence} from './windows-model-ui.mjs'
import {observe021Ipc,install021NoticeObservation,sample021Notice,assert021NoticeTiming,install021CandidateObservation} from './windows-021-observer.mjs'

const js=(fn,...args)=>'('+fn.toString()+')('+args.map(value=>JSON.stringify(value)).join(',')+')'
const hash=bytes=>createHash('sha256').update(bytes).digest('hex')
const notice='.notif-layer [data-notification-key^="model-save:"]'
export function assert021Admission(facts,expected){
 assertModelSandbox(facts,expected)
 assert.equal(facts.about.version,'0.2.21','021 probes require the exact installed 0.2.21 version')
 return modelConfigPath(facts,expected)
}
async function freshAdmission(invoke,expected){
 const facts={about:await invoke('app_about'),dataRoot:await invoke('get_data_root_config')}
 return {facts,config:assert021Admission(facts,expected)}
}
async function assertFixtureFile(config,root){
 for(const path of [root,win32.dirname(config)]){
  const stat=await lstat(path);assert(stat.isDirectory()&&!stat.isSymbolicLink(),'021 fixture parent must be an owned directory')
 }
 const stat=await lstat(config);assert(stat.isFile()&&!stat.isSymbolicLink(),'021 fixture config must be a regular sandbox file')
 const relative=win32.relative(await realpath(root),await realpath(config))
 assert(relative&&!relative.startsWith('..')&&!win32.isAbsolute(relative),'021 fixture resolved outside the admitted root')
}
export function assert021RenameReadback({before,after,beforeSnapshot,afterSnapshot,channelId,from,to,updatesDisplayName=false}){
 assert(Object.hasOwn(before.providers,from)&&!Object.hasOwn(before.providers,to),'021 rename preimage is ambiguous')
 const expected=structuredClone(before)
 expected.providers[to]=expected.providers[from];delete expected.providers[from]
 if(updatesDisplayName)expected.providers[to].name=to
 if(expected.defaultProvider===from)expected.defaultProvider=to
 for(const field of ['modelPickerOrder','disabledModels'])if(expected[field])expected[field]=expected[field].map(slug=>slug.startsWith(from+'/')?to+slug.slice(from.length):slug)
 assert.deepEqual(after,expected,'021 rename must migrate exact routed/default references and preserve all other file content/Keys')
 const old=beforeSnapshot.policy.channels.find(c=>c.id===channelId),current=afterSnapshot.policy.channels.find(c=>c.id===channelId)
 assert(old&&current,'021 rename lost the stable channel.id')
 assert.equal(current.id,old.id);assert.equal(current.label,to);assert.equal(current.connection.providerId,to)
 const intended=structuredClone(beforeSnapshot.policy),renamed=intended.channels.find(c=>c.id===channelId)
 renamed.label=to;renamed.connection.providerId=to
 assert.deepEqual(afterSnapshot.policy,intended,'021 rename changed channel order, default identity, nodes, rules or unrelated channels')
 assert.equal(afterSnapshot.policy.defaultChannelId,channelId,'021 rename changed default channel identity')
 assert.notEqual(afterSnapshot.policyDigest,beforeSnapshot.policyDigest,'021 rename did not commit a new file digest')
}
export async function assert021RenameBackups(before,after,root){
 assert(!after.bytes.equals(before.bytes),'021 rename did not change actual config.json')
 assert.deepEqual(after.backups.filter(id=>before.backups.includes(id)),before.backups,'021 rename lost an older backup')
 const added=after.backups.filter(id=>!before.backups.includes(id)),transactions=added.filter(id=>/^rename-[a-f0-9-]{36}$/.test(id)),ordinary=added.filter(id=>!transactions.includes(id))
 assert.equal(ordinary.length,1,'021 rename requires exactly one ordinary prewrite backup')
 assert.equal(transactions.length,1,'021 rename requires exactly one recovery image directory')
 assert((await readFile(join(root,'backups','model-config',ordinary[0],'config.json'))).equals(before.bytes),'021 rename backup lost exact prewrite bytes')
 const directory=join(root,'backups','model-config',transactions[0]),files=await readdir(directory)
 const preimages=await Promise.all(files.filter(id=>/^[0-9]+[.]before$/.test(id)).map(id=>readFile(join(directory,id))))
 const postimages=await Promise.all(files.filter(id=>/^[0-9]+[.]after$/.test(id)).map(id=>readFile(join(directory,id))))
 assert(preimages.some(bytes=>bytes.equals(before.bytes)),'021 recovery directory lacks exact config preimage')
 assert(postimages.some(bytes=>bytes.equals(after.bytes)),'021 recovery directory lacks exact config postimage')
 await assert.rejects(readFile(join(root,'manager-state','provider-rename-pending.json')),error=>error.code==='ENOENT','Successful rename left a pending recovery journal')
 return {ordinary:ordinary[0],transaction:transactions[0],preimageSha256:hash(before.bytes),postimageSha256:hash(after.bytes)}
}
export async function native021Click({call,evaluate,wait},selector){
 await evaluate(js(selector=>{const e=document.querySelector(selector);if(!e)throw Error('021 native click target missing');e.scrollIntoView({block:'center'});return true},selector))
 const point=await wait(js(selector=>{const e=document.querySelector(selector);if(!e||e.disabled)return null;const r=e.getBoundingClientRect(),x=r.left+r.width/2,y=r.top+r.height/2;return r.width&&r.height&&x>0&&y>0&&x<innerWidth&&y<innerHeight&&e.contains(document.elementFromPoint(x,y))?{x,y}:null},selector),'021 hit-testable native target '+selector)
 await call('Input.dispatchMouseEvent',{type:'mouseMoved',...point})
 await call('Input.dispatchMouseEvent',{type:'mousePressed',...point,button:'left',buttons:1,clickCount:1})
 await call('Input.dispatchMouseEvent',{type:'mouseReleased',...point,button:'left',buttons:0,clickCount:1})
}
export async function close021Dialog(driver,selector,exit){
 const {call,evaluate,wait,click}=driver
 if(exit==='x')await native021Click(driver,selector+' > .ui-dialog-header .modal-close')
 else if(exit==='escape')for(const type of ['keyDown','keyUp'])await call('Input.dispatchKeyEvent',{type,key:'Escape',code:'Escape',windowsVirtualKeyCode:27,nativeVirtualKeyCode:27})
 else if(exit==='cancel-button')await click('取消',selector+' > footer')
 else if(exit==='native-cancel')await evaluate(js(selector=>{const d=document.querySelector(selector);if(!(d instanceof HTMLDialogElement)||!d.open)throw Error('Open native dialog required');d.dispatchEvent(new Event('cancel',{cancelable:true}));return true},selector))
 else if(exit==='backdrop'){
  const point=await evaluate(js(selector=>{const d=document.querySelector(selector),r=d.getBoundingClientRect();return [{x:2,y:2},{x:innerWidth-2,y:2},{x:2,y:innerHeight-2}].find(p=>(p.x<r.left||p.x>r.right||p.y<r.top||p.y>r.bottom)&&document.elementFromPoint(p.x,p.y)===d)??null},selector))
  assert(point,'021 native backdrop is not hit-testable')
  await call('Input.dispatchMouseEvent',{type:'mouseMoved',...point})
  await call('Input.dispatchMouseEvent',{type:'mousePressed',...point,button:'left',buttons:1,clickCount:1})
  await call('Input.dispatchMouseEvent',{type:'mouseReleased',...point,button:'left',buttons:0,clickCount:1})
 }else assert.fail('Unknown 021 dialog exit: '+exit)
 await wait('!document.querySelector('+JSON.stringify(selector)+')','021 dialog closes via '+exit)
}
async function screenshot({call,output},name){const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(image.data,'base64'))}
async function channelList({click,wait}){
 await click('模型','.side');await wait("!!document.querySelector('.models-page')",'021 model route')
 await click('渠道模型');await wait("!!document.querySelector('.channel-table')",'021 channels')
}
async function openChannel({evaluate,wait},id,key){
 await evaluate(js(id=>{const row=[...document.querySelectorAll('.channel-table tr[data-channel]')].find(e=>e.dataset.channel===id),button=[...row?.querySelectorAll('button')??[]].find(e=>e.textContent.trim()==='编辑');if(!button)throw Error('021 fixture channel missing');button.click();return true},id))
 await wait(js(key=>{const input=document.querySelector('input[aria-label="API Key"]');return input&&!input.disabled&&input.value===key&&input.type==='password'},key),'021 actual file Key ready and masked')
}
async function input({evaluate},selector,value){
 await evaluate(js((selector,value)=>{const e=document.querySelector(selector);if(!e||e.disabled)throw Error('021 editable input missing');e.value=value;e.dispatchEvent(new Event('input',{bubbles:true}));return true},selector,value))
}

async function verifyNotice(driver){
 const {evaluate,wait,click}=driver,confirm='dialog.ui-dialog[open]'
 const sample=js(sample021Notice)
 const first=await wait('('+sample+".phase==='running')&&"+sample,'021 save notification running')
 assert(first.present&&first.ratio>0,'021 save notification must start its countdown')
 assert(first.help.includes('8秒'),'021 notification must advertise the real eight-second TTL')
 assert(!first.hidden&&!first.hovered,'021 timing needs a visible, unhovered native page')
 assert(first.focusOnAnchor&&!first.focusInNotice&&!first.focus.some(event=>event.inNotice),'021 save notification stole focus when it appeared')
 const consumed=await wait('('+sample+".phase==='running'&&"+sample+'.ratio<=0.75)&&'+sample,'021 real countdown consumes two seconds before confirmation')
 assert(consumed.focusOnAnchor,'021 countdown moved the existing editing focus')
 const cancellations=[]
 for(const exit of ['x','escape','cancel-button','native-cancel','backdrop']){
  await click('立即重启 Codex',notice)
  await wait("document.querySelectorAll('dialog[open]').length===1&&document.querySelector('dialog[open] h2')?.textContent==='重启 Codex'",'021 restart confirmation')
  const paused=await wait('('+sample+".phase==='paused')&&"+sample,'021 confirmation pauses countdown')
  await evaluate('new Promise(resolve=>setTimeout(()=>resolve(true),1100))')
  const held=await evaluate(sample)
  assert(held.present&&held.phase==='paused'&&Math.abs(held.ratio-paused.ratio)<0.01,'021 confirmation consumed the paused countdown')
  await close021Dialog(driver,confirm,exit)
  const resumed=await wait('('+sample+".phase==='running')&&"+sample,'021 cancellation resumes without manual blur '+exit)
  assert(resumed.focusInNotice,'021 confirmation did not return focus to its real trigger')
  assert.equal(resumed.key,first.key,'021 cancellation replaced the original save notification')
  assert(resumed.ratio<=paused.ratio+0.01,'021 cancellation reset the remaining eight-second countdown')
  cancellations.push({exit,pausedAt:paused.now,heldMs:held.now-paused.now,resumedAt:resumed.now,pausedRatio:paused.ratio,resumedRatio:resumed.ratio})
 }
 await evaluate('globalThis.__ocx021Notice.anchor.focus();true')
 await wait('!document.querySelector('+JSON.stringify(notice)+')','021 eight-second notification auto-collapse after cancellations')
 const trace=await evaluate(sample),timing=assert021NoticeTiming(trace)
 assert(!await evaluate("!!document.querySelector('dialog[open]')"),'021 cancellation left a confirmation open')
 return {first,cancellations,...timing,trace,clock:'real WebView performance.now/setTimeout; no virtual time',restartConfirmed:false}
}

async function verifyRenameAndNotice(driver,expected){
 const {call,invoke,evaluate,wait,click}=driver,{config}=await freshAdmission(invoke,expected)
 const suffix=randomUUID().slice(0,8),from='native021-old-'+suffix,to='native021-new-'+suffix,neighbor=from+'0',id=randomUUID(),node=randomUUID(),neighborId=randomUUID(),neighborNode=randomUUID()
 const ids=['vendor/a:8b','vendor-a:8b','Mixed_Case/V2'],rules=Object.fromEntries(['protocol','contextWindow','inputModalities','reasoningEfforts','defaultEffort','fast','pinnedEffort','effortMap','noReasoning','reasoningAuthoritative','fastWire'].map(k=>[k,{mode:'inherit'}]))
 const fixture={id,label:from,templateRefs:[],modelRefs:{},selection:{mode:'whitelist',models:ids},modelOrder:ids,discovered:ids.map(id=>({id,label:id,rules})),defaults:rules,overrides:{},familyOverrides:{},connection:{providerId:from,protocol:'openai-chat',nodes:[{id:node,name:'Never requested',url:'https://native021.invalid/v1'}],activeNodeId:node,credentialRef:null,enabled:false,allowPrivateNetwork:false,authMode:'key'}}
 const other={...structuredClone(fixture),id:neighborId,label:neighbor,connection:{...fixture.connection,providerId:neighbor,nodes:[{id:neighborNode,name:'Neighbor never requested',url:'https://neighbor.native021.invalid/v1'}],activeNodeId:neighborNode}}
 const baseline=await invoke('model_file_snapshot')
 const seeded=await invoke('model_file_save',{request:{expectedFile:baseline.policyDigest,policy:{...baseline.policy,channels:[...baseline.policy.channels,fixture,other]},credentials:[{channelId:id,action:'replace',value:'synthetic-native021-key'},{channelId:neighborId,action:'replace',value:'synthetic-native021-neighbor-key'}]}})
 assert(seeded.changed,'021 fixture must be saved by the production writer')
 await freshAdmission(invoke,expected);await assertFixtureFile(config,expected)
 const raw=JSON.parse(await readFile(config))
 assert.equal(raw.providers[from].apiKey,'synthetic-native021-key')
 assert(!JSON.stringify(raw.providers[from]).includes('keychain:'),'021 rename fixture must never reference system credentials')
 raw.defaultProvider=from
 raw.modelPickerOrder=[...(raw.modelPickerOrder??[]).filter(slug=>!slug.startsWith(from+'/')&&!slug.startsWith(neighbor+'/')),from+'/'+ids[0],neighbor+'/'+ids[0],from+'/'+ids[1],neighbor+'/'+ids[1],from+'/'+ids[2],neighbor+'/'+ids[2]]
 raw.disabledModels=[...(raw.disabledModels??[]).filter(slug=>!slug.startsWith(from+'/')),from+'/'+ids[2]]
 raw.providers[from].native021Opaque={literal:from+'/leave-opaque',keep:42}
 await writeFile(config,JSON.stringify(raw,null,2)+'\n')
 const beforeSnapshot=await invoke('model_file_snapshot'),before=await modelFileEvidence(config,expected)
 assert.equal(beforeSnapshot.policy.defaultChannelId,id,'021 fixture must read its default from the actual file')
 await call('Page.reload',{ignoreCache:true})
 await wait("document.readyState==='complete'&&!!document.querySelector('.side')",'021 fixture reload')
 await channelList(driver)
 const observer=await observe021Ipc(driver)
 let noticeResult,ipc,updatesDisplayName=false
 try{
  await openChannel(driver,id,'synthetic-native021-key')
  // Historical 0.2.21 exposes a separate runtime ID; the integrated name-only
  // contract also persists the channel display name in the provider document.
  updatesDisplayName=await evaluate(js(()=>!document.querySelector('input[aria-label="运行渠道标识"]')))
  await input(driver,'#channel-config input[maxlength="80"]',to)
  const draft=await wait(js(to=>{const preview=document.querySelector('[data-channel-rename-preview]');const field=document.querySelector('input[aria-label="运行渠道标识"]');return (field?field.value===to:document.querySelector('#channel-config input[maxlength="80"]')?.value===to)&&preview?.textContent.includes(to)?{preview:preview.textContent,keyMasked:document.querySelector('input[aria-label="API Key"]').type==='password'}:null},to),'021 exact rename preview from native handler')
  assert(draft.keyMasked)
  await click('返回渠道列表');await wait("!!document.querySelector('.channel-table')",'021 unsaved rename return')
  const unsaved=await modelFileEvidence(config,expected)
  assert(unsaved.bytes.equals(before.bytes));assert.deepEqual(unsaved.backups,before.backups,'021 unsaved rename created a backup')
  await openChannel(driver,id,'synthetic-native021-key')
  assert.equal(await evaluate('document.querySelector('+JSON.stringify('#channel-config input[maxlength="80"]')+').value'),to,'021 rename draft lost on return')
  await freshAdmission(invoke,expected)
  await call('Input.dispatchMouseEvent',{type:'mouseMoved',x:4,y:4})
  assert(await evaluate(js(install021NoticeObservation)))
  await click('保存渠道草稿')
  await wait('!!document.querySelector('+JSON.stringify(notice)+')','021 actual rename Save result')
  // Observe immediately so file checks cannot consume the eight-second TTL.
  noticeResult=await verifyNotice(driver)
 }finally{
  try{ipc=await observer.finish(['model_file_save'])}
  finally{
   try{await writeFile(join(driver.output,'native021-notice-trace.json'),JSON.stringify(await evaluate(js(sample021Notice)),null,2)+'\n')}catch{}
   await evaluate('(()=>{globalThis.__ocx021Notice?.dispose();delete globalThis.__ocx021Notice;return true})()')
  }
 }
 assert.equal(ipc.commands.filter(c=>c==='model_file_save').length,1,'021 notification/cancellation performed another save')
 const after=await modelFileEvidence(config,expected),afterSnapshot=await invoke('model_file_snapshot')
 assert021RenameReadback({before:JSON.parse(before.bytes),after:JSON.parse(after.bytes),beforeSnapshot,afterSnapshot,channelId:id,from,to,updatesDisplayName})
 const backups=await assert021RenameBackups(before,after,expected)
 await freshAdmission(invoke,expected)
 await call('Page.reload',{ignoreCache:true});await wait("document.readyState==='complete'&&!!document.querySelector('.side')",'021 saved rename full reopen')
 await channelList(driver);await openChannel(driver,id,'synthetic-native021-key')
 assert.equal(await evaluate('document.querySelector('+JSON.stringify('#channel-config input[maxlength="80"]')+').value'),to,'021 saved namespace lost after reload')
 assert.equal(await evaluate('document.querySelector('+JSON.stringify('button[aria-label="保存渠道草稿"]')+').disabled'),true,'021 reopened rename is dirty')
 assert.deepEqual(await evaluate("[...document.querySelectorAll('.channel-model-table tr[data-model]')].map(e=>e.dataset.model)"),ids,'021 full model IDs/order lost on reopen')
 const reopened=await invoke('model_file_snapshot');assert.deepEqual(reopened.policy,afterSnapshot.policy)
 await screenshot(driver,'native021-channel-rename')
 await click('返回渠道列表');await wait("!!document.querySelector('.channel-table')",'021 detection list')
 return {channelId:id,provider:to,from,modelIds:ids,backups,notice:noticeResult,ipc,updatesDisplayName}
}

export function assert021DetectionLayout(value){
 assert(value.native&&value.open&&value.title.endsWith(' · 节点检测'),'021 detection must be the actual native dialog')
 assert.equal(value.closeCount,1,'021 detection needs one header X');assert(value.closeEnabled,'021 detection header X is disabled')
 assert.equal(value.footerCount,0,'021 detection observation must have no footer')
 assert(value.width>0&&value.height>0&&value.left>=-1&&value.top>=-1&&value.right<=value.viewportWidth+1&&value.bottom<=value.viewportHeight+1,'021 detection escaped the native viewport')
 assert(Math.abs(value.centerX-value.viewportWidth/2)<=3&&Math.abs(value.centerY-value.viewportHeight/2)<=3,'021 detection dialog must be centered')
 assert(value.statuses.length>0&&value.statuses.every(s=>s==='未检测'),'Opening 021 detection started or fabricated a measurement')
}
async function verifyDetection(driver,expected,channelId){
 const {call,evaluate,wait,invoke}=driver,{config}=await freshAdmission(invoke,expected),before=await modelFileEvidence(config,expected),snapshot=await invoke('model_file_snapshot'),observer=await observe021Ipc(driver),checks=[]
 let ipc
 try{
  for(const exit of ['x','escape','native-cancel','backdrop']){
   const trigger='.channel-table tr[data-channel="'+channelId+'"] .channel-node-detect'
   await native021Click(driver,trigger)
   const layout=await wait(js(()=>{const d=document.querySelector('dialog.channel-node-dialog[open]');if(!d)return null;const r=d.getBoundingClientRect(),x=d.querySelectorAll('.ui-dialog-header .modal-close');return {native:d instanceof HTMLDialogElement,open:d.open,title:d.querySelector('h2')?.textContent,closeCount:x.length,closeEnabled:!!x[0]&&!x[0].disabled,footerCount:d.querySelectorAll('footer,.modal-actions').length,left:r.left,top:r.top,right:r.right,bottom:r.bottom,width:r.width,height:r.height,centerX:r.left+r.width/2,centerY:r.top+r.height/2,viewportWidth:innerWidth,viewportHeight:innerHeight,statuses:[...d.querySelectorAll('.channel-node-result')].map(e=>e.textContent.trim())}}),'021 centered detection observation')
   assert021DetectionLayout(layout)
   if(exit==='x'){
    for(const modifiers of [0,8]){
     for(let i=0;i<6;i++){
      for(const type of ['keyDown','keyUp'])await call('Input.dispatchKeyEvent',{type,key:'Tab',code:'Tab',modifiers,windowsVirtualKeyCode:9,nativeVirtualKeyCode:9})
      assert(await evaluate("document.querySelector('dialog.channel-node-dialog[open]').contains(document.activeElement)"),'021 Tab/Shift+Tab escaped the owning dialog')
     }
    }
    await screenshot(driver,'native021-node-detection')
   }
   await close021Dialog(driver,'dialog.channel-node-dialog[open]',exit)
   assert(await evaluate(js(selector=>document.activeElement===document.querySelector(selector),trigger)),'021 detection did not restore its actual mouse trigger')
   checks.push({exit,...layout})
  }
 }finally{ipc=await observer.finish()}
 const after=await modelFileEvidence(config,expected)
 assert(after.bytes.equals(before.bytes),'021 detection observation wrote config');assert.deepEqual(after.backups,before.backups)
 assert.deepEqual((await invoke('model_file_snapshot')).policy,snapshot.policy,'021 detection observation changed the saved policy')
 return {checks,ipc,configSha256:hash(after.bytes),configAndBackupsUnchanged:true}
}

// Read the mounted production store only. The startup environment gate can
// legitimately hide overview cards; it must never be bypassed with DEV state.
export function inspect021Webdav(){
 const sync=document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?.state.value.sync
 const row=document.querySelector('[data-testid=overview-remote-version]'),card=row?.closest('.mod'),gate=document.querySelector('[data-testid=environment-gate]')
 const observation=sync?.observation
 // CDP returnByValue does not materialize Vue Proxy own properties. Read only
 // the explicit scalar facts, never return a live store object to the driver.
 return {state:sync?.webdavState,revision:sync?.config?.endpoint?.revision,hasObservation:!!observation,observation:observation?{revision:observation.revision,source:observation.source,remoteRead:observation.remoteRead,remoteTime:observation.remoteTime??null,remoteObservedAt:observation.remoteObservedAt}:null,checkingVersion:!!sync?.checkingVersion,
  versionCheckAvailable:[...card?.querySelectorAll?.('button')??[]].some(button=>button.textContent.trim()==='检测版本'),
  card:card?{text:card.textContent,remote:row.textContent,dots:row.querySelectorAll('.ov-state-dot').length}:null,
  gate:{present:!!gate,settled:!!gate?.querySelector('.overview-gate-count')?.textContent.includes('/ 3'),checks:gate?.querySelectorAll('.overview-checks li').length??0}}
}
export function assert021WebdavState(value,revision,{startupVersionCheck=false}={}){
 if(startupVersionCheck){
  assert.equal(value.state,'failed','Synthetic invalid endpoint must report a real failed version read')
  assert.equal(value.checkingVersion,false,'Version read must have settled')
  assert.equal(value.hasObservation,true,'Startup version check needs an actual observation')
  assert.equal(value.observation?.revision,revision,'Startup observation belongs to another revision')
  assert.equal(value.observation?.source,'version-check','Startup observation must be readonly version check')
  assert.equal(value.observation?.remoteRead,'failed','Invalid endpoint cannot fabricate remote content')
  assert(Number.isFinite(Date.parse(value.observation?.remoteObservedAt)),'Version failure needs an actual observation time')
  assert(value.observation?.remoteTime==null,'Failed read cannot fabricate a saved content time')
 }else{
  assert.equal(value.state,'configured','021 saved WebDAV must remain configured, not disconnected/connected')
  assert.equal(value.hasObservation,false,'021 WebDAV has a fabricated test/sync observation')
 }
 assert.equal(value.revision,revision,'021 DOM consumer used another endpoint revision')
 if(value.card){
  assert(value.card.text.includes(startupVersionCheck?'最近检测失败':'已配置 · 未检测'),'021 overview lost the actual connection state')
  if(startupVersionCheck)assert(value.card.remote.includes('读取失败'),'Overview must distinguish version read failure')
  assert(!/最近检测成功|最近同步成功|连接失败/.test(value.card.text),'021 overview fabricated a connection result')
  assert.equal(value.card.dots,0,'021 untested endpoint must have no remote status dot')
  return 'rendered'
 }
 assert(value.gate.present&&value.gate.settled&&value.gate.checks===3,'021 missing WebDAV card requires the actual settled environment gate')
 return 'environment-gated; production configured state verified; overview label requires a ready native environment'
}
async function verifyWebdav(driver,expected,{startupVersionCheck=false}={}){
 const {call,invoke,click,evaluate,wait}=driver,{config}=await freshAdmission(invoke,expected),before=await modelFileEvidence(config,expected)
 assert.equal((await invoke('get_sync_config')).endpoint,null,'021 WebDAV fixture requires the fresh startup sandbox, not an existing connection')
 assert.deepEqual((await invoke('list_imported_sync_connections')).items,[],'021 empty candidates must come from the actual backend')
 // Production save only: credentials.rs stores this synthetic password under
 // the admitted root using Windows DPAPI, without system keychain or network.
 const saved=await invoke('save_sync_endpoint',{request:{baseUrl:'https://webdav.native021.invalid',remotePath:'native021-only',username:'synthetic-native021',password:'synthetic-native021-webdav-password',conflictPolicy:'ask'}})
 assert(saved.endpoint&&/^[a-f0-9]{64}$/.test(saved.endpoint.revision),'021 configured endpoint must have a real backend revision')
 const stored=await readFile(join(expected,'manager-state','sync-endpoints.json')),disk=JSON.parse(stored)
 assert.equal(disk.endpoints[0].endpoint_id,saved.endpoint.endpointId);assert.equal(disk.endpoints[0].url,saved.endpoint.url)
 assert.equal(disk.endpoints[0].credential_ref.service_name,'OpenCodex Desktop Sandbox','021 fixture left the sandbox credential identity')
 assert(!stored.includes(Buffer.from('synthetic-native021-webdav-password')),'021 endpoint file contains the synthetic password')
 const candidates=await invoke('list_imported_sync_connections');assert.deepEqual(candidates.items,[]);assert.equal(typeof candidates.baseline,'string')
 await call('Page.reload',{ignoreCache:true});await wait("document.readyState==='complete'&&!!document.querySelector('.side')",'021 configured endpoint full reload')
 await freshAdmission(invoke,expected)
 const observer=await observe021Ipc(driver)
 let state,connection,ipc
 try{
  await click('概览','.side')
  await wait("!!document.querySelector('[data-testid=overview-remote-version]')||!!document.querySelector('[data-testid=environment-gate] .overview-gate-count')?.textContent.includes('/ 3')",'021 real overview state settled')
  state=await wait('('+js(inspect021Webdav)+'.state==='+JSON.stringify(startupVersionCheck?'failed':'configured')+')&&'+js(inspect021Webdav),'021 actual production configured/version-read state')
  state.presentation=assert021WebdavState(state,saved.endpoint.revision,{startupVersionCheck})
  await screenshot(driver,'native021-webdav-configured')
  await click('同步','.side');await wait("!!document.querySelector('.sync-route')",'021 WebDAV settings entry')
  await click('WebDAV 同步','.sync-route');await wait("!!document.querySelector('#sync-webdav-panel')",'021 WebDAV panel')
  await wait("document.querySelector('#sync-webdav-panel .sync-connection-summary')?.textContent.includes('当前WebDAV连接')&&document.querySelector('#sync-webdav-panel .sync-connection-summary')?.textContent.includes('https://webdav.native021.invalid')",'021 actual configured connection entry')
  await evaluate(js(install021CandidateObservation))
  await click('连接设置','.sync-route')
  await wait("!!document.querySelector('dialog.sync-connection-dialog[open]')&&[...document.querySelectorAll('dialog.sync-connection-dialog[open] input')].some(e=>e.value==='https://webdav.native021.invalid')",'021 actual saved connection form')
  const candidateObservation=await wait("(()=>{const t=globalThis.__ocx021Candidates;return t?.loadingSeen&&t.settledAt!==null&&!document.querySelector('dialog.sync-connection-dialog[open] .imported-connections')?{loadingSeen:t.loadingSeen,settledAt:t.settledAt}:null})()",'021 backend empty-candidate consumer settled after real loading DOM')
  connection=await evaluate("(()=>{const d=document.querySelector('dialog.sync-connection-dialog[open]');return {native:d instanceof HTMLDialogElement,title:d.querySelector('h2').textContent,closeCount:d.querySelectorAll('.ui-dialog-header .modal-close').length,actions:[...d.querySelectorAll('footer button')].map(e=>e.textContent.trim()),candidates:d.querySelectorAll('.sx-imported-connections,summary,.imported-row').length,password:d.querySelector('input[type=password]').value,alerts:[...d.querySelectorAll('[role=alert]')].map(e=>e.textContent)}})()")
  assert(connection.native&&connection.title==='WebDAV 连接'&&connection.closeCount===1)
  assert.deepEqual(connection.actions,['取消','保存连接']);assert.equal(connection.candidates,0);assert.equal(connection.password,'');assert.deepEqual(connection.alerts,[])
  connection.observation=candidateObservation
  await screenshot(driver,'native021-webdav-empty-candidates')
  await close021Dialog(driver,'dialog.sync-connection-dialog[open]','escape')
 }finally{
  try{ipc=await observer.finish(['save_transfer_choices'],startupVersionCheck?['config_sync_observe']:[])} // Local preference watcher + adopted readonly version read, never sync.
  finally{await evaluate('(()=>{globalThis.__ocx021Candidates?.dispose();delete globalThis.__ocx021Candidates;return true})()')}
 }
 assert(ipc.commands.includes('get_sync_config')&&ipc.commands.includes('list_imported_sync_connections'),'021 WebDAV DOM did not load through real IPC')
 assert.deepEqual(await invoke('get_sync_config'),saved,'021 settings observation changed the saved endpoint')
 assert((await readFile(join(expected,'manager-state','sync-endpoints.json'))).equals(stored),'021 cancelled settings changed its actual file')
 const after=await modelFileEvidence(config,expected);assert(after.bytes.equals(before.bytes));assert.deepEqual(after.backups,before.backups)
 return {revision:saved.endpoint.revision,fixture:'production save_sync_endpoint; synthetic sandbox-only DPAPI credential; invalid synthetic endpoint; no test/connection/sync request',startupVersionCheck,state,connection,ipc,modelConfigAndBackupsUnchanged:true,endpointBytesUnchanged:true}
}

export async function verifyWindows021Probes(driver,facts,expected){
 assert021Admission(facts,expected) // Invalid facts/version fail before any IPC/write.
 const result={schemaVersion:1,result:'fail',version:'0.2.21',platform:'Windows WebView2',admittedRoot:expected}
 try{
  result.admission=(await freshAdmission(driver.invoke,expected)).facts
  result.rename=await verifyRenameAndNotice(driver,expected)
  result.detection=await verifyDetection(driver,expected,result.rename.channelId)
  result.webdav=await verifyWebdav(driver,expected,{startupVersionCheck:result.rename.updatesDisplayName})
  result.limitations=result.webdav.state.card?[]:[result.webdav.state.presentation]
  result.finalAdmission=(await freshAdmission(driver.invoke,expected)).facts
  result.result='pass'
  return result
 }catch(error){
  result.error=error.message
  if(driver.call)try{await screenshot(driver,'native021-failure')}catch{}
  throw error
 }
 finally{await writeFile(join(driver.output,'native021-behavior.json'),JSON.stringify(result,null,2)+'\n')}
}
