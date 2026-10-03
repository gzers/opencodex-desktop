// 原型「日志历史」两分类 + 官方面板入口：真实浏览器（无头 Chromium）检查与截图。
// 运行：node "<本文件>"   （可选 PLAYWRIGHT_CORE=<playwright-core 目录>）
// 环境缺失时以退出码 2 报告 BLOCKED，不把「没跑」当成通过。
import { createRequire } from 'module';
import fs from 'fs';
import path from 'path';
import os from 'os';
import { fileURLToPath, pathToFileURL } from 'url';

// 从脚本位置向上找到仓库根（含 .git），避免写死绝对路径。
function findRepo(start){
  let d=start;
  for(let i=0;i<12;i++){
    if(fs.existsSync(path.join(d,'.git'))) return d;
    const up=path.dirname(d);
    if(up===d) break;
    d=up;
  }
  return null;
}
const here=path.dirname(fileURLToPath(import.meta.url));
const repo=findRepo(here);
if(!repo){ console.log('BLOCKED: 未找到仓库根（.git）。'); process.exit(2); }

// playwright-core 未随仓库安装（仓库不新增该依赖）；按 PLAYWRIGHT_CORE 或 npx 缓存解析。
// 解析不到 → 报告环境缺失（退出码 2），不把「没跑」当成通过。
function resolvePlaywright(){
  const candidates=[process.env.PLAYWRIGHT_CORE];
  const npx=path.join(os.homedir(),'.npm/_npx');
  if(fs.existsSync(npx)){
    for(const d of fs.readdirSync(npx)) candidates.push(path.join(npx,d,'node_modules/playwright-core'));
  }
  for(const c of candidates){
    if(!c) continue;
    if(fs.existsSync(path.join(c,'package.json'))) return c;
  }
  return null;
}
function newestChromiumShell(){
  const base=path.join(os.homedir(),'Library/Caches/ms-playwright');
  if(!fs.existsSync(base)) return null;
  const dirs=fs.readdirSync(base).filter(d=>d.startsWith('chromium_headless_shell-')).sort();
  for(const d of dirs.reverse()){
    const p=path.join(base,d,'chrome-headless-shell-mac-arm64/chrome-headless-shell');
    if(fs.existsSync(p)) return p;
  }
  return null;
}

const pwPath=resolvePlaywright();
const exe=newestChromiumShell();
if(!pwPath||!exe){
  console.log('BLOCKED: 环境缺失，未执行浏览器检查。');
  console.log('  playwright-core: '+(pwPath||'未找到（可设 PLAYWRIGHT_CORE）'));
  console.log('  chromium headless shell: '+(exe||'未找到'));
  process.exit(2);
}
const require=createRequire(pwPath+'/');
const { chromium }=require('playwright-core');

const protoDir=path.join(here,'..');          // 05-原型/原型
const proto=path.join(protoDir,'index.html');
const outDir=path.join(protoDir,'..','文档','截图','原型-日志两分类-20260922');
fs.mkdirSync(outDir,{recursive:true});

const url=(theme,hash='#logs')=>pathToFileURL(proto).href+`?theme=${theme}${hash}`;

const results=[];
const check=(c,m)=>results.push((c?'PASS':'FAIL')+' '+m);

const browser=await chromium.launch({executablePath:exe});
const page=await browser.newPage({viewport:{width:1692,height:916},deviceScaleFactor:2});
const consoleErrors=[];
page.on('console',m=>{ if(m.type()==='error') consoleErrors.push(m.text()); });
page.on('pageerror',e=>consoleErrors.push('pageerror: '+e.message));

