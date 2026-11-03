import fs from 'node:fs/promises'
import path from 'node:path'
import crypto from 'node:crypto'
import {pickEngine} from './engine.mjs'
const out=process.env.AUDIT_OUT;if(!out)throw Error('AUDIT_OUT required');await fs.mkdir(out,{recursive:true})
const identity=JSON.parse(await fs.readFile('/tmp/ocx020-final-review-agent/final-current-version-details/receipts/source-identity-before.json','utf8'))
const rawResponse=await fetch('http://127.0.0.1:52422/src/features/updates/UpdateCenter.vue?raw').then(r=>{if(!r.ok)throw Error('UpdateCenter raw HTTP '+r.status);return r.text()})
const rawLine=rawResponse.split(String.fromCharCode(10))[0];if(!rawLine.startsWith('export default '))throw Error('Missing raw-source export');let rawJson=rawLine.slice('export default '.length).trim();if(rawJson.endsWith(';'))rawJson=rawJson.slice(0,-1);
const actualSha=crypto.createHash('sha256').update(JSON.parse(rawJson)).digest('hex');if(actualSha!==identity.UpdateCenterSha256)throw Error('Source changed before expanded run: '+actualSha)
const engine=await pickEngine(),browser=await engine.launch(),checks=[],samples=[],screens=[],sourceSha256=actualSha
let completed=false,failure=null,key=''
const check=(name,passed,detail)=>{checks.push({key,name,passed:!!passed,detail});if(!passed)throw Error(key+' '+name+' '+JSON.stringify(detail))}
try{for(const owner of ['overview','settings'])for(const v of [{width:1180,height:760},{width:900,height:600}])for(const scale of [100,150,200]){
 key=[owner,v.width,v.height,scale].join('-')
 const p=await browser.newPage({viewport:v,deviceScaleFactor:1})
 await p.goto('http://127.0.0.1:52422/?__audit=1&route=models&env=ready&runtime=running&theme=dark&effects=low');await p.locator('.channel-table').waitFor()
 await p.evaluate(async({owner,scale})=>{
  (await import('/src/app/appearance/scale.ts')).applyInterfaceScale(scale)
  const app=(await import('/src/stores/app.ts')).useAppStore(),u=(await import('/src/features/updates/store.ts')).useUpdatesStore(),m=await import('/src/features/updates/update.ts')
  window.__notes=['Owned long release notes. Browser fixture only; no download or native install.',...Array.from({length:90},(_,i)=>'Owned paragraph '+i+' : '+('LongExplanationWithoutExternalRequest '.repeat(5))),'END-OWNED-LONG-NOTES'].join(String.fromCharCode(10))
  window.__status={channel:'stable',currentVersion:'0.2.19',availableVersion:'0.2.20',lastCheckedAt:'2026-11-04T00:00:00Z',signatureVerified:true,error:null,notes:window.__notes,pendingRestart:null}
  window.__installs=[];window.__calls=[];let callback=0
  window.__TAURI_INTERNALS__={metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},transformCallback:()=>++callback,unregisterCallback:()=>{},invoke:async(command,args)=>{
   window.__calls.push(command)
   if(command==='get_update_status')return {...window.__status}
   if(command==='check_for_update')return{status:'available',update:{...window.__status}}
   if(command==='update_release_details')return{target:args.target,channel:args.channel,candidateVersion:args.candidateVersion,notesState:'ready',notes:window.__notes,notesSource:'updater',releaseUrl:null,publishedAt:null,fetchedAt:1,cached:false}
   if(command==='install_update'){window.__installs.push({candidateVersion:args.candidateVersion,channel:args.channel,backup:args.backup});return new Promise((resolve,reject)=>{window.__rejectInstall=reject;window.__resolveInstall=resolve})}
   throw Error('Unapproved fixture IPC: '+command)
  }}
  m.managerUpdateStatus.value={...window.__status};m.managerInstallProgress.value=null
  u.officialProject={displayName:'Owned panel',version:'2.83.0',rawVersion:'2.83.0',truncated:false};u.officialRemote={tag:'owned',version:'2.84.0',integrity:null};u.officialProjectError=false;u.officialRemoteError=false
  app.loadOfficialProject=async()=>{};app.loadOfficialRemoteLatest=async()=>{}
  ;(await import('/src/app/feedback/store.ts')).useFeedbackStore().clearToast()
  ;(await import('/src/stores/routes.ts')).useRouteStore().go(owner,owner==='settings'?{section:'upgrade'}:{})
 },{owner,scale})
 await p.getByRole('button',{name:'检查更新',exact:true}).waitFor();const ownerHash=new URL(p.url()).hash
 const root=p.getByTestId('update-center'),dialog=root.locator('.update-center')
 const open=async()=>{if(owner==='settings')await p.getByRole('button',{name:'查看管理器更新',exact:true}).click();else await p.locator('button[aria-label^="查看"]').filter({hasText:'管理器'}).click();await root.waitFor()}
 const measure=async state=>{
  const g=await dialog.evaluate(el=>{const box=e=>{const r=e.getBoundingClientRect();return{x:r.x,y:r.y,right:r.right,bottom:r.bottom,width:r.width,height:r.height}},header=el.querySelector('.update-center-head'),foot=el.querySelector('.update-center-foot'),body=el.querySelector('.update-center-body'),shell=document.querySelector('.app-window'),button=foot.querySelector('button');return{
   viewport:{width:innerWidth,height:innerHeight},modal:box(el),header:box(header),footer:box(foot),body:box(body),bodyScroll:{top:body.scrollTop,client:body.clientHeight,height:body.scrollHeight,width:body.clientWidth,scrollWidth:body.scrollWidth,overflowY:getComputedStyle(body).overflowY},
   rootZoom:getComputedStyle(document.documentElement).getPropertyValue('--ui-zoom'),shellZoom:getComputedStyle(shell).zoom,modalZoom:getComputedStyle(el).zoom,bodyFont:getComputedStyle(document.body).fontSize,buttonFont:getComputedStyle(button).fontSize,titleFont:getComputedStyle(header.querySelector('h3')).fontSize,buttonHeight:button.getBoundingClientRect().height,
   headShrink:getComputedStyle(header).flexShrink,footShrink:getComputedStyle(foot).flexShrink,
   controls:[...foot.querySelectorAll('button')].filter(e=>e.offsetWidth>0).map(e=>{const r=e.getBoundingClientRect(),x=r.x+r.width/2,y=r.y+r.height/2,hit=document.elementFromPoint(x,y);return{label:e.getAttribute('aria-label')||e.textContent.trim(),disabled:e.disabled,rect:box(e),hit:e.contains(hit),coveredBy:hit?.tagName+'.'+hit?.className}})}})
  samples.push({key,owner,...v,scale,state,...g})
  check(state+' all four modal edges within actual viewport',g.modal.x>=-1&&g.modal.y>=-1&&g.modal.right<=v.width+1&&g.modal.bottom<=v.height+1,g)
  check(state+' header footer and content retain height',g.header.height>0&&g.footer.height>0&&g.body.height>0&&g.headShrink==='0'&&g.footShrink==='0',g)
  check(state+' close and confirmation controls fit and receive pointer',g.controls.length>0&&g.controls.every(c=>c.rect.x>=-1&&c.rect.y>=-1&&c.rect.right<=v.width+1&&c.rect.bottom<=v.height+1&&c.rect.height>0&&c.hit),g.controls)
  check(state+' no horizontal content clipping',g.bodyScroll.scrollWidth<=g.bodyScroll.width+1,g.bodyScroll)
  check(state+' exact shared zoom and compact logical typography',Math.abs(Number(g.modalZoom)-scale/100)<.001&&Math.abs(Number(g.shellZoom)-scale/100)<.001&&g.bodyFont==='13px'&&g.buttonFont==='12px'&&g.titleFont==='18px',g)
  const file=key+'-'+state+'.png';await p.screenshot({path:path.join(out,file)});screens.push(file);return g
 }
 const clickCenter=async locator=>{const g=await locator.evaluate(e=>{const r=e.getBoundingClientRect(),x=r.x+r.width/2,y=r.y+r.height/2;return{x,y,hit:e.contains(document.elementFromPoint(x,y)),disabled:e.disabled}});check('actual action center is enabled and exposed',g.hit&&!g.disabled&&g.x>=0&&g.x<v.width&&g.y>=0&&g.y<v.height,g);await p.mouse.click(g.x,g.y)}
 await p.getByRole('button',{name:'检查更新',exact:true}).click();await root.getByTestId('manager-update-detail').waitFor();await measure('results');check('result controls remain two detail entries',await root.locator('.update-result-row').count()===2&&await root.getByTestId('manager-update-detail').count()===1&&await root.getByTestId('official-update-detail').count()===1)
 await clickCenter(root.getByRole('button',{name:'关闭更新中心',exact:true}));await root.waitFor({state:'detached'});check('result close returns exact owner',new URL(p.url()).hash===ownerHash)
 await open();await root.getByTestId('install-manager-update').waitFor();await root.locator('.release-notes-body').waitFor();await measure('details-long-top')
 const scrolled=await root.locator('.release-notes-body').evaluate(el=>{const before={top:el.scrollTop,height:el.clientHeight,scroll:el.scrollHeight,overflowY:getComputedStyle(el).overflowY};el.scrollTop=el.scrollHeight;return{before,after:el.scrollTop,end:el.textContent.endsWith('END-OWNED-LONG-NOTES')}})
 check('long release notes have independent working internal scroll',scrolled.before.scroll>scrolled.before.height+1&&scrolled.before.height>0&&['auto','scroll'].includes(scrolled.before.overflowY)&&scrolled.after>0&&scrolled.end,scrolled)
 const outer=await root.locator('.update-center-body').evaluate(el=>{const max=el.scrollHeight-el.clientHeight;el.scrollTop=max;return{max,top:el.scrollTop,overflowY:getComputedStyle(el).overflowY}})
 check('constrained outer content scroll works without moving footer',outer.max<=1||outer.top>0&&['auto','scroll'].includes(outer.overflowY),outer);await measure('details-long-bottom')
 const footerClose=root.getByRole('button',{name:'关闭更新中心',exact:true}),confirm=root.getByTestId('install-manager-update')
 await clickCenter(footerClose);await root.waitFor({state:'detached'});check('detail close never installs and retains owner',await p.evaluate(()=>window.__installs.length)===0&&new URL(p.url()).hash===ownerHash)
 await open();await clickCenter(confirm);await p.waitForFunction(()=>window.__installs.length===1);check('reachable confirmation invokes exact memory-only intent',await p.evaluate(()=>window.__installs[0].candidateVersion==='0.2.20'&&window.__installs[0].channel==='stable'&&window.__installs[0].backup===true));await p.evaluate(()=>window.__rejectInstall(Error('Owned expanded check completed; no real install')));await root.getByRole('alert').waitFor();await measure('confirm-refused')
 await p.keyboard.press('Escape');await root.waitFor({state:'detached'});check('Escape after confirmation failure returns exact host',new URL(p.url()).hash===ownerHash);await p.close()
}
 for(const owner of ['overview','settings'])for(const width of [1180,900])for(const scale of [150,200]){
  key=[owner,width,'scale-ratio',scale].join('-');const a=samples.find(x=>x.owner===owner&&x.width===width&&x.scale===100&&x.state==='details-long-top'),z=samples.find(x=>x.owner===owner&&x.width===width&&x.scale===scale&&x.state==='details-long-top'),ratio=z.buttonHeight/a.buttonHeight
  if(scale===200)check('original 200% painted-size threshold retained',ratio>=1.8,{ratio,button100:a.buttonHeight,buttonScaled:z.buttonHeight})
  check('painted control ratio equals requested scale',Math.abs(ratio-scale/100)<.03,{ratio,expected:scale/100,button100:a.buttonHeight,buttonScaled:z.buttonHeight})
 }
 completed=true
}catch(error){failure={key,message:error.message};const p=browser.contexts().flatMap(c=>c.pages()).at(-1);if(p){await p.screenshot({path:path.join(out,'failure.png')});await fs.writeFile(path.join(out,'failure.txt'),await p.locator('body').innerText())}throw error}
finally{await browser.close();await fs.writeFile(path.join(out,'report.json'),JSON.stringify({engine:engine.name,completed,failure,result:completed&&checks.every(x=>x.passed)?'pass':'fail',sourceSha256,sourceHead:identity.head,scope:'Actual Overview/Settings consumers; real browser viewport900x600/1180x760 and100/150/200%; pointer-hit and actual mouse actions, long notes/body scroll. Explicit memory-only IPC; no native/network/install/OTA.',checks,samples,screens},null,2))}
console.log(engine.name,checks.length,'checks',screens.length,'screens')
