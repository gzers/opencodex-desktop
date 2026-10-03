// 原型「OpenCodex 运行来源 / 托管安装 / 离线导入 / 两级卸载」真实浏览器（无头 Chromium）检查与截图。
// 运行：node "<本文件>"   （可选 PLAYWRIGHT_CORE=<playwright-core 目录>）
// 覆盖 2026-09-24 用户决定：只允许官方包；网络不通可导入 file.tgz（提供下载地址 + 拖拽）；
// 托管安装只写数据根内私有前缀；卸载分两级且破坏性一级必须二次确认。
// 覆盖 2026-09-25 用户决定：安装弹窗像真实安装器——向导提示不放大、不显示「原型演示」脚注、
// 演示用开包开关移到右侧测试器（外面）。
// 覆盖 2026-09-25 追加：安装后进入进度视图——顶部进度条 + 联网显示命令行明细（含 HTTP / SOCKS5 代理）、
// 离线只显示包体安装进度（不给命令行）。
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
    const up=path.dirname(d); if(up===d) break; d=up;
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
const outDir=path.join(here,'..','..','文档','截图','原型-托管安装-20260924');
fs.mkdirSync(outDir,{recursive:true});
const url=hash=>pathToFileURL(proto).href+'?theme=light'+hash;

const results=[];
const check=(c,m)=>results.push((c?'PASS':'FAIL')+' '+m);
const browser=await chromium.launch({executablePath:exe});
const page=await browser.newPage({viewport:{width:1692,height:916},deviceScaleFactor:2});
const consoleErrors=[];
page.on('console',m=>{ if(m.type()==='error') consoleErrors.push(m.text()); });
page.on('pageerror',e=>consoleErrors.push('pageerror: '+e.message));
const shot=async(name,sel)=>{
  const el=await page.$(sel||'body');
  if(!el){ check(false,'截图目标缺失: '+(sel||'body')); return; }
  await el.scrollIntoViewIfNeeded().catch(()=>{});
  await page.waitForTimeout(150);
  const box=await el.boundingBox();
  if(!box){ check(false,'截图目标不可见: '+(sel||'body')); return; }
  // 用页面坐标裁图：元素截图会做 actionability 检查，原型里部分区域（缩放 + 折叠）
  // 会被判成「不可见」，这里改成按 boundingBox 裁剪，同时先断言可见性。
  check(true,'截图目标可见: '+name);
  const off=await page.evaluate(()=>({x:window.scrollX,y:window.scrollY}));
  await page.screenshot({path:path.join(outDir,name),clip:{
    x:box.x+off.x,y:box.y+off.y,width:box.width,height:box.height}});
};
async function boot(hash){ await page.goto('about:blank'); await page.goto(url(hash)); await page.waitForTimeout(1100); }

// ── 0. 概览未安装引导（先做：不依赖前面任何状态） ──
await boot('#overview');
await page.evaluate(()=>window.setEnv('missing_ocx'));
await page.waitForTimeout(400);
check(await page.$eval('#envGate',el=>!el.hidden),'缺 OpenCodex 时门禁卡片可见');
const gate=await page.textContent('#envGate');
check(gate.includes('安装 OpenCodex'),'缺 OpenCodex 时门禁给出「安装 OpenCodex」');
check(gate.includes('导入离线包'),'缺 OpenCodex 时门禁给出「导入离线包」');
// 已确认原型把门禁脚注文案改为面向用户的「安装仅写管理器数据目录」，
// 不再出现内部路径 runtime/opencodex；此处按当前文案的语义断言。
check(/仅写[^。；\n]{0,16}数据目录/.test(gate),'门禁说明托管安装只写数据根内');
check(!gate.includes('ocx --version'),'不再出现依赖 PATH 的校验命令');
await shot('12-概览-缺OpenCodex引导.png','#envGate');

