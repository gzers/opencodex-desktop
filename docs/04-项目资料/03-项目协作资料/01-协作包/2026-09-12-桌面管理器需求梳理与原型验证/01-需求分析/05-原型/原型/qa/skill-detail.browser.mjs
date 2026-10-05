import {sharedRoot,sharedPrototype} from './prototype-location.mjs';
// 原型「扩展管理 → Skills」Skill 详情弹窗：真实浏览器（无头 Chromium）检查与截图。
// 运行：node "<本文件>"   （可选 PLAYWRIGHT_CORE=<playwright-core 目录>）
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

const protoDir=path.join(here,'..');
const proto=sharedPrototype;
const outDir=path.join(protoDir,'..','文档','截图','原型-扩展Skills详情-20260923');
fs.mkdirSync(outDir,{recursive:true});

const url=(theme)=>pathToFileURL(proto).href+`?theme=${theme}#extensions`;

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
}
async function shot(name,sel){
  const el=await page.$(sel);
  if(!el){ check(false,'截图目标缺失: '+sel+' ('+name+')'); return; }
  await el.scrollIntoViewIfNeeded().catch(()=>{});
  await page.waitForTimeout(120);
  const box=await el.boundingBox();
  if(!box){ check(false,'截图目标不可见: '+sel+' ('+name+')'); return; }
  const vp=page.viewportSize();
  const x=Math.max(0,box.x-8), y=Math.max(0,box.y-8);
  const w=Math.min(vp.width-x, box.width+16), h=Math.min(vp.height-y, box.height+16);
  if(w<=1||h<=1){ check(false,'截图区域无效: '+sel+' ('+name+')'); return; }
  await page.screenshot({path:path.join(outDir,name),clip:{x,y,width:w,height:h}});
}
const modalShown=()=>page.$eval('#modalMask',el=>el.style.display==='flex');
const modalTitle=()=>page.$eval('#modalTitle',el=>el.textContent.trim());
const modalBody=()=>page.$eval('#modalBody',el=>el.textContent);
const rowHeight=()=>page.$eval('#skillsList .skill-row',el=>Math.round(el.getBoundingClientRect().height));

// ── 1. 列表：可点单元格 ──
await boot('light');
const cells=await page.$$('#skillsList .skill-info[data-ext-detail]');
check(cells.length===6,'Skills 6 行的「名称 + 描述」单元格都可点开详情');
check((await page.$$('#mcpList .skill-info[data-ext-detail]')).length===1,'MCP 行用同一套详情机制');
check(await page.$eval('#skillsList .skill-info[data-ext-detail]',el=>el.getAttribute('role')==='button'&&el.getAttribute('tabindex')==='0'),'单元格是 role=button 且可聚焦');

// 可点击提示：指针 + 悬停时单元格文字提亮 + 描述底部渐隐；没有外框 / 下划线 / 提示文字
const cell='#skillsList .skill-info[data-ext-detail]';
const bodyColor=await page.$eval('#skillsList .skill-info .skill-title h3',el=>getComputedStyle(el).color);
const before=await page.$eval(`${cell} p`,el=>getComputedStyle(el).color);
check(await page.$eval(cell,el=>getComputedStyle(el).cursor==='pointer'),'悬停指针为可点击');
await page.hover(cell);
await page.waitForTimeout(250);
const after=await page.$eval(`${cell} p`,el=>getComputedStyle(el).color);
check(after!==before,'悬停时描述文字颜色发生变化');
check(after===bodyColor,'悬停时描述文字提亮到正文色（'+bodyColor+'）');
check(await page.$eval(`${cell} .skill-meta span`,el=>getComputedStyle(el).color)===bodyColor,'悬停时版本 / 更新时间一并提亮');
check(!(await page.$eval(`${cell} .skill-title h3`,el=>getComputedStyle(el).textDecorationLine.includes('underline'))),'悬停不加标题下划线');
check(await page.$eval(cell,el=>{const s=getComputedStyle(el);return s.outlineStyle==='none'&&s.borderTopStyle==='none'&&s.boxShadow==='none'&&s.backgroundColor==='rgba(0, 0, 0, 0)';}),'悬停不加外框 / 边框 / 阴影 / 底色');
await shot('03-描述渐隐提示-亮.png','#skillsList');
check(!(await modalShown()),'仅悬停不会打开弹窗');

