import {sharedRoot,sharedPrototype} from './prototype-location.mjs';
// 原型「卡片内块间距」真实浏览器（无头 Chromium）检查。
// 运行：node "<本文件>"   （可选 PLAYWRIGHT_CORE=<playwright-core 目录>）
// 覆盖 2026-09-24 用户要求：卡片里的按钮行不能贴在上一块（尤其是输入框）上，必须有上间距。
// 环境缺失以退出码 2 报 BLOCKED，不把「没跑」当成通过。
import { createRequire } from 'module';
import fs from 'fs';
import path from 'path';
import os from 'os';
import { fileURLToPath, pathToFileURL } from 'url';

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
if(!findRepo(here)){ console.log('BLOCKED: 未找到仓库根（.git）。'); process.exit(2); }

function resolvePlaywright(){
  const candidates=[process.env.PLAYWRIGHT_CORE];
  const npx=path.join(os.homedir(),'.npm/_npx');
  if(fs.existsSync(npx)) for(const d of fs.readdirSync(npx)) candidates.push(path.join(npx,d,'node_modules/playwright-core'));
  for(const c of candidates){ if(c && fs.existsSync(path.join(c,'package.json'))) return c; }
  return null;
}
function newestChromiumShell(){
  const base=path.join(os.homedir(),'Library/Caches/ms-playwright');
  if(!fs.existsSync(base)) return null;
  for(const d of fs.readdirSync(base).filter(d=>d.startsWith('chromium_headless_shell-')).sort().reverse()){
    const p=path.join(base,d,'chrome-headless-shell-mac-arm64/chrome-headless-shell');
    if(fs.existsSync(p)) return p;
  }
  return null;
}
const pwPath=resolvePlaywright(), exe=newestChromiumShell();
if(!pwPath||!exe){
  console.log('BLOCKED: 环境缺失，未执行浏览器检查。');
  console.log('  playwright-core: '+(pwPath||'未找到（可设 PLAYWRIGHT_CORE）'));
  console.log('  chromium headless shell: '+(exe||'未找到'));
  process.exit(2);
}
const require=createRequire(pwPath+'/');
const { chromium }=require('playwright-core');

const proto=sharedPrototype;
const url=hash=>pathToFileURL(proto).href+'?theme=light'+hash;
const results=[];
const check=(c,m)=>results.push((c?'PASS':'FAIL')+' '+m);

// 规则：卡片里「上一个可见块」→「按钮行」的垂直间距 ≥ 12px（设计值 14px，留 2px 渲染容差）。
const MIN_GAP=12;
const collect=`()=>{
  const out=[];
  for(const card of document.querySelectorAll('.card')){
    const kids=[...card.children].filter(el=>el.offsetParent!==null);
    for(let i=1;i<kids.length;i++){
      const cur=kids[i], prev=kids[i-1];
      if(!cur.matches('.controls')&&!cur.querySelector('.btn')) continue;
      if(!prev.querySelector('input,select,textarea,.select')&&!prev.matches('.field-grid,.setting-list,.path-action-row')) continue;
      const a=prev.getBoundingClientRect(), c=cur.getBoundingClientRect();
      out.push({id:card.id||card.querySelector('h2,h3')?.textContent?.trim()||'(card)', label:prev.className,
                gap:Math.round(c.top-a.bottom), text:(cur.textContent||'').trim().replace(/\\s+/g,' ').slice(0,30)});
    }
  }
  return out;
}`;

const browser=await chromium.launch({executablePath:exe});
const page=await browser.newPage({viewport:{width:1692,height:916},deviceScaleFactor:2});
const consoleErrors=[];
page.on('console',m=>{ if(m.type()==='error') consoleErrors.push(m.text()); });
page.on('pageerror',e=>consoleErrors.push('pageerror: '+e.message));

const ROUTES=['#settings?section=installation','#settings?section=migration','#settings?section=sync','#settings?section=cleanup','#settings?section=extensions','#settings?section=cli','#settings?section=upgrade','#logs'];
for(const route of ROUTES){
  await page.goto('about:blank');
  await page.goto(url(route));
  await page.waitForTimeout(700);
  const rows=await page.evaluate(eval('('+collect+')'));
  for(const r of rows){
    check(r.gap>=MIN_GAP, route+' · '+r.id+'：「'+r.text+'」与上一块 .'+String(r.label).split(' ').join('.')+' 的上间距 '+r.gap+'px（要求 ≥'+MIN_GAP+'px）');
  }
  check(true, route+'：检查完成（候选 '+(rows.length)+' 处）');
}
await browser.close();

console.log(results.join('\n'));
console.log('\n== console errors ==');
console.log(consoleErrors.length?consoleErrors.join('\n'):'(none)');
const fails=results.filter(r=>r.startsWith('FAIL')).length;
console.log(`\nTOTAL: ${results.length}  FAIL: ${fails}  CONSOLE_ERRORS: ${consoleErrors.length}`);
process.exit(fails||consoleErrors.length?1:0);
