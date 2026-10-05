import {sharedRoot,sharedPrototype} from './prototype-location.mjs';
// 原型「托盘状态（mock 投影）」行为测试（jsdom）。
// 运行：node "<本文件>"
// 覆盖 2026-09-24 用户决策：托盘只讲进程事实——at_risk（应用内「存在风险」）在托盘标题里
// 显示为「未运行」，风险结论与状态说明句都不写进托盘（托盘菜单项会被长文案撑宽）。
import { createRequire } from 'module';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

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
const proto=sharedPrototype;
// jsdom 装在 apps/desktop/ui 下；ESM 的 NODE_PATH 不生效，故显式从该目录解析。
const require=createRequire(path.join(repo,'apps/desktop/ui/'));
const { JSDOM }=require('jsdom');

const html=fs.readFileSync(proto,'utf8');

const errors=[];
const dom=new JSDOM(html,{runScripts:'dangerously',pretendToBeVisual:true,url:'file:///proto/index.html',beforeParse(win){
  win.matchMedia=win.matchMedia||(()=>({matches:false,addEventListener(){},removeEventListener(){},addListener(){},removeListener(){}}));
  win.addEventListener('error',e=>errors.push('window error: '+(e.error&&e.error.stack||e.message)));
  for(const fn of ['replaceState','pushState']){
    const orig=win.history[fn].bind(win.history);
    win.history[fn]=(state,title,url)=>{ try{ orig(state,title,url); }catch{ /* file:// 下被 jsdom 拒绝 */ } };
  }
}});
const {window}=dom;
const doc=window.document;

const log=[];const assert=(c,m)=>log.push((c?'PASS':'FAIL')+' '+m);
const click=el=>el.dispatchEvent(new window.MouseEvent('click',{bubbles:true,cancelable:true}));
const stateBtn=s=>doc.querySelector(`#trayStates button[data-tray-state="${s}"]`);
// 托盘标题：macOS 菜单栏菜单与 Windows 通知区菜单各一份。
const titles=()=>[...doc.querySelectorAll('[data-tray-shell] .tray-title b[data-tray-label]')];
const trayText=()=>[...doc.querySelectorAll('[data-tray-shell] .tray-menu, [data-tray-shell] .tray-menubar, [data-tray-shell] .tray-statusbar')]
  .map(el=>el.textContent).join('\n');

await new Promise(r=>setTimeout(r,1200));

// 1. 切到 at_risk：托盘标题显示进程事实，不写风险结论
assert(!!stateBtn('at_risk'),'托盘状态切换器提供「存在风险」场景');
click(stateBtn('at_risk'));
assert(stateBtn('at_risk').classList.contains('active'),'点击后切换器停在 at_risk 场景');
const atRiskTitles=titles().map(el=>el.textContent.trim());
assert(atRiskTitles.length>=1,'托盘标题存在（macOS / Windows 各一份）');
assert(atRiskTitles.every(t=>t==='未运行'),`at_risk 的托盘标题显示「未运行」（实际：${JSON.stringify(atRiskTitles)}）`);
const atRiskTray=trayText();
assert(!atRiskTray.includes('存在风险'),'托盘不再出现「存在风险」结论');
assert(!atRiskTray.includes('启动保护')&&!atRiskTray.includes('代理未在运行')&&!atRiskTray.includes('版本偏差'),
  '托盘不再出现状态说明句（启动保护 / 代理未在运行 / 版本偏差）');
assert(!/；/.test(atRiskTitles.join('')),'托盘标题是短标签，不带说明句分隔符');

// 2. 其余场景不受影响
for(const [state,label] of [['running','运行中'],['stopped','未运行'],['starting_failed','启动失败'],['external_takeover','外部接管']]){
  click(stateBtn(state));
  assert(titles().every(el=>el.textContent.trim()===label),`${state} 的托盘标题仍是「${label}」`);
}

// 3. 菜单栏 / 任务栏只给图标，标签文本不占位置。
//    jsdom 的层叠计算不覆盖后置的 `.tray-state{display:flex}` 与分组后代选择器比较，
//    因此这里断言样式表仍然声明了这条隐藏规则（视觉结果在真实制品上验收）。
assert(/\.tray-menubar\s+\.tray-state\s*,\s*\.tray-statusbar\s+\.tray-state\s*\{\s*display\s*:\s*none/.test(html),
  '样式表声明菜单栏 / 任务栏的状态文本不显示（不占位置）');

// 4. 托盘菜单结构未被改动：动作项与标题层级保持原型形状
const menu=doc.querySelector('[data-tray-shell] .tray-menu.mac');
assert(!!menu,'macOS 托盘菜单存在');
assert(menu.querySelectorAll('.tray-item[data-tray-action="start"]').length===1,'托盘菜单仍有「启动 OpenCodex」');
assert(menu.querySelectorAll('.tray-item[data-tray-action="quit"]').length===1,'托盘菜单仍有「退出桌面壳」');
// 2026-09-24 命名统一：托盘入口与概览 / 设置一致，用全名「打开诊断中心」。
assert(menu.querySelector('.tray-item[data-tray-action="open-logs"]').textContent.includes('打开诊断中心'),
  '托盘菜单的入口项为「打开诊断中心」');

console.log(log.join('\n'));
console.log('\n== 捕获的 JS 错误 ==');
console.log(errors.length?errors.join('\n'):'(none)');
const fails=log.filter(l=>l.startsWith('FAIL')).length;
console.log(`\nTOTAL: ${log.length}  FAIL: ${fails}  JS_ERRORS: ${errors.length}`);
process.exit(fails||errors.length?1:0);