// 键盘聚焦也不加外框（与悬停同一套提亮）
await page.mouse.move(10,10);
await page.waitForTimeout(150);
await page.focus(cell);
await page.waitForTimeout(200);
check(await page.$eval(cell,el=>getComputedStyle(el).outlineStyle==='none'),'键盘聚焦也不加外框');
check(await page.$eval(`${cell} p`,el=>getComputedStyle(el).color)===bodyColor,'键盘聚焦同样提亮文字');

// 截断才渐隐：长描述带遮罩，短描述不带
check(await page.$eval('#skillsList .skill-row:first-child .skill-info p',el=>el.classList.contains('is-clamped')),'长描述（design-studio）标记为已截断');
check(await page.$eval('#skillsList .skill-row:first-child .skill-info p',el=>{const s=getComputedStyle(el);return s.maskImage&&s.maskImage!=='none';}),'被截断的描述应用底部渐隐');
const shortRow=await page.$eval('#skillsList .skill-row:last-child .skill-info p',el=>({clamped:el.classList.contains('is-clamped'),mask:getComputedStyle(el).maskImage}));
check(!shortRow.clamped&&(shortRow.mask==='none'||!shortRow.mask),'短描述（skill-audit）不加渐隐，避免读成残缺');

// ── 2. 打开详情弹窗 ──
const hBefore=await rowHeight();
await page.click('#skillsList .skill-info[data-ext-detail]');
await page.waitForTimeout(300);
check(await modalShown(),'点击单元格打开详情弹窗');
check((await modalTitle())==='design-studio','弹窗标题是 Skill 名称');
check(await page.$('#modalBody .skill-detail-name')===null,'名称由弹窗标题承载，正文不重复');
const body=await modalBody();
check(body.includes('创作表达与自然度校准（去 AI 味、说人话、自然一点、别像模板）'),'弹窗展示完整描述（列表里只露两行）');
check(body.includes('跨项目、跨 agent 使用的全局设计 skill'),'超长描述的第二段也在弹窗里');
check((await page.$$('#modalBody .skill-detail-label')).length===0,'详情弹窗不再有任何区块小标题（描述 / 信息 / 同步目标 均已去掉）');
check(await page.$eval('#modalBody',el=>el.firstElementChild.classList.contains('skill-detail-doc')),'描述面板是正文第一块');
// 描述面板：带外框与加深底色、独立滚动、内部按 Markdown 渲染（不再是被截断的纯文本）
check(await page.$eval('#modalBody .skill-detail-md',el=>{const s=getComputedStyle(el);return s.borderTopStyle==='solid'&&s.backgroundColor!=='rgba(0, 0, 0, 0)';}),'描述面板有外框与加深底色');
check(await page.$eval('#modalBody .skill-detail-md',el=>getComputedStyle(el).overflowY==='auto'),'描述面板是独立滚动容器');
check(await page.$eval('#modalBody .skill-detail-md',el=>el.clientHeight<=el.parentElement.parentElement.clientHeight),'描述面板高度受弹窗约束，不会把弹窗撑高');
check(await page.$eval('#modalBody .skill-detail-md',el=>el.textContent.includes('结构化图表')&&el.textContent.includes('COL-*/')),'描述内容完整在面板里（只是需要滚动查看）');
check(await page.$eval('#modalBody .skill-detail-md',el=>el.querySelectorAll('h4').length>=3&&el.querySelectorAll('ol>li').length>0&&!!el.querySelector('pre code')),'描述内部已渲染 Markdown（标题 / 列表 / 代码块）');
check(await page.$eval('#modalBody .skill-detail-md p',el=>{const s=getComputedStyle(el);return s.color===getComputedStyle(document.querySelector('#skillsList .skill-info .skill-title h3')).color;}),'描述面板正文用正文色（与列表里的次要色区分）');
check(await page.$('#modalBody .modal-prototype-note')===null,'详情弹窗不显示「原型演示」脚注');
check((await page.$eval('#modal',el=>el.getBoundingClientRect().width))>=700,'长描述用加宽弹窗阅读');
check(body.includes('~/.agents/skills/design-studio')&&body.includes('SKILL.md'),'展示源目录与入口文件');
check(body.includes('28 个文件')&&body.includes('412 KB'),'展示文件数 / 体量');
check((await page.$$('#modalBody .skill-detail-facts>div')).length===7,'信息区恰为 7 项，且已并入来源 / 版本 / 更新时间');
check(await page.$('#modalBody .skill-detail-head')===null&&await page.$('#modalBody .modal-facts')===null,'信息区不再单占一行、也不套外框');
check(await page.$eval('#modalBody .skill-detail-facts',el=>{const s=getComputedStyle(el);return s.borderTopStyle==='none'&&s.backgroundColor==='rgba(0, 0, 0, 0)';}),'信息区无边框无底色');
const dens=await page.$eval('#modalBody .skill-detail-facts',el=>{const s=getComputedStyle(el);return{cols:s.gridTemplateColumns.split(' ').length,gap:s.rowGap};});
check(dens.cols===2,'信息区为两列布局（提高密度）');
check(parseFloat(dens.gap)>=8,'信息区间距足够（行距 '+dens.gap+'）');
check((await page.$$('#modalBody .skill-detail-targets li')).length===6,'展示 6 个同步目标');
check((await page.$$('#modalBody .skill-detail-targets li.on')).length===3,'已同步数量与行内开关一致（3）');
check(await page.$eval('#modalConfirm',el=>el.hidden)&&(await page.$eval('#modalCancel',el=>el.textContent.trim()))==='关闭','弹窗只读，底部只有「关闭」');
await shot('01-Skill详情-亮.png','#modal');

