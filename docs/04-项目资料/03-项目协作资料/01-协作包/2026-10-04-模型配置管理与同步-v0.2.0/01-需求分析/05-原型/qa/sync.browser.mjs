import {repoRoot as root, prototypeRoot} from '../../../../2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/原型/qa/prototype-location.mjs';
// 原型 QA：只访问静态本机 HTML，mock 不连接渠道或写入运行配置。
// node 本文件；可用 PLAYWRIGHT_CORE / CHROMIUM_EXECUTABLE 指定既有运行环境。
import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath, pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';
const here=path.dirname(fileURLToPath(import.meta.url));
const shared=prototypeRoot;
const old=path.join(root,'docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型');
const npx=path.join(os.homedir(),'.npm/_npx'),cache=path.join(os.homedir(),'Library/Caches/ms-playwright');
const pw=[process.env.PLAYWRIGHT_CORE,...(fs.existsSync(npx)?fs.readdirSync(npx).map(d=>path.join(npx,d,'node_modules/playwright-core')):[])].find(p=>p&&fs.existsSync(path.join(p,'package.json')));
const exe=[process.env.CHROMIUM_EXECUTABLE,...(fs.existsSync(cache)?fs.readdirSync(cache).filter(d=>d.startsWith('chromium_headless_shell-')).sort().reverse().map(d=>path.join(cache,d,'chrome-headless-shell-mac-arm64/chrome-headless-shell')):[])].find(p=>p&&fs.existsSync(p));
if(!pw||!exe){console.error('BLOCKED：需要既有 playwright-core / Chromium；本脚本不会安装依赖。');process.exit(2);}
const {chromium}=createRequire(pw+'/')('playwright-core');
const browser=await chromium.launch({executablePath:exe});
const page=await browser.newPage({viewport:{width:1692,height:916}}),errors=[],results=[];
page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
page.setDefaultTimeout(6000);

