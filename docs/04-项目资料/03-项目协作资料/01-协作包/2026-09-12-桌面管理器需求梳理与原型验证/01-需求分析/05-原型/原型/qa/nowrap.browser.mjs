// 原型「短文本不许断行」真实浏览器（无头 Chromium）检查。
// 运行：node "<本文件>"   （可选 PLAYWRIGHT_CORE=<playwright-core 目录>）
//
// 覆盖 2026-09-25 用户报告：概览摘要行里「数据目录」被长路径挤成两行（「数据目」/「录」）。
// 用户要求：逐个页面检查描述 / label 等短文本不要断行，并给出合理宽度。
// 判定手段：用 Range 行盒（`getClientRects()`）数行——同一个元素里的文本占两个行盒就是断行，
// 比量高度更准（不受行高、内边距与同行不同字号影响）。
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

const proto=path.join(here,'..','index.html');
const url=hash=>pathToFileURL(proto).href+'?theme=light'+hash;
const results=[];
const check=(c,m)=>results.push((c?'PASS':'FAIL')+' '+m);

// 控件类：任何文案都不许断行（UI规范 §16「按钮永不换行」）。
const NEVER_WRAP=['.btn','.tag','.state-pill','.cli-badge','.icon-btn','.runtime-pill','.n-tag','.seg-tabs button','.preset-chip','.launch-grid button'];
// 标签类：短文案不许断行；长描述允许换行，所以按字数设上限。
const LABEL_MAX_CHARS=10;
const SHORT_LABELS=['label','.metric label','.ovc-fact','.setting-title','.field label','th','.card-head h2','.card-head h3','.nav button','.settings-tabs button','.diag-tabs button','.log-cats button','.notification-cats button','.skill-detail-label','.ovc-fact','.runtime-fact>b','.wiz-label','.modal-facts b','.dl-row>span','.proto-note-title'];

const collect=`(cfg)=>{
  // 行盒计数：同一行内的多个 rect（例如徽标 + 文字、不同字号）**垂直区间重叠**，
  // 换行产生的两个行盒则互不重叠。用区间重叠判同行，比比较 top 更稳。
  const lineCount=(el)=>{
    const range=document.createRange();
    range.selectNodeContents(el);
    const rects=[...range.getClientRects()].filter(r=>r.width>0.5&&r.height>0.5);
    const lines=[];
    for(const r of rects){
      if(!lines.some(l=>r.top<l.bottom-1&&r.bottom>l.top+1)) lines.push({top:r.top,bottom:r.bottom});
    }
    return lines.length;
  };
  const visible=(el)=>{
    if(el.offsetParent===null&&getComputedStyle(el).position!=='fixed') return false;
    const r=el.getBoundingClientRect();
    return r.width>0&&r.height>0&&getComputedStyle(el).visibility!=='hidden';
  };
  const out=[];
  const push=(el,kind,text,lines)=>{
    out.push({kind,selector:el.className||el.tagName.toLowerCase(),text:text.replace(/\\s+/g,' ').slice(0,40),lines});
  };
  for(const sel of cfg.neverWrap){
    for(const el of document.querySelectorAll(sel)){
      if(!visible(el)) continue;
      const text=(el.textContent||'').trim();
      if(!text) continue;
      push(el,'control',text,lineCount(el));
    }
  }
  for(const sel of cfg.shortLabels){
    for(const el of document.querySelectorAll(sel)){
      if(!visible(el)) continue;
      const text=(el.textContent||'').trim();
      if(!text) continue;
      // .ovc-fact 里带长路径：整行内容断行同样算破相（长值应当截断而不是换行）。
      if(!sel.includes('ovc-fact')&&text.length>cfg.maxChars) continue;
      push(el,'label',text,lineCount(el));
    }
  }
  return out;
}`;

const browser=await chromium.launch({executablePath:exe});
const consoleErrors=[];

const ROUTES=[
  '#overview','#panel','#extensions','#logs','#logs?tab=notifications','#logs?tab=doctor','#tray',
  '#settings?section=general','#settings?section=installation','#settings?section=backup',
  '#settings?section=extensions','#settings?section=migration','#settings?section=sync',
  '#settings?section=cleanup','#settings?section=cli','#settings?section=upgrade','#settings?section=about'
];
// 默认窗口与最小窗口各跑一遍：断行问题只在窄宽度暴露。
const WIDTHS=[{w:1180,h:760,name:'默认 1180'},{w:900,h:600,name:'最小 900'}];