// ── 1. 安装配置：运行来源卡片 ──
await boot('#settings?section=installation');
check((await page.textContent('#runtimeSourceTag')).trim()==='自动发现','默认来源标签是「自动发现」');
const facts=await page.textContent('#runtimeSourceBody');
check(facts.includes('~/.local/bin/ocx'),'显示可执行文件路径');
check(facts.includes('@bitkyc08/opencodex'),'显示官方包名');
check(facts.includes('应用不读取 PATH'),'显示「不读取 PATH」这一事实');
check(facts.includes('runtime/opencodex'),'显示托管安装的目标前缀');
check((await page.$('[data-prototype-action="runtime-install"]'))!==null,'有安装入口');
check((await page.$('[data-prototype-action="runtime-import"]'))!==null,'有离线导入入口');
// 2026-09-28 用户决定：任意「已解析来源」都可卸载，不再只在托管安装下出现。
check((await page.$('[data-prototype-action="runtime-uninstall"]'))!==null,'自动发现来源下也出现卸载入口');
await shot('01-运行来源-自动发现.png','#card-runtime-source');

// 切到托管安装：出现更新 / 卸载
await page.click('#runtimeStates button[data-runtime="managed"]');
await page.waitForTimeout(250);
check((await page.textContent('#runtimeSourceTag')).trim()==='托管安装','可切到托管安装');
check((await page.$('[data-prototype-action="runtime-uninstall"]'))!==null,'托管来源下出现卸载入口');
check((await page.$('[data-prototype-action="runtime-update"]'))!==null,'托管来源下出现更新入口');
await shot('02-运行来源-托管安装.png','#card-runtime-source');
await page.click('#runtimeStates button[data-runtime="missing"]');
await page.waitForTimeout(250);
check((await page.textContent('#runtimeSourceBody')).includes('没有可用来源'),'未解析来源给出可读解释与出口');
await shot('03-运行来源-未解析.png','#card-runtime-source');

// ── 2. 安装弹窗：安装器形态 + 自定义安装位置 ──
await page.click('#runtimeStates button[data-runtime="auto"]');
await page.click('[data-prototype-action="runtime-install"]');
await page.waitForTimeout(400);
const wizard=await page.textContent('#modalBody');
check(wizard.includes('安装位置')&&wizard.includes('安装源')&&wizard.includes('确认'),'安装弹窗是步骤式（安装位置 / 安装源 / 确认）');
check((await page.$('#modalBody .modal-prototype-note'))===null,'安装弹窗内不再显示「原型演示」脚注（移到外面）');
check(!wizard.includes('原型演示'),'安装弹窗内不再出现原型演示控件（按钮移到外面）');
check((await page.$('#installDemo'))!==null,'演示开关放在右侧测试器（外面）');
const hintSize=await page.$eval('#installHost .wiz-hint',el=>parseFloat(getComputedStyle(el).fontSize));
check(hintSize<=11,'安装位置提示保持向导小字（'+hintSize+'px ≤ 11px），不被弹窗正文放大');
check(wizard.includes('安装进度'),'步骤条有第 4 步「安装进度」');
check((await page.$('#installProxyType'))!==null&&(await page.$('#installProxyAddr'))!==null,'联网安装提供代理配置（协议 + 地址）');
const proxyOpts=await page.$$eval('#installProxyType option',els=>els.map(o=>o.value));
check(proxyOpts.includes('http')&&proxyOpts.includes('socks5'),'代理协议含 HTTP 与 SOCKS5');
check(wizard.includes('--proxy'),'代理说明写明以 --proxy 传给 npm');
check(await page.$eval('#installPathInput',el=>el.value)==='~/OpenCodexData/runtime/opencodex','安装位置默认是数据根内的私有前缀');
check(await page.$eval('#installPathInput',el=>!el.readOnly&&!el.disabled),'安装位置文本框可编辑（可覆盖）');
check((await page.$('[data-prototype-action="install-choose-path"]'))!==null,'有「选择…」入口（系统目录选择器）');
check(await page.$eval('[data-prototype-action="install-path-reset"]',el=>el.disabled),'默认值下「恢复默认」禁用');
check(!(await page.$eval('.wiz-more',el=>el.open)),'说明 / 下载地址默认折叠（不抢视线）');
check(wizard.includes('registry.npmjs.org'),'折叠区里有官方下载地址（tarball 直链）');
check(wizard.includes('不写系统目录'),'确认项写明不写系统目录');
check(await page.$eval('#modalConfirm',el=>el.disabled),'未勾选确认前主行动禁用');
await shot('04-安装弹窗-安装器-默认.png','#modal');