const entry=pathToFileURL(path.join(shared,'原型/index.html')).href;
// 原型工具板结构（两顶层折叠段 + 各页面 / Tab 专属 Mock）证据独立输出，保留历史结果与截图。
const evidence=path.join(here,'../配置同步/原型工具板结构验收');
const shots=path.join(evidence,'截图');fs.mkdirSync(shots,{recursive:true});
const sx=()=>page.evaluate(()=>syncPrototype.state());
const act=a=>page.locator('[data-sx="'+a+'"]:visible').first().click();
const expand=async id=>{const b=page.locator('[data-sx="expand"][data-id="'+id+'"]');if(await b.getAttribute('aria-expanded')!=='true')await b.click();};
const confirm=()=>page.locator('#modalConfirm').click();
const close=()=>page.evaluate(()=>closeModal());
const select=(selector,value)=>page.locator(selector).selectOption(value,{force:true});
const reset=async(route='sync')=>{await page.goto('about:blank');await page.goto(entry+'?theme=light#'+route);await page.evaluate(()=>setWinSize(1180,760));await page.waitForTimeout(70);};
const open=options=>page.evaluate(options=>syncPrototype.open(options),options);
const expandAll=async()=>{for(const id of await page.locator('[data-sx="expand"]').evaluateAll(items=>items.map(e=>e.dataset.id)))await expand(id);};
const compareExpand=async id=>{const b=page.locator('[data-sx="compare-expand"][data-id="'+id+'"]');if(await b.getAttribute('aria-expanded')!=='true')await b.click();};
const compareExpandAll=async()=>{for(const id of await page.locator('[data-sx="compare-expand"]').evaluateAll(items=>items.map(e=>e.dataset.id)))await compareExpand(id);};
const sample=async(kind,expanded=true)=>{await select('#sxImportSample',kind);await act('import');await act('choose-import-file');if(kind==='encrypted')await page.locator('#sxUnlock').fill('demo-sync');await confirm();if(expanded)await compareExpandAll();};
async function check(name,fn){try{await reset();await fn();results.push({name,status:'PASS'});}catch(e){results.push({name,status:'FAIL',error:e.message});await close();}console.log(results.at(-1).status+' '+name);}
try{
 await check('S-P01 独立同步菜单、两个 Tab 与账户登录状态默认联动',async()=>{
  assert.equal(await page.locator('button[data-route="sync"] .nav-label').innerText(),'同步');assert.equal(await page.locator('#syncContent [role=tab]').count(),2);
  assert.equal((await sx()).tab,'file');assert((await sx()).selected.includes('channel:legacy'));assert((await sx()).selected.includes('endpoint:personal'));assert.deepEqual((await sx()).secrets.endpointPasswords,['endpoint:personal']);assert.deepEqual((await sx()).secrets.accounts,['account:chatgpt']);assert.deepEqual((await sx()).secrets.keys,[]);assert.deepEqual((await sx()).secrets.extensions,[]);assert.equal(await page.locator('[data-sx-secret="accounts"]').count(),0);
  assert.equal(await page.locator('[data-sx="expand"][aria-expanded="false"]').count(),7);assert.equal(await page.locator('[data-sx-object]').count(),0);
  await page.locator('.window').screenshot({path:path.join(shots,'01-文件同步-1180-light.png')});
  await open({tab:'webdav'});assert.equal(await page.locator('[data-sx="expand"][aria-expanded="false"]').count(),6);assert.equal(await page.locator('[data-sx-object]').count(),0);
 });
 await check('S-P02 旧迁移 / WebDAV 路由归并，设置仍有模型项',async()=>{
  for(const [route,expected] of [['settings?section=migration','file'],['settings?section=sync','webdav'],['webdav','webdav']]){await reset(route);assert.equal((await sx()).tab,expected);assert(await page.locator('#route-sync').isVisible());}
  await reset('settings');assert.equal(await page.locator('#settings-migration,#settings-sync').count(),0);assert.equal(await page.locator('#settings-models').count(),1);
 });
 await check('S-P03 双 Tab 键盘与各自的选择范围',async()=>{
  await act('clear');await page.locator('#sxTab-file').focus();await page.keyboard.press('ArrowRight');assert.equal((await sx()).tab,'webdav');assert((await sx()).selected.length>0);
  await page.keyboard.press('Home');assert.equal((await sx()).selected.length,0);assert.equal(await page.locator('#sxTab-file').evaluate(e=>e===document.activeElement),true);
 });
 await check('S-P04 搜索保留隐藏勾选，分组半选',async()=>{
  const before=(await sx()).selected;await page.locator('#sxSearch').fill('NewApi');assert.deepEqual((await sx()).selected,before);
  await page.locator('[data-sx-object="channel:newapi"]').uncheck();assert((await sx()).selected.includes('channel:a6api'));assert.equal(await page.locator('[data-sx-object="channel:newapi"]').evaluate(e=>e===document.activeElement),true);
  await page.locator('#sxSearch').fill('');assert.equal(await page.locator('[data-sx-group="channels"]').evaluate(e=>e.indeterminate),true);
 });
 await check('S-P05 所选渠道补齐精确模版依赖，不带凭据',async()=>{
  await open({scope:['channel:newapi']});assert((await sx()).effective.includes('template:gpt'));await expand('templates');
  assert.equal(await page.locator('[data-sx-object="template:gpt"]').isDisabled(),true);
  await act('export');await confirm();assert((await sx()).lastExport.dependencies.some(x=>x.id==='template:gpt'&&x.revisions.length>0));assert.equal((await sx()).lastExport.encryptionRequired,false);assert.equal((await sx()).lastExport.source,'committed');
 });
 await check('S-P06 对象详情与设备绑定登录态说明',async()=>{
  await expand('channels');await page.locator('[data-sx="detail"][data-id="channel:newapi"]').click();assert.match(await page.locator('#modalBody').innerText(),/已选模型/);await close();await expand('accounts');await page.locator('[data-sx="detail"][data-id="account:device"]').click();assert.match(await page.locator('#modalBody').innerText(),/需重新登录/);assert.equal(await page.locator('#modalConfirm').isVisible(),false);
 });
 await check('S-P07 分享预设后另选 Key，含凭据强制加密 / 口令校验',async()=>{
  await select('[data-sx-preset]','share');await page.locator('[data-sx-secret="keys"]').check();assert((await sx()).secrets.keys.length>0);assert.equal((await sx()).secrets.accounts.length,0);
  await page.locator('[data-sx-secret-object="channel:newapi"]').uncheck();assert.equal(await page.locator('[data-sx-secret="keys"]').evaluate(e=>e.indeterminate),true);
  await act('export');assert.equal(await page.locator('#sxEncrypt').isDisabled(),true);await confirm();assert.match(await page.locator('#sxError').innerText(),/8 位/);
  await page.locator('#sxPass').fill('demo-sync');await page.locator('#sxPassAgain').fill('mismatch');await confirm();assert.equal((await sx()).lastExport,null);
  await page.locator('#sxPassAgain').fill('demo-sync');assert.equal(await page.locator('#sxError').innerText(),'');await page.locator('#modal').screenshot({path:path.join(shots,'03-含Key导出确认.png')});await confirm();assert.equal((await sx()).lastExport.encrypted,true);assert.deepEqual((await sx()).lastExport.credentials.accounts,[]);
 });
 await check('S-P08 换设备预设排除设备绑定登录态，取消对象清理凭据选择',async()=>{
  await select('[data-sx-preset]','device');assert.deepEqual((await sx()).secrets.accounts,['account:chatgpt']);assert.deepEqual((await sx()).secrets.endpointPasswords,['endpoint:personal']);
  await expand('accounts');await page.locator('[data-sx-object="account:chatgpt"]').uncheck();assert.deepEqual((await sx()).secrets.accounts,[]);
  await select('[data-sx-preset]','all');assert.deepEqual((await sx()).secrets.accounts,['account:chatgpt']);assert.deepEqual((await sx()).secrets.keys,[]);assert.deepEqual((await sx()).secrets.extensions,[]);
  await select('[data-sx-preset]','share');assert(Object.values((await sx()).secrets).every(x=>!x.length));
 });
 await check('S-P09 损坏包 / 解密错误不创建应用结果',async()=>{
  await act('import');await act('choose-import-file');await page.locator('#sxUnlock').fill('wrong');await confirm();assert.match(await page.locator('#sxError').innerText(),/不正确/);assert.equal((await sx()).serial,0);
  await close();await select('#sxImportSample','corrupt');await act('import');await act('choose-import-file');await confirm();assert.match(await page.locator('#sxError').innerText(),/完整性校验失败/);assert.equal((await sx()).serial,0);
 });
 await check('S-P10 无 Key 导入、依赖、同名异端点与冲突预览',async()=>{
  const before=await page.evaluate(()=>modelPrototype.state());await sample('keyless');await confirm();assert.match(await page.locator('#sxError').innerText(),/身份映射/);
  assert.equal(await page.locator('[data-sx-import-pick="template:gpt"]').isDisabled(),true);assert.equal(await page.locator('[data-sx-import-secret]').count(),0);
  assert.equal(await page.locator('[data-sx-import-choice="channel:newapi"]').count(),0);await select('[data-sx-import-choice="channel:legacy"]','new');
  await page.locator('#modal').screenshot({path:path.join(shots,'04-导入冲突预览.png')});await confirm();assert.match((await sx()).result.title,/演示完成/);assert.deepEqual(await page.evaluate(()=>modelPrototype.state()),before);
 });
 await check('S-P11 导入账户默认恢复检查，详情可排除登录状态',async()=>{
  await sample('encrypted');await page.locator('[data-sx-import-pick="channel:newapi"]').uncheck();await page.locator('[data-sx-import-pick="channel:legacy"]').uncheck();
  assert.equal(await page.locator('[data-sx-import-secret="login"]').count(),0);assert.equal(await page.locator('[data-sx-import-account-info="account:chatgpt"]').isChecked(),false);
  await confirm();assert.equal((await sx()).result.canVerify,false);assert.deepEqual((await sx()).result.accounts.map(x=>x.status),['restored','needs-login']);assert.match((await sx()).result.detail,/自动检查通过（模拟）/);
  await sample('encrypted');await page.locator('[data-sx-import-pick="channel:newapi"]').uncheck();await page.locator('[data-sx-import-pick="channel:legacy"]').uncheck();
  await page.locator('[data-sx-import-account-info="account:chatgpt"]').check();assert.equal(await page.locator('[data-sx-import-account-info]').locator('..').locator('..').getAttribute('class'),'sx-compare-ops');assert.equal(await page.locator('[data-sx-import-account-info]').evaluate(e=>e===document.activeElement),true);
  await confirm();assert.equal((await sx()).result.accounts[0].status,'info-only');assert.match((await sx()).result.detail,/保留本机已有登录状态/);
 });
 await check('S-P12 旧版包范围有限，不冒充完整 Skill / MCP',async()=>{
  await sample('legacy');assert.equal(await page.locator('[data-sx-import-pick]').count(),2);assert.match(await page.locator('#modalBody').innerText(),/不冒充完整 Skill/);await confirm();assert.match((await sx()).result.title,/演示完成/);
 });
 await check('S-P13 WebDAV 范围变更先保存，清空不意味着远端删除',async()=>{
  await open({tab:'webdav'});await expand('channels');await page.locator('[data-sx-object="channel:newapi"]').uncheck();assert((await sx()).dirty);assert.equal(await page.locator('[data-sx="check"]').isDisabled(),true);
  await act('save-scope');assert.equal((await sx()).dirty,false);assert.match((await sx()).result.detail,/未选远端对象保留/);
  await act('clear');await act('save-scope');await act('check');await compareExpandAll();assert.match((await sx()).result.title,/尚未选择/);
 });
 await check('S-P14 WebDAV 只比较所选项，凭据关闭不出现登录态冲突',async()=>{
  await open({tab:'webdav',scope:['mcp:docs']});await act('save-scope');await act('check');await compareExpandAll();assert.equal(await page.locator('.sx-compare-item').count(),1);assert.match(await page.locator('#modalBody').innerText(),/Docs MCP/);assert.equal(await page.locator('#sxSyncPass').count(),0);await confirm();
 });
 await check('S-P15 登录态自动核对，冲突选择与独立加密口令仍必需',async()=>{
  await open({tab:'webdav',scope:['account:chatgpt']});await act('save-scope');await act('check');await compareExpandAll();assert.equal(await page.locator('[data-sx="verify-token"]').count(),0);assert.match(await page.locator('#sxTokenCheck').innerText(),/已自动检查/);
  await confirm();assert.match(await page.locator('#sxError').innerText(),/冲突/);await select('[data-sx-diff-choice="account:chatgpt"]','remote');await confirm();assert.match(await page.locator('#sxError').innerText(),/同步口令/);
  await page.locator('#sxSyncPass').fill('demo-sync');await confirm();assert.equal((await sx()).result.canVerify,false);assert.equal((await sx()).result.accounts[0].status,'restored');assert.match((await sx()).result.detail,/自动检查通过（模拟）/);
 });
 await check('S-P16 连接配置校验与按钮文案，密码不进入状态',async()=>{
  await open({tab:'webdav'});await act('connection');assert.equal(await page.locator('[data-sx="test-connection"]').innerText(),'测试连接');assert.equal(await page.locator('#modalConfirm').innerText(),'保存连接');await page.locator('#sxDavURL').fill('https://user:pass@dav.example.com');await confirm();assert.match(await page.locator('#sxError').innerText(),/内嵌凭据/);
  await page.locator('#sxDavURL').fill('https://dav.example.com');await act('test-connection');assert.match(await page.locator('#sxConnectionTest').innerText(),/未发起网络请求/);await confirm();
  await page.locator('.window').screenshot({path:path.join(shots,'02-WebDAV同步-1180-light.png')});
 });
 await check('S-P17 备份失败 / 部分失败不报告同步成功',async()=>{
  for(const scenario of ['backup-fail','partial']){await page.evaluate(s=>syncPrototype.setScenario(s),scenario);await sample('legacy');await confirm();assert.match((await sx()).result.title,scenario==='partial'?/部分完成/:/未应用/);assert.equal((await sx()).result.canVerify,false);}
 });
 await check('S-P18 取消导入与 Escape 不应用，焦点回到入口',async()=>{
  const trigger=page.locator('[data-sx="import"]');await trigger.focus();await page.keyboard.press('Enter');await page.keyboard.press('Escape');assert.equal((await sx()).serial,0);assert.equal(await trigger.evaluate(e=>e===document.activeElement),true);
 });
 await check('S-P19 右栏样例键盘选择、弹窗 Tab 循环',async()=>{
  const picker=page.locator('#sxImportSample');await picker.focus();await page.keyboard.press('ArrowDown');await page.keyboard.press('Enter');assert.equal(await picker.inputValue(),'keyless');
  await act('import');await act('choose-import-file');await page.locator('#modalConfirm').focus();await page.keyboard.press('Tab');assert.equal(await page.locator('#modal').evaluate(e=>e.contains(document.activeElement)),true);
  assert.equal(await page.locator('#modalBody #sxImportSample').count(),0);assert.equal(await page.locator('#sxUnlockField').isVisible(),false);
 });
 await check('S-P20 渠道快捷导出转入同步，保留草稿',async()=>{
  await reset('models');await page.evaluate(()=>modelPrototype.scene('pending-draft'));const before=await page.evaluate(()=>modelPrototype.state().draft);assert.notDeepEqual(before,await page.evaluate(()=>modelPrototype.state().saved));await page.locator('[data-mp="more"]:visible').first().click();await page.locator('#modalBody [data-mp="export"]').click();assert.equal((await sx()).dialogKind,'export');assert((await sx()).selected.every(x=>x.startsWith('channel:')));assert.deepEqual(await page.evaluate(()=>modelPrototype.state().draft),before);
 });
 await check('S-P21 模版快捷导入转入同步并预选模版范围',async()=>{
  await reset('models');await page.locator('[data-mp="tab"][data-tab="templates"]').click();await page.locator('[data-mp="more"]:visible').first().click();await page.locator('#modalBody [data-mp="import"]').click();assert.equal((await sx()).dialogKind,'import');assert((await sx()).selected.every(x=>x.startsWith('template:')));
  await page.evaluate(()=>syncPrototype.setImportSample('keyless'));await act('choose-import-file');await confirm();assert.equal(await page.locator('[data-sx-import-pick="channel:newapi"]').isChecked(),false);
 });
 await check('S-P22 浅深色与 1180 / 900 窗口内独立滚动',async()=>{
  for(const mode of ['file','webdav'])for(const theme of ['light','dark'])for(const [w,h] of [[1180,760],[900,600]]){
   await open({tab:mode});await page.evaluate(({theme,w,h})=>{applyTheme(theme);setWinSize(w,h);}, {theme,w,h});
   await select('[data-sx-preset]','device');await expandAll();if(mode==='webdav')await act('save-scope');
   for(const target of ['.main','#syncContent','.sx-table','.sx-secrets'])assert.equal(await page.locator(target).evaluate(e=>e.scrollWidth<=e.clientWidth+2),true,target+' overflow '+w+theme);
   assert.equal(await page.locator('.main').evaluate(e=>e.scrollHeight<=e.clientHeight+2),true,'external overflow '+mode+w+theme);
   const geometry=await page.evaluate(()=>{const main=document.querySelector('.main').getBoundingClientRect(),list=document.querySelector('.sx-table'),r=list.getBoundingClientRect(),rail=document.querySelector('.sx-secrets').getBoundingClientRect();return {height:r.height,contained:r.top>=main.top&&r.bottom<=main.bottom+1,overflow:list.scrollHeight>list.clientHeight,side:rail.left>=r.right,scroll:getComputedStyle(list).overflowY};});
   assert(geometry.height>40);assert(geometry.contained);assert(geometry.overflow);assert(geometry.side);assert.equal(geometry.scroll,'auto');
   await page.locator('.sx-table').evaluate(e=>e.scrollTop=e.scrollHeight);
   assert.equal(await page.locator('.sx-table').evaluate(e=>Math.abs(e.querySelector('.sx-table-head').getBoundingClientRect().top-e.getBoundingClientRect().top)<=2),true,'sticky list header');
  }
  await page.locator('.window').screenshot({path:path.join(shots,'05-WebDAV同步-900-dark.png')});
 });
 await check('S-P23 与 Windows 窗口专题共存',async()=>{
  await page.goto('about:blank');await page.goto(entry+'?theme=light&topic=windows#sync');await page.waitForTimeout(100);assert(await page.locator('#route-sync').isVisible());assert.equal(await page.locator('script[src="windows-window.js"]').count(),1);assert.equal(await page.locator('body').evaluate(e=>e.classList.contains('windows-mode')),true);await page.locator('[data-win-action="leave"]').click();assert.equal(await page.locator('body').evaluate(e=>e.classList.contains('windows-mode')),false);assert(await page.locator('#route-sync').isVisible());
 });
 await check('S-P25 Skills / MCP 独立勾选，展开不改变范围与凭据',async()=>{
  const before=await sx();await expand('skills');await expand('mcp');assert.deepEqual((await sx()).selected,before.selected);assert.deepEqual((await sx()).secrets,before.secrets);
  await page.locator('[data-sx-group="skills"]').uncheck();assert(!(await sx()).selected.includes('skill:review'));assert((await sx()).selected.includes('mcp:docs'));
  await page.locator('[data-sx-secret="extensions"]').check();await page.locator('[data-sx-group="mcp"]').uncheck();assert.deepEqual((await sx()).secrets.extensions,[]);
 });
 await check('S-P26 搜索自动展开且可收起，清除后恢复普通折叠状态',async()=>{
  await expand('accounts');await page.locator('#sxSearch').fill('NewApi');assert.equal(await page.locator('[data-sx="expand"][data-id="channels"]').getAttribute('aria-expanded'),'true');
  await page.locator('[data-sx="expand"][data-id="channels"]').click();assert.equal(await page.locator('[data-sx-object]').count(),0);
  await page.locator('#sxSearch').fill('');assert.equal(await page.locator('[data-sx="expand"][data-id="channels"]').getAttribute('aria-expanded'),'false');assert.equal(await page.locator('[data-sx="expand"][data-id="accounts"]').getAttribute('aria-expanded'),'true');
 });
 await check('S-P27 无摘要与独立 Tab 滚动位置保留',async()=>{
  await expandAll();await page.locator('.sx-table').evaluate(e=>e.scrollTop=120);const before=await page.locator('.sx-table').evaluate(e=>e.scrollTop);assert(before>0);
  await page.locator('[data-sx-secret="keys"]').check();assert.equal(await page.locator('.sx-table').evaluate(e=>e.scrollTop),before);
  assert.equal(await page.locator('.sx-selection').count(),0);assert.equal(await page.locator('.sx-toolbar [data-sx="clear"]').count(),1);
  await page.locator('#sxTab-webdav').click();assert.equal(await page.locator('.sx-selection').count(),0);assert.equal(await page.locator('.sx-top [data-sx="save-scope"]').count(),1);assert.equal(await page.locator('[data-sx-object]').count(),0);await page.locator('#sxTab-file').click();assert.equal(await page.locator('.sx-table').evaluate(e=>e.scrollTop),before);
  await act('clear');assert.equal((await sx()).effective.length,0);assert.equal(await page.locator('.sx-toolbar [data-sx="clear"]').isDisabled(),true);assert.equal(await page.locator('[data-sx="export"]').isDisabled(),true);
 });
 await check('S-P28 账户详情仅信息、加密要求与取消重选',async()=>{
  await open({scope:['account:chatgpt']});await expand('accounts');await page.locator('[data-sx="detail"][data-id="account:chatgpt"]').click();assert.equal(await page.locator('.sx-account-options').evaluate(e=>e.open),false);
  await page.locator('.sx-account-options summary').click();await page.locator('[data-sx-account-info="account:chatgpt"]').check();assert.deepEqual((await sx()).secrets.accounts,[]);await close();
  await act('export');assert.equal(await page.locator('#sxEncrypt').isDisabled(),false);await confirm();assert.equal((await sx()).lastExport.encryptionRequired,false);
  await page.locator('[data-sx-object="account:chatgpt"]').uncheck();await page.locator('[data-sx-object="account:chatgpt"]').check();assert.deepEqual((await sx()).secrets.accounts,['account:chatgpt']);
  await act('export');assert.equal(await page.locator('#sxEncrypt').isDisabled(),true);assert.equal(await page.locator('#sxEncrypt').isChecked(),true);await close();
 });
 await check('S-P29 失效导入需重新登录，失效远端不能采用',async()=>{
  await page.evaluate(()=>syncPrototype.setScenario('login-invalid'));await sample('encrypted');await page.locator('[data-sx-import-pick="channel:newapi"]').uncheck();await page.locator('[data-sx-import-pick="channel:legacy"]').uncheck();await confirm();assert.equal((await sx()).result.accounts[0].status,'needs-login');assert.match((await sx()).result.detail,/已失效/);
  await open({tab:'webdav',scope:['account:chatgpt']});await act('save-scope');await act('check');await compareExpandAll();assert.match(await page.locator('#sxTokenCheck').innerText(),/自动检查未通过/);await select('[data-sx-diff-choice="account:chatgpt"]','remote');await page.locator('#sxSyncPass').fill('demo-sync');const serial=(await sx()).serial;await confirm();assert.equal((await sx()).serial,serial);assert.match(await page.locator('#sxError').innerText(),/已失效/);
  await select('[data-sx-diff-choice="account:chatgpt"]','local');await confirm();assert.deepEqual((await sx()).result.accounts,[]);
 });
 await check('S-P30 WebDAV 仅账户信息须保存，无令牌冲突或假恢复',async()=>{
  await open({tab:'webdav',scope:['account:chatgpt']});await act('save-scope');await expand('accounts');await page.locator('[data-sx="detail"][data-id="account:chatgpt"]').click();await page.locator('.sx-account-options summary').click();await page.locator('[data-sx-account-info="account:chatgpt"]').check();await close();assert((await sx()).dirty);assert.equal(await page.locator('[data-sx="check"]').isDisabled(),true);
  await act('save-scope');await act('check');await compareExpandAll();assert.equal(await page.locator('#sxSyncPass,[data-sx-diff-choice]').count(),0);await confirm();assert.equal((await sx()).result.accounts[0].status,'info-only');
  await open({tab:'file'});assert.deepEqual((await sx()).secrets.accounts,['account:chatgpt']);await open({tab:'webdav'});assert.deepEqual((await sx()).secrets.accounts,[]);
 });
 await check('S-P31 导入账户取消后不恢复，重选回到默认',async()=>{
  await sample('encrypted');await page.locator('[data-sx-import-pick="channel:newapi"]').uncheck();await page.locator('[data-sx-import-pick="channel:legacy"]').uncheck();await page.locator('[data-sx-import-account-info="account:chatgpt"]').check();
  await page.locator('[data-sx-import-pick="account:chatgpt"]').uncheck();assert.equal(await page.locator('[data-sx-import-account-info]').isDisabled(),true);await page.locator('[data-sx-import-pick="account:chatgpt"]').check();assert.equal(await page.locator('[data-sx-import-account-info]').isChecked(),false);
  await page.locator('[data-sx-import-pick="account:chatgpt"]').uncheck();await confirm();assert.deepEqual((await sx()).result.accounts.map(x=>x.id),['account:device']);
 });
 await check('S-P32 WebDAV 配置默认带密码，详情排除与取消重选',async()=>{
  await open({scope:['endpoint:personal']});await expand('endpoints');await page.locator('[data-sx="detail"][data-id="endpoint:personal"]').click();assert.equal(await page.locator('.sx-account-options').evaluate(e=>e.open),false);await page.locator('.sx-account-options summary').click();await page.locator('[data-sx-endpoint-config="endpoint:personal"]').check();assert.deepEqual((await sx()).secrets.endpointPasswords,[]);await close();
  await act('export');assert.equal(await page.locator('#sxEncrypt').isDisabled(),false);await confirm();assert.equal((await sx()).lastExport.encryptionRequired,false);
  await page.locator('[data-sx-object="endpoint:personal"]').uncheck();await page.locator('[data-sx-object="endpoint:personal"]').check();assert.deepEqual((await sx()).secrets.endpointPasswords,['endpoint:personal']);await act('export');assert.equal(await page.locator('#sxEncrypt').isDisabled(),true);await close();
  await open({tab:'webdav',scope:['endpoint:personal']});assert(!(await sx()).selected.includes('endpoint:personal'));assert.deepEqual((await sx()).secrets.endpointPasswords,[]);
 });
 await check('S-P33 WebDAV 密码随导入配置采用，连接保持待启用',async()=>{
  await open({scope:['endpoint:personal'],action:'import'});await act('choose-import-file');await page.locator('#sxUnlock').fill('demo-sync');await confirm();await compareExpandAll();assert.equal(await page.locator('[data-sx-import-endpoint-config="endpoint:personal"]').isChecked(),false);await confirm();assert.deepEqual((await sx()).result.endpoints,[{id:'endpoint:personal',status:'password-imported',enabled:false}]);
  await open({scope:['endpoint:personal'],action:'import'});await act('choose-import-file');await page.locator('#sxUnlock').fill('demo-sync');await confirm();await compareExpandAll();await page.locator('[data-sx-import-endpoint-config="endpoint:personal"]').check();await confirm();assert.equal((await sx()).result.endpoints[0].status,'config-only');assert.match((await sx()).result.detail,/同身份保留本机密码/);
 });
 await check('S-P34 无密码包只采用配置，两个 Tab 无底部原型说明',async()=>{
  await sample('config-only');assert.equal(await page.locator('[data-sx-import-endpoint-config]').count(),0);await confirm();assert.deepEqual((await sx()).result.endpoints,[{id:'endpoint:personal',status:'config-only',enabled:false}]);
  for(const tab of ['file','webdav']){await open({tab});assert.equal(await page.locator('.sx-demo-note').count(),0);assert.doesNotMatch(await page.locator('#sxPanel').innerText(),/交互原型 · 演示数据/);}
 });
await check('S-P35 上部公共设置、下部当前页面与 Tab Mock',async()=>{
  assert.equal(await page.locator('#protoCommon').isVisible(),true,'global group visible');
  assert.equal(await page.locator('#syncFileDemoCard').isVisible(),true,'file mock visible');
  assert.equal(await page.locator('#mpDemoCard').isVisible(),false,'models mock hidden');
  assert.equal(await page.locator('#protoCommon').evaluate(e=>e.open),false,'global group collapsed by default');
  assert.equal(await page.locator('#protoStdCards').evaluate(e=>e.open),true,'page mock expanded by default');
  assert.equal(await page.locator('#protoCommon').evaluate(e=>e.nextElementSibling.id),'protoStdCards');
  await page.locator('#sxTab-webdav').click();await page.waitForFunction(()=>document.querySelector('.prototype-side').dataset.context==='sync:webdav');assert.equal(await page.locator('#syncFileDemoCard').isVisible(),false);assert.equal(await page.locator('#syncDemoCard').isVisible(),true);
  await page.locator('#protoCommon > summary').click();await page.waitForTimeout(70);
  await page.locator('#launchGrid [data-launch="models"]').click();await page.waitForFunction(()=>document.querySelector('.prototype-side').dataset.context==='models:channels');assert.equal(await page.locator('#mpDemoCard').isVisible(),true);assert.equal(await page.locator('#syncDemoCard').isVisible(),false);
  await page.locator('[data-mp="tab"][data-tab="templates"]').click();await page.waitForFunction(()=>document.querySelector('.prototype-side').dataset.context==='models:templates');assert.equal(await page.locator('#mpDemoCard [data-scene="discovery-fail"]').isVisible(),false);assert.equal(await page.locator('#mpDemoCard [data-scene="late-generation"]').isVisible(),true);
  await page.locator('#launchGrid [data-launch="windows"]').click();await page.waitForFunction(()=>document.querySelector('.prototype-side').dataset.context==='windows');assert.equal(await page.locator('#protoStdCards #windowsDemoCard').isVisible(),true);assert.equal(await page.locator('#protoCommon').isVisible(),true);
});
await check('S-P36 公共设置和 Mock 一起滚动，锚点保持产品路由',async()=>{
  await page.setViewportSize({width:1805,height:1128});await page.evaluate(()=>setWinSize(1180,760));
  await page.locator('#protoCommon').evaluate(e=>{e.open=true});
  const geometry=await page.locator('.prototype-side').evaluate(side=>{
   const common=side.querySelector('#protoCommon'),mock=side.querySelector('#protoStdCards'),rail=side.querySelector('.proto-rail');
   const before=[common,mock].map(e=>e.getBoundingClientRect().top);side.scrollTop=160;const after=[common,mock].map(e=>e.getBoundingClientRect().top);
   return {overflow:getComputedStyle(side).overflowY,rail:getComputedStyle(rail).position,inner:[common,mock].map(e=>getComputedStyle(e).overflowY),delta:before.map((v,i)=>v-after[i]),scroll:side.scrollTop};
  });
  assert.match(geometry.overflow,/auto|scroll/);assert.equal(geometry.rail,'static');assert(geometry.inner.every(v=>!['auto','scroll'].includes(v)));assert(geometry.scroll>0);assert(geometry.delta.every(v=>Math.abs(v-geometry.scroll)<2));
  await page.locator('.prototype-side').evaluate(e=>e.scrollTop=0);const hash=await page.evaluate(()=>location.hash);await page.locator('.prototype-side a[href="#ctl-mock"]').click();await page.waitForTimeout(500);assert.equal(await page.evaluate(()=>location.hash),hash);assert(await page.locator('.prototype-side').evaluate(e=>e.scrollTop>0));
  await page.setViewportSize({width:1692,height:916});
 });
 await check('S-P37 导入文件选择、解密字段与已选文件快照',async()=>{
  await act('import');assert.equal(await page.locator('#modalConfirm').isDisabled(),true);assert.equal(await page.locator('#sxUnlockField').isVisible(),false);assert.doesNotMatch(await page.locator('#modalBody').innerText(),/示例|演示|demo-sync|正式产品/);
  await act('choose-import-file');assert.equal(await page.locator('#modalConfirm').isDisabled(),false);assert.equal(await page.locator('#sxUnlockField').isVisible(),true);assert.equal(await page.locator('#sxImportFileName').innerText(),'opencodex-device-config.ocx');
  await page.evaluate(()=>syncPrototype.setImportSample('corrupt'));await page.locator('#sxUnlock').fill('demo-sync');await confirm();assert(await page.locator('[data-sx-import-pick]').count()>0);await close();
  await act('import');await act('choose-import-file');await confirm();assert.match(await page.locator('#sxError').innerText(),/完整性校验失败/);assert.equal((await sx()).serial,0);
 });
 await check('S-P38 比较树默认收起、分类半选与明确删除',async()=>{
  await sample('encrypted',false);assert.equal(await page.locator('[data-sx="compare-expand"][aria-expanded="false"]').count(),7);
  assert.equal(await page.locator('.sx-compare-item:visible').count(),0);assert.equal(await page.locator('[data-sx-compare-all]').evaluate(e=>e.indeterminate),true);
  assert.match(await page.locator('.sx-compare-tree').innerText(),/新增/);assert.match(await page.locator('.sx-compare-tree').innerText(),/更新/);assert.match(await page.locator('.sx-compare-tree').innerText(),/删除/);
  assert.equal(await page.locator('[data-sx-import-choice="channel:newapi"],[data-sx-import-secret="secrets"]').count(),0);
  await compareExpand('skills');assert.equal(await page.locator('[data-sx-import-pick="skill:legacy-review"]').isChecked(),false);assert.equal(await page.locator('[data-sx="compare-expand"][data-id="skills"]').evaluate(e=>e===document.activeElement),true);
 });
 await check('S-P39 顶层分类选择、精确依赖与凭据个别覆盖',async()=>{
  await sample('encrypted');await page.locator('[data-sx-compare-all]').uncheck();assert.equal((await sx()).comparison.selected.length,0);
  await page.locator('[data-sx-compare-group="channels"]').check();assert((await sx()).comparison.selected.includes('template:gpt'));assert.equal(await page.locator('[data-sx-import-pick="template:gpt"]').isDisabled(),true);
  await page.locator('[data-sx-import-pick="channel:legacy"]').uncheck();assert.equal(await page.locator('[data-sx-compare-group="channels"]').evaluate(e=>e.indeterminate),true);
  await page.locator('[data-sx-compare-all]').check();await page.locator('[data-sx-compare-rule="all"]').check();assert.equal((await sx()).comparison.rules['channel:newapi'],true);assert.equal((await sx()).comparison.rules['mcp:docs'],true);
  await page.locator('[data-sx-compare-credential="mcp:docs"]').uncheck();assert.equal(await page.locator('[data-sx-compare-rule="all"]').evaluate(e=>e.indeterminate),true);assert.equal((await sx()).comparison.rules['channel:newapi'],true);
  assert.equal(await page.locator('[data-sx-compare-credential="channel:legacy"]').count(),0);
 });
 await check('S-P40 七类只读详情、独立滚动与 Escape 返回比较',async()=>{
  await sample('encrypted');await page.locator('[data-sx-compare-credential="channel:newapi"]').check();
  for(const [id,field] of [['channel:newapi','推理等级'],['template:gpt','匹配规则'],['account:chatgpt','登录材料'],['skill:review','SKILL.md'],['mcp:docs','环境变量'],['preferences:desktop','窗口坐标'],['endpoint:personal','同步加密口令']]){
   const origin=page.locator('[data-sx="compare-detail"][data-id="'+id+'"]');await origin.scrollIntoViewIfNeeded();await origin.focus();const before=(await sx()).comparison,scroll=await page.locator('.sx-compare-tree').evaluate(e=>e.scrollTop);
   await origin.click();assert((await sx()).detailOpen);assert.match(await page.locator('#sxCompareDetail').innerText(),new RegExp(field));assert.equal(await page.locator('#sxCompareDetail input,#sxCompareDetail select,#sxCompareDetail textarea').count(),0);
   assert.equal(await page.locator('#sxCompareMain').evaluate(e=>e.inert),true);await page.keyboard.press('Tab');assert.equal(await page.locator('#sxCompareDetailClose').evaluate(e=>e===document.activeElement),true);
   await page.locator('.sx-detail-body').evaluate(e=>e.scrollTop=e.scrollHeight);await page.keyboard.press('Escape');assert(!(await sx()).detailOpen);assert.equal((await sx()).dialogKind,'import');assert.deepEqual((await sx()).comparison,before);assert.equal(await page.locator('.sx-compare-tree').evaluate(e=>e.scrollTop),scroll);assert.equal(await origin.evaluate(e=>e===document.activeElement),true);assert.equal((await sx()).serial,0);
  }
 });
 await check('S-P41 WebDAV 同步配置按钮、同构树与勾选应用',async()=>{
  await open({tab:'webdav',scope:['channel:newapi','skill:review','mcp:docs','preferences:desktop']});await act('save-scope');assert.equal(await page.locator('[data-sx="check"]').innerText(),'同步配置');await act('check');await compareExpandAll();
  assert.equal(await page.locator('.sx-compare-tree').count(),1);await page.locator('[data-sx-compare-all]').uncheck();await page.locator('[data-sx-diff-pick="mcp:docs"]').check();await confirm();assert.deepEqual((await sx()).lastComparison.items.map(x=>x.id),['mcp:docs']);assert.equal((await sx()).lastComparison.items[0].direction,'upload');
 });
 await check('S-P42 WebDAV 重绘与只读详情保留方向和口令',async()=>{
  await open({tab:'webdav',scope:['account:chatgpt']});await act('save-scope');await act('check');await compareExpandAll();await select('[data-sx-diff-choice="account:chatgpt"]','remote');await page.locator('#sxSyncPass').fill('demo-sync');
  await page.locator('[data-sx-diff-account-info="account:chatgpt"]').check();assert.equal(await page.locator('#sxSyncPass').inputValue(),'demo-sync');assert.equal(await page.locator('[data-sx-diff-choice="account:chatgpt"]').inputValue(),'remote');
  await page.locator('[data-sx="compare-detail"][data-id="account:chatgpt"]').click();await page.keyboard.press('Escape');await confirm();assert.equal((await sx()).result.accounts[0].status,'info-only');
 });
 await check('S-P43 比较与第二层详情在浅深色短窗内可达',async()=>{
  for(const theme of ['light','dark'])for(const [w,h] of [[1180,760],[900,600]]){
   await page.evaluate(({theme,w,h})=>{applyTheme(theme);setWinSize(w,h);},{theme,w,h});await sample('encrypted');
   const geometry=await page.locator('#modal').evaluate(e=>{const r=e.getBoundingClientRect(),w=document.querySelector('.window').getBoundingClientRect(),tree=e.querySelector('.sx-compare-tree'),body=e.querySelector('#modalBody');return {contained:r.top>=w.top&&r.bottom<=w.bottom+1,noX:tree.scrollWidth<=tree.clientWidth+2,outer:body.scrollHeight<=body.clientHeight+2,inner:tree.scrollHeight>tree.clientHeight};});
   assert(geometry.contained);assert(geometry.noX);assert(geometry.outer);assert(geometry.inner);await page.locator('[data-sx="compare-detail"][data-id="channel:newapi"]').click();
   assert.equal(await page.locator('.sx-detail-body').evaluate(e=>getComputedStyle(e).overflowY),'auto');assert.equal(await page.locator('#sxCompareDetailClose').isVisible(),true);await page.keyboard.press('Escape');assert.equal(await page.locator('#modalConfirm').isVisible(),true);await close();
  }
  await sample('encrypted');await page.locator('#modal').screenshot({path:path.join(shots,'06-树形比较-900-dark.png')});await page.locator('[data-sx="compare-detail"][data-id="channel:newapi"]').click();await page.locator('#modal').screenshot({path:path.join(shots,'07-只读详情-900-dark.png')});await page.keyboard.press('Escape');
 });
 await check('S-P24 浏览器脚本与资源无错误',async()=>assert.deepEqual(errors,[]));
}finally{await browser.close();}
const report={date:'2026-10-08',runtime:{playwright:pw,chromium:exe},boundary:'静态交互原型；不读取真实凭据、不生成加密文件、不连接 WebDAV、不等于生产验收',results,consoleErrors:errors};
fs.writeFileSync(path.join(evidence,'浏览器验收结果.json'),JSON.stringify(report,null,2)+'\n');
console.log('TOTAL '+results.length+' / FAIL '+results.filter(x=>x.status==='FAIL').length);process.exitCode=results.some(x=>x.status==='FAIL')?1:0;
