// Real built ESM bootstrap regression. Explicit synthetic IPC only; no real HOME,
// service, upstream endpoint, configuration or API key is accessed.
import {chromium} from 'playwright-core'
import {createServer} from 'node:http'
import {createHash} from 'node:crypto'
import {readFile,mkdir,writeFile,stat} from 'node:fs/promises'
import {resolve,extname,sep,dirname} from 'node:path'
import {fileURLToPath} from 'node:url'
import assert from 'node:assert/strict'
const dist=resolve(process.env.OCXD_PRODUCTION_BOOT_DIST??fileURLToPath(new URL('../dist/',import.meta.url)))
const output=resolve(process.env.OCXD_PRODUCTION_BOOT_EVIDENCE??fileURLToPath(new URL('../../../../test/out/production-ui-boot-'+Date.now()+'-'+process.pid,import.meta.url)))
await mkdir(dirname(output),{recursive:true});await mkdir(output)
const source={commit:process.env.GITHUB_SHA??null,distEntrySha256:createHash('sha256').update(await readFile(resolve(dist,'index.html'))).digest('hex')}
const types={'.html':'text/html','.js':'text/javascript','.css':'text/css','.svg':'image/svg+xml','.png':'image/png','.woff2':'font/woff2'}
const server=createServer(async(req,res)=>{try{const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);const path=resolve(dist,'.'+(pathname==='/'?'/index.html':pathname));assert(path.startsWith(dist+sep));assert((await stat(path)).isFile());res.writeHead(200,{'content-type':types[extname(path)]??'application/octet-stream','cache-control':'no-store'});res.end(await readFile(path))}catch{res.writeHead(404);res.end()}})
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve))
let browser;const errors=[],blocked=[],checks=[]
try{
 browser=await chromium.launch(process.env.OCXD_PRODUCTION_BOOT_BROWSER?{executablePath:process.env.OCXD_PRODUCTION_BOOT_BROWSER}:undefined)
 const page=await browser.newPage({viewport:{width:1180,height:760},reducedMotion:'reduce'})
 page.on('pageerror',error=>errors.push(error.stack??error.message))
 const base='http://127.0.0.1:'+server.address().port
 await page.route('**/*',route=>{if(new URL(route.request().url()).origin===base)return route.continue();blocked.push(route.request().url());return route.abort()})
await page.addInitScript(()=>{
 let saved={mode:'auto',port:null,revision:'r1',reconciliationRequired:false};const snapshot={matrix:{runtime:'stopped',connection:'unconfigured',operation:'idle'},facts:{health:'unknown',runtime_label:null,opencodex_home:'/Users/demo/.codex-desktop/opencodex/home',data_root:'/Users/demo/.codex-desktop/desktop',protection:null,reboot_safe:null,service_present:null,shim_present:null,version_drift:null},port:null,pid:null,can_start:true,can_stop:false,can_restart:false,source:'fixture'};
 const config={aggregateRoot:'/Users/demo/.codex-desktop',activeDataRoot:'/Users/demo/.codex-desktop/desktop',currentDataRoot:'/Users/demo/.codex-desktop/desktop',opencodexHomeMode:'inside',opencodexHome:'/Users/demo/.codex-desktop/opencodex/home',currentOpencodexHome:'/Users/demo/.codex-desktop/opencodex/home',runtimeActive:true};
 const path=(path)=>({path,status:'valid'});const I='/Users/demo/Applications/OpenCodeX Desktop';const paths={schemaVersion:1,aggregateRoot:path(config.aggregateRoot),desktopRoot:path(config.activeDataRoot),opencodexHome:path(config.opencodexHome),configPath:path(config.opencodexHome+'/config.json'),installationRoot:path(I),runtimePrefix:path(I+'/runtime/opencodex'),runtimeEntry:path(I+'/runtime/bin/ocx'),managerExecutable:path(I+'/OpenCodeX Desktop.app'),program:{packageVersion:'2.97.0-desktop023.names.2',manifestVersion:'2.97.0-desktop023.names.2',sourceVersion:'2.97.0-desktop023.names.2',consistency:'consistent'},currentAggregateRoot:config.aggregateRoot,pendingRestart:false};
 window.__portCalls=[];window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},convertFileSrc:p=>p,invoke:async(cmd,args)=>{
 window.__portCalls.push(cmd);
 if(cmd==='get_status_snapshot')return structuredClone(snapshot);if(cmd==='model_paths_v2_snapshot')return structuredClone(paths);if(cmd==='get_data_root_config')return {...config};if(cmd==='runtime_protection_status')return{state:'none',version:null,backupId:null};
 if(cmd==='runtime_port_load')return{...saved};if(cmd==='runtime_port_save'){saved={mode:args.request.mode,port:args.request.port??55581,revision:'r2',reconciliationRequired:false};return{...saved}};
 if(cmd==='process_action'){snapshot.matrix.runtime=args.request.action==='stop'?'stopped':'running';snapshot.port=saved.port;snapshot.pid='42';snapshot.can_start=false;snapshot.can_stop=true;return{lifecycleState:'running',result:'started',canStart:false,canStop:true,canRestart:true}};
 if(cmd==='get_preferences')return{schemaVersion:1,theme:'light',interfaceScale:100,launchMain:true,autoPanel:false,visualEffects:'low',glowRender:'css',appUpdateAutoCheck:false,panelMode:'embedded'};
 if(cmd==='discover_environment')return{gate:'ready',node:{found:true,path:'/synthetic/node',version:null},npm:{found:true,path:'/synthetic/npm',version:null},ocx:{found:true,path:'/synthetic/ocx',version:null},shortCircuited:false,brewFound:true};
 if(cmd==='runtime_source')return{kind:'managed',path:I+'/runtime/bin/ocx',version:'2.97.0-desktop023.names.2',managedPrefix:I+'/runtime/opencodex',defaultPrefix:I+'/runtime/opencodex',managedEntry:I+'/runtime/bin/ocx',explicitPath:null,history:[],insideDataRoot:false,resolvedAt:null};
 if(cmd==='get_data_root_partitions')return[];if(cmd==='get_notifications')return[];
 if(cmd.startsWith('plugin:'))return 1;
 return null;
 }};
});
 await page.goto(base)
 await page.locator('.side').waitFor({timeout:20000})
 assert.equal(errors.length,0,JSON.stringify(errors));checks.push('built production entry mounted without ESM initialization error')
 await page.locator('.side').getByRole('button',{name:'设置',exact:true}).click()
 await page.getByRole('button',{name:'安装配置',exact:true}).click()
 await page.locator('[data-testid=runtime-port-settings]').waitFor({timeout:20000})
 assert.equal(await page.locator('.settings-panel.active>.card').first().getAttribute('data-testid'),'runtime-port-settings');checks.push('production installation page service card is first')
 await page.getByRole('button',{name:'检查并保存',exact:true}).click()
 await page.waitForFunction(()=>document.querySelector('.port-summary')?.textContent.includes('55581'))
 checks.push('production radio component and synthetic IPC save result render')
 await page.locator('[data-testid=settings-paths-v2]').waitFor()
 assert.equal(await page.locator('[data-testid=paths-select]').count(),0)
 checks.push('production directory consumer renders without removed selection entry')
 assert.equal(errors.length,0,JSON.stringify(errors));assert.equal(blocked.length,0,'production boot tried an external request')
 await page.screenshot({path:resolve(output,'production-boot.png')})
 await writeFile(resolve(output,'result.json'),JSON.stringify({pass:true,source,checks,errors,blocked,adapter:'actual dist ESM and production Vue, synthetic IPC',nativeAccepted:false},null,2))
 console.log(JSON.stringify({pass:true,checks:checks.length}))
}catch(error){await writeFile(resolve(output,'result.json'),JSON.stringify({pass:false,source,checks,errors,blocked,firstError:String(error),nativeAccepted:false},null,2));throw error}
finally{await browser?.close();await new Promise(resolve=>server.close(resolve))}
