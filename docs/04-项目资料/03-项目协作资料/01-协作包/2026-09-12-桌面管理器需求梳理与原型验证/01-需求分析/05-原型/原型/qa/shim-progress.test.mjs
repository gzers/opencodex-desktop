import {sharedRoot,sharedPrototype} from './prototype-location.mjs';
// 原型「设置 → 官方共享配置 → 随 Codex 启动 OpenCodex」写入进行态：行为测试（jsdom）。
// 运行：node "<本文件>"
// 覆盖：点开关后先「准备」延迟 → 再走不确定进度（不编百分比）→ 完成才改开关；
// 进行态长在该行内部的小框里；框内可展开下拉查看实际执行的具体指令；期间开关禁用。
import { createRequire } from 'module';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

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
const proto=sharedPrototype;
const require=createRequire(path.join(repo,'apps/desktop/ui/'));
let JSDOM;
try{ ({ JSDOM }=require('jsdom')); }
catch{ console.log('BLOCKED: 未找到 jsdom（应装在 apps/desktop/ui 依赖下）。'); process.exit(2); }

const log=[];
const jsErrors=[];
const assert=(cond,label)=>{ log.push(`${cond?'PASS':'FAIL'}  ${label}`); if(!cond) jsErrors.push(label); };
const sleep=ms=>new Promise(r=>setTimeout(r,ms));

const dom=new JSDOM(fs.readFileSync(proto,'utf8'),{
  url:'file:///proto/index.html#settings?section=general',
  runScripts:'dangerously', pretendToBeVisual:true,
  beforeParse(win){
    win.matchMedia=win.matchMedia||(()=>({matches:false,addEventListener(){},removeEventListener(){},addListener(){},removeListener(){}}));
    win.addEventListener('error',e=>jsErrors.push('window error: '+(e.error&&e.error.stack||e.message)));
  },
});
const { window }=dom;
await new Promise(r=>window.addEventListener('load',r,{once:true}));
await sleep(300);

const doc=window.document;
const toggle=doc.querySelector('[data-prototype-action="codex-shim-toggle"]');
assert(!!toggle,'「随 Codex 启动 OpenCodex」开关接入了 codex-shim-toggle 动作');
const panel=doc.querySelector('[data-shim-panel]');
assert(!!panel,'存在行内进行态框 [data-shim-panel]');
const title=panel.querySelector('[data-shim-panel-title]');
const meta=panel.querySelector('[data-shim-panel-meta]');
const hint=panel.querySelector('[data-shim-panel-hint]');
const cmds=panel.querySelector('[data-shim-cmds]');
const details=panel.querySelector('[data-shim-details]');
const result=panel.querySelector('[data-shim-panel-result]');
const detailsToggle=panel.querySelector('[data-shim-details-toggle]');
const bar=panel.querySelector('[data-shim-panel-bar]');
const state=()=>panel.dataset.shimState;
const click=el=>el.dispatchEvent(new window.MouseEvent('click',{bubbles:true}));

assert(panel.hidden===true,'进行态框默认隐藏');
assert(toggle.getAttribute('aria-checked')==='false','原型默认关闭，方便点开看安装效果');
// 默认那行保持同一行：标题/说明列 + 开关列，面板另起一行独占。
const row=panel.parentElement;
assert(row.classList.contains('setting-row'),'进行态框是 setting-row 的直接子元素（独占该行底部一行）');
assert(row.querySelector('.controls')&&row.querySelector('.setting-desc'),'标题/说明与开关仍在同一行内');

// —— 点开关 = 安装（原型默认关闭，方便看效果）——
click(toggle);
await sleep(60);
assert(panel.hidden===false,'点开关后行内出现小框');
assert(state()==='prepare','先进入「准备」延迟阶段，不立刻跳进度');
assert(/正在准备/.test(title.textContent),'准备阶段标题为「正在准备…」');
assert(/准备阶段/.test(hint.textContent),'准备阶段行内说明写明正式写入尚未开始');
assert(toggle.disabled===true,'写入期间开关被禁用');
assert(details.hidden===true,'详情下拉默认收起');
assert(cmds.querySelectorAll('li').length===3,'准备阶段已把「安装」的指令清单渲染进框内');
assert([...cmds.querySelectorAll('code')].some(c=>c.textContent.includes('ocx codex-shim install')),'安装指令清单含 ocx codex-shim install');

