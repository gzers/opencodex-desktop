const {chromium}=require('/Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-f5de-7c53-89a3-cad653af6685/dialog-pixel-ci2880/tools/node_modules/playwright-core');
const fs=require('node:fs'),assert=require('node:assert/strict');
const out='/private/tmp/ocxd023-directory-card-20261108/';
const entry='http://127.0.0.1:57285/docs/'+encodeURI('04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html')+'#settings?section=installation&card=ocxd023Settings';
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:'/Users/ezio/Library/Caches/ms-playwright/chromium-1248/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'});
 const page=await browser.newPage({viewport:{width:1699,height:1128},reducedMotion:'reduce',colorScheme:'dark'}),errors=[],checks=[];page.on('pageerror',e=>errors.push(e.message));
 const check=name=>checks.push({name,pass:true});
 await page.goto(entry);await page.locator('#ocxd023Settings').waitFor();
 await page.evaluate(()=>{envState='ready';setState('stopped');});
 await page.locator('#ocxd023Settings').scrollIntoViewIfNeeded();
 await page.screenshot({path:out+'01-dark-macos.png'});
 assert.equal(await page.locator('[data-o23=path-select]').count(),0);assert.equal(await page.locator('[data-o23=path-restart]').count(),0);check('主卡片无目录选择或切换入口');
 assert.match(await page.locator('#ocxd023Settings').innerText(),/\/Users\/demo\/\.codex-desktop\/opencodex\/home/);check('默认新目录按当前用户分类');
 async function layout(){return page.locator('#ocxd023Settings').evaluate(el=>({width:el.clientWidth,scroll:el.scrollWidth,rows:[...el.querySelectorAll('dt')].map(dt=>({label:dt.textContent,labelHeight:dt.getBoundingClientRect().height,line:parseFloat(getComputedStyle(dt).lineHeight),delta:Math.abs(dt.getBoundingClientRect().top-dt.nextElementSibling.getBoundingClientRect().top)})),gaps:[...el.querySelectorAll('.dr023-actions')].map(a=>a.getBoundingClientRect().top-a.previousElementSibling.getBoundingClientRect().bottom)}));}
 let l=await layout();assert.ok(l.scroll<=l.width+1);assert.ok(l.rows.every(r=>r.delta<=1&&r.labelHeight<=r.line+1));assert.ok(l.gaps.every(g=>g>=16));check('标签不折行且与路径顶对齐，动作行保留独立间距');
 await page.locator('[data-o23=path-details]').click();assert.match(await page.locator('#modalTitle').innerText(),/完整路径/);assert.match(await page.locator('[aria-label=完整路径文本]').inputValue(),/config.json/);assert.equal(await page.locator('#modalConfirm').isVisible(),false);await page.locator('#modalClose').click();check('完整路径查看沿用居中查看弹窗和右上角关闭');
 await page.locator('[data-o23=path-json]').click();assert.match(await page.locator('#modalBody').innerText(),/当前支持格式/);assert.doesNotMatch(await page.locator('#modalBody').innerText(),/迁移旧目录|扫描旧/);await page.locator('#modalClose').click();check('渠道说明只指向新HOME中的当前支持格式JSON');
 await page.evaluate(()=>{directoryRuntime023.controller.useSnapshot(directoryRuntime023.snapshots.windows);directoryRuntime023.render();});l=await layout();assert.ok(l.scroll<=l.width+1);assert.ok(l.rows.every(r=>r.delta<=1));await page.screenshot({path:out+'02-dark-windows.png'});check('Windows长路径无横向溢出且行首对齐');
 await page.emulateMedia({colorScheme:'light'});await page.screenshot({path:out+'03-light-windows.png'});check('亮色主题实际页面检查');
 await page.setViewportSize({width:1200,height:900});await page.evaluate(()=>{document.querySelector('[data-size="900x600"]').click();document.querySelector('#ocxd023Settings').scrollIntoView({block:'start'});});l=await layout();assert.ok(l.scroll<=l.width+1);await page.screenshot({path:out+'04-narrow-windows.png'});check('900×600窗口保持完整路径且无卡片横向溢出');
 await page.evaluate(()=>{directoryRuntime023.controller.scene('loading');directoryRuntime023.render();});assert.equal(await page.locator('[data-o23=path-details]').isDisabled(),true);check('读取态保留状态和禁用语义');
 await page.evaluate(()=>{directoryRuntime023.controller.scene('missing');directoryRuntime023.render();});await page.locator('[data-o23=path-init]').click();assert.match(await page.locator('#modalBody').innerText(),/仅初始化新版/);await page.locator('#modalCancel').click();assert.equal(await page.evaluate(()=>directoryRuntime023.controller.state().scene),'missing');check('尚未初始化和取消保持原状态');
 await page.evaluate(()=>{directoryRuntime023.controller.useSnapshot(directoryRuntime023.snapshots.macos);directoryRuntime023.render();});await page.locator('[data-o23=install-open]').click();assert.match(await page.locator('#modalTitle').innerText(),/安装身份诊断/);await page.locator('#modalClose').click();check('安装恢复消费者仍可打开');
 assert.equal(errors.length,0);
 fs.writeFileSync(out+'browser-result.json',JSON.stringify({checks,errors,lastLayout:l,productAccepted:false},null,2));await browser.close();console.log(JSON.stringify({checks:checks.length,errors}));
})().catch(e=>{console.error(e);process.exit(1);});
