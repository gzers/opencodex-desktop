import {createServer} from 'vite'
import {chromium} from 'playwright-core'
import fs from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
const output=process.argv[2],ui=process.cwd(),name='.preparation-preview-'+Date.now(),dir=path.join(ui,'qa',name)
await fs.mkdir(dir);let server,browser
try{
 await fs.writeFile(path.join(dir,'index.html'),'<div id="app"></div><script type="module" src="./main.ts"></script>')
 await fs.writeFile(path.join(dir,'main.ts'),"import {createApp} from 'vue';import {createPinia} from 'pinia';import App from './App.vue';import '@/styles/tokens.css';import '@/styles/base.css';import '@/styles/materials.css';import '@/styles/effects.css';createApp(App).use(createPinia()).mount('#app')")
 await fs.writeFile(path.join(dir,'App.vue'),[
 '<script setup lang="ts">',"import IntegratedPreparation from '@/features/environment/components/IntegratedPreparation.vue';",
 "const fixture={phase:'extracting',revision:1,attempt:'original-attempt',calls:[] as string[]};(window as any).fixture=fixture;const state=()=>({attemptId:fixture.attempt,revision:fixture.revision,phase:fixture.phase,failedPhase:fixture.phase==='failed'?'verifying':null,failure:fixture.phase==='failed'?'integrity':null,retryable:fixture.phase==='failed'});",
 "(window as any).__TAURI_INTERNALS__={invoke:async(command:string,args:any)=>{fixture.calls.push(command);if(command==='runtime_preparation_status')return state();if(command==='runtime_preparation_retry'){if(args.request.attemptId!=='original-attempt')throw Error('wrong attempt');fixture.phase='assembling';fixture.attempt='new-attempt';fixture.revision++;return state()};if(command==='discover_environment'){const found={found:true,path:'/fixture',version:'2.99.0'};return {node:found,npm:found,ocx:found,gate:'ready',shortCircuited:false,brewFound:false}};throw Error('unexpected '+command)}};",
 '</script><template><main style="padding:24px;max-width:800px;margin:auto"><h1>概览</h1><section class="card" style="padding:24px"><IntegratedPreparation/></section></main></template>'
 ].join('\n'))
 server=await createServer({server:{host:'127.0.0.1',port:0,strictPort:false}});await server.listen();const address=server.httpServer.address()
 const caches=path.join(os.homedir(),'Library/Caches/ms-playwright'),versions=(await fs.readdir(caches)).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()
 browser=await chromium.launch({executablePath:path.join(caches,versions[0],'chrome-headless-shell-mac-arm64/chrome-headless-shell')});const results=[]
 for(const [width,height] of [[1040,760],[580,640]]){
  const page=await browser.newPage({viewport:{width,height}}),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('http://127.0.0.1:'+address.port+'/qa/'+name+'/index.html');await page.getByTestId('integrated-preparation').waitFor();await page.screenshot({path:path.join(output,'visual-extract-'+width+'.png')})
  await page.evaluate(()=>{window.fixture.phase='failed';window.fixture.revision++});await page.getByRole('button',{name:'重新准备内置环境',exact:true}).waitFor();await page.screenshot({path:path.join(output,'visual-failed-'+width+'.png')})
  await page.getByRole('button',{name:'重新准备内置环境',exact:true}).click();await page.evaluate(()=>{window.fixture.phase='supplied';window.fixture.revision++});await page.getByText('内置依赖检查通过；服务运行状态单独显示。',{exact:true}).waitFor();await page.screenshot({path:path.join(output,'visual-supplied-'+width+'.png')})
  const calls=await page.evaluate(()=>window.fixture.calls);if(calls.filter(c=>c==='runtime_preparation_retry').length!==1||calls.some(c=>!['runtime_preparation_status','runtime_preparation_retry','discover_environment'].includes(c)))throw Error('wrong call scope')
  const geometry=await page.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth}));if(errors.length||geometry.scroll>width)throw Error(JSON.stringify({errors,geometry}))
  results.push({width,height,geometry,errors,realPhasePresentation:true,oneExplicitRetry:true,noServiceAction:true,calls});await page.close()
 }
 await fs.writeFile(path.join(output,'visual-result.json'),JSON.stringify({results,nativeIpc:false,fixture:'actual IntegratedPreparation/UiButton/store; fixed memory stage and discovery IPC'},null,2)+'\n');console.log('visual passed')
}finally{if(browser)await browser.close();if(server)await server.close();await fs.rm(dir,{recursive:true,force:true})}