// 弹窗不越界、正文不横向溢出（三段式：只有正文滚动）
const vp=page.viewportSize();
const mbox=await page.$eval('#modal',el=>{const r=el.getBoundingClientRect();return{x:r.x,y:r.y,w:r.width,h:r.height};});
check(mbox.x>=0&&mbox.y>=0&&mbox.x+mbox.w<=vp.width+1&&mbox.y+mbox.h<=vp.height+1,'弹窗完整落在视口内');
check(await page.$eval('#modalBody',el=>el.scrollWidth<=el.clientWidth+1),'弹窗正文无横向溢出');

// ── 3. 关闭方式 ──
// ── 2b. 标题右侧动作 + 目标格切换 ──
const headSel='#modalHeadActions button';
check((await page.$$(headSel)).length===2,'标题右侧有 2 个图标动作');
const headBox=await page.$eval('#modalHeadActions',el=>{const r=el.getBoundingClientRect();return{x:r.x,y:r.y,w:r.width,h:r.height};});
const titleBox=await page.$eval('#modalTitle',el=>{const r=el.getBoundingClientRect();return{x:r.x,y:r.y,w:r.width,h:r.height};});
const bodyTop=await page.$eval('#modalBody',el=>el.getBoundingClientRect().y);
check(headBox.x>titleBox.x+titleBox.w-1,'动作按钮位于标题右侧');
check(headBox.y+headBox.h<=bodyTop+1,'动作按钮在标题行内、正文之上');
check(await page.$eval(headSel+'[title="卸载"]',el=>el.classList.contains('danger')),'「卸载」用危险色');

const targetSel='#modalBody .skill-detail-target';
await page.click(targetSel);
await page.waitForTimeout(200);
check((await page.$eval(targetSel,el=>el.getAttribute('aria-pressed')))==='false','点击目标格切成未同步');
check((await page.$eval(targetSel,el=>el.querySelector('em').textContent.trim()))==='未同步','状态文案随点击更新');
check((await page.$eval('#skillsList .skill-row:first-child .skill-targets .target-icon',el=>el.getAttribute('aria-checked')))==='false','列表那一行的开关同步写回');
await page.click(targetSel);
await page.waitForTimeout(200);
check((await page.$eval(targetSel,el=>el.getAttribute('aria-pressed')))==='true','再点一次切回已同步');
check((await page.$$('#modalBody .skill-detail-targets li.on')).length===3,'两次点击后回到 3 个已同步');

