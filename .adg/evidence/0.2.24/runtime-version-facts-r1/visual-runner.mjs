import {createServer} from 'vite'
import {chromium} from 'playwright-core'
import fs from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
const output=process.argv[2],ui=process.cwd(),name='.version-facts-preview-'+Date.now(),dir=path.join(ui,'qa',name)
await fs.mkdir(dir);let server,browser
try{
 await fs.writeFile(path.join(dir,'index.html'),'<div id="app"></div><script type="module" src="./main.ts"></script>')
 await fs.writeFile(path.join(dir,'main.ts'),"import {createApp} from 'vue';import {createPinia} from 'pinia';import App from './App.vue';import '@/styles/tokens.css';import '@/styles/base.css';import '@/styles/materials.css';import '@/styles/effects.css';createApp(App).use(createPinia()).mount('#app')")
 await fs.writeFile(path.join(dir,'App.vue'),[
 '<script setup lang="ts">',"import RuntimeVersionFacts from '@/features/updates/RuntimeVersionFacts.vue';import {useUpdatesStore} from '@/features/updates/store';",
 "const fixture={fail:false,calls:[] as string[]};(window as any).fixture=fixture;(window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{fixture.calls.push(command);if(command==='runtime_version_facts'){if(fixture.fail)throw Error('fixture observation unavailable');return {disk:{diskVersion:'2.99.0',selectedVersion:'2.98.0',recordedVersion:'2.97.0',recordedAt:'2026-11-01T00:00:00Z',integrity:'changed',checkedAt:'2026-11-11T00:00:00Z',confirmationDigest:'original-observation'},running:{version:'2.98.0',pid:42,port:55581},runningState:'attested'}};throw Error('unexpected '+command)}};void useUpdatesStore().loadVersions();",
 '</script><template><main style="padding:24px;max-width:800px;margin:auto"><h1>OpenCodex 更新</h1><section class="card" style="padding:24px"><RuntimeVersionFacts/></section></main></template>'
 ].join('\n'))
 server=await createServer({server:{host:'127.0.0.1',port:0,strictPort:false}});await server.listen();const address=server.httpServer.address()
 const caches=path.join(os.homedir(),'Library/Caches/ms-playwright'),versions=(await fs.readdir(caches)).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()
 browser=await chromium.launch({executablePath:path.join(caches,versions[0],'chrome-headless-shell-mac-arm64/chrome-headless-shell')});const results=[]
 for(const [width,height] of [[1040,760],[580,640]]){
  const page=await browser.newPage({viewport:{width,height}}),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('http://127.0.0.1:'+address.port+'/qa/'+name+'/index.html');await page.getByText('磁盘与运行实例版本不同，当前进程尚未采用磁盘版本。',{exact:true}).waitFor();await page.screenshot({path:path.join(output,'visual-drift-'+width+'.png')})
  await page.evaluate(()=>window.fixture.fail=true);await page.getByRole('button',{name:'重新核对版本',exact:true}).click();await page.getByText('实际版本尚未核实，请重新检查。下列信息为上次观察。',{exact:true}).waitFor();await page.screenshot({path:path.join(output,'visual-unknown-'+width+'.png')})
  const calls=await page.evaluate(()=>window.fixture.calls);if(calls.some(c=>c!=='runtime_version_facts'))throw Error('unexpected mutation')
  const geometry=await page.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth}));if(errors.length||geometry.scroll>width)throw Error(JSON.stringify({errors,geometry}))
  results.push({width,height,geometry,errors,separateVersions:true,failedReadMarkedOld:true,noInstallOrProcessCall:true,calls});await page.close()
 }
 await fs.writeFile(path.join(output,'visual-result.json'),JSON.stringify({results,nativeIpc:false,fixture:'actual RuntimeVersionFacts/UiButton/store; fixed memory observation IPC'},null,2)+'\n');console.log('visual passed')
}finally{if(browser)await browser.close();if(server)await server.close();await fs.rm(dir,{recursive:true,force:true})}
