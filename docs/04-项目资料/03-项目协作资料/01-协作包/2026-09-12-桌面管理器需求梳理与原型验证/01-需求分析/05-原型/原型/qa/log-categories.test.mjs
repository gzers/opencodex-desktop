// 原型「日志历史」两分类 + 官方面板入口 行为测试（jsdom）。
// 运行：node "<本文件>"
// 覆盖产品方案要求的原型场景：两类切换、刷新、审计详情展开、长行、截断、加载、
// 空记录、缺文件、失败、超时，以及面板入口的可用 / 代理未运行 / 加载失败。
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
const protoDir=path.join(here,'..');
const proto=path.join(protoDir,'index.html');
// jsdom 装在 apps/desktop/ui 下；ESM 的 NODE_PATH 不生效，故显式从该目录解析。
const require=createRequire(path.join(repo,'apps/desktop/ui/'));
const { JSDOM }=require('jsdom');

const html=fs.readFileSync(proto,'utf8');

const errors=[];
const dom=new JSDOM(html,{runScripts:'dangerously',pretendToBeVisual:true,url:'file:///proto/index.html',beforeParse(win){
  win.matchMedia=win.matchMedia||(()=>({matches:false,addEventListener(){},removeEventListener(){},addListener(){},removeListener(){}}));
  win.addEventListener('error',e=>errors.push('window error: '+(e.error&&e.error.stack||e.message)));
  // jsdom 对 file:// 的 history.replaceState 抛 SecurityError（原型初始化与路由都用它）。
  // 这里改为记录目标 URL，使「是否导航」可断言；真实导航在无头 Chromium 中另测。
  win.__nav=[];
  for(const fn of ['replaceState','pushState']){
    const orig=win.history[fn].bind(win.history);
    win.history[fn]=(state,title,url)=>{ win.__nav.push(String(url||'')); try{ orig(state,title,url); }catch{ /* file:// 下被 jsdom 拒绝 */ } };
  }
}});
const {window}=dom;
const doc=window.document;
window.addEventListener('error',e=>errors.push('err: '+(e.error&&e.error.stack||e.message)));

const log=[];const assert=(c,m)=>log.push((c?'PASS':'FAIL')+' '+m);

const status=()=>doc.getElementById('logStatus').textContent.trim();
const view=()=>doc.getElementById('logView').textContent.trim();
const activeCat=()=>[...doc.querySelectorAll('.log-cats button')].find(b=>b.classList.contains('active'))?.dataset.logCat;
const catBtn=c=>doc.querySelector(`.log-cats button[data-log-cat="${c}"]`);
const scenBtn=s=>doc.querySelector(`#logScenarios button[data-log-scenario="${s}"]`);
const runBtn=r=>doc.querySelector(`#logRunStates button[data-log-run="${r}"]`);
const panelBtn=p=>doc.querySelector(`#logPanelStates button[data-log-panel="${p}"]`);
const entry=()=>doc.getElementById('logPanelEntry');
const modalShown=()=>doc.getElementById('modalMask').style.display==='flex';
const modalTitle=()=>doc.getElementById('modalTitle').textContent.trim();
const modalBody=()=>doc.getElementById('modalBody').textContent.trim();
const navigatedToPanel=()=>window.__nav.some(u=>u.includes('#panel'))||window.location.hash==='#panel';
const click=el=>el.dispatchEvent(new window.MouseEvent('click',{bubbles:true,cancelable:true}));

await new Promise(r=>setTimeout(r,1200));

// 1. 两分类存在，且不再有第三个分类
assert(!!catBtn('app')&&!!catBtn('audit'),'分类只有「应用日志 / 调用日志」两个');
assert(catBtn('proxy')===null,'不再存在「代理请求」分类标签');
assert(doc.querySelector('#logView[role=tabpanel]'),'logView 是 tabpanel');
assert(activeCat()==='app','首次进入默认「应用日志」');
assert(view().includes('[manager]'),'应用日志显示纯文本记录');
assert(status().includes('app.log'),'状态栏显示来源路径（应用日志）');
assert(status().includes('刷新'),'状态栏提供刷新');
assert(status().includes('清理日志'),'应用日志保留清理日志入口');
assert(!!entry(),'分类行右侧存在「面板请求日志」入口');
assert(entry().textContent.trim()==='面板请求日志','面板入口文案是「面板请求日志」');
assert(entry().classList.contains('btn')&&entry().classList.contains('ghost'),'面板入口用标准按钮规格（.btn ghost），与其他按钮一致');

