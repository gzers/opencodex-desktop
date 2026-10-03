// 原型「表格列宽分配」真实浏览器（无头 Chromium）检查与截图。
// 运行：node "<本文件>"   （可选 PLAYWRIGHT_CORE=<playwright-core 目录>）
// 覆盖 2026-09-24 用户要求：所有表格合理分配列宽，按钮 / 状态这类短内容不换行。
// 环境缺失时以退出码 2 报告 BLOCKED，不把「没跑」当成通过。
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
const repo=findRepo(here);
if(!repo){ console.log('BLOCKED: 未找到仓库根（.git）。'); process.exit(2); }

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

const proto=path.join(here,'..','index.html');
const outDir=path.join(here,'..','..','文档','截图','原型-表格列宽-20260924');
fs.mkdirSync(outDir,{recursive:true});
const url=hash=>pathToFileURL(proto).href+'?theme=light'+hash;

const results=[];
const check=(c,m)=>results.push((c?'PASS':'FAIL')+' '+m);

const browser=await chromium.launch({executablePath:exe});
const page=await browser.newPage({viewport:{width:1692,height:916},deviceScaleFactor:2});
const consoleErrors=[];
page.on('console',m=>{ if(m.type()==='error') consoleErrors.push(m.text()); });
page.on('pageerror',e=>consoleErrors.push('pageerror: '+e.message));

// 行数：Range 的行盒 top 去重——单行元素只会有 1 个 top。
const LINE_COUNT=`(el)=>{ const r=document.createRange(); r.selectNodeContents(el);
  const tops=[...r.getClientRects()].map(x=>Math.round(x.top)); return new Set(tops).size; }`;

// 先离开当前文档再进入目标 hash：同文档改 hash 不会重新加载，会残留上一步的弹窗 / 面板状态。
async function boot(hash){
  await page.goto('about:blank');
  await page.goto(url(hash));
  await page.waitForTimeout(700);
}
async function shot(name,sel){
  const el=await page.$(sel);
  if(!el){ check(false,'截图目标缺失: '+sel+' ('+name+')'); return; }
  await el.scrollIntoViewIfNeeded().catch(()=>{});
  await page.waitForTimeout(120);
  const box=await el.boundingBox();
  if(!box){ check(false,'截图目标不可见: '+sel+' ('+name+')'); return; }
  await el.screenshot({path:path.join(outDir,name)});
}

// 通用断言：表格不横向溢出；含按钮 / 状态的单元格单行；短文本列单行。
async function auditTable(label,selector,index=0){
  const info=await page.evaluate(({selector,index,LINE_COUNT})=>{
    const lineCount=eval(LINE_COUNT);
    const table=document.querySelectorAll(selector)[index];
    if(!table) return {missing:true};
    const host=table.parentElement;
    const out={overflow:table.scrollWidth-(host?host.clientWidth:0), rows:[], headerLine:0};
    const th=table.querySelector('thead th');
    if(th) out.headerLine=lineCount(th);
    for(const tr of table.querySelectorAll('tbody tr')){
      const cells=[...tr.children];
      const row={cellLines:cells.map(c=>lineCount(c)), btnHeights:[], pathLine:0};
      row.textLines=cells.map((c,i)=>{
        const full=(c.textContent||'').trim();
        // 只有「纯文本单元格」才用行盒数判单行：含按钮 / 标签 / 内联块的单元格
        // 会因内联盒与自身行盒产生两个 top，属测量噪声（这些已由按钮级断言覆盖）。
        const plain=c.children.length===0;
        return {i, text:full.slice(0,24), short:full.length<=30&&plain, lines:lineCount(c)};
      });
      for(const btn of tr.querySelectorAll('.btn,.icon-btn')){
        row.btnHeights.push({text:(btn.textContent||'').trim(), h:Math.round(btn.getBoundingClientRect().height), lines:lineCount(btn)});
      }
      for(const tag of tr.querySelectorAll('.tag,.state-pill,.cli-badge,.cell-note')){
        row.btnHeights.push({text:(tag.textContent||'').trim(), h:Math.round(tag.getBoundingClientRect().height), lines:lineCount(tag)});
      }
      const code=tr.querySelector('code');
      if(code) row.pathLine=lineCount(code);
      out.rows.push(row);
    }
    return out;
  },{selector,index,LINE_COUNT});

  if(info.missing){ check(false,label+'：表格存在（'+selector+'）'); return; }
  check(info.overflow<=1, label+'：表格不横向溢出（overflow='+info.overflow+'px）');
  check(info.headerLine<=1, label+'：表头单行');
  info.rows.forEach((row,i)=>{
    row.btnHeights.forEach(b=>{
      check(b.lines===1, label+'：第 '+(i+1)+' 行「'+b.text+'」单行（实测 '+b.lines+' 行）');
      check(b.h<=34, label+'：第 '+(i+1)+' 行「'+b.text+'」高度是单行控件（'+b.h+'px）');
    });
    if(row.pathLine) check(row.pathLine===1, label+'：第 '+(i+1)+' 行路径单行（实测 '+row.pathLine+' 行）');
    // 短文本列（用途 / 状态 / 结果这类，≤30 字）必须单行；
    // 长文本列（说明 / 正文）按设计允许多行——列宽分配上它们让位给短列。
    row.textLines.forEach(cell=>{
      if(cell.i===0) return;
      if(!cell.short) return;
      check(cell.lines===1, label+'：第 '+(i+1)+' 行第 '+(cell.i+1)+' 列短内容「'+cell.text+'」单行（实测 '+cell.lines+' 行）');
    });
  });
}

