import {createServer} from 'vite'
import {chromium} from 'playwright-core'
import fs from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
const output=process.argv[2],ui=process.cwd(),name='.network-notes-preview-'+Date.now(),dir=path.join(ui,'qa',name)
await fs.mkdir(dir);let server,browser
try{
 await fs.writeFile(path.join(dir,'index.html'),'<div id="app"></div><script type="module" src="./main.ts"></script>')
 await fs.writeFile(path.join(dir,'main.ts'),"import {createApp} from 'vue';import {createPinia} from 'pinia';import App from './App.vue';import '@/styles/tokens.css';import '@/styles/base.css';import '@/styles/materials.css';import '@/styles/effects.css';createApp(App).use(createPinia()).mount('#app')")
 const component=[
 '<script setup lang="ts">',
 "import NetworkProbe from '@/features/preferences/NetworkProbe.vue';import ReleaseNotes from '@/features/updates/ReleaseNotes.vue';",
 "const fixture={calls:[] as string[],retry:false};(window as any).fixture=fixture;",
 "(window as any).__TAURI_INTERNALS__={invoke:async(command:string,args:any)=>{fixture.calls.push(command);if(command==='check_network_proxy')return {ok:false,detail:'部分连接未通过',results:[{target:'github',state:'rate_limited',httpStatus:429,retryAfterSeconds:12,checkedAt:'2026-11-11T10:00:00Z'},{target:'npm',state:'ready',httpStatus:200,retryAfterSeconds:null,checkedAt:'2026-11-11T10:00:00Z'}]};if(command==='update_release_details')return {target:'official',channel:'official',candidateVersion:'2.99.0',notesState:'ready',notes:'该版本的官方发布说明。本页保留原有的版本与来源，当前请求未成功时可查看上次说明。',notesSource:'github_release',releaseUrl:'https://github.com/lidge-jun/opencodex/releases/tag/v2.99.0',publishedAt:'2026-11-10T00:00:00Z',fetchedAt:1794268800,checkedAt:1794355200,cached:!args.force,failure:args.force?null:{kind:'timeout',httpStatus:null}};throw Error('unexpected native '+command)}};",
 '</script><template><main style="padding:24px;max-width:860px;margin:auto"><h1>网络与更新说明</h1><section class="card" style="padding:20px;margin-bottom:20px"><h2>网络</h2><NetworkProbe policy-key="fixture-saved-policy"/></section><section class="card" style="padding:20px"><ReleaseNotes target="official" channel="official" version="2.99.0"/></section></main></template>'
 ].join('\n');await fs.writeFile(path.join(dir,'App.vue'),component)
 server=await createServer({server:{host:'127.0.0.1',port:0,strictPort:false}});await server.listen();const address=server.httpServer.address()
 const caches=path.join(os.homedir(),'Library/Caches/ms-playwright'),versions=(await fs.readdir(caches)).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()
 browser=await chromium.launch({executablePath:path.join(caches,versions[0],'chrome-headless-shell-mac-arm64/chrome-headless-shell')});const results=[]
 for(const [width,height] of [[1040,760],[580,640]]){
  const page=await browser.newPage({viewport:{width,height}}),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('http://127.0.0.1:'+address.port+'/qa/'+name+'/index.html')
  await page.getByRole('button',{name:'检查连接',exact:true}).click();await page.getByLabel('网络连接检查结果').waitFor();await page.getByTestId('notes-cache-provenance').waitFor()
  await page.screenshot({path:path.join(output,'visual-failure-cache-'+width+'.png')})
  await page.getByRole('button',{name:'重试说明',exact:true}).click();await page.waitForFunction(()=>!document.querySelector('[data-testid="notes-cache-provenance"]'))
  if(!(await page.locator('pre').textContent()).includes('官方发布说明'))throw Error('retry lost notes')
  const calls=await page.evaluate(()=>window.fixture.calls);if(calls.some(c=>!['check_network_proxy','update_release_details'].includes(c)))throw Error('unexpected product mutation')
  const geometry=await page.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth}));if(errors.length||geometry.scroll>width)throw Error(JSON.stringify({errors,geometry}))
  await page.screenshot({path:path.join(output,'visual-retry-'+width+'.png')});results.push({width,height,geometry,errors,separateDestinations:true,cachedVersionAndTime:true,notesOnlyRetry:true,calls});await page.close()
 }
 await fs.writeFile(path.join(output,'visual-result.json'),JSON.stringify({results,nativeIpc:false,fixture:'actual NetworkProbe/ReleaseNotes/UiButton; fixed memory IPC'},null,2)+'\n');console.log('visual passed')
}finally{if(browser)await browser.close();if(server)await server.close();await fs.rm(dir,{recursive:true,force:true})}