async function boot(theme){
  await page.goto(url(theme));
  await page.waitForTimeout(700);
  // 2026-09-28：诊断中心默认落在「环境诊断」Tab；本用例针对日志历史，需显式切到日志 Tab。
  await page.click('.diag-tabs button[data-diag-tab="logs"]');
  await page.waitForTimeout(150);
}
async function clickDemo(sel){ await page.evaluate(s=>{const el=document.querySelector(s); if(el) el.click();},sel); await page.waitForTimeout(220); }
async function shot(name,sel){
  const el=await page.$(sel);
  if(!el){ check(false,'截图目标缺失: '+sel+' ('+name+')'); await page.screenshot({path:path.join(outDir,name),fullPage:true}); return; }
  await el.scrollIntoViewIfNeeded().catch(()=>{});
  await page.waitForTimeout(120);
  const box=await el.boundingBox();
  if(!box){ check(false,'截图目标不可见: '+sel+' ('+name+')'); await page.screenshot({path:path.join(outDir,name),fullPage:true}); return; }
  const vp=page.viewportSize();
  const x=Math.max(0,box.x-8), y=Math.max(0,box.y-8);
  const w=Math.min(vp.width-x, box.width+16), h=Math.min(vp.height-y, box.height+16);
  if(w<=1||h<=1){ check(false,'截图区域无效: '+sel+' ('+name+')'); await page.screenshot({path:path.join(outDir,name),fullPage:true}); return; }
  await page.screenshot({path:path.join(outDir,name),clip:{x,y,width:w,height:h}});
}

await boot('light');

// 结构：只有两个分类 + 面板入口
const tabs=await page.$$eval('.log-cats button',els=>els.map(e=>e.dataset.logCat));
check(tabs.length===2&&tabs[0]==='app'&&tabs[1]==='audit','分类恰为 应用日志 / 调用日志（'+tabs.join(',')+'）');
check(await page.$('.log-cats button[data-log-cat="proxy"]')===null,'无代理请求分类');
check(!!(await page.$('#logPanelEntry')),'存在官方面板入口');

// 1. 应用日志-亮
await shot('01-应用日志-亮.png','#card-diag-history');

// 2. 调用日志-悬停详情-亮（详情走浮层，不展开、不占行高）
await page.click('.log-cats button[data-log-cat="audit"]');
await page.waitForTimeout(200);
check(!!(await page.$('#logView .t-detail-hit')),'执行结果提供可悬停标识');
check(await page.$('#logView .t-err-full')===null,'不存在展开块');
check(await page.$('#logView .t-detail-hint')===null,'不存在「悬停查看」提示文字');
check(!(await page.$eval('#logView .t-detail-hit',el=>el.textContent)).includes('悬停查看'),'标识不含多余提示文字');
check(await page.$eval('#logView .t-detail-hit',el=>getComputedStyle(el).borderBottomStyle)!=='dotted','标识无虚线下划线');
const hits=await page.$$('#logView .t-detail-hit');
const rowH=sel=>page.$eval(sel,el=>Math.round(el.getBoundingClientRect().height));
const hBefore=await rowH('#logView tbody tr');
await hits[hits.length-1].hover();
await page.waitForTimeout(300);
const tipShown=await page.$eval('#logTip',el=>getComputedStyle(el).display!=='none').catch(()=>false);
check(tipShown,'悬停显示详情浮层');
const tipText=await page.$eval('#logTip',el=>el.textContent).catch(()=>'');
check(tipText.includes('query 已脱敏'),'浮层内容已脱敏');
check(tipText.includes('已保留备份'),'浮层显示完整详情');
const hAfter=await rowH('#logView tbody tr');
check(hBefore===hAfter,'悬停不改变行高（不占高度）');
const tipBox=await page.$eval('#logTip',el=>{const r=el.getBoundingClientRect();return {x:r.x,y:r.y,w:r.width,h:r.height};});
const tblBox=await page.$eval('#logView .log-table-wrap',el=>{const r=el.getBoundingClientRect();return {x:r.x,y:r.y,w:r.width,h:r.height};});
const ux=Math.max(0,Math.min(tipBox.x,tblBox.x)-8), uy=Math.max(0,Math.min(tipBox.y,tblBox.y)-8);
const uw=Math.min(1692-ux, Math.max(tipBox.x+tipBox.w,tblBox.x+tblBox.w)+8-ux);
const uh=Math.min(916-uy, Math.max(tipBox.y+tipBox.h,tblBox.y+tblBox.h)+8-uy);
await page.screenshot({path:path.join(outDir,'02-调用日志-悬停详情-亮.png'),clip:{x:ux,y:uy,width:uw,height:uh}});
await page.mouse.move(10,10);
await page.waitForTimeout(200);
check(!(await page.$eval('#logTip',el=>getComputedStyle(el).display!=='none').catch(()=>false)),'移开后浮层收起');

