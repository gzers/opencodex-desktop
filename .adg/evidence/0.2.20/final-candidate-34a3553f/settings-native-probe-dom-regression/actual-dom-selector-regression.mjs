import fs from 'node:fs/promises'
import assert from 'node:assert/strict'
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs'
const out=process.env.AUDIT_OUT;await fs.mkdir(out,{recursive:true});const e=await pickEngine(),b=await e.launch(),checks=[]
try{
 const p=await b.newPage({viewport:{width:1028,height:749}});await p.route('**/*',r=>new URL(r.request().url()).hostname==='127.0.0.1'?r.continue():r.abort())
 await p.goto('http://127.0.0.1:52422/?__audit=1&route=models&env=ready&runtime=running&theme=dark&effects=low');await p.locator('.channel-table').waitFor()
 await p.locator('.nav button').filter({hasText:'设置'}).click();await p.locator('.settings-tabs button').filter({hasText:'版本升级'}).waitFor()
 const before=await p.evaluate(()=>({wrongRoleTabs:[...document.querySelectorAll('[role=tab]')].filter(b=>b.textContent.trim()==='版本升级').length,realButtons:[...document.querySelectorAll('.settings-tabs button')].filter(b=>b.textContent.trim()==='版本升级').length}));assert.equal(before.wrongRoleTabs,0);assert.equal(before.realButtons,1)
 assert(await p.evaluate(()=>{const tab=[...document.querySelectorAll('.settings-tabs button')].find(b=>b.textContent.trim()==='版本升级'&&b.getClientRects().length>0&&!b.disabled);if(!tab)return false;tab.click();return true}))
 await p.getByRole('button',{name:'查看管理器更新',exact:true}).waitFor();await p.getByRole('button',{name:'查看管理器更新',exact:true}).click();await p.getByTestId('update-center').waitFor()
 for(const scale of [100,150,200]){
  await p.evaluate(scale=>document.documentElement.style.setProperty('--ui-zoom',String(scale/100)),scale)
  const sample=await p.evaluate(()=>{const m=document.querySelector('[data-testid=update-center] .update-center'),b=m.querySelector('.update-center-foot button'),r=m.getBoundingClientRect(),a=b.getBoundingClientRect();return {buttonHeight:a.height,x:r.x,y:r.y,right:r.right,bottom:r.bottom,viewportWidth:innerWidth,viewportHeight:innerHeight,buttonBottom:a.bottom}})
  assert(sample.x>=-1&&sample.y>=-1&&sample.right<=sample.viewportWidth+1&&sample.bottom<=sample.viewportHeight+1&&sample.buttonBottom<=sample.viewportHeight+1);assert(Math.abs(sample.buttonHeight/(checks[0]?.sample.buttonHeight??sample.buttonHeight)-scale/100)<.03);checks.push({scale,sample,passed:true})
 }
 await p.screenshot({path:out+'/settings-200.png'});await p.close();await fs.writeFile(out+'/report.json',JSON.stringify({result:'pass',engine:e.name,source:'actual production SettingsRoute/UpdateCenter with explicit DEV adapters',before,checks,boundary:'browser DOM driver precheck, not native Windows proof; wrong role selector fails, actual button selector opens production window'},null,2));console.log(e.name+' actual Settings selector and strict scales pass')
}finally{await b.close()}
