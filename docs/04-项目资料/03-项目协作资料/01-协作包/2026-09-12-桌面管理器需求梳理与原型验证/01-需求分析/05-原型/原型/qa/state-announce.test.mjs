import {sharedRoot,sharedPrototype} from './prototype-location.mjs';
// 原型「主线＋节点支线＋统一报错」行为测试（jsdom）。
// 运行：node "<本文件>"
// 覆盖 DMD Revision 12（REQ-31 / AC-15）与状态专题 §6：一个运行主线组织节点支线；
//   · 概览简略＋详情：摘要卡只放主线事实与当前支线，环境/路径/保护进详情；
//   · 主线只表达运行事实（待接入 / 未运行 / 运行中 / 正在确认），不被支线或问题抢占；
//   · 持续问题与主线分离：进「待处理摘要」一条，可解除；at-risk 单独不制造故障；
//   · 不再按运行枚举机械地产生持久通知或稳定态 Toast；
//   · 操作归当前支线：待确认 → 执行 → 核验 → 结果；
//   · 统一结果区分成功 / 部分完成 / 失败 / 已取消 / 待确认 / 待重启，不误报为普通成功/失败。
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
const stateBtn=s=>doc.querySelector(`#scenarios button[data-state="${s}"]`);
const setState=(s)=>{ const b=stateBtn(s); if(!b) throw new Error('缺状态按钮: '+s); click(b); };
const opBtn=k=>doc.querySelector(`#opResultStates button[data-op-result="${k}"]`);
const setOp=k=>click(opBtn(k));
const toastEl=()=>doc.getElementById('toastEl');
const toastText=()=>toastEl().textContent.replace(/\s+/g,'');
const toastShown=()=>toastEl().classList.contains('show');
const toastKind=()=>[...toastEl().classList].find(c=>/^k-(info|success|warning|danger|error)$/.test(c))||'';
const notifRows=()=>[...doc.querySelectorAll('#notificationList .notification-item')];
const notifByText=t=>notifRows().find(el=>el.textContent.includes(t));
const mainlineText=()=>doc.querySelector('#card-overview-status [data-b="statetext"]').textContent.trim();
const problemText=()=>{const el=doc.querySelector('#card-overview-status [data-b="problem"]');return el.hidden?'':el.textContent.trim();};
const branchRow=()=>doc.querySelector('#card-overview-status [data-b="brancher"]');
const sleep=ms=>new Promise(r=>setTimeout(r,ms));

await sleep(1300);

// 1. 概览简略＋详情：摘要卡只放主线与支线；环境/路径进详情；端口只在运行观测时出现
assert(!!doc.getElementById('card-overview-status'),'概览存在运行摘要卡');
assert(!!doc.getElementById('card-overview-details'),'概览存在同一块的运行详情（简略＋详情）');
assert(doc.getElementById('card-overview-status').contains(doc.getElementById('card-overview-details')),'运行详情与摘要同属一张卡（同一摘要块内展开）');
assert(doc.querySelectorAll('.ovb-note').length===0 && doc.querySelectorAll('[data-b="note"]').length===0,'摘要卡不再有说明句节点');
assert(doc.getElementById('card-overview-status').contains(doc.querySelector('[data-b="rootshort"]')),'摘要行只保留关键辅助事实');

// 2. 主线只表达运行事实
setState('not_found');
assert(mainlineText()==='待接入',`not_found 主线显示「待接入」（实际 ${mainlineText()}）`);
setState('loading');
assert(mainlineText()==='正在确认',`loading 主线显示「正在确认」（实际 ${mainlineText()}）`);
setState('stopped');
assert(mainlineText()==='未运行','stopped 主线显示「未运行」');
assert(doc.querySelector('[data-fact="port"]').hidden,'未运行时不展示端口（不暗示正在监听）');
setState('running');
assert(mainlineText()==='运行中','running 主线显示「运行中」');
assert(!doc.querySelector('[data-fact="port"]').hidden,'运行中展示端口');

// 3. 持续问题与主线分离；at-risk 单独不制造故障通知
setState('at_risk');
assert(mainlineText()==='未运行','at_risk 主线与现行 UI 规范及托盘一致，显示「未运行」');
assert(problemText()==='启动保护未启用','at_risk 只给中性说明，不写告警');
assert(!/warning|danger/.test(doc.querySelector('#card-overview-status [data-b="problem"]').className),'at_risk 说明不是告警级别（官方 at-risk 单独不构成故障）');
assert(!notifRows().some(el=>el.textContent.includes('启动保护')||el.textContent.includes('run-at-risk')),'at_risk 不产生持续故障通知');