await page.click(headSel+'[title="检查 / 更新"]');
await page.waitForTimeout(200);
check((await modalTitle())==='design-studio'&&(await page.$$(headSel)).length===2,'标题「检查 / 更新」不换窗，就地执行当前 Skill');
check(await page.$eval('#skillsList .skill-row:first-child [data-prototype-action="skills-update"]',el=>el.classList.contains('is-busy')),'进行中状态表现在行内按钮上');
await page.waitForTimeout(1000);
check(await page.$eval('#skillsList .skill-row:first-child .skill-meta span:last-child',el=>el.textContent.trim())==='更新 2026-09-13','已是最新的 Skill 不改更新时间');
check((await page.$eval('#modalBody .skill-detail-facts>div:nth-child(3) span',el=>el.textContent.trim()))==='2026-09-13','详情弹窗的更新时间保持一致');

// 可更新的 Skill：就地更新这一条，日期刷新
await page.keyboard.press('Escape');
await page.waitForTimeout(150);
await page.click('#skillsList .skill-row:nth-child(2) .skill-info[data-ext-detail]');
await page.waitForTimeout(250);
check((await page.$eval('#modalBody .skill-detail-facts>div:nth-child(3) span',el=>el.textContent.trim()))==='2026-09-12','development-knowledge 更新前是 2026-09-12');
await page.click(headSel+'[title="检查 / 更新"]');
await page.waitForTimeout(1000);
check((await page.$eval('#modalBody .skill-detail-facts>div:nth-child(3) span',el=>el.textContent.trim()))==='2026-09-23','详情弹窗的更新时间随更新刷新');
check(await page.$eval('#skillsList .skill-row:nth-child(2) .skill-meta span:last-child',el=>el.textContent.trim())==='更新 2026-09-23','列表那一行也刷新了更新时间');
await page.keyboard.press('Escape');
await page.waitForTimeout(150);
await page.click('#skillsList .skill-info[data-ext-detail]');
await page.waitForTimeout(250);
await page.click(headSel+'[title="卸载"]');
await page.waitForTimeout(250);
check((await page.$eval('#modalTitle',el=>el.textContent.trim()))==='卸载 Skill','「卸载」打开警告弹窗');
check((await modalBody()).includes('design-studio'),'卸载弹窗点名当前 Skill');
await page.keyboard.press('Escape');
await page.waitForTimeout(150);
await page.click('#skillsList .skill-info[data-ext-detail]');
await page.waitForTimeout(250);

// ── 3. 关闭方式 ──
await page.keyboard.press('Escape');
await page.waitForTimeout(200);
check(!(await modalShown()),'Esc 关闭弹窗');
check(!(await page.$eval('#modal',el=>el.classList.contains('modal-wide'))),'关闭后收起加宽样式');
check((await rowHeight())===hBefore,'关闭后列表行高不变');
await page.focus('#skillsList .skill-info[data-ext-detail]');
await page.keyboard.press('Enter');
await page.waitForTimeout(250);
check(await modalShown(),'键盘 Enter 打开详情');
await page.keyboard.press('Escape');
await page.waitForTimeout(200);

// ── 4. 默认窗口 1180×760：内容不应撑出内部滚动条 ──
const def=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2});
def.on('pageerror',e=>consoleErrors.push('default pageerror: '+e.message));
await def.goto(url('light'));
await def.waitForTimeout(700);
await def.click('#skillsList .skill-info[data-ext-detail]');
await def.waitForTimeout(300);
check(await def.$eval('#modalMask',el=>el.style.display==='flex'),'默认窗口下可打开详情弹窗');
check(await def.$eval('#modalBody',el=>el.scrollHeight<=el.clientHeight+1),'默认窗口下弹窗框架不滚动（信息 / 目标 / 关闭始终在场）');
check(await def.$eval('#modalBody',el=>el.offsetHeight-el.clientHeight<=1),'弹窗框架没有第二层滚动条');
check(await def.$eval('#modalBody .skill-detail-md',el=>el.scrollHeight>el.clientHeight),'默认窗口下滚动只发生在描述面板内部');
check(await def.$eval('#modal',el=>{const r=el.getBoundingClientRect();return r.x>=0&&r.y>=0&&r.x+r.width<=window.innerWidth+1&&r.y+r.height<=window.innerHeight+1;}),'默认窗口下弹窗不越界');
await def.screenshot({path:path.join(outDir,'05-默认窗口-1180x760.png')});
await def.close();

