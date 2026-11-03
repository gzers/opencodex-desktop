import fs from 'node:fs/promises'
import path from 'node:path'
import crypto from 'node:crypto'
import {pickEngine} from './engine.mjs'
const R='/tmp/ocx020-final-review-agent/final-current-version-details',out=process.env.AUDIT_OUT;await fs.mkdir(out,{recursive:true})
const identity=JSON.parse(await fs.readFile(process.env.REVIEW_SOURCE_IDENTITY||R+'/receipts/source-identity-before.json','utf8')),actual=await fs.readFile('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/src/features/updates/UpdateCenter.vue')
if(crypto.createHash('sha256').update(actual).digest('hex')!==identity.sourceSha256)throw Error('Source changed before matrix')
const engine=await pickEngine(),browser=await engine.launch(),checks=[],states=[],screens=[];let key='',completed=false,failure=null
const check=(name,passed,detail)=>{checks.push({key,name,passed:!!passed,detail});if(!passed)throw Error(key+' '+name+' '+JSON.stringify(detail))}
async function create(owner,target,condition,progress=false){
 const p=await browser.newPage({viewport:{width:1180,height:760}})
 await p.goto('http://127.0.0.1:52422/?__audit=1&route=models&env=ready&runtime=running&theme=dark&effects=low');await p.locator('.channel-table').waitFor()
 const local=progress?(target==='manager'?'0.2.19':'2.83.0'):(condition==='unknownCurrent'?null:(target==='manager'?'0.2.20':'2.84.0'))
 const remote=progress?(target==='manager'?'0.2.20':'2.84.0'):(condition==='none'?null:target==='manager'?({same:'0.2.20',lower:'0.2.19',newer:'0.2.21',unknownCurrent:'0.2.20'}[condition]):({same:'2.84.0',lower:'2.83.0',newer:'2.85.0',unknownCurrent:'2.84.0'}[condition]))
 await p.evaluate(async({owner,target,local,remote})=>{
  const app=(await import('/src/stores/app.ts')).useAppStore(),u=(await import('/src/features/updates/store.ts')).useUpdatesStore(),m=await import('/src/features/updates/update.ts')
  window.__state={channel:'stable',currentVersion:target==='manager'?local:'0.2.20',availableVersion:target==='manager'?remote:'0.2.20',lastCheckedAt:'2026-11-04T00:00:00Z',signatureVerified:true,error:null,notes:null,pendingRestart:null}
  window.__calls=[];window.__managerInstalls=[];window.__officialInstalls=[];window.__details=[];let callback=0
  window.__TAURI_INTERNALS__={metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},transformCallback:()=>++callback,unregisterCallback:()=>{},invoke:async(command,args)=>{
   window.__calls.push(command)
   if(command==='get_update_status')return {...window.__state}
   if(command==='check_for_update')return{status:'available',update:{...window.__state}}
   if(command==='update_release_details'){window.__details.push({...args});return{target:args.target,channel:args.channel,candidateVersion:args.candidateVersion,notesState:'ready',notes:'OWNED-EXACT-RELEASE-'+args.target+'-'+args.candidateVersion,notesSource:'updater',releaseUrl:null,publishedAt:null,fetchedAt:1,cached:false}}
   if(command==='install_update'){window.__managerInstalls.push({candidateVersion:args.candidateVersion,channel:args.channel,backup:args.backup});return new Promise(resolve=>window.__finishManager=resolve)}
   throw Error('Unexpected browser fixture command '+command)
  }}
  m.managerUpdateStatus.value={...window.__state};m.managerInstallProgress.value=null;app.aboutApp={...(app.aboutApp||{}),version:target==='manager'&&!local?'未知':'0.2.20'}
  u.officialProject=target==='official'&&!local?null:{displayName:'Owned panel',version:target==='official'?local:'2.84.0',rawVersion:target==='official'?local:'2.84.0',truncated:false}
  u.officialRemote=target==='official'&&!remote?null:{tag:'owned',version:target==='official'?remote:'2.84.0',integrity:null};u.officialRemoteError=false;u.officialProjectError=false
  app.loadOfficialProject=async()=>{};app.loadOfficialRemoteLatest=async()=>{}
  u.createUpgradeBackup=async()=>true
  app.applyOfficialUpdate=async version=>{window.__officialInstalls.push(version);return new Promise(resolve=>window.__finishOfficial=resolve)}
  ;(await import('/src/app/feedback/store.ts')).useFeedbackStore().clearToast();(await import('/src/stores/routes.ts')).useRouteStore().go(owner,owner==='settings'?{section:'upgrade'}:{})
 },{owner,target,local,remote})
 await p.getByRole('button',{name:'检查更新',exact:true}).waitFor();const ownerHash=new URL(p.url()).hash
 await p.getByRole('button',{name:'检查更新',exact:true}).click();const root=p.getByTestId('update-center');await root.getByTestId(target+'-update-detail').waitFor()
 const resultRow=root.getByTestId(target+'-update-row');check('result-row candidate only for proven newer',((await resultRow.locator('.update-result-version').innerText()).includes('→'))===(condition==='newer'||progress))
 await root.getByTestId(target+'-update-detail').click();await root.getByTestId(target+'-update-card').waitFor()
 return{p,root,local,remote,ownerHash}
}
async function snapshot(p,root,owner,target,condition,phase){
 const x=await root.evaluate(el=>({title:el.querySelector('h3')?.textContent,hero:el.querySelector('.update-version-hero')?.textContent,current:el.querySelector('.update-version-hero>span')?.textContent,candidate:el.querySelector('.update-version-hero>strong')?.textContent??null,arrow:[...el.querySelectorAll('.update-version-hero>span')].some(x=>x.textContent==='→'),footer:[...el.querySelectorAll('.update-center-foot button')].map(x=>({text:x.textContent.trim(),label:x.getAttribute('aria-label'),disabled:x.disabled})),hints:[...el.querySelectorAll('.update-hint')].map(x=>x.textContent.trim()),text:el.innerText,notes:el.querySelector('.release-notes-body')?.textContent??null}))
 states.push({key,owner,target,condition,phase,...x});const f=key+'-'+phase+'.png';await p.screenshot({path:path.join(out,f)});screens.push(f);return x
}
try{
 for(const owner of ['overview','settings'])for(const target of ['manager','official'])for(const condition of (process.env.REVIEW_BODY_ONLY?['same','lower','none','unknownCurrent']:['same','lower','none','newer','unknownCurrent'])){
  key=[owner,target,condition].join('-');const{p,root,local,remote,ownerHash}=await create(owner,target,condition)
  const hasCandidate=condition==='newer'||condition==='unknownCurrent',available=condition==='newer',prefix=target==='manager'?'桌面管理器':'OpenCodex 面板'
  if(hasCandidate)await root.locator('.release-notes-body').waitFor()
  const x=await snapshot(p,root,owner,target,condition,'detail')
  check('release notes block exists only with readable candidate',await root.locator('.release-notes').count()===(hasCandidate?1:0))
  check('current version stays exact',x.current===(local??'未知'),x)
  check('cached equal or lower does not produce a candidate arrow',x.arrow===hasCandidate,x)
  check('candidate version is exact or absent',x.candidate===(hasCandidate?remote:null),x)
  check('title only requests confirmation for actual newer version',x.title===prefix+(available?'更新详情与确认':'更新详情'),x)
  check('footer says close for read-only details',x.footer[0].text===(available?'取消':'关闭'),x)
  check('update confirmation exists only for actual newer version',x.footer.length===(available?2:1)&&((await root.getByTestId('install-'+target+'-update').count())===(available?1:0)),x)
  const hint=target==='manager'?'安装后需重启管理器':'更新已登记的面板包体'
  check('install or restart explanation absent from latest/unknown details',x.hints.some(t=>t.includes(hint))===available,x.hints)
  check('no restart button without pending progress',await root.getByRole('button',{name:'重启并应用更新',exact:true}).count()===0)
  if(hasCandidate){check('exact remote release description stays readable',x.notes==='OWNED-EXACT-RELEASE-'+target+'-'+remote,x.notes);check('release query binds exact target and candidate',await p.evaluate(({target,remote})=>window.__details.some(d=>d.target===target&&d.candidateVersion===remote),{target,remote}))}
  else check('same/lower/none does not fetch an obsolete candidate description',await p.evaluate(()=>window.__details.length)===0)
  await root.getByRole('button',{name:'关闭更新中心',exact:true}).click();await root.waitFor({state:'detached'});check('read-only close retains original owner',new URL(p.url()).hash===ownerHash);check('reading details performs zero install/restart actions',await p.evaluate(()=>window.__managerInstalls.length===0&&window.__officialInstalls.length===0&&!window.__calls.includes('restart_after_update')));await p.close()
 }
 if(!process.env.REVIEW_BODY_ONLY)for(const owner of ['overview','settings'])for(const target of ['manager','official']){
  key=[owner,target,'pending-frozen'].join('-');const{p,root,local,remote,ownerHash}=await create(owner,target,'newer',true)
  await root.getByTestId('install-'+target+'-update').click();await p.waitForFunction(target=>target==='manager'?window.__managerInstalls.length===1:window.__officialInstalls.length===1,target)
  await p.evaluate(async target=>{
   const u=(await import('/src/features/updates/store.ts')).useUpdatesStore(),m=await import('/src/features/updates/update.ts')
   if(target==='manager'){
    window.__state={...window.__state,currentVersion:'0.2.20',availableVersion:'0.2.21',pendingRestart:'0.2.20'};m.managerUpdateStatus.value={...window.__state};m.managerInstallProgress.value={operationId:'owned-pending',target:'manager',channel:'stable',candidateVersion:'0.2.20',generation:1,sequence:1,stage:'pending_restart',downloadedBytes:1,totalBytes:1};window.__finishManager()
   }else{u.officialProject={displayName:'Owned panel',version:'2.84.0',rawVersion:'2.84.0',truncated:false};u.officialRemote={tag:'newer-cached',version:'2.85.0',integrity:null}}
  },target)
  if(target==='manager')await root.getByRole('button',{name:'重启并应用更新',exact:true}).waitFor();else await root.getByTestId('official-update-progress').waitFor()
  const x=await snapshot(p,root,owner,target,'pending','frozen')
  check('progress title remains progress',x.title===(target==='manager'?'桌面管理器':'OpenCodex 面板')+'更新进度',x)
  check('progress retains the starting current version',x.current===local,x)
  check('progress retains confirmed candidate despite changed cached versions',x.arrow&&x.candidate===remote,x)
  check('progress close is labelled close window',x.footer[0].text==='关闭窗口',x)
  check('progress does not offer a second update confirmation',await root.locator('.update-center-foot [data-testid^=install-]').count()===0)
  if(target==='manager'){check('pending restart explains frozen actual transition',x.hints.some(t=>t.includes('当前仍为 0.2.19')&&t.includes('重启后核验 0.2.20')),x.hints);check('pending restart action remains present',await root.getByRole('button',{name:'重启并应用更新',exact:true}).count()===1)}
  await p.keyboard.press('Escape');await root.waitFor({state:'detached'});check('progress dismissal retains owner route',new URL(p.url()).hash===ownerHash)
  if(target==='manager'){if(owner==='settings')await p.getByRole('button',{name:'查看管理器更新',exact:true}).click();else await p.getByRole('button',{name:'查看管理器进度',exact:true}).click()}
  else{if(owner==='settings')await p.getByRole('button',{name:'查看面板更新',exact:true}).click();else await p.locator('button[aria-label^="查看"]').filter({hasText:'面板'}).click()}
  await root.waitFor();const reopened=await snapshot(p,root,owner,target,'pending','reopened');check('reopening progress preserves frozen version pair',reopened.current===local&&reopened.candidate===remote&&reopened.title.endsWith('更新进度'),reopened)
  if(target==='official'){await p.evaluate(()=>window.__finishOfficial(false));await root.getByRole('alert').waitFor();const failed=await snapshot(p,root,owner,target,'pending','failed');check('failed official progress retains confirmed candidate',failed.current===local&&failed.candidate===remote,failed)}
  await p.close()
 }
 completed=true
}catch(error){failure={key,message:error.message};const p=browser.contexts().flatMap(c=>c.pages()).at(-1);if(p){await p.screenshot({path:path.join(out,'failure.png')});await fs.writeFile(path.join(out,'failure.txt'),await p.locator('body').innerText())}throw error}
finally{await browser.close();await fs.writeFile(path.join(out,'report.json'),JSON.stringify({engine:engine.name,completed,failure,result:completed?'pass':'fail',sourceSha256:identity.sourceSha256,scope:'Actual production Overview/Settings UpdateCenter with explicit memory-only state/IPC. Covers2hosts*2targets*same/lower/none/newer/unknownCurrent plus frozen pending manager/official sessions. No real network/native/install/restart.',checks,states,screens},null,2))}
console.log(engine.name,checks.length,'checks',screens.length,'screens')