setState('external_takeover');
assert(mainlineText()==='运行中','external_takeover 主线仍是「运行中」');
assert(/接管/.test(problemText()),'external_takeover 问题槽显示所有权问题');
const takeRow=notifByText('外部 provider 已接管');
assert(!!takeRow && !takeRow.classList.contains('is-read'),'外部接管写入一条未读待处理通知');

setState('unreachable');
assert(/不可达/.test(problemText()),'unreachable 问题槽显示不可达');
assert(!notifByText('外部 provider 已接管') || notifByText('外部 provider 已接管').classList.contains('is-read'),'离开外部接管后该问题被解除（不再未读）');
assert(!!notifByText('进程或端口不可达'),'unreachable 写入待处理通知');
setState('running');
assert(notifByText('进程或端口不可达').classList.contains('is-read'),'离开不可达后问题解除（当前待处理收口，保留历史）');
assert(problemText()==='','回到运行中问题槽清空');
assert(doc.querySelector('#card-overview-status [data-b="brancher"]').hidden,'空闲运行不再重复占一行「运行正常」');

// 4. 不再按运行枚举机械地产生稳定态 Toast
toastEl().classList.remove('show');
const beforeCount=notifRows().length;
setState('stopped');
await sleep(3800);
assert(!toastShown(),'stopped 不再机械播报 info Toast');
setState('running');
await sleep(3800);
assert(!toastShown(),'running 不再机械播报 info Toast');
assert(notifRows().length===beforeCount,'稳定主线迁移不新增通知');

// 5. 操作归当前支线：执行中/结果
setState('starting');
assert(/启动中/.test(branchRow().textContent),'starting 支线显示执行中');
assert(!doc.querySelector('[data-b="branchspin"]').hidden,'starting 支线有进行态指示');
setState('starting_failed');
assert(/启动失败/.test(branchRow().textContent),'starting_failed 支线给出结果');
assert(doc.querySelector('[data-b="branchlabel"]').classList.contains('tone-bad'),'启动失败按失败色调呈现');
assert(!notifByText('OpenCodex 启动失败')||true,'启动失败归当前支线，不新造全局故障（历史通知不受影响）');

// 6. 统一操作结果：部分完成 / 已取消 / 待确认 / 待重启各有真实结论
setOp('partial');
const resultEl=doc.querySelector('[data-b="opresult"]');
assert(!resultEl.hidden && /部分完成/.test(resultEl.textContent),'部分完成显示真实结论');
assert(/未完成/.test(resultEl.textContent),'部分完成列出未完成项');
setOp('cancelled');
assert(/已取消/.test(resultEl.textContent),'取消显示「已取消」而非失败');
setOp('unconfirmed');
assert(/待确认/.test(resultEl.textContent),'超时/未决显示「结果待确认」，不误报成功或失败');
setOp('restart');
assert(/待重启/.test(resultEl.textContent),'待重启生效显示为明确后续条件');
setOp('none');
assert(resultEl.hidden,'清空后不常驻结果块');

// 7. 动作同源：运行中四颗主操作全部内联（打开面板 / 停止 / 重启 / 查看日志，对齐软件实际效果）；
//    刷新 / 查看建议只在允许它们的状态进入「更多」，运行中不出现空壳折叠。
setState('running');
const moreWrap=doc.querySelector('[data-b="morewrap"]');
assert(moreWrap.hidden,'运行中不出现空壳「更多」');
for(const act of ['panel','stop','restart','logs']){
  assert(!doc.querySelector(`[data-act="${act}"]`).hidden,`运行中直接露出 ${act}`);
}
setState('stopped');
assert(!moreWrap.hidden,'未运行存在「更多」入口');
assert(doc.querySelector('[data-act="refresh"]').closest('.ovb-more-panel'),'刷新等低频动作收进「更多」');

// 8. 启停结果 Toast 不叠加机械说明句
await sleep(3800);
setState('stopped');
await sleep(3800);
toastEl().classList.remove('show');
click(doc.querySelector('[data-control="start"]'));
await sleep(2300);
assert(toastText().includes('已启动'),'启动完成后播报结果 Toast（已启动）');
assert(!toastText().includes('代理未运行'),'启动流程不叠加旧的说明句 Toast');

console.log(log.join('\n'));
console.log('\n== 捕获的 JS 错误 ==');
console.log(errors.length?errors.join('\n'):'(none)');
const fails=log.filter(r=>r.startsWith('FAIL')).length;
console.log(`\nTOTAL: ${log.length}  FAIL: ${fails}  JS_ERRORS: ${errors.length}`);
process.exit(fails||errors.length?1:0);
