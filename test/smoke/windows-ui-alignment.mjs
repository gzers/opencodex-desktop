// Runs only inside the already admitted, disposable Windows startup sandbox.
// Uses actual Vue event handlers and WebView2 layout, not a DEV fixture renderer.
import assert from 'node:assert/strict'
import {writeFile} from 'node:fs/promises'
import {join} from 'node:path'
export function overviewScene(rows,gate){
 if(rows.length>=6)return 'ready'
 assert.equal(rows.length,0,'partial information cards are not a valid overview state')
 assert(gate.present&&gate.settled&&gate.checks===3,'missing cards require the real, settled environment gate')
 return 'environment-gated'
}
export async function verifyUiAlignment({call,evaluate,wait,click,output,templateLabel}){
 const receipts=[]
 async function shot(name){const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'ui021-'+name+'.png'),Buffer.from(image.data,'base64'))}
 async function measureDetail(label){
  const expression=(`(()=>{const nav=document.querySelector('.detail-navigation'),save=nav?.querySelector('.detail-navigation-primary button'),r=save?.getBoundingClientRect(),main=document.querySelector('main.main')?.getBoundingClientRect();return {label:${JSON.stringify(label)},save:r?{top:r.top,bottom:r.bottom,height:r.height}:null,viewport:innerHeight,mainBottom:main?.bottom??innerHeight,topActions:document.querySelector('.sticky-page-tabs')?.textContent,icons:[...nav?.querySelectorAll('.btn-content svg')??[]].map(e=>{const a=e.getBoundingClientRect(),b=e.parentElement.getBoundingClientRect();return Math.abs(a.top+a.height/2-b.top-b.height/2)})}})()`);
  await wait('(r=>r.save&&r.save.height>0&&r.save.top>=0&&r.save.bottom<=Math.min(r.viewport,r.mainBottom)+2)('+expression+')','visible primary save '+label)
  const result=await evaluate(expression)
  assert(result.save&&result.save.height>0&&result.save.top>=0&&result.save.bottom<=Math.min(result.viewport,result.mainBottom)+2,'primary save must be visible in native viewport: '+JSON.stringify(result))
  if(label.startsWith('channel'))assert(!result.topActions?.includes('保存'),'detail save duplicated in top tabs');
  assert(result.icons.every(delta=>delta<=1.1),'icon/content center drift');receipts.push(result)
 }
 await measureDetail('channel-expanded');await shot('channel-detail')
 await click('收起渠道导航');await wait("!!document.querySelector('.detail-collapsed')",'collapsed channel sidebar');await measureDetail('channel-collapsed')
 await click('返回渠道列表');await wait("!!document.querySelector('.channel-table tbody tr')",'channel list for column geometry')
 const columns=await evaluate("(()=>{const t=document.querySelector('.channel-table'),a=[...t.tHead.rows[0].cells],b=[...t.tBodies[0].rows[0].cells];return {width:t.getBoundingClientRect().width,selection:b[0].getBoundingClientRect().width,offsets:a.map((e,i)=>Math.abs(e.getBoundingClientRect().left-b[i].getBoundingClientRect().left))}})()")
 assert(columns.selection<=40&&columns.selection/columns.width<0.06);assert(columns.offsets.every(delta=>delta<=1));receipts.push({label:'channel-columns',...columns});await shot('channels')
 await click('模型模版');await wait("!!document.querySelector('.template-table tbody tr')",'template column geometry')
 const templates=await evaluate("(()=>{const t=document.querySelector('.template-table'),row=t.tBodies[0].rows[0];return {columns:row.cells.length,selection:row.cells[0].getBoundingClientRect().width,agent:[...document.querySelectorAll('.sticky-page-tabs button')].some(e=>e.textContent.trim()==='Agent 配置')}})()")
 assert.equal(templates.columns,4);assert(templates.selection<=40&&templates.agent);receipts.push({label:'template-columns',...templates});await shot('templates')
 await evaluate(`(()=>{const row=[...document.querySelectorAll('.template-table tbody tr')].find(e=>e.textContent.includes(${JSON.stringify(templateLabel)}));[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑').click()})()`);await wait("!!document.querySelector('#model-basic')",'template identity')
 assert(await evaluate("document.querySelector('.detail-workspace-head').textContent.includes('编辑基于')"));await measureDetail('template-expanded');await shot('template-detail')
 await click('同步','.side');await wait("!!document.querySelector('#sync-file-panel .sync-scope-workspace')",'inline file scope')
 assert(await evaluate("!!document.querySelector('#sync-file-panel .sync-scope-credentials')&&!document.querySelector('#sync-file-panel dialog[open]')"));await shot('file-scope')
 await click('WebDAV 同步','.sync-route');await wait("!!document.querySelector('#sync-webdav-panel .sync-scope-workspace')",'inline WebDAV scope')
 const remote=await evaluate("(()=>{const parent=document.querySelector('#sync-webdav-panel .u-stack'),cards=parent?[...parent.children].filter(e=>e.getClientRects().length):[];return {scope:!!document.querySelector('#sync-webdav-panel .sync-scope-credentials'),gaps:cards.slice(1).map((e,i)=>e.getBoundingClientRect().top-cards[i].getBoundingClientRect().bottom),settings:!![...document.querySelectorAll('#sync-webdav-panel button')].find(e=>e.textContent.trim()==='连接设置')}})()")
 assert(remote.scope&&remote.settings);assert(remote.gaps.every(gap=>gap>=8),'WebDAV cards require spacing');receipts.push({label:'remote-scope',...remote});await shot('webdav-scope')
 await click('连接设置','.sync-route');await wait("!!document.querySelector('dialog[open]')",'on-demand connection settings');await shot('connection-dialog');await click('关闭','dialog[open]')
 await click('设置','.side');await wait("!!document.querySelector('[role=radiogroup][aria-label=视觉特效]')",'quality segmented radios')
 assert.equal(await evaluate("document.querySelectorAll('[role=radiogroup][aria-label=视觉特效] [role=radio]').length"),3)
 for(const [effect,label] of [['high','高'],['mid','中'],['low','低']]){
  await click(label,'[role=radiogroup][aria-label=视觉特效]');await wait('document.documentElement.dataset.effects==='+JSON.stringify(effect),'actual quality preference '+effect)
  for(const [theme,button] of [['light','浅色'],['dark','深色']]){
   await click(button,'.theme-seg');await wait('document.documentElement.dataset.theme==='+JSON.stringify(theme),'actual theme '+theme);await shot(theme+'-'+effect)
  }
 }
 await click('高','[role=radiogroup][aria-label=视觉特效]');await click('浅色','.theme-seg')
 for(const scale of [50,150,200,100]){
  await evaluate("(()=>{const input=document.querySelector('[aria-label=界面缩放数值]');input.value="+scale+";input.dispatchEvent(new Event('change',{bubbles:true}))})()")
  await wait("document.documentElement.style.getPropertyValue('--ui-zoom')==="+JSON.stringify(String(scale/100)),'production UI scale '+scale)
  await click('模型','.side');await wait("!!document.querySelector('#model-basic')",'preserved template workspace at scale '+scale);await measureDetail('template-zoom-'+scale);await shot('template-zoom-'+scale)
  await click('设置','.side');await wait("!!document.querySelector('[aria-label=界面缩放数值]')",'return to actual scale control')
 }
 await click('Agent 配置','.settings-tabs');await wait("!![...document.querySelectorAll('.settings-panel button')].find(e=>e.textContent.trim()==='Agent 配置')",'persistent Agent configuration');await shot('agent-settings')
 await click('概览','.side');await wait("document.querySelectorAll('.info-row').length>=6||!!document.querySelector('[data-testid=environment-gate] .overview-gate-count')?.textContent.includes('/ 3')",'settled overview environment state')
 const information=await evaluate("[...document.querySelectorAll('.info-row')].map(e=>{const s=getComputedStyle(e),r=e.getBoundingClientRect(),label=e.querySelector('.info-row-label').getBoundingClientRect();return {font:s.fontSize,line:s.lineHeight,padding:s.padding,labelInset:label.left-r.left}})")
 const gate=await evaluate("(()=>{const e=document.querySelector('[data-testid=environment-gate]');return {present:!!e,settled:!!e?.querySelector('.overview-gate-count')?.textContent.includes('/ 3'),checks:e?.querySelectorAll('.overview-checks li').length??0}})()")
 const scene=overviewScene(information,gate)
 if(scene==='ready'){assert.equal(new Set(information.map(x=>x.font+'/'+x.line+'/'+x.padding)).size,1);assert(information.every(x=>x.labelInset>=8))}
 receipts.push({label:'overview-information',state:scene,rows:information,...(scene==='environment-gated'?{gate,limitation:'This isolated no-HOME/no-runtime admission intentionally shows setup, not ready cards. InformationRow rendering is checked in the same-source Gallery and ready macOS sandbox; no injected environment bypass.'}:{})});await shot('overview')
 const result={result:'pass',platform:'Windows WebView2',checks:receipts,scope:'actual production consumers and 6 theme/quality combinations; no network sync or model generation'}
 await writeFile(join(output,'ui021-alignment.json'),JSON.stringify(result,null,2)+'\n');return result
}