// ── 5. 暗色 ──
await boot('dark');
// 暗色下同样靠提亮表达可点击（不能只用「更亮」的浅色假设）
const darkBody=await page.$eval('#skillsList .skill-info .skill-title h3',el=>getComputedStyle(el).color);
const darkBefore=await page.$eval('#skillsList .skill-info[data-ext-detail] p',el=>getComputedStyle(el).color);
await page.hover('#skillsList .skill-info[data-ext-detail]');
await page.waitForTimeout(250);
check((await page.$eval('#skillsList .skill-info[data-ext-detail] p',el=>getComputedStyle(el).color))===darkBody,'暗色下悬停同样把描述提亮到正文色');
check(darkBefore!==darkBody,'暗色下未悬停时描述确实是次要色（'+darkBefore+' → '+darkBody+'）');
await page.mouse.move(10,10);
await page.waitForTimeout(150);
await page.click('#skillsList .skill-info[data-ext-detail]');
await page.waitForTimeout(300);
check(await modalShown(),'暗色下同样可打开详情');
check((await page.$eval('#modal',el=>getComputedStyle(el).backgroundColor))!=='rgba(0, 0, 0, 0)','暗色下弹窗取到主题底色');
await shot('02-Skill详情-暗.png','#modal');
await page.keyboard.press('Escape');
await page.waitForTimeout(150);

// ── 6. 行内操作不误触 ──
await boot('light');
await page.click('#skillsList .skill-row .skill-targets .target-icon');
await page.waitForTimeout(200);
check(!(await modalShown()),'点击行内同步开关不打开弹窗');
await page.click('#skillsList .skill-row .row-actions .icon-btn');
await page.waitForTimeout(250);
check(!(await modalShown()),'行内「检查 / 更新」就地执行，不打开任何弹窗');
await page.waitForTimeout(1000);
check(!(await modalShown()),'行内更新结束后仍无弹窗');
await page.click('#skillsList .skill-row .row-actions .icon-btn.danger');
await page.waitForTimeout(250);
check((await modalShown())&&(await modalTitle())==='卸载 Skill','行内「卸载」仍弹确认窗');
check((await modalBody()).includes('design-studio'),'行内卸载点名当前 Skill（不再写死 design-studio）');
await page.keyboard.press('Escape');
await page.waitForTimeout(150);

// ── 7. 最小窗口 900×600 ──
const min=await browser.newPage({viewport:{width:900,height:600},deviceScaleFactor:2});
min.on('pageerror',e=>consoleErrors.push('min pageerror: '+e.message));
await min.goto(url('light'));
await min.waitForTimeout(700);
const minErrors=[];
await min.click('#skillsList .skill-info[data-ext-detail]').catch(e=>minErrors.push(String(e.message)));
await min.waitForTimeout(300);
check(await min.$eval('#modalMask',el=>el.style.display==='flex'),'最小窗口下可打开详情弹窗');
check(await min.$eval('#modalBody',el=>el.scrollWidth<=el.clientWidth+1),'最小窗口下弹窗正文无横向溢出');
// 原型是把模拟应用窗口 .window 嵌在带评审工具条的页面里；「不越界」应相对该模拟窗口，
  // 与 state-convergence 的「弹出层不越出模拟窗口」同一参照系（用浏览器视口比较会误判）。
  check(await min.evaluate(()=>{const r=document.getElementById('modal').getBoundingClientRect();const w=document.querySelector('.window').getBoundingClientRect();return r.x>=w.x-1&&r.y>=w.y-1&&r.right<=w.right+1&&r.bottom<=w.bottom+1;}),'最小窗口下弹窗不越界');
// 只有描述面板滚动；弹窗框架不滚动，标题与底部操作始终可见，不产生第二层滚动条。
check(await min.$eval('#modalBody .skill-detail-md',el=>el.scrollHeight>el.clientHeight),'最小窗口下超长描述在面板内滚动');
check(await min.$eval('#modalBody',el=>el.scrollHeight<=el.clientHeight+1),'最小窗口下弹窗框架仍不滚动（无第二层滚动条）');
check(await min.$eval('#modalCancel',el=>{const r=el.getBoundingClientRect();return r.y>=0&&r.y+r.height<=window.innerHeight+1;}),'滚动时底部「关闭」仍固定可见');
check(await min.$eval('#modal h3',el=>el.getBoundingClientRect().height>0),'滚动时标题仍固定可见');
check(await min.$eval('#modalBody .skill-detail-facts',el=>{const r=el.getBoundingClientRect();return r.y>=0&&r.bottom<=window.innerHeight+1;}),'滚动时信息区仍固定可见');
await min.screenshot({path:path.join(outDir,'04-最小窗口-900x600.png')});
await min.close();

