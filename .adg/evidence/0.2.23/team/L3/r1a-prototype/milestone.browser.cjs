const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const pw=require('/Users/ezio/.npm/_npx/e41f203b7505f1fb/node_modules/playwright');
const out=__dirname,url='http://127.0.0.1:43123/01-产品原型/原型/index.html';
const reports=[];
(async()=>{
 for(const [engine,viewport,theme] of [['chromium',{width:1180,height:760},'light'],['webkit',{width:900,height:650},'dark']]){
  const browser=await pw[engine].launch({headless:true});const errors=[],blocked=[];const checks=[];
  try{
   const page=await browser.newPage({viewport});page.setDefaultTimeout(7000);
   page.on('pageerror',e=>errors.push(e.message));
   await page.route('**/*',r=>{if(r.request().url().startsWith('http://127.0.0.1:43123/'))return r.continue();blocked.push(r.request().url());return r.abort();});
   await page.goto(url+'#models');await page.evaluate(t=>document.documentElement.dataset.theme=t,theme);
   await page.locator('[data-mp="tab"]').filter({hasText:'模型模版'}).click();
   await page.locator('[data-mp="open-template-editor"][data-id="gpt"]').click();
   await page.getByRole('button',{name:'编辑条件与预览',exact:true}).click();
   await page.getByRole('button',{name:'预览匹配',exact:true}).click();
   assert.match(await page.locator('#c23Result').textContent(),/命中/);checks.push('real template route -> conditions -> declared preview');
   await page.locator('[data-o23-select="condition-scene"]').selectOption('before',{force:true});
   await page.getByRole('button',{name:'预览匹配',exact:true}).click();
   assert.match(await page.locator('#c23Result').textContent(),/claude-opus-5.5/);
   assert.match(await page.locator('#c23Result').textContent(),/before_version/);checks.push('name before version AND complete numeric view');
   await page.locator('#modalBody').evaluate(el=>el.scrollTop=el.scrollHeight);
   await page.screenshot({path:path.join(out,engine+'-condition-'+theme+'.png')});
   await page.getByLabel('预览原始模型 ID',{exact:true}).fill('unregistered-id');
   assert.match(await page.locator('#c23Result').textContent(),/旧命中结果已失效/);
   await page.getByRole('button',{name:'预览匹配',exact:true}).click();assert.match(await page.locator('#c23Result').textContent(),/暂无|没有对应/);checks.push('arbitrary edit invalidates mock response');
   await page.locator('[data-o23-select="condition-scene"]').selectOption('matched',{force:true});
   await page.getByRole('button',{name:'预览匹配',exact:true}).click();
   await page.locator('#modalConfirm').click();assert.match(await page.locator('#modalTitle').textContent(),/保存条件/);
   await page.locator('#modalCancel').click();checks.push('cancel save stays uncommitted');
   await page.evaluate(()=>modelConditions023.open('gpt'));
   await page.locator('#modalConfirm').click();await page.locator('#modalConfirm').click();
   await page.getByRole('button',{name:'查看已保存版本',exact:true}).click();assert.match(await page.locator('#modalBody').textContent(),/contentVersion/);checks.push('explicit immutable memory revision and reopen');
   await page.locator('#modalClose').click();
   await page.locator('.nav [data-route="settings"]').click();
   await page.locator('[data-settings-section="installation"]').click();
   await page.getByRole('button',{name:'选择新版目录',exact:true}).click();
   const before=await page.evaluate(()=>directoryRuntime023.controller.state().writes);
   await page.locator('#modalConfirm').click();await page.locator('#modalCancel').click();assert.equal(await page.evaluate(()=>directoryRuntime023.controller.state().writes),before);checks.push('directory prepare and cancel no binding mutation');
   await page.getByRole('button',{name:'诊断与恢复安装',exact:true}).click();
   await page.locator('#modalConfirm').click();await page.locator('#modalConfirm').click();
   assert.equal(await page.evaluate(()=>runtimeRecovery023.controller.state().backups),0);
   await page.screenshot({path:path.join(out,engine+'-installation-confirm-'+theme+'.png')});
   const box=await page.locator('#modalConfirm').boundingBox();assert.ok(box&&box.y+box.height<=viewport.height+1);checks.push('confirm reachable at viewport height');
   await page.locator('#modalConfirm').click();
   for(let i=0;i<4;i++)await page.locator('#modalConfirm').click();
   assert.equal(await page.evaluate(()=>runtimeRecovery023.controller.state().operation.phase),'done');checks.push('backup -> owned service -> replace -> verify -> done');
   await page.locator('#modalClose').click();
   await page.getByRole('button',{name:'诊断与恢复安装',exact:true}).click();assert.match(await page.locator('#modalBody').textContent(),/恢复完成/);checks.push('reopen original installation record');
   assert.deepEqual(errors,[]);reports.push({engine,viewport,theme,status:'passed',checks,pageErrors:errors,blockedExternalRequests:blocked.length});
  }catch(e){reports.push({engine,viewport,theme,status:'failed',checks,pageErrors:errors,error:e.stack});throw e;}finally{fs.writeFileSync(path.join(out,'milestone-results.json'),JSON.stringify(reports,null,2));await browser.close();}
 }
 console.log(JSON.stringify(reports,null,2));
})().catch(e=>{console.error(e);process.exitCode=1;});