await page.click('[data-prototype-action="install-choose-path"]');
await page.waitForTimeout(300);
check(await page.$eval('#installPathInput',el=>el.value)==='~/dev/opencodex-runtime','「选择…」可把位置改到数据根外');
check((await page.textContent('#modalBody')).includes('不在数据根内'),'数据根外落点有明确提示');
await shot('05-安装弹窗-自定义位置.png','#modal');

await page.fill('#installPathInput','/usr/local/lib/opencodex');
await page.waitForTimeout(300);
check((await page.textContent('#modalBody')).includes('系统保护目录'),'系统保护目录给出可读拒绝原因');
check(await page.$eval('#modalConfirm',el=>el.disabled),'系统保护目录下主行动禁用');
await shot('06-安装弹窗-系统目录被拒.png','#modal');

await page.click('[data-prototype-action="install-path-reset"]');
await page.waitForTimeout(250);
await page.click('#installAck');
await page.waitForTimeout(250);
check(!(await page.$eval('#modalConfirm',el=>el.disabled)),'位置合法且已勾选确认后主行动可用');

await page.click('input[name="installSrc"][value="file"]');
await page.waitForTimeout(300);
check((await page.$('#dropzone'))!==null,'选「导入离线包」后出现拖拽区');
check((await page.$('#installProxyType'))===null,'离线导入时不再显示代理配置');
check((await page.textContent('#dropzone')).includes('opencodex-2.64.0.tgz'),'拖拽区写明期望文件名');
check(await page.$eval('#modalConfirm',el=>el.disabled),'离线模式下未导入包时主行动禁用');
await shot('07-安装弹窗-离线拖拽默态.png','#modal');
await page.click('#installDemo [data-prototype-action="drop-demo-bad"]');
await page.waitForTimeout(250);
check(await page.$eval('#dropzone',el=>el.classList.contains('bad')),'非官方包进入拒绝态');
check((await page.textContent('#dropzone')).includes('拒绝'),'非官方包给出可读拒绝原因');
await shot('08-安装弹窗-离线非官方包.png','#modal');
await page.click('#installDemo [data-prototype-action="drop-demo-good"]');
await page.waitForTimeout(250);
check(await page.$eval('#dropzone',el=>el.classList.contains('picked')),'合法包进入已选态');
check((await page.textContent('#dropzone')).includes('SHA-256'),'已选态显示 SHA-256');
check(!(await page.$eval('#modalConfirm',el=>el.disabled)),'导入合法包后主行动可用');
await shot('09-安装弹窗-离线已选.png','#modal');

// ── 2b. 安装进度：离线只给包体进度，不给命令行 ──
await page.click('#modalConfirm');
await page.waitForTimeout(300);
check((await page.$('.wiz-progress-bar'))!==null,'开始安装后出现顶部进度条');
check((await page.$('#installConsole'))===null,'离线安装不显示命令行明细');
check((await page.$('.wiz-substeps'))!==null,'离线安装显示包体分包进度');
check((await page.textContent('#modalConfirm')).trim()==='安装中…'&&await page.$eval('#modalConfirm',el=>el.disabled),'安装进行中主行动变为「安装中…」且禁用');
check((await page.textContent('#modalCancel')).trim()==='取消安装','安装进行中取消按钮变为「取消安装」');
check((await page.textContent('#modalBody')).includes('导入离线包 · '),'进度视图标明离线导入与百分比');
await page.waitForTimeout(900);
await shot('13-安装进度-离线.png','#modal');
await page.waitForTimeout(3200);
check((await page.textContent('#modalBody')).includes('安装完成'),'离线安装走到完成态');
check((await page.textContent('#modalBody')).includes('运行来源已切到'),'完成态说明运行来源已切换');
check((await page.textContent('#modalConfirm')).trim()==='完成','完成态主行动变为「完成」');
await shot('14-安装完成-离线.png','#modal');
await page.click('#modalConfirm');
await page.waitForTimeout(300);
check(await page.$eval('#modalMask',el=>getComputedStyle(el).display)==='none','点「完成」关闭弹窗');
check((await page.textContent('#runtimeSourceTag')).trim()==='托管安装','安装完成后运行来源切到「托管安装」');