// ── 8. MCP：同一套详情机制 ──
await boot('light');
await page.click('.ext-tabs button[data-ext-tab="mcp"]');
await page.waitForTimeout(300);
check((await page.$$('#mcpList .skill-info[data-ext-detail]')).length===1,'MCP 条目可点开详情');
await page.click('#mcpList .skill-info[data-ext-detail]');
await page.waitForTimeout(300);
check((await modalTitle())==='node_repl','MCP 详情标题为条目名');
check((await page.$$('#modalBody .skill-detail-label')).length===0&&(await page.$('#modalBody .modal-prototype-note'))===null,'MCP 详情同样无小标题、无原型脚注');
const mcpBody=await modalBody();
check(mcpBody.includes('类型')&&mcpBody.includes('stdio')&&mcpBody.includes('--input-type=module')&&mcpBody.includes('mcp_servers'),'信息区按 MCP 语义给类型 / 参数 / 配置落点');
check((await page.$$('#modalBody .skill-detail-facts>div')).length===7,'MCP 信息区恰为 7 项');
check(await page.$$eval('#modalBody .skill-detail-facts>div b',els=>new Set(els.map(el=>Math.round(el.getBoundingClientRect().width))).size===1),'信息区标签固定宽度，取值左边界对齐');
check(await page.$eval('#modalBody .skill-detail-facts>div.span-2 span.scrollable',el=>el.scrollHeight>0),'环境变量整行且取值区可滚动');
check(!!(await page.$('#modalBody .skill-detail-md pre code')),'MCP 详情用 Markdown 渲染配置代码块');
check(await page.$eval('#modalBody .skill-detail-md',el=>{const s=getComputedStyle(el);return s.borderTopStyle==='solid'&&s.backgroundColor!=='rgba(0, 0, 0, 0)'&&s.overflowY==='auto';}),'MCP 描述面板同样有外框、加深底色与独立滚动容器');
check((await page.$$eval('#modalHeadActions button',els=>els.map(e=>e.getAttribute('title')))).join(',')==='编辑,删除','MCP 标题动作是「编辑 / 删除」');
await shot('06-MCP详情-亮.png','#modal');
await page.click('#modalBody .skill-detail-target');
await page.waitForTimeout(200);
check((await page.$eval('#modalBody .skill-detail-target',el=>el.getAttribute('aria-pressed')))==='true','MCP 目标格点击可切换同步');
check((await page.$eval('#mcpList .skill-row .skill-targets .target-icon',el=>el.getAttribute('aria-checked')))==='true','写回 MCP 列表那一行的开关');
await page.click('#modalHeadActions button[title="删除"]');
await page.waitForTimeout(250);
check((await modalTitle())==='删除 MCP'&&(await modalBody()).includes('node_repl'),'「删除」点名当前 MCP 条目');
await page.keyboard.press('Escape');
await page.waitForTimeout(150);
await boot('dark');
await page.click('.ext-tabs button[data-ext-tab="mcp"]');
await page.waitForTimeout(300);
await page.click('#mcpList .skill-info[data-ext-detail]');
await page.waitForTimeout(300);
check(await page.$eval('#modalMask',el=>el.style.display==='flex'),'暗色下 MCP 详情可打开');
await shot('07-MCP详情-暗.png','#modal');
await page.keyboard.press('Escape');
await page.waitForTimeout(150);

await browser.close();

console.log(results.join('\n'));
const fails=results.filter(r=>r.startsWith('FAIL'));
console.log('\n== console errors ==\n'+(consoleErrors.join('\n')||'(none)'));
console.log('\nTOTAL: '+results.length+'  FAIL: '+fails.length+'  CONSOLE_ERRORS: '+consoleErrors.length);
console.log('shots -> '+outDir);
process.exit(fails.length||consoleErrors.length?1:0);