// 容器内可能有多张表（CLI 说明、体检结果各含多张）：逐张审计。
async function auditTablesIn(label,containerSel,tableSel='table'){
  const count=await page.evaluate(({containerSel,tableSel})=>document.querySelectorAll(containerSel+' '+tableSel).length,{containerSel,tableSel});
  if(count===0){ check(false,label+'：容器内存在表格（'+containerSel+' '+tableSel+'）'); return; }
  for(let i=0;i<count;i++) await auditTable(label+' · 表'+(i+1),containerSel+' '+tableSel,i);
}

// ── 1. 设置 → 安装配置：数据目录两张表（用户报告的现场）
for(const [w,h,tag] of [[1692,916,'默认窗口'],[900,600,'最小窗口']]){
  await page.setViewportSize({width:w,height:h});
  await boot('#settings?section=installation');
  await page.waitForTimeout(300);
  await auditTable(tag+' · 数据目录与 OPENCODEX_HOME','#card-install-paths .data-root-table');
  await auditTable(tag+' · 数据目录分区','#card-install-partitions .data-root-table');
  await shot('tables-'+tag+'-数据目录.png','#card-install-paths');
}

// ── 2. CLI 控制面指令表
await page.setViewportSize({width:1692,height:916});
await boot('#settings?section=cli');
await page.waitForTimeout(300);
// #cliPromptRow 默认 hidden，按钮不可点击；直接派发 click 触发委托监听（原型行为不变）。
await page.evaluate(()=>document.querySelector('[data-prototype-action="cli-help"]').click());
await page.waitForTimeout(400);
await auditTablesIn('CLI 说明弹窗','#modalBody','.cli-help-table');

// ── 3. 日志历史：调用日志表格（执行结果列是状态标识）
await boot('#logs?tab=logs');
await page.click('.log-cats button[data-log-cat="audit"]').catch(()=>{});
await page.waitForTimeout(300);
await auditTable('调用日志表','#logView .log-table');
await shot('tables-调用日志.png','#logView');

// ── 4. 扩展管理：同步体检弹窗里的体检表（状态列是 tag）
await boot('#settings?section=extensions');
await page.waitForTimeout(300);
await page.click('[data-prototype-action="skills-health-check"]').catch(()=>{});
await page.waitForTimeout(500);
await auditTablesIn('Skills 同步体检弹窗','#modalBody','.health-table');
await shot('tables-同步体检.png','#modal');

await browser.close();

console.log(results.join('\n'));
console.log('\n== console errors ==');
console.log(consoleErrors.length?consoleErrors.join('\n'):'(none)');
const fails=results.filter(r=>r.startsWith('FAIL')).length;
console.log(`\nTOTAL: ${results.length}  FAIL: ${fails}  CONSOLE_ERRORS: ${consoleErrors.length}`);
console.log('shots -> '+outDir);
process.exit(fails||consoleErrors.length?1:0);
