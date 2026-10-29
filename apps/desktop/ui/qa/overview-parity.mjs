// Paired prototype/production-component QA. Fixtures exercise UI only, not native acceptance.
import fs from 'node:fs'
import path from 'node:path'
import os from 'node:os'
import net from 'node:net'
import {spawn} from 'node:child_process'
import {fileURLToPath} from 'node:url'
import {pickEngine} from './engine.mjs'
const ui=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..')
const prototype=process.env.PROTOTYPE_FILE
if(!prototype || !fs.existsSync(prototype))throw Error('Explicit frozen PROTOTYPE_FILE required')
const output=process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(),'ocx018-overview-'))
fs.mkdirSync(output,{recursive:true})
const server=net.createServer();await new Promise(r=>server.listen(0,'127.0.0.1',r));const port=server.address().port;await new Promise(r=>server.close(r))
const vite=spawn(process.execPath,[path.join(ui,'node_modules/vite/bin/vite.js'),'--host','127.0.0.1','--port',String(port),'--strictPort'],{cwd:ui,stdio:['ignore','pipe','pipe']})
let viteOutput='';vite.stderr.on('data',d=>viteOutput+=d)
for(let n=0;n<100;n++){try{await fetch('http://127.0.0.1:'+port);break}catch{await new Promise(r=>setTimeout(r,100))}}
const e=await pickEngine(),browser=await e.launch(),checks=[],errors=[]
function check(name,passed,detail){checks.push({name,passed:!!passed,detail});if(!passed)console.log('FAIL',name,detail)}
async function info(page,selector){return page.locator(selector).evaluate(n=>{const b=n.getBoundingClientRect();return {width:b.width,height:b.height,top:b.top,bottom:b.bottom,viewHeight:innerHeight,title:n.querySelector('h2,h3')?.textContent,buttons:[...n.querySelectorAll('button')].filter(x=>x.getClientRects().length).map(x=>({text:x.textContent.trim(),disabled:x.disabled})),horizontal:n.scrollWidth>n.clientWidth+1}})}
try{
 for(const c of [{w:1180,h:760,theme:'light'},{w:900,h:600,theme:'dark'}].flatMap(c=>['high','mid','low'].map(effects=>({...c,effects})))){
  const tag=c.w+'-'+c.theme+'-'+c.effects
  const proto=await browser.newPage({viewport:{width:c.w+520,height:c.h+368}}),app=await browser.newPage({viewport:{width:c.w,height:c.h}})
  app.on('pageerror',x=>errors.push(x.message));proto.on('pageerror',x=>errors.push('prototype: '+x.message))
  await proto.goto('file://'+encodeURI(prototype)+'#overview');await proto.waitForTimeout(500)
  await proto.evaluate(({w,h,theme,effects})=>{const win=document.querySelector('.window');win.style.width=w+'px';win.style.height=h+'px';win.style.flex='none';document.documentElement.dataset.theme=theme;document.documentElement.dataset.effects=effects;if(window.applyTheme)window.applyTheme(theme)},c)
  await app.goto('http://127.0.0.1:'+port+'/qa/overview-harness/index.html?effects='+c.effects+'&theme='+c.theme);await app.waitForSelector('.mods')
  await app.getByRole('button',{name:'导入配置',exact:true}).click();await app.waitForSelector('dialog[open]')
  await proto.evaluate(()=>{window.setEnv('ready');document.querySelector('[data-prototype-action=import]').click()})
  await proto.waitForTimeout(200)
  const pi=await info(proto,'#modal'),ii=await info(app,'dialog[open]')
  check(tag+' import initial disabled',ii.buttons.some(x=>x.text==='读取并比较'&&x.disabled))
  check(tag+' import file action accessible',await app.getByRole('button',{name:'选择文件',exact:true}).isEnabled())
  check(tag+' import title',pi.title===ii.title,{pi,ii})
  check(tag+' import remains overview',await app.locator('.ov').isVisible())
  check(tag+' import within window',ii.top>=0&&ii.bottom<=c.h&&!ii.horizontal,ii)
  await proto.locator('#modal').screenshot({path:path.join(output,tag+'-prototype-import.png')});await app.locator('dialog[open]').screenshot({path:path.join(output,tag+'-implementation-import.png')})
  await app.getByRole('button',{name:'取消',exact:true}).click();await proto.evaluate(()=>window.closeModal())
  await app.getByRole('button',{name:'查看桌面管理器更新详情',exact:true}).click();await app.waitForSelector('[data-testid=manager-update-card]')
  await proto.evaluate(()=>{document.querySelector('[data-maint-action=update-detail][data-target=manager]').click()});await proto.waitForTimeout(250)
  // If the prototype is initially unchecked, the explicit check has no side effects.
  if(!await proto.locator('#modal').innerText().then(t=>t.includes('更新详情与确认'))){await proto.evaluate(()=>{window.closeModal();document.querySelector('[data-maint-action=check-all]').click()});await proto.waitForTimeout(1100);await proto.evaluate(()=>document.querySelector('[data-maint-action=update-detail][data-target=manager]').click())}
  await app.waitForTimeout(100)
  const pd=await info(proto,'#modal'),id=await info(app,'.update-center')
  check(tag+' update single title',await app.locator('.update-center h3').count()===1)
  check(tag+' update title matches',pd.title===id.title,{pd,id})
  check(tag+' update width matches',Math.abs(pd.width-id.width)<=2,{prototype:pd.width,implementation:id.width})
  check(tag+' no repeated detail page',await app.locator('.update-back').count()===0)
  await proto.locator('#modal').screenshot({path:path.join(output,tag+'-prototype-update-confirm.png')});await app.locator('.update-center').screenshot({path:path.join(output,tag+'-implementation-update-confirm.png')})
  await app.getByTestId('install-manager-update').click();await proto.locator('#modalConfirm').click();await app.waitForTimeout(600);await proto.waitForTimeout(3600)
  const pp=await info(proto,'#modal'),ip=await info(app,'.update-center')
  check(tag+' progress title matches',pp.title===ip.title,{pp,ip})
  check(tag+' confirmation replaced',await app.locator('.release-notes').count()===0&&await app.locator('.update-backup').count()===0)
  check(tag+' pending restart still current',await app.locator('.update-center').innerText().then(t=>t.includes('当前仍为 0.1.9')))
  check(tag+' progress stage list',await app.locator('.update-stage-list li').count()===4)
  check(tag+' progress fits',ip.top>=0&&ip.bottom<=c.h&&!ip.horizontal,ip)
  await proto.locator('#modal').screenshot({path:path.join(output,tag+'-prototype-update-progress.png')});await app.locator('.update-center').screenshot({path:path.join(output,tag+'-implementation-update-progress.png')})
  await app.getByRole('button',{name:'关闭更新中心',exact:true}).click();await proto.evaluate(()=>window.closeModal())
  for(const state of ['checking','missing_node_brew','missing_node_nobrew','missing_npm','missing_ocx','ready']){
   const gate=state.startsWith('missing_node')?'missing_node':state
   await app.goto('http://127.0.0.1:'+port+'/qa/overview-harness/index.html?effects='+c.effects+'&theme='+c.theme+'&env='+gate+'&brew='+(state==='missing_node_nobrew'?'0':'1'))
   await app.waitForSelector('.ov');await proto.evaluate(state=>window.setEnv(state),state);await app.waitForTimeout(100)
   const prep=state!=='ready'
   check(tag+' '+state+' gate',await app.locator('[data-testid=environment-gate]').count()===(prep?1:0))
   if(prep)check(tag+' '+state+' check order',await app.locator('.overview-checks b').allTextContents().then(x=>x.join(',')==='Node.js,npm,OpenCodex'))
   await proto.locator('.window').screenshot({path:path.join(output,tag+'-prototype-env-'+state+'.png')});await app.screenshot({path:path.join(output,tag+'-implementation-env-'+state+'.png')})
  }
  await app.goto('http://127.0.0.1:'+port+'/qa/overview-harness/index.html?effects='+c.effects+'&theme='+c.theme+'&env=missing_ocx');await app.waitForSelector('[data-testid=environment-gate]');await app.getByRole('button',{name:'安装 OpenCodex',exact:true}).click();await app.waitForSelector('#wiz-prefix')
  await proto.evaluate(()=>{window.setEnv('missing_ocx');window.openRuntimeInstall('net')});await proto.waitForTimeout(100)
  const pw=await info(proto,'#modal'),iw=await info(app,'.runtime-install-dialog')
  check(tag+' install width matches',Math.abs(pw.width-iw.width)<=2,{pw,iw})
  check(tag+' install remains overview',await app.locator('.ov').isVisible())
  check(tag+' named steps',await app.locator('.wiz-steps li').count()===4)
  check(tag+' configuration same screen',await app.locator('#wiz-prefix').count()===1&&await app.locator('input[aria-label=版本]').count()===1&&await app.locator('[data-testid=wiz-ack]').count()===1)
  check(tag+' install requires ack',await app.getByRole('button',{name:'开始安装',exact:true}).isDisabled())
  check(tag+' install fits',iw.top>=0&&iw.bottom<=c.h&&!iw.horizontal,iw)
  await proto.locator('#modal').screenshot({path:path.join(output,tag+'-prototype-install.png')});await app.locator('.runtime-install-dialog').screenshot({path:path.join(output,tag+'-implementation-install.png')})
  await app.locator('[data-testid=wiz-ack]').check();check(tag+' valid ack enables',await app.getByRole('button',{name:'开始安装',exact:true}).isEnabled());await app.locator('#wiz-prefix').fill('relative');check(tag+' relative still blocked',await app.getByRole('button',{name:'开始安装',exact:true}).isDisabled())
  await app.getByRole('button',{name:'取消',exact:true}).click();check(tag+' cancel remains overview',await app.locator('.ov').isVisible()&&await app.locator('#wiz-prefix').count()===0)
  await app.close();await proto.close()
 }
 check('no unhandled page errors',errors.length===0,errors)
}finally{await browser.close();vite.kill('SIGTERM');fs.writeFileSync(path.join(output,'overview-report.json'),JSON.stringify({engine:e.name,prototype,checks,failed:checks.filter(x=>!x.passed).length,errors,viteOutput,scope:'UI fixture / paired screenshots; no native or real install claim'},null,2))}
console.log(output,checks.filter(x=>!x.passed).length+' failed / '+checks.length)
process.exitCode=checks.some(x=>!x.passed)?1:0