// 2. 切到调用日志
click(catBtn('audit'));
assert(activeCat()==='audit','可切换到「调用日志」');
assert(!status().includes('清理日志'),'调用日志不显示清理日志');
assert(!!doc.querySelector('#logView table'),'调用日志渲染为表格');
assert(doc.querySelectorAll('#logView tbody tr').length>=4,'调用日志显示多条记录');
assert(view().includes('调用来源'),'调用日志含「调用来源」列');

// 2b. 执行结果：标识与提示同行；详情只用悬停浮层，不展开、不占行高
const hits=[...doc.querySelectorAll('#logView .t-detail-hit')];
const hit=hits.find(h=>(h.getAttribute('data-detail')||'').includes('query 已脱敏'))||hits[0];
assert(!!hit,'有详情的结果单元格提供可悬停的标识');
assert(hits.length>=2,'多行提供详情（已取消 / 失败）');
assert(doc.querySelector('#logView .t-err-full')===null,'不存在展开块（不靠展开占高度）');
assert(doc.querySelector('#logView [data-log-expand]')===null,'不存在展开按钮');
assert(!hit.textContent.includes('悬停查看'),'标识不再附带「悬停查看」提示文字');
assert(doc.querySelector('#logView .t-detail-hint')===null,'不存在多余的提示文字元素');
assert(doc.querySelector('#logView .t-detail').textContent.trim().length>0,'执行结果单元格有内容');
const tip=()=>doc.getElementById('logTip');
assert(!tip()||!tip().classList.contains('show'),'默认不显示详情浮层');
hit.dispatchEvent(new window.MouseEvent('mouseover',{bubbles:true}));
assert(tip()&&tip().classList.contains('show'),'悬停显示详情浮层');
assert(tip().textContent.includes('query 已脱敏'),'浮层内容已脱敏（URL query）');
assert(tip().textContent.includes('已保留备份')&&tip().textContent.includes('\n'),'浮层显示完整多行详情');
assert(doc.querySelector('#logView .t-err-full')===null,'悬停不产生展开块（行高不变）');
hit.dispatchEvent(new window.MouseEvent('mouseout',{bubbles:true,relatedTarget:doc.body}));
assert(!tip().classList.contains('show'),'移开后浮层收起');

// 3. 缺文件：不自动跨分类回退，提供主动切换
click(catBtn('audit'));
click(scenBtn('missing'));
assert(view().includes('尚未生成调用日志文件'),'调用日志缺文件文案');
assert(activeCat()==='audit','缺文件不自动切换分类');
const vab=[...doc.querySelectorAll('#logView [data-log-act]')].find(b=>b.dataset.logAct==='view-app');
assert(!!vab,'缺文件提供「查看应用日志」主动切换');
click(vab);
assert(activeCat()==='app','用户主动切换后选中项更新为应用日志');
click(catBtn('app'));
click(scenBtn('missing'));
assert(![...doc.querySelectorAll('#logView [data-log-act]')].some(b=>b.dataset.logAct==='view-app'),'应用日志自身缺文件不提示切换到应用日志');

// 4. 各分类状态文案
const chk=(s,needle,label)=>{click(scenBtn(s));assert(view().includes(needle),label);};
click(catBtn('app'));
chk('empty','暂无应用日志记录','应用日志空态文案');
chk('missing','尚未生成应用日志文件','应用日志缺文件文案');
chk('error','本次读取失败','应用日志失败文案');
chk('timeout','读取超时','应用日志超时文案');
chk('capped','已截断','应用日志截断提示');
click(scenBtn('normal'));
click(scenBtn('error'));
assert(view().includes('上次读取'),'有旧内容时失败显示「上次读取内容」提示');
click(catBtn('audit'));
chk('empty','暂无调用日志记录','调用日志空态文案');
chk('missing','尚未生成调用日志文件','调用日志缺文件文案');
chk('capped','已截断','调用日志截断提示');

// 5. 长行截断（仅应用日志）
click(catBtn('app'));
click(scenBtn('longtext'));
assert(view().includes('已截断 8 KiB'),'应用日志超长单行显示截断标记');
click(scenBtn('normal'));

// 6. 演示状态适用性置灰
assert(scenBtn('longtext').disabled===false,'应用日志：超长文本可用');
click(catBtn('audit'));
assert(scenBtn('longtext').disabled===true,'调用日志：超长文本置灰');
assert(scenBtn('missing').disabled===false,'调用日志：缺文件可用');

// 7. 刷新加载态与不叠加
click(catBtn('app'));
click(scenBtn('normal'));
let refreshBtn=[...doc.querySelectorAll('#logStatus button')].find(b=>b.dataset.logAct==='refresh');
click(refreshBtn);
assert(view().includes('正在读取应用日志'),'刷新显示加载态');
assert([...doc.querySelectorAll('#logStatus button')].find(b=>b.dataset.logAct==='refresh').disabled,'加载中刷新置灰，不叠加任务');
await new Promise(r=>setTimeout(r,900));
assert(view().includes('[manager]'),'刷新结束后回到记录');

