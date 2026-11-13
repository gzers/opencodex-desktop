import {createServer} from 'vite'
import {chromium} from 'playwright-core'
import fs from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
const ui=process.cwd(), output=process.argv[2], name='.read-notification-preview-'+Date.now(), dir=path.join(ui,'qa',name)
const summary='已读取 305 个模型：自动匹配 280 项，更新草稿 270 项，保留人工指定 5 项，未匹配 12 项，冲突 3 项；缺默认版本 2 项，缺依赖 3 项，协议待配置 9 项。保存配置后才生效。'
const notice={id:'read-operation-one',kind:'info',category:'system',action:'models',title:'现有渠道读取完成',detail:summary,read:false,resolved:true,time:'2026-11-11T00:00:00Z'}
await fs.mkdir(dir)
let server,browser
try {
 await fs.writeFile(path.join(dir,'index.html'),'<div id="app"></div><script type="module" src="./main.ts"></script>')
 await fs.writeFile(path.join(dir,'main.ts'),"import {createApp} from 'vue';import {createPinia} from 'pinia';import App from './App.vue';import '@/styles/tokens.css';import '@/styles/base.css';import '@/styles/materials.css';import '@/styles/effects.css';createApp(App).use(createPinia()).mount('#app')")
 await fs.writeFile(path.join(dir,'App.vue'),`<script setup lang="ts">
 import {ref} from 'vue';import ToastHost from '@/components/ToastHost.vue';import NotificationItem from '@/features/notifications/components/NotificationItem.vue';import {useNotificationsStore} from '@/features/notifications/store';import {useFeedbackStore} from '@/app/feedback/store';
 const notices=useNotificationsStore(),feedback=useFeedbackStore(),center=ref(false),opened=ref(false);
 (window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{if(command==='list_notifications')return {items:[${JSON.stringify(notice)}]};throw Error('unexpected native invocation '+command)}};
 (window as any).fixture={announce:()=>notices.announce(),clear:()=>feedback.clearToast()};
 </script><template><main style="padding:24px"><h1>模型配置</h1><p>共享通知组件 · 隔离验证</p><button @click="center=!center" aria-label="通知中心">铃铛</button></main><div class="notif-layer"><ToastHost/><section v-if="center" class="notification-panel show" style="position:static;width:100%"><h3>通知中心</h3><NotificationItem v-for="item in notices.list" :key="item.id" :item="item" @open="opened=true" /></section></div><section v-if="opened" role="dialog" aria-label="读取结果详情" style="position:fixed;bottom:20px;left:24px;right:24px;padding:20px;background:var(--panel);border:1px solid var(--border)"><h2>现有渠道读取完成</h2><p>${summary}</p></section></template>`)
 server=await createServer({server:{host:'127.0.0.1',port:0,strictPort:false}});await server.listen();const address=server.httpServer.address()
 const caches=path.join(os.homedir(),'Library/Caches/ms-playwright'),versions=(await fs.readdir(caches)).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()
 const executablePath=path.join(caches,versions[0],'chrome-headless-shell-mac-arm64/chrome-headless-shell')
 browser=await chromium.launch({executablePath});const results=[]
 for (const [width,height] of [[1040,760],[420,640]]) {
  const page=await browser.newPage({viewport:{width,height}}),errors=[];page.on('pageerror',e=>errors.push(e.message))
  await page.goto('http://127.0.0.1:'+address.port+'/qa/'+name+'/index.html');await page.evaluate(()=>window.fixture.announce());await page.locator('.toast').waitFor()
  if((await page.locator('.toast-txt').textContent())!=='现有渠道读取完成：'+summary)throw Error('actual toast summary mismatch')
  await page.screenshot({path:path.join(output,'visual-toast-'+width+'.png')})
  await page.getByRole('button',{name:'收起这条操作反馈到铃铛'}).click();await page.waitForFunction(()=>!document.querySelector('.toast'))
  await page.getByRole('button',{name:'通知中心',exact:true}).click();await page.locator('.notification-item').waitFor();await page.locator('.n-main').click()
  if(!(await page.getByRole('dialog',{name:'读取结果详情'}).textContent()).includes(summary))throw Error('detail omitted counts')
  await page.evaluate(()=>window.fixture.announce());if(await page.locator('.toast').count())throw Error('replayed historical read')
  const geometry=await page.evaluate(()=>({viewport:innerWidth,scroll:document.documentElement.scrollWidth}));if(geometry.scroll>width||errors.length)throw Error(JSON.stringify({geometry,errors}))
  await page.screenshot({path:path.join(output,'visual-bell-'+width+'.png')});results.push({width,height,geometry,errors,oneResult:true,collapsePreservedHistory:true,noReplay:true});await page.close()
 }
 await fs.writeFile(path.join(output,'visual-result.json'),JSON.stringify({results,nativeIpc:false,fixture:'real ToastHost/NotificationItem and store; memory list IPC; detail shell is fixture'},null,2)+'\n')
 console.log('visual checks passed')
}finally {if(browser)await browser.close();if(server)await server.close();await fs.rm(dir,{recursive:true,force:true})}
