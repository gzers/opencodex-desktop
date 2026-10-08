// 静态 mock QA；使用已有 Playwright/Chromium，不安装依赖、不调用原生接口。
import {createRequire} from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath,pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
const here=path.dirname(fileURLToPath(import.meta.url));
let root=here;while(!fs.existsSync(path.join(root,'.git'))){const up=path.dirname(root);if(up===root)throw Error('找不到仓库根');root=up;}
const npx=path.join(os.homedir(),'.npm/_npx'),cache=path.join(os.homedir(),'Library/Caches/ms-playwright');
const pw=[process.env.PLAYWRIGHT_CORE,...fs.readdirSync(npx).map(d=>path.join(npx,d,'node_modules/playwright-core'))].find(p=>p&&fs.existsSync(path.join(p,'package.json')));
const exe=[process.env.CHROMIUM_EXECUTABLE,...fs.readdirSync(cache).filter(d=>d.startsWith('chromium_headless_shell-')).sort().reverse().map(d=>path.join(cache,d,'chrome-headless-shell-mac-arm64/chrome-headless-shell'))].find(p=>p&&fs.existsSync(p));
if(!pw||!exe)throw Error('需要已有 Playwright/Chromium；不自动安装');
const {chromium}=createRequire(pw+'/')('playwright-core');
const browser=await chromium.launch({executablePath:exe});
const page=await browser.newPage({viewport:{width:1692,height:916}});
const errors=[],results=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});page.setDefaultTimeout(5000);
const entry=pathToFileURL(process.env.PROTOTYPE_ENTRY||path.join(root,'docs/04-项目资料/03-项目协作资料/03-共享原型/原型/index.html')).href;
const evidence=process.env.EVIDENCE_DIR||path.join(root,'.adg/evidence/MNT-OPENCODEX-DESKTOP-20261008-01/replay');fs.mkdirSync(evidence,{recursive:true});
const shots=process.env.SHOTS_DIR||path.join(evidence,'截图');fs.mkdirSync(shots,{recursive:true});
const choose=(key,value)=>page.locator('[data-win-key="'+key+'"][data-win-value="'+value+'"]').click();
const act=name=>page.locator('.windows-caption [data-win-action="'+name+'"]').click();
const state=()=>page.evaluate(()=>windowsPrototype.state());
const shot=async name=>{await page.locator('.prototype-side').evaluate(e=>e.scrollTop=0);await page.waitForTimeout(200);await page.screenshot({path:path.join(shots,name+'.png')});};
async function check(name,fn){try{await fn();results.push({name,status:'PASS'});}catch(e){results.push({name,status:'FAIL',error:e.message});}console.log(results.at(-1).status+' '+name);}
try{
 await page.goto(entry+'?theme=light#overview');await page.waitForTimeout(900);
 await check('常规入口保持红绿灯，Windows 控件不外泄',async()=>{assert(await page.locator('.traffic').isVisible());assert(!await page.locator('.windows-caption').isVisible());});
 await page.locator('[data-launch="windows"]').click();
 await check('右侧专题入口复用主内容且无原生菜单行',async()=>{assert(await page.locator('#windowsDemoCard').isVisible());assert(!await page.locator('#protoStdCards').isVisible());assert(!await page.locator('.windows-legacy-menu').isVisible());assert(!await page.locator('.traffic').isVisible());assert.equal(await page.locator('.windows-buttons button').count(),3);assert(page.url().includes('topic=windows'));});
 await check('Win11 云母示意与 Win10、透明关闭实色回退',async()=>{assert.equal(await page.locator('body').getAttribute('data-win-material'),'mica');await choose('platform','win10');assert.equal(await page.locator('body').getAttribute('data-win-material'),'solid');await choose('platform','win11');await choose('transparency','off');assert.equal(await page.locator('body').getAttribute('data-win-material'),'solid');await choose('transparency','on');});
 await check('深色主题与非活动回退',async()=>{await choose('theme','dark');assert.equal(await page.locator('html').getAttribute('data-theme'),'dark');await choose('focus','inactive');assert.equal(await page.locator('body').getAttribute('data-win-material'),'solid');await choose('focus','active');assert.equal(await page.locator('body').getAttribute('data-win-material'),'mica');await shot('02-Windows11深色');await choose('theme','light');});
 await check('现有双栏对照与单行布局',async()=>{await choose('layout','legacy');assert(await page.locator('.windows-legacy-menu').isVisible());assert.equal(await page.locator('.titlebar').evaluate(e=>e.clientHeight),72);await shot('03-现有双栏对照');await choose('layout','integrated');assert.equal(await page.locator('.titlebar').evaluate(e=>e.clientHeight),40);});
 await check('最大化、还原按钮与标题双击',async()=>{await act('maximize');assert((await state()).maximized);assert.equal(await page.locator('[data-win-action="maximize"]').getAttribute('aria-label'),'还原');await act('maximize');assert(!(await state()).maximized);await page.locator('.windows-drag').dblclick();assert((await state()).maximized);await page.locator('.windows-drag').dblclick();assert(!(await state()).maximized);});
 await check('最小化与重新打开保留代理 mock',async()=>{await page.evaluate(()=>setState('running'));await act('minimize');assert(!await page.locator('.window').isVisible());assert.equal((await state()).visibility,'minimized');await page.locator('.windows-hidden button').click();assert(await page.locator('.window').isVisible());assert.equal(await page.evaluate(()=>currentState),'running');});
 await check('关闭留托盘与键盘焦点恢复，代理 mock 保留',async()=>{await page.locator('[data-win-action="close"]').focus();await page.keyboard.press('Enter');assert.equal((await state()).visibility,'tray');assert.equal(await page.evaluate(()=>currentState),'running');assert(await page.locator('.windows-hidden button').evaluate(e=>e===document.activeElement));await shot('04-关闭到托盘');await page.locator('.windows-hidden button').click();});
 await check('关闭偏好关闭时退出，再打开停止态',async()=>{await choose('close','exit');await act('close');assert.equal((await state()).visibility,'exited');assert.equal(await page.evaluate(()=>currentState),'stopped');await page.locator('.windows-hidden button').click();assert.equal(await page.evaluate(()=>currentState),'stopped');await choose('close','tray');});
 await check('产品导航保留 Windows 顶栏与专题入口选中',async()=>{await page.locator('.nav [data-route="settings"]').click();await page.waitForTimeout(40);assert(await page.locator('.windows-caption').isVisible());assert(await page.locator('[data-launch="windows"]').evaluate(e=>e.classList.contains('active')));await page.locator('.nav [data-route="overview"]').click();});
 await check('两主题、两系统、两尺寸无标题内容重叠',async()=>{for(const theme of ['light','dark'])for(const platform of ['win11','win10'])for(const size of ['1180','960']){await choose('theme',theme);await choose('platform',platform);await choose('size',size);assert(await page.locator('.windows-caption').evaluate(e=>e.scrollWidth<=e.clientWidth));const title=await page.locator('.titlebar').boundingBox(),main=await page.locator('.main').boundingBox();assert(main.y>=title.y+title.height-1);assert(await page.locator('.main').evaluate(e=>e.scrollWidth<=e.clientWidth+2));}await choose('theme','light');await choose('platform','win10');await shot('05-Windows10实色960');await choose('platform','win11');await choose('size','1180');await shot('01-Windows11单行浅色');});
 await check('返回常规原型恢复主题、红绿灯和尺寸',async()=>{await choose('theme','dark');await page.locator('#windowsDemoCard [data-win-action="leave"]').click();assert.equal(await page.locator('html').getAttribute('data-theme'),'light');assert(await page.locator('.traffic').isVisible());assert(!await page.locator('.windows-caption').isVisible());assert(!page.url().includes('topic=windows'));assert.equal(await page.locator('html').evaluate(e=>getComputedStyle(e).getPropertyValue('--win-w').trim()),'1180px');});
 await check('通知专题与常规导航仍可进入',async()=>{await page.locator('[data-launch="windows"]').click();await page.locator('[data-launch="notify"]').click();assert(await page.locator('#notifDemoCard').isVisible());assert(!await page.locator('#windowsDemoCard').isVisible());await page.locator('[data-launch="overview"]').click();assert(await page.locator('#protoStdCards').isVisible());});
 await check('深链接保留模型路由并只加载一份模型内容',async()=>{await page.goto(entry+'?topic=windows&theme=light#models');await page.waitForTimeout(900);assert(await page.locator('#mpContent').isVisible());assert(await page.locator('#windowsDemoCard').isVisible());assert.equal(await page.locator('#mpContent').count(),1);assert.equal((await state()).close,'tray');});
 await check('深色深链接与产品内主题切换同步专题',async()=>{await page.goto(entry+'?topic=windows&theme=dark#overview');await page.waitForTimeout(900);assert.equal((await state()).theme,'dark');await page.evaluate(()=>applyTheme('light'));await page.waitForTimeout(40);assert.equal((await state()).theme,'light');});
 await check('尺寸控件与真实 mock 窗口宽度一致',async()=>{for(const size of ['960','1180']){await choose('size',size);assert.equal(Math.round((await page.locator('.window').boundingBox()).width),Number(size));assert.equal(await page.locator('[data-win-key="size"][data-win-value="'+size+'"]').getAttribute('aria-pressed'),'true');}});
 await check('1280 评审视口控件可见且标题可读',async()=>{await page.setViewportSize({width:1280,height:900});await choose('size','960');assert(await page.locator('.windows-caption').evaluate(e=>e.scrollWidth<=e.clientWidth));assert(await page.locator('#windowsDemoCard').isVisible());});
 await check('浏览器无脚本或控制台错误',async()=>assert.deepEqual(errors,[]));
}finally{
 const report={testedAt:new Date().toISOString(),scope:'browser mock only; no native Mica, drag, Snap, DPI or installer verification',results,errors};
 fs.writeFileSync(path.join(evidence,'browser-results.json'),JSON.stringify(report,null,2));await browser.close();
}
if(results.some(r=>r.status==='FAIL'))process.exitCode=1;