await sleep(1000);
assert(state()==='run','约 1 秒后转入写入进度阶段');
assert(title.textContent.includes('正在安装官方 shim'),'进度阶段标题为「正在安装官方 shim」');
assert(!bar.hasAttribute('aria-valuenow'),'不确定进度条不编造百分比（无 aria-valuenow）');
assert(/探针/.test(hint.textContent)&&/10–30 秒/.test(hint.textContent),'安装提示写明探针校验与时长');

click(detailsToggle);
await sleep(30);
assert(details.hidden===false,'点「查看执行的具体指令」展开详情下拉');
assert(detailsToggle.getAttribute('aria-expanded')==='true','展开时 aria-expanded=true');
const cmdText=[...cmds.querySelectorAll('code')].map(c=>c.textContent).join('|');
assert(cmdText.includes('ocx codex-shim status'),'指令清单含 ocx codex-shim status（先读状态/读回确认）');

await sleep(1200);
assert(/已用 [12]s/.test(meta.textContent),'已用时间在走');
assert(cmds.querySelector('li').dataset.cmdState!=='todo','指令按顺序从「待执行」推进到「执行中/已完成」');

await sleep(3400);
assert(state()==='done','写入完成后进入完成阶段');
assert(result.hidden===false&&/已安装官方 shim/.test(result.textContent),'完成时把「指令做了什么」写进结果说明（安装）');
assert(result.textContent.includes('codex.opencodex-real'),'结果说明点明备份文件落点');
assert(!panel.classList.contains('is-close'),'开启不是「关闭」形态（保留长耗时进度条）');
assert(toggle.getAttribute('aria-checked')==='true','完成后 aria-checked=true');
assert(toggle.disabled===false,'完成后开关恢复可用');
await sleep(1600);
assert(panel.hidden===true,'完成后小框收起');

// —— 再点一次 = 关闭 ——
click(toggle);
await sleep(60);
assert(state()==='prepare','关闭同样先进入「准备」延迟阶段');
assert(panel.classList.contains('is-close'),'关闭进入「关闭」形态：不复用开启那根进度条');
assert(/正在准备/.test(title.textContent),'关闭准备阶段标题为「正在准备…」');
assert(cmds.querySelectorAll('li').length===2,'准备阶段已把「关闭」的指令清单渲染进框内');
assert([...cmds.querySelectorAll('code')].some(c=>c.textContent.includes('ocx codex-shim uninstall')),'关闭指令清单含 ocx codex-shim uninstall');
await sleep(1000);
assert(state()==='run','约 1 秒后转入关闭进度阶段');
assert(title.textContent.includes('正在关闭官方 shim'),'关闭进度阶段标题');
await sleep(1800);
assert(state()==='done','关闭完成后进入完成阶段');
assert(result.hidden===false&&/已关闭官方 shim/.test(result.textContent),'关闭完成时把「指令做了什么」写进结果说明（移除 wrapper / 还原原可执行文件）');
assert(/还原/.test(result.textContent),'关闭结果说明点明「还原」动作');
assert(toggle.getAttribute('aria-checked')==='false','关闭完成后 aria-checked=false');
assert(toggle.disabled===false,'关闭完成后开关恢复可用');
await sleep(2200);
assert(panel.hidden===true,'关闭完成后小框收起');

console.log(log.join('\n'));
console.log('\n== 捕获的 JS 错误 ==\n'+(jsErrors.filter(e=>String(e).startsWith('window error')).join('\n')||'(none)'));
const fails=log.filter(l=>l.startsWith('FAIL'));
console.log('\nTOTAL: '+log.length+'  FAIL: '+fails.length);
// 清掉原型里未跑完的定时器，避免 jsdom 事件循环挂住。
if(typeof window.__shimPanelReset==='function') window.__shimPanelReset();
dom.window.close();
process.exit(fails.length?1:0);