// 8. 清理仅应用日志并有确认
click(catBtn('audit'));
assert(![...doc.querySelectorAll('#logStatus button')].some(b=>b.dataset.logAct==='clear'),'调用日志无清理入口');
click(catBtn('app'));
const clearBtn=[...doc.querySelectorAll('#logStatus button')].find(b=>b.dataset.logAct==='clear');
click(clearBtn);
assert(modalShown(),'清理弹出确认 Modal');
assert(modalBody().includes('不涉及官方请求数据'),'清理说明不涉及官方请求数据 / 审计文件');
window.closeModal&&window.closeModal();

// 9. 官方面板入口：运行中且正常加载 → 导航到面板
click(panelBtn('ok'));
click(runBtn('running'));
const navBefore=window.__nav.length;
click(entry());
assert(window.__nav.length>navBefore&&navigatedToPanel(),'运行中点击入口导航到应用内官方面板');

// 10. 代理未运行 → 只说明原因，不导航、不切分类
click(catBtn('app'));
click(runBtn('stopped'));
const catBefore=activeCat();
const navBefore2=window.__nav.length;
click(entry());
assert(modalShown(),'代理未运行时点击入口给出说明');
assert(modalBody().includes('代理未运行，请先启动后查看'),'说明文案为「代理未运行，请先启动后查看」');
assert(activeCat()===catBefore,'代理未运行不切换当前日志分类');
assert(window.__nav.length===navBefore2,'代理未运行不触发导航');

// 11. 面板加载失败 → 如实失败并可重试，不改分类
click(runBtn('running'));
click(panelBtn('fail'));
click(catBtn('audit'));
const navBefore3=window.__nav.length;
click(entry());
assert(modalShown(),'面板加载失败时给出失败反馈');
assert(modalTitle().includes('加载失败'),'失败反馈标题如实说明加载失败');
assert(modalBody().includes('不会')&&modalBody().includes('空'),'失败反馈不伪装成空日志');
assert(activeCat()==='audit','面板加载失败不影响当前本地日志分类');
assert(window.__nav.length===navBefore3,'面板加载失败不导航');
assert(!!doc.getElementById('modalConfirm'),'失败反馈提供重试按钮');
click(panelBtn('ok'));

// 12. 代码级：不再存在代理请求专属实现
const src=html;
assert(!src.includes('logProxyTable'),'已删除 logProxyTable 实现');
assert(!src.includes("LOG_CATS.proxy")&&!src.includes("proxy:{label"),'已删除代理请求分类定义');
assert(!src.includes("'unsupported'")&&!src.includes('命令不支持'),'已删除「命令不支持」查询专属状态');
assert(!src.includes('t-err-full')&&!src.includes('data-log-expand'),'详情已改为悬停浮层，不再有展开实现');
assert(!src.includes('调用审计'),'用户可见分类名已统一为「调用日志」');
assert(!src.includes('t-detail-hint'),'已删除「悬停查看」提示样式与元素');
assert(!src.includes('dotted'),'已删除虚线下划线等多余修饰');

// 13. 诊断中心第一个 Tab =「环境诊断」（2026-09-24 用户要求：Doctor 做成 Tab、标题四个字、放第一个）
const diagTabs=[...doc.querySelectorAll('.diag-tabs[aria-label="诊断分区"] button[data-diag-tab]')];
assert(diagTabs.length===3,'诊断中心共三个 Tab');
assert(diagTabs[0].dataset.diagTab==='doctor','第一个 Tab 是环境诊断');
assert(diagTabs[0].textContent.trim()==='环境诊断','第一个 Tab 标题是「环境诊断」');
assert(diagTabs[0].textContent.trim().length===4,'第一个 Tab 标题恰好四个字');
assert(diagTabs.slice(1).map(b=>b.dataset.diagTab).join(',')==='logs,notifications','其余 Tab 仍是日志历史 / 通知历史');
assert(doc.querySelectorAll('#card-diag-history .diag-view').length===3,'三个 Tab 共用一张内容卡');
assert(doc.getElementById('card-diag-doctor')===null,'不再有独立的 Doctor 卡片');
assert(!!doc.getElementById('diag-doctor'),'环境诊断面板存在且与日志历史同卡');
// 2026-09-28 用户要求：「环境诊断」是进入诊断中心后的默认落地 Tab。
assert(diagTabs[0].classList.contains('active'),'默认落地 Tab 是「环境诊断」（第一个）');
assert(!diagTabs[1].classList.contains('active')&&!diagTabs[2].classList.contains('active'),'默认不落在日志历史 / 通知历史');
assert(doc.getElementById('diag-doctor').classList.contains('active'),'默认展示环境诊断面板');
assert(!doc.getElementById('diag-logs').classList.contains('active'),'默认不展示日志历史面板');
click(diagTabs[0]);
assert(doc.getElementById('diag-doctor').classList.contains('active'),'切到环境诊断：该面板激活');
assert(!doc.getElementById('diag-logs').classList.contains('active')&&!doc.getElementById('diag-notifications').classList.contains('active'),'切到环境诊断：日志与通知面板隐藏');
assert(diagTabs[0].classList.contains('active'),'切到环境诊断：Tab 高亮跟随');
const doctorText=doc.getElementById('diag-doctor').textContent;
assert(doctorText.includes('ocx doctor')&&doctorText.includes('默认只读'),'环境诊断面板保留只读说明');
click(diagTabs[1]);
assert(doc.getElementById('diag-logs').classList.contains('active'),'切回日志历史：日志面板恢复');
assert(!doc.getElementById('diag-doctor').classList.contains('active'),'切回日志历史：环境诊断面板隐藏');
assert(doc.getElementById('doctorRunBtn')!==null,'环境诊断面板保留「环境诊断」入口按钮');

