import {sharedRoot,sharedPrototype} from './prototype-location.mjs';
// 原型「主线＋支线＋统一报错」宽/窄窗口验证（无头 Chromium）。
// 运行：node "<本文件>"   （可选 PLAYWRIGHT_CORE=<playwright-core 目录>）
//
// 覆盖 DMD Revision 12（REQ-31 / AC-15）：概览在默认与最小窗口下——
//   · 主线标签只落在「待接入 / 未运行 / 运行中 / 正在确认」；
//   · 问题槽与主线分离且不被裁切；摘要卡不横向溢出；
//   · 当前支线可见；详情可展开、可关闭；「更多」弹出层不越出模拟窗口；
//   · 各状态截图留档。
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
const url=pathToFileURL(proto).href+'?theme=light#overview';
const shotsDir=path.join(here,'..','..','文档','截图','原型-状态收敛-20260928');
fs.mkdirSync(shotsDir,{recursive:true});

const results=[];const check=(c,m)=>results.push((c?'PASS':'FAIL')+' '+m);
const MAINLINE=['待接入','未运行','运行中','正在确认'];
const STATES=['not_found','stopped','running','loading','starting','pending','starting_failed','at_risk','external_takeover','unreachable'];
const BRANCH_STATES=['not_found','loading','starting','pending','starting_failed'];
const SIZES=[{preset:'1180x760',w:1280,h:840,name:'宽 1180×760'},{preset:'900x600',w:1000,h:680,name:'窄 900×600'}];

const browser=await chromium.launch({executablePath:exe});
const consoleErrors=[];

// 已确认的双形态概览（overview-dual.css 隐藏 .ovb-main）把「问题槽 / 支线行」并入门线下的
// 可见说明行 .motion-caption；旧 .ovb-problem/.ovb-brancher 节点随 .ovb-main 一并退役为隐藏。
// 因此：支线是否活动改用支线槽自身的 hidden 标志判断（与视觉落点无关）；
// 「问题槽未被裁切」用可见说明行是否越出卡片代理。
const probe=`()=>{
  const card=document.getElementById('card-overview-status');
  const label=document.querySelector('#card-overview-status [data-b="statetext"]');
  const branch=document.querySelector('#card-overview-status [data-b="brancher"]');
  const caption=document.querySelector('#card-overview-status .motion-caption');
  const rect=el=>el?el.getBoundingClientRect():null;
  const cardR=rect(card);
  const capR=rect(caption);
  const visible=el=>{ if(!el) return false; const r=el.getBoundingClientRect(); return r.width>0.5&&r.height>0.5&&getComputedStyle(el).visibility!=='hidden'; };
  return {
    label:label?label.textContent.trim():'',
    labelVisible:visible(label),
    caption:caption?caption.textContent.trim():'',
    captionVisible:visible(caption),
    captionClipped:capR&&cardR?(capR.left<cardR.left-1||capR.right>cardR.right+1):false,
    branchActive:branch?!branch.hidden:false,
    branchText:branch?branch.textContent.trim():'',
    cardOverflow:card.scrollWidth>card.clientWidth+2,
    docOverflow:document.documentElement.scrollWidth>document.documentElement.clientWidth+2
  };
}`;
const resetScroll=async(page)=>{
  await page.evaluate(()=>{
    document.querySelectorAll('.main,.window,.content,.route-section').forEach(el=>{el.scrollTop=0;el.scrollLeft=0;});
    if(document.scrollingElement) document.scrollingElement.scrollTop=0;
    window.scrollTo(0,0);
    document.querySelectorAll('.is-scrolling').forEach(el=>el.classList.remove('is-scrolling'));
  });
  await page.waitForTimeout(250);
};
const shotCard=async(page,file)=>{
  const clip=await page.evaluate(()=>{
    const el=document.getElementById('card-overview-status');
    const r=el.getBoundingClientRect();
    return {x:Math.max(0,r.x-6),y:Math.max(0,r.y-6),width:r.width+12,height:r.height+12};
  });
  await page.screenshot({path:file,clip});
};

