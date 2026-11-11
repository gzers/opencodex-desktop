import {createServer} from 'vite'
import {chromium} from 'playwright-core'
import fs from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
const output=process.argv[2],ui=process.cwd(),name='.builtin-repair-preview-'+Date.now(),dir=path.join(ui,'qa',name)
await fs.mkdir(dir);let server,browser
try{
 await fs.writeFile(path.join(dir,'index.html'),'<div id="app"></div><script type="module" src="./main.ts"></script>')
 await fs.writeFile(path.join(dir,'main.ts'),"import {createApp} from 'vue';import {createPinia} from 'pinia';import App from './App.vue';import '@/styles/tokens.css';import '@/styles/base.css';import '@/styles/materials.css';import '@/styles/effects.css';createApp(App).use(createPinia()).mount('#app')")
 const component=[
 '<script setup lang="ts">',
 "import RuntimeProgramReinstall from '@/features/runtime/RuntimeProgramReinstall.vue';",
 "const fixture={resume:false,calls:[] as string[]};(window as any).fixture=fixture;",
 "(window as any).__TAURI_INTERNALS__={invoke:async(command:string,args:any)=>{fixture.calls.push(command);if(command==='runtime_builtin_repair_preview')return {version:'2.99.0',resume:fixture.resume,programRoot:'/Users/demo/Library/Application Support/OpenCodeX Desktop/program',configPath:'/Users/demo/.codex-desktop/opencodex/home/config.json',confirmationDigest:'original-candidate'};if(command==='runtime_builtin_repair'){if(args.request.confirmationDigest!=='original-candidate')throw Error('wrong review');return {version:'2.99.0',configurationPreserved:true}};if(command==='runtime_source')return null;if(command==='model_paths_v2_snapshot')throw Error('isolated path refresh unavailable');throw Error('unexpected native '+command)}};",
 '</script><template><main style="padding:28px"><h1>安装配置</h1><p>共享重装组件 · 隔离演示</p><RuntimeProgramReinstall :repair="true"/></main></template>'
 ].join('\n');await fs.writeFile(path.join(dir,'App.vue'),component)
 server=await createServer({server:{host:'127.0.0.1',port:0,strictPort:false}});await server.listen();const address=server.httpServer.address()
 const caches=path.join(os.homedir(),'Library/Caches/ms-playwright'),versions=(await fs.readdir(caches)).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()
 browser=await chromium.launch({executablePath:path.join(caches,versions[0],'chrome-headless-shell-mac-arm64/chrome-headless-shell')});const results=[]
 for(const [width,height] of [[1040,760],[580,480]]){
  const page=await browser.newPage({viewport:{width,height}}),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('http://127.0.0.1:'+address.port+'/qa/'+name+'/index.html')
  await page.getByRole('button',{name:'修复程序',exact:true}).click();await page.getByRole('button',{name:'确认修复程序',exact:true}).waitFor()
  await page.screenshot({path:path.join(output,'visual-preview-'+width+'.png')});await page.getByRole('button',{name:'取消',exact:true}).click()
  if((await page.evaluate(()=>window.fixture.calls)).includes('runtime_builtin_repair'))throw Error('cancel mutated')
  await page.evaluate(()=>window.fixture.resume=true);await page.getByRole('button',{name:'修复程序',exact:true}).click();await page.getByRole('button',{name:'确认继续原修复',exact:true}).click()
  await page.getByText('OpenCodex 2.99.0 已修复，配置和备份已保留。',{exact:true}).waitFor();await page.screenshot({path:path.join(output,'visual-result-'+width+'.png')})
  const calls=await page.evaluate(()=>window.fixture.calls);if(calls.filter(c=>c==='runtime_builtin_repair').length!==1)throw Error('duplicate install')
  const geometry=await page.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth}));if(errors.length||geometry.scroll>width)throw Error(JSON.stringify({errors,geometry}))
  results.push({width,height,geometry,errors,cancelDidNotInstall:true,oneExplicitResume:true,refreshFailureDoesNotEraseCompletion:true});await page.close()
 }
 await fs.writeFile(path.join(output,'visual-result.json'),JSON.stringify({results,nativeIpc:false,fixture:'actual RuntimeProgramReinstall/UiDialog and store, memory IPC; path refresh intentionally fails'},null,2)+'\n');console.log('visual passed')
}finally{if(browser)await browser.close();if(server)await server.close();await fs.rm(dir,{recursive:true,force:true})}