// 13b. 界面异常自检（IMP-04 §14.3）：与环境诊断同 Tab，提供「异常提示弹窗」的可复现入口。
// 2026-09-28 用户要求：该提示带恢复动作，改为居中弹窗（原为窗口顶部横条）。
click(diagTabs[0]);
const selfCheckBtn=doc.getElementById('uiSelfCheckBtn');
const faultModal=doc.getElementById('uiFaultModal');
assert(selfCheckBtn!==null,'环境诊断面板有「界面诊断」入口');
assert(selfCheckBtn.textContent.trim()==='界面诊断','自检按钮文案是「界面诊断」');
assert(doc.getElementById('doctorRunBtn').textContent.trim()==='环境诊断','Doctor 按钮用中文名「环境诊断」');
assert(faultModal.hidden===true,'异常提示弹窗初始隐藏');
click(selfCheckBtn);
assert(faultModal.hidden===false,'触发自检后异常提示弹窗出现');
assert(faultModal.getAttribute('role')==='alertdialog','异常提示是 role=alertdialog 的弹窗');
assert(faultModal.getAttribute('aria-modal')==='true','异常提示声明 aria-modal');
assert(!!doc.getElementById('uiFaultMask'),'异常提示弹窗有遮罩');
assert(!!doc.getElementById('uiFaultText'),'弹窗有异常说明文本');
const faultActs=[...faultModal.querySelectorAll('.ui-fault-actions button')].map(b=>b.textContent.trim());
assert(faultActs.join(',')==='关闭提示,重新加载界面','弹窗提供「关闭提示 / 重新加载界面」两个恢复动作');
click(doc.getElementById('uiFaultDismiss'));
assert(faultModal.hidden===true,'「关闭提示」关闭弹窗');
click(selfCheckBtn);
assert(faultModal.hidden===false,'可重复触发自检');
click(doc.getElementById('uiFaultMask'));
assert(faultModal.hidden===true,'点遮罩关闭弹窗');
click(selfCheckBtn);
assert(faultModal.hidden===false,'可再次触发自检');
click(doc.getElementById('uiFaultReload'));
assert(faultModal.hidden===true,'「重新加载界面」关闭弹窗');

// 14. 命名统一：左侧导航用 2 字短名「诊断」，全名「诊断中心」留给页面标题与入口动作
const navLabel=doc.querySelector('nav.nav button[data-route="logs"] .nav-label');
assert(!!navLabel,'左侧导航存在诊断入口');
assert(navLabel.textContent.trim()==='诊断','导航短名是「诊断」');
assert(navLabel.textContent.trim().length===2,'导航短名保持两个字（与概览 / 面板 / 拓展 / 设置同宽）');
assert(doc.querySelector('nav.nav button[data-route="logs"]').title==='诊断','导航 title 与短名一致');
assert(doc.querySelector('.route-subtitle,h2')!==null,'页面标题区存在');
assert(html.includes('打开诊断中心'),'跨入口动作用全名「打开诊断中心」');
assert(!html.includes('>打开日志<'),'不再有「打开日志」这种半截说法');

console.log(log.join('\n'));
const fails=log.filter(l=>l.startsWith('FAIL'));
console.log('\n== 捕获的 JS 错误 ==\n'+(errors.join('\n')||'(none)'));
console.log('\nTOTAL: '+log.length+'  FAIL: '+fails.length+'  JS_ERRORS: '+errors.length);
process.exit(fails.length||errors.length?1:0);