// ── 2c. 安装进度：联网给命令行明细，代理传 --proxy ──
await page.click('[data-prototype-action="runtime-install"]');
await page.waitForTimeout(350);
await page.selectOption('#installProxyType','socks5');
await page.fill('#installProxyAddr','127.0.0.1:1080');
await page.waitForTimeout(200);
await page.click('#installAck');
await page.waitForTimeout(200);
await page.click('#modalConfirm');
await page.waitForTimeout(400);
check((await page.$('#installConsole'))!==null,'联网安装显示命令行明细');
check((await page.$('.wiz-substeps'))===null,'联网安装不再重复渲染色块分包列表');
const consoleText=await page.textContent('#installConsole');
check(consoleText.includes('npm install @bitkyc08/opencodex'),'命令行明细含 npm 安装命令');
check(consoleText.includes('--proxy socks5h://127.0.0.1:1080'),'SOCKS5 代理以 socks5h:// 传给 npm');
await page.waitForTimeout(1400);
await shot('15-安装进度-联网.png','#modal');
await page.waitForTimeout(2600);
check((await page.textContent('#installConsole')).includes('运行来源已切换为'),'联网安装命令行明细走到收尾行');
await page.click('#modalConfirm');
await page.waitForTimeout(300);

// ── 3. 卸载弹窗：任意来源可卸载 · 完整 / 仅包体两级 · 备份只提醒 · 残留核验 ──
// 2026-09-28 用户决定：只要查出来允许来源都允许卸载；卸载要干净；备份只检测与提醒、不阻断。
await page.click('#runtimeStates button[data-runtime="auto"]');
await page.waitForTimeout(250);
await page.click('[data-prototype-action="runtime-uninstall"]');
await page.waitForTimeout(400);
const plan=await page.textContent('#modalBody');
check(plan.includes('完整卸载'),'卸载弹窗给出「完整卸载」范围');
check(plan.includes('仅移除包体与入口'),'卸载弹窗给出「仅移除包体与入口」范围');
check(plan.includes('npm 全局前缀'),'自动发现来源按 npm 全局前缀给出方案');
check(plan.includes('npm uninstall -g @bitkyc08/opencodex'),'外部包体给出将执行的官方卸包命令');
check((await page.$('#uninstallBackup'))!==null&&(await page.textContent('#uninstallBackup')).includes('未检测到可恢复备份'),'无备份时给出提醒');
check((await page.$('#uninstallAutoBackup'))!==null&&await page.$eval('#uninstallAutoBackup',el=>el.checked),'无备份时默认勾选「卸载前自动生成备份」');
check((await page.$('#uninstallAck'))!==null,'卸载要求显式确认');
check(await page.$eval('#modalConfirm',el=>el.disabled),'未勾选确认前主行动禁用');
// 2026-09-28 用户反馈「卸载按钮点不动」：确认项必须与按钮同时可见，否则用户看不到为何灰着。
const ackGeom=await page.evaluate(()=>{
  const a=document.getElementById('uninstallAckChoice').getBoundingClientRect();
  const m=document.getElementById('modal').getBoundingClientRect();
  return {ackTop:Math.round(a.top),ackBottom:Math.round(a.bottom),mTop:Math.round(m.top),mBottom:Math.round(m.bottom)};
});
check(ackGeom.ackTop>=ackGeom.mTop&&ackGeom.ackBottom<=ackGeom.mBottom,'确认项在弹窗可见区内（不滚动也能看到）');
check((await page.$('.ack-hint'))!==null&&!(await page.$eval('.ack-hint',el=>el.hidden)),'主行动禁用时给出可读原因');
// 2026-09-28 用户要求「不想要弹窗滚动条」：默认窗口下正文不应出现滚动条。
const cfgOverflow=await page.$eval('#modalBody',el=>el.scrollHeight-el.clientHeight);
check(cfgOverflow<=0,'确认视图正文无滚动条（溢出 '+cfgOverflow+'px）');
check((await page.textContent('#uninstallBodyNote')).includes('由应用代执行'),'说明写明外部包体由应用代执行');
await shot('10-卸载弹窗-完整卸载-无备份.png','#modal');
// 备份只提醒、不阻断：有备份切提示文案（不改主行动可用性）
await page.click('#uninstallDemo button[data-uninstall-backup="exists"]');
await page.waitForTimeout(250);
check((await page.textContent('#uninstallBackup')).includes('已检测到可恢复备份'),'有备份时给出复用提示');
check((await page.$('#uninstallReuseBackup'))!==null,'有备份时提供「复用这份备份」');
await shot('10b-卸载弹窗-已检测到备份.png','#modal');
await page.click('#uninstallDemo button[data-uninstall-backup="none"]');
await page.waitForTimeout(200);
// 切「仅移除包体与入口」：运行态一行消失
await page.click('input[name="uninstallScope"][value="body"]');
await page.waitForTimeout(250);
check(!(await page.textContent('.removal-list')).includes('由官方'),'仅移除包体时不列出运行态（service / shim / config）');
check(await page.$eval('#uninstallData',el=>el.disabled),'仅移除包体时 OPENCODEX_HOME 清理项禁用（不在该范围）');
await shot('11-卸载弹窗-仅移除包体.png','#modal');
// 未勾选确认 → 主行动保持禁用（不进入进度）
check(await page.$eval('#modalConfirm',el=>el.disabled),'未勾选确认时主行动保持禁用');
check((await page.$('.wiz-substeps'))===null,'未确认时不会进入卸载进度');
// 勾选后走完整卸载进度 → 残留核验
await page.click('input[name="uninstallScope"][value="full"]');
await page.waitForTimeout(200);
await page.click('#uninstallAck');
await page.waitForTimeout(200);
check(!(await page.$eval('#modalConfirm',el=>el.disabled)),'勾选确认后主行动可用');
check(await page.$eval('.ack-hint',el=>el.hidden),'勾选后禁用原因提示消失');
await page.click('#modalConfirm');
await page.waitForTimeout(300);
check((await page.$('.wiz-substeps'))!==null,'卸载开始后进入进度视图');
check((await page.textContent('#modalConfirm')).trim()==='卸载中…','卸载进行中主行动变为「卸载中…」');
check((await page.$('#uninstallConsole'))!==null,'完整卸载显示命令行明细（官方 ocx uninstall 输出口径）');
check((await page.textContent('#uninstallConsole')).includes('ocx uninstall'),'命令行明细含官方 ocx uninstall');
check((await page.$('.wiz-progress-fill.run'))!==null,'进行中进度条带动效类（run）');
check((await page.$('.wiz-substeps li.on, .wiz-substeps li.ok'))!==null,'进行中有当前步骤态');
const consoleH1=Math.round(await page.$eval('#uninstallConsole',el=>el.getBoundingClientRect().height));
await page.waitForTimeout(5200);
const uRes=await page.textContent('#modalBody');
check(uRes.includes('卸载完成'),'卸载走到完成态');
check((await page.$('#uninstallResidue'))!==null,'完成态给出「残留核验」');
check((await page.$('.residue-list li'))!==null,'残留核验逐项列出扫描对象');
check((await page.textContent('#uninstallResidue')).includes('由应用执行'),'残留核验说明外部包体由应用执行移除');
check((await page.$('.residue-badge'))!==null,'完成态有绿色通过标记');
const consoleH2=Math.round(await page.$eval('#uninstallConsole',el=>el.getBoundingClientRect().height));
check(consoleH1===consoleH2 && consoleH1>0,'命令明细框高度固定（'+consoleH1+'px → '+consoleH2+'px）');
const doneOverflow=await page.$eval('#modalBody',el=>el.scrollHeight-el.clientHeight);
check(doneOverflow<=0,'完成视图正文无滚动条（溢出 '+doneOverflow+'px）');
await shot('11b-卸载完成-残留核验.png','#modal');
await page.click('#modalConfirm');
await page.waitForTimeout(300);
check(await page.$eval('#modalMask',el=>getComputedStyle(el).display)==='none','点「完成」关闭弹窗');


await browser.close();
console.log(results.join('\n'));
console.log('\n== console errors ==');
console.log(consoleErrors.length?consoleErrors.join('\n'):'(none)');
const fails=results.filter(r=>r.startsWith('FAIL')).length;
console.log(`\nTOTAL: ${results.length}  FAIL: ${fails}  CONSOLE_ERRORS: ${consoleErrors.length}`);
console.log('shots -> '+outDir);
process.exit(fails||consoleErrors.length?1:0);