// 3. 调用日志-截断-暗
await boot('dark');
await page.click('.log-cats button[data-log-cat="audit"]');
await clickDemo('#logScenarios button[data-log-scenario="capped"]');
check((await page.$eval('#logView',el=>el.textContent)).includes('已截断'),'审计截断提示可见');
await shot('03-调用日志-截断-暗.png','#card-diag-history');

// 4. 应用日志-超长截断-暗
await page.click('.log-cats button[data-log-cat="app"]');
await clickDemo('#logScenarios button[data-log-scenario="longtext"]');
check((await page.$eval('#logView',el=>el.textContent)).includes('已截断 8 KiB'),'超长单行截断标记可见');
await shot('04-应用日志-超长截断-暗.png','#card-diag-history');

// 5. 缺文件 + 主动切换（亮）
await boot('light');
await page.click('.log-cats button[data-log-cat="audit"]');
await clickDemo('#logScenarios button[data-log-scenario="missing"]');
const missingTxt=await page.$eval('#logView',el=>el.textContent);
check(missingTxt.includes('尚未生成调用日志文件'),'调用日志缺文件文案');
check(!!(await page.$('#logView [data-log-act="view-app"]')),'缺文件提供主动切换');
await shot('05-调用日志-缺文件-主动切换-亮.png','#card-diag-history');

// 6. 面板入口：代理未运行
await clickDemo('#logRunStates button[data-log-run="stopped"]');
await page.click('#logPanelEntry');
await page.waitForTimeout(250);
const modalTitle=await page.$eval('#modalTitle',el=>el.textContent).catch(()=>'');
const modalBody=await page.$eval('#modalBody',el=>el.textContent).catch(()=>'');
check(modalTitle.includes('官方面板'),'代理未运行弹出说明（标题）');
check(modalBody.includes('代理未运行，请先启动后查看'),'代理未运行说明文案');
check(page.url().includes('#panel')===false,'代理未运行不导航到面板');
await shot('06-面板入口-代理未运行-亮.png','#modal');
await page.evaluate(()=>document.getElementById('modalCancel').click());
await page.waitForTimeout(200);

// 7. 面板入口：加载失败
await clickDemo('#logRunStates button[data-log-run="running"]');
await clickDemo('#logPanelStates button[data-log-panel="fail"]');
await page.click('#logPanelEntry');
await page.waitForTimeout(250);
const failTitle=await page.$eval('#modalTitle',el=>el.textContent).catch(()=>'');
check(failTitle.includes('加载失败'),'面板加载失败标题如实');
check(page.url().includes('#panel')===false,'加载失败不导航');
await shot('07-面板入口-加载失败-亮.png','#modal');
await page.evaluate(()=>document.getElementById('modalCancel').click());
await page.waitForTimeout(200);

// 8. 面板入口：可用 → 导航到面板
await clickDemo('#logPanelStates button[data-log-panel="ok"]');
await page.click('#logPanelEntry');
await page.waitForTimeout(700);
check(page.url().includes('#panel'),'入口可用时导航到应用内官方面板（'+page.url().split('#')[1]+'）');
const panelVisible=await page.$eval('#panelSection',el=>getComputedStyle(el).display!=='none').catch(()=>false);
check(panelVisible,'面板路由已激活');
await shot('08-面板入口-可用-进入面板-亮.png','#panelSection');

// 9. 最小窗口 900x600
await boot('light');
const min=await browser.newPage({viewport:{width:900,height:600},deviceScaleFactor:2});
await min.goto(url('light'));
await min.waitForTimeout(700);
const overflow=await min.$eval('#diag-logs',el=>el.scrollWidth<=el.clientWidth+1);
check(overflow,'最小窗口下日志区无横向溢出');
await min.screenshot({path:path.join(outDir,'09-最小窗口-900x600.png')});
await min.close();

// 10. 原型工具区
await page.evaluate(()=>window.scrollTo(0,0));
await shot('10-原型工具区-日志分类演示.png','#logDemoCard');

await browser.close();

console.log(results.join('\n'));
const fails=results.filter(r=>r.startsWith('FAIL'));
console.log('\n== console errors ==\n'+(consoleErrors.join('\n')||'(none)'));
console.log('\nTOTAL: '+results.length+'  FAIL: '+fails.length+'  CONSOLE_ERRORS: '+consoleErrors.length);
console.log('shots -> '+outDir);
process.exit(fails.length||consoleErrors.length?1:0);
