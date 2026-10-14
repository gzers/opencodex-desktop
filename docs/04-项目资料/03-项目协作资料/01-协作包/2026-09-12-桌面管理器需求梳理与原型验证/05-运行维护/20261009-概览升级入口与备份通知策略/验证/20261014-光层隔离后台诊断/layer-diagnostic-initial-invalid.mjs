import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
const ui='/Users/ezio/.codex/worktrees/ocx-0110-main/apps/desktop/ui';
const out='/private/tmp/ocx-0110-perf-review-20261014/headless';
const core='/Users/ezio/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core';
const executable='/Users/ezio/Library/Caches/ms-playwright/chromium_headless_shell-1248/chrome-headless-shell-mac-arm64/chrome-headless-shell';
const require=createRequire(path.join(core,'package.json'));
const { chromium }=require('playwright-core');
const { createServer }=await import(pathToFileURL(path.join(ui,'node_modules/vite/dist/node/index.js')).href);
const server=await createServer({root:ui,configFile:path.join(ui,'vite.config.ts'),server:{host:'127.0.0.1',port:0,strictPort:false}});
await mkdir(out,{recursive:true});
const cases=[];let browser;
try {
 await server.listen();
 const port=server.httpServer.address().port;
 browser=await chromium.launch({executablePath:executable,headless:true});
 for(const mode of ['setup','ready']) {
  for(const render of ['mesh','css']) {
   const name=mode+'-'+render;
   const page=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2});
   const errors=[];const blocked=[];page.on('pageerror',e=>errors.push(String(e)));
   await page.route('**/*',async route=>{
     const u=new URL(route.request().url());
     if(u.hostname==='127.0.0.1'&&u.port===String(port))return route.continue();
     blocked.push(u.origin);await route.abort();
   });
   await page.addInitScript(()=>{
     const pending=new Map();let id=0;let now=1000;
     window.requestAnimationFrame=callback=>{pending.set(++id,callback);return id};
     window.cancelAnimationFrame=id=>pending.delete(id);
     window.__stepDiagnosticFrames=count=>{for(let i=0;i<count;i++){now+=16;const q=[...pending.values()];pending.clear();q.forEach(f=>f(now))}return {pending:pending.size,now}};
   });
   const url=`http://127.0.0.1:${port}/?__audit=1&route=overview&env=${mode==='setup'?'missing_ocx':'ready'}&runtime=${mode==='setup'?'not_found':'running'}&prefs=local&effects=high&theme=light`;
   await page.goto(url);
   await page.waitForSelector('html[data-audit-harness=installed]');
   await page.waitForSelector('.motion-hero svg');
   await page.evaluate(render=>{
     const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;
     p._s.get('theme').apply('light',{persistCache:false});
     p._s.get('effects').setSetting('high');p._s.get('effects').setVisible(true);p._s.get('effects').setForeground(true);
     p._s.get('glowRender').setSetting(render);
   },render);
   await page.waitForTimeout(100);
   const time=await page.evaluate(()=>window.__stepDiagnosticFrames(120));
   const read=()=>page.evaluate(()=>{
     const selectors=['.main','.motion-overview','.motion-stage','.motion-mark','.motion-backdrop','.motion-mesh-host','.motion-ambient','.motion-hero svg'];
     const nodes=Object.fromEntries(selectors.map(selector=>{const element=document.querySelector(selector);const style=getComputedStyle(element);return [selector,{rect:element.getBoundingClientRect().toJSON(),position:style.position,isolation:style.isolation,display:style.display,zIndex:style.zIndex,overflow:style.overflow,offsetParent:element.offsetParent?.className??null}]}));
     return {nodes,attributes:{theme:document.documentElement.dataset.theme,effects:document.documentElement.dataset.effects,glow:document.documentElement.dataset.glowRender,mode:document.querySelector('.ov').dataset.mode,size:document.documentElement.dataset.overviewSize},svg:document.querySelector('.motion-hero').innerHTML,canvasSize:[document.querySelector('.motion-mesh')?.width??null,document.querySelector('.motion-mesh')?.height??null],backdropParent:document.querySelector('.motion-backdrop').parentElement.className};
   });
   const before=await read();
   assert.equal(before.attributes.mode,mode);assert.equal(before.attributes.glow,render);assert.equal(before.attributes.effects,'high');
   await page.screenshot({path:path.join(out,name+'-current.png')});
   await page.addStyleTag({content:'.motion-overview { isolation: isolate; }'});
   const after=await read();
   await page.screenshot({path:path.join(out,name+'-isolated.png')});
   for(const selector of Object.keys(before.nodes))assert.deepEqual(after.nodes[selector].rect,before.nodes[selector].rect,selector+' geometry changed');
   assert.equal(after.svg,before.svg,'animation frame changed during static CSS comparison');assert.deepEqual(after.canvasSize,before.canvasSize);
   cases.push({name,time,before,after,errors,blockedRequests:blocked,scope:'headless Chromium, audit fixture, synthetic rAF, no native performance or real runtime state'});
   console.log(name+' geometry equal; restoration experiment only');
   await page.close();
  }
 }
 await writeFile(path.join(out,'results.json'),JSON.stringify({browserVersion:browser.version(),sourceSha:'d3b83b3822bf10a511556d80dc851c2ccfb17cf5',productionFilesEdited:false,cases,rootCauseProven:false,nativePerformanceAccepted:false},null,2)+'
');
} catch(error) {
 await writeFile(path.join(out,'results-failed.json'),JSON.stringify({error:String(error),cases,nativePerformanceAccepted:false},null,2)+'
');throw error;
} finally {await browser?.close();await server.close()}