for(const size of WIDTHS){
  const page=await browser.newPage({viewport:{width:size.w,height:size.h},deviceScaleFactor:2});
  page.on('console',m=>{ if(m.type()==='error') consoleErrors.push(m.text()); });
  page.on('pageerror',e=>consoleErrors.push('pageerror: '+e.message));
  // 把原型模拟窗口调到对应尺寸：概览卡片在窄窗口下才会暴露断行。
  const preset=size.w===900?'900x600':'1180x760';
  for(const route of ROUTES){
    await page.goto('about:blank');
    await page.goto(url(route));
    await page.waitForTimeout(500);
    await page.click(`#sizePresets button[data-size="${preset}"]`).catch(()=>{});
    await page.waitForTimeout(250);
    const rows=await page.evaluate(eval('('+collect+')'),{neverWrap:NEVER_WRAP,shortLabels:SHORT_LABELS,maxChars:LABEL_MAX_CHARS});
    const bad=rows.filter(r=>r.lines>1);
    for(const r of bad){
      check(false, `${size.name} ${route}：${r.kind}「${r.text}」断成 ${r.lines} 行（${r.selector}）`);
    }
    check(bad.length===0, `${size.name} ${route}：无断行（候选 ${rows.length} 处）`);
  }

  // 长路径压力：概览的「label 单行 + 长值截断」现在由运行详情承载。
  // 已确认的双形态概览把摘要事实收进 Logo 舞台的「运行详情」弹层，
  // 并由 overview-dual.css 隐藏旧的 .ovb-main 摘要行；因此改在运行详情
  // 的路径行上取证。「数据目录」是用户 2026-09-25 报告会被长路径断行的字段。
  await page.goto('about:blank');
  await page.goto(url('#overview'));
  await page.waitForTimeout(500);
  await page.click(`#sizePresets button[data-size="${preset}"]`).catch(()=>{});
  await page.waitForTimeout(200);
  await page.evaluate(({longPath})=>{
    document.querySelectorAll('[data-b="root"],[data-b="home"],[data-b="rootshort"]').forEach(el=>{ el.textContent=longPath; });
  },{longPath:'/Users/example/Library/Application Support/com.example.opencodex.desktop/SuperLongSegmentForWrapping'});
  await page.click('#card-overview-status .motion-detail-icon');
  await page.waitForTimeout(300);
  const stress=await page.evaluate(()=>{
    const lineCount=(el)=>{
      const range=document.createRange();
      range.selectNodeContents(el);
      const rects=[...range.getClientRects()].filter(r=>r.width>0.5&&r.height>0.5);
      const lines=[];
      for(const r of rects){ if(!lines.some(l=>r.top<l.bottom-1&&r.bottom>l.top+1)) lines.push({top:r.top,bottom:r.bottom}); }
      return lines.length;
    };
    const out=[];
    for(const row of document.querySelectorAll('.motion-runtime-paths>div,.motion-runtime-meta>div')){
      const dt=row.querySelector('dt'), dd=row.querySelector('dd');
      const value=row.querySelector('.motion-runtime-value');
      out.push({
        label:(dt?dt.textContent:'').trim(),
        lines:dt?lineCount(dt):0,
        valueLines:value?lineCount(value):(dd?lineCount(dd):0),
        valueTruncated:value?value.scrollWidth>value.clientWidth+1:false,
      });
    }
    return out;
  });
  const worst=Math.max(0,...stress.map(s=>s.lines));
  check(stress.length>0&&worst===1, `${size.name} 长路径压力：运行详情 label 保持单行（实测最大 ${worst} 行，候选 ${stress.length} 项）`);
  const pathRow=stress.filter(s=>s.label==='数据目录');
  check(pathRow.length>0&&pathRow.every(s=>s.valueTruncated||s.valueLines===1), `${size.name} 长路径压力：长值在自己行内截断/单行，不挤走 label`);
  await page.close();
}

await browser.close();
console.log(results.join('\n'));
console.log('\n== console errors ==');
console.log(consoleErrors.length?consoleErrors.join('\n'):'(none)');
const fails=results.filter(r=>r.startsWith('FAIL')).length;
console.log(`\nTOTAL: ${results.length}  FAIL: ${fails}  CONSOLE_ERRORS: ${consoleErrors.length}`);
process.exit(fails||consoleErrors.length?1:0);