for(const size of SIZES){
  const page=await browser.newPage({viewport:{width:size.w,height:size.h},deviceScaleFactor:2});
  page.on('console',m=>{ if(m.type()==='error') consoleErrors.push(m.text()); });
  page.on('pageerror',e=>consoleErrors.push('pageerror: '+e.message));
  await page.goto(url);
  await page.waitForTimeout(1000);
  await page.evaluate(()=>{ document.getElementById('protoCommon').open=true; });
  await page.click(`#sizePresets button[data-size="${size.preset}"]`).catch(()=>{});
  await page.waitForTimeout(300);

  for(const st of STATES){
    await page.click(`#scenarios button[data-state="${st}"]`).catch(()=>{});
    await page.waitForTimeout(120);
    const r=await page.evaluate(eval('('+probe+')'));
    check(MAINLINE.includes(r.label), `${size.name} ${st}：主线标签在允许集合（${r.label}）`);
    check(r.labelVisible, `${size.name} ${st}：主线标签可见`);
    check(r.captionVisible&&!r.captionClipped, `${size.name} ${st}：问题/支线说明可见且未被裁切`);
    check(!r.cardOverflow, `${size.name} ${st}：摘要卡不横向溢出`);
    check(r.branchActive===BRANCH_STATES.includes(st), `${size.name} ${st}：支线仅在操作/待办态激活（实际 ${r.branchActive}）`);
    check(!r.docOverflow, `${size.name} ${st}：页面不横向溢出`);
  }

  // 运行详情可展开/关闭：已确认的双形态概览把它从「页内折叠」移到 Logo 舞台的详情弹层
  // （页内 #card-overview-details 已被退役为 display:none）。这里改为打开该弹层取证。
  await page.click('#scenarios button[data-state="external_takeover"]').catch(()=>{});
  await page.waitForTimeout(150);
  await page.click('#card-overview-status .motion-detail-icon');
  await page.waitForTimeout(250);
  const detailOpen=await page.evaluate(()=>{
    const mask=document.getElementById('modalMask');
    const modal=document.getElementById('modal');
    const body=document.getElementById('modalBody');
    return {
      shown:mask.style.display==='flex',
      isRuntime:modal.classList.contains('motion-runtime-modal'),
      bodyVisible:body.getBoundingClientRect().height>0,
      overflow:body.scrollWidth>body.clientWidth+2,
    };
  });
  check(detailOpen.shown && detailOpen.isRuntime && detailOpen.bodyVisible, `${size.name}：运行详情可展开且正文可见`);
  check(!detailOpen.overflow, `${size.name}：运行详情正文不横向溢出`);
  await resetScroll(page);
  await shotCard(page,path.join(shotsDir,`概览-详情展开-${size.preset}.png`));
  await page.click('#motionModalClose').catch(()=>page.keyboard.press('Escape'));
  await page.waitForTimeout(200);
  check(await page.evaluate(()=>document.getElementById('modalMask').style.display==='none'), `${size.name}：运行详情可关闭`);

  // 运行中四颗主操作全部内联（打开面板 / 停止 / 重启 / 查看日志），不再出现「更多」空壳折叠。
  await page.click('#scenarios button[data-state="running"]').catch(()=>{});
  await page.waitForTimeout(150);
  const runningActs=await page.evaluate(()=>[...document.querySelectorAll('#card-overview-status .ovb-actions > .btn')]
    .filter(el=>!el.hidden&&el.offsetParent!==null).map(el=>(el.textContent||'').trim().replace(/\s+/g,' ')));
  check(JSON.stringify(runningActs)===JSON.stringify(['打开面板','停止','重启','查看日志']), `${size.name}：运行中动作=${JSON.stringify(runningActs)}`);

  // 2026-10-09 软件实证：软件无「更多」折叠层，原型同步移除，不再做弹出层取证。
  await page.click('#scenarios button[data-state="stopped"]').catch(()=>{});
  await page.waitForTimeout(150);
  const moreGone=await page.evaluate(()=>!document.querySelector('#card-overview-status .ovb-more'));
  check(moreGone, `${size.name}：未运行也不存在「更多」折叠层`);
  await page.click('#scenarios button[data-state="running"]').catch(()=>{});
  await page.waitForTimeout(150);
  await page.click('#card-overview-status .motion-detail-icon');
  await page.waitForTimeout(250);
  await resetScroll(page);
  await shotCard(page,path.join(shotsDir,`概览-运行中详情-${size.preset}.png`));
  await page.keyboard.press('Escape');
  await page.close();
}
await browser.close();

console.log(results.join('\n'));
console.log('\n== console errors ==');
console.log(consoleErrors.length?consoleErrors.join('\n'):'(none)');
console.log('\nshots -> '+shotsDir);
const fails=results.filter(r=>r.startsWith('FAIL')).length;
console.log(`\nTOTAL: ${results.length}  FAIL: ${fails}  CONSOLE_ERRORS: ${consoleErrors.length}`);
process.exit(fails||consoleErrors.length?1:0);
