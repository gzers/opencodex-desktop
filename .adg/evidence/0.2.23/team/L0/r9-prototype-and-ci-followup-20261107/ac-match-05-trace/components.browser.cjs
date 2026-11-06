const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const pw=require('/Users/ezio/.npm/_npx/e41f203b7505f1fb/node_modules/playwright');
const out=__dirname,reports=[];
(async()=>{for(const [engine,width,theme] of [['chromium',1180,'light'],['webkit',900,'dark']]){
const browser=await pw[engine].launch({headless:true});
const report={engine,width,height:760,theme,checks:[],scenes:[],pageErrors:[],console:[],requests:[],failedRequests:[]};reports.push(report);
const p=await browser.newPage({viewport:{width,height:760}});p.setDefaultTimeout(6000);
p.on('pageerror',e=>report.pageErrors.push(e.message));p.on('console',m=>report.console.push({type:m.type(),text:m.text()}));p.on('requestfailed',r=>report.failedRequests.push({url:r.url(),error:r.failure()?.errorText}));
await p.route('**/*',r=>{const request=r.request();report.requests.push({url:request.url(),method:request.method()});return request.url().startsWith('http://127.0.0.1:43130/')&&(request.method()==='GET'||request.url().endsWith('/__rust-preview'))?r.continue():r.abort();});
const check=async(name,fn)=>{try{await fn();report.checks.push({name,status:'passed'});}catch(e){report.checks.push({name,status:'failed',error:e.stack});}};
const state=()=>p.evaluate(()=>acMatch05.snapshot());
const scene=async s=>{await p.getByLabel('夹具场景',{exact:true}).selectOption(s);};
const choose=async(label,option)=>{await p.getByRole('button',{name:label,exact:true}).click();await p.getByRole('option',{name:option,exact:true}).click();};
const preview=p.getByLabel('条件匹配预览',{exact:true});
const run=async()=>{await p.getByRole('button',{name:'预览当前条件',exact:true}).click();await p.waitForFunction(()=>!acMatch05.store.previewBusy);};
const shot=async name=>{await p.screenshot({path:path.join(out,engine+'-'+name+'-'+theme+'.png'),fullPage:true});};
try{
await p.goto('http://127.0.0.1:43130/__l3-component');await p.getByLabel('覆盖项名称1',{exact:true}).waitFor();await p.evaluate(t=>document.documentElement.dataset.theme=t,theme);
await check('formal components mount with production store and IPC adapter',async()=>{assert.equal(await p.getByLabel('条件覆盖编辑器',{exact:true}).count(),1);assert.match(await preview.innerText(),/尚无当前输入/);});
await check('matching scope / UUID / patterns / priority / case controls emit exact content',async()=>{await choose('模版渠道范围','仅所选渠道 UUID');await p.getByLabel('模版范围渠道UUID',{exact:true}).fill('uuid-A\nuuid-B');await p.getByRole('button',{name:'添加包含',exact:true}).click();await choose('include匹配方式2','精确 ID');await p.getByLabel('include匹配文本2',{exact:true}).fill('exact/model');await p.getByRole('button',{name:'添加排除',exact:true}).click();await p.getByLabel('exclude匹配文本1',{exact:true}).fill('gpt-repeated-');await p.getByLabel('优先级',{exact:true}).fill('17');await p.getByLabel('区分大小写',{exact:true}).check();const c=(await state()).draft.content;assert.equal(c.matching.channelMode,'whitelist');assert.deepEqual(c.matching.channels,['uuid-A','uuid-B']);assert.equal(c.matching.include[1].kind,'exact');assert.equal(c.matching.priority,17);assert.equal(c.matching.caseSensitive,true);});
await scene('matched');
await check('complete string numeric components preserve arity, leading zero, dot/hyphen input',async()=>{for(const [input,expected] of [['06-01-001',['06','01','001']],['6.10',['6','10']],['6.1.0',['6','1','0']]]){await p.getByLabel('覆盖1条件值1',{exact:true}).fill(input);assert.deepEqual((await state()).draft.content.conditionOverrides[0].selector.all[0].components,expected);}await shot('numeric-editor');});
await check('NameLayout fixed policies retained through prefix, exact tail and source ID edits',async()=>{await p.getByLabel('结构前缀1',{exact:true}).fill('literal-');await p.getByLabel('精确剩余文本1',{exact:true}).fill('-sol-20261107-build001');await p.getByLabel('来源精确ID1',{exact:true}).fill('literal-06-01-001-sol-20261107-build001');const c=(await state()).draft.content;assert.deepEqual(c.nameLayouts[0],{id:'44444444-4444-4444-8444-444444444444',literalPrefix:'literal-',versionCapture:'complete_numeric_components',numericJoiners:['.','-'],tailCapture:'literal_remainder',trailingDatePolicy:'retain_valid_YYYYMMDD_after_hyphen',ambiguousNumericTail:'no_equivalence'});assert.equal(c.conditionOverrides[0].selector.literalTail,'-sol-20261107-build001');});
await check('literal segment any / before_version / after_version selected by actual menu',async()=>{for(const [label,value] of [['版本前','before_version'],['版本后','after_version'],['不限位置','any']]){await choose('覆盖1名称位置2',label);assert.equal((await state()).draft.content.conditionOverrides[0].selector.all[1].position,value);}await shot('position-editor');});
await check('AND add, atom kind exact ID, removal, override add/remove',async()=>{await p.getByRole('button',{name:'添加 AND 条件',exact:true}).click();await choose('覆盖1条件类型3','完整模型 ID');await p.getByLabel('覆盖1条件值3',{exact:true}).fill('exact/raw-id-001');assert.deepEqual((await state()).draft.content.conditionOverrides[0].selector.all[2],{kind:'exact_id',value:'exact/raw-id-001'});await p.getByRole('button',{name:'删除条件',exact:true}).nth(2).click();assert.equal((await state()).draft.content.conditionOverrides[0].selector.all.length,2);await p.getByRole('button',{name:'添加覆盖项',exact:true}).click();await p.getByLabel('覆盖项名称2',{exact:true}).fill('extra');await p.getByLabel('覆盖项2',{exact:true}).getByRole('button',{name:'删除此覆盖项',exact:true}).click();assert.equal((await state()).draft.content.conditionOverrides.length,1);});
await check('without layout relative positions disabled; exact empty tail distinct from absent tail',async()=>{await choose('数字版本结构1','无数字结构');let c=(await state()).draft.content;assert.equal(Object.hasOwn(c.conditionOverrides[0].selector,'literalTail'),false);assert.equal(await p.getByLabel('精确剩余文本1',{exact:true}).isDisabled(),true);await p.getByRole('button',{name:'覆盖1名称位置2',exact:true}).click();assert.equal(await p.getByRole('option',{name:'版本前',exact:true}).isDisabled(),true);assert.equal(await p.getByRole('option',{name:'版本后',exact:true}).isDisabled(),true);await p.getByRole('option',{name:'不限位置',exact:true}).click();await choose('数字版本结构1','literal-〈数字版本〉');assert.equal((await state()).draft.content.conditionOverrides[0].selector.literalTail,'');});
await check('layout add/remove; referenced deletion preserves unresolved selector and alerts',async()=>{await p.getByRole('button',{name:'添加名称结构',exact:true}).click();await p.getByLabel('结构前缀2',{exact:true}).fill('other-');await p.getByRole('button',{name:'移除此结构',exact:true}).nth(1).click();const before=(await state()).draft.content.conditionOverrides[0].selector;await p.getByRole('button',{name:'移除此结构',exact:true}).first().click();const after=(await state()).draft.content.conditionOverrides[0].selector;assert.deepEqual(after,before);assert.match(await p.getByLabel('覆盖项1',{exact:true}).innerText(),/所引用的名称结构不存在/);await shot('missing-layout');});
await scene('matched');
await check('condition rule edits and disabled state use production controls',async()=>{await choose('Fast支持','不支持');assert.deepEqual((await state()).draft.content.conditionOverrides[0].rules.fast,{mode:'custom',value:false});await p.getByLabel('禁用编辑',{exact:true}).check();assert.equal(await p.getByLabel('结构前缀1',{exact:true}).isDisabled(),true);assert.equal(await p.getByLabel('覆盖1条件值1',{exact:true}).isDisabled(),true);assert.equal(await p.getByRole('button',{name:'预览当前条件',exact:true}).isDisabled(),true);await p.getByLabel('禁用编辑',{exact:true}).uncheck();});
for(const [i,c] of (await p.evaluate(()=>acMatch05.frozenCases)).entries())for(const [j,raw] of c.request.rawModelIds.entries()){
 const s='frozen/'+i+'/'+j;await scene(s);
 await check('frozen Rust case '+c.case+' row '+j,async()=>{
  const before=await state();await run();const snapshot=await state(),text=await preview.innerText();
  if(c.error){assert.equal(snapshot.preview,null);assert.match(text,new RegExp(c.error.code));assert.deepEqual(snapshot.issues,c.error.issues);assert.deepEqual(snapshot.draft.content,before.draft.content);}
  else {
   const row=c.result.rows[j];assert.equal(snapshot.preview.rawModelId,raw);assert.equal(snapshot.preview.matchingView,row.matchingView);assert.equal(snapshot.preview.reason,row.reason);assert.equal(snapshot.preview.reasonCode,row.reasonCode);assert.deepEqual(snapshot.preview.family,row.family);assert.deepEqual(snapshot.preview.conditions,row.conditions);
   for(const field of ['rules','protocol','fieldOrigins'])assert.deepEqual(snapshot.preview.configuration[field],row[field]);
   assert.ok(text.includes(raw));assert.ok(text.includes(row.matchingView));assert.ok(text.includes(row.family.reason));assert.ok(text.includes(row.family.reasonCode));assert.ok(text.includes(row.reason));
   assert.equal(await preview.locator('article').count(),row.conditions.length);
   for(const [k,condition] of row.conditions.entries()){
    const article=preview.getByLabel('条件核对'+(k+1),{exact:true}),content=await article.innerText();
    assert.ok(content.includes(condition.reason));assert.ok(content.includes(condition.reasonCode));
    assert.match(content,condition.applied?/已采用/:/未采用/);
    if(!condition.applicable)assert.ok(content.includes('不适用'));
    for(const [at,precondition] of condition.preconditions.entries()){
     const t=await article.locator('ul li').nth(at).innerText();assert.ok(t.includes(precondition.reason));assert.ok(t.includes(precondition.reasonCode));assert.ok(t.includes(precondition.matched===null?'未评估／不适用':precondition.matched?'满足':'不满足'));
    }
    for(const atom of condition.atoms){
     const t=await article.locator('ol li').nth(atom.index).innerText();assert.ok(t.includes(atom.reason));assert.ok(t.includes(atom.reasonCode));assert.ok(t.includes(atom.matched===null?'未评估／不适用':atom.matched?'满足':'不满足'));if(atom.matched===null)assert.doesNotMatch(t,/ · 不满足 · /);
    }
   }
  }
  report.scenes.push({scene:s,case:c.case,row:j,raw,text,snapshot});await shot('frozen-'+c.case+'-'+j);
 });
}
for(const s of ['matched','all_conditions','missing','unsupported','duplicate','empty','layout_failure','atom_failure','network_error','malformed','wrong_id']){
 await scene(s);await check('live Rust UI scene '+s,async()=>{
  const before=await state();await run();const snapshot=await state(),text=await preview.innerText();
  if(['unsupported','duplicate','empty','network_error','malformed','wrong_id'].includes(s)){
   assert.equal(snapshot.preview,null);assert.ok(snapshot.error);assert.deepEqual(snapshot.draft.content,before.draft.content);
   assert.match(text,{unsupported:/model_protocol_unsupported/,duplicate:/model_selector_duplicate/,empty:/model_selector_invalid/,network_error:/隔离传输故障/,malformed:/缺少完整匹配过程/,wrong_id:/预览结果不属于当前模型/}[s]);
  }else {
   assert.equal(snapshot.preview.rawModelId,snapshot.calls.filter(c=>c.command==='model_v2_preview_conditions').at(-1).args.request.rawModelIds[0]);
   if(s==='all_conditions'){assert.equal(snapshot.preview.conditions.length,2);assert.deepEqual(snapshot.preview.conditions.map(c=>c.applied),[true,false]);}
   if(s==='missing')assert.match(text,/请求协议待配置/);
   if(s==='layout_failure')assert.deepEqual(snapshot.preview.conditions[0].atoms.map(a=>a.matched),[null,null]);
   if(s==='atom_failure')assert.deepEqual(snapshot.preview.conditions[0].atoms.map(a=>a.matched),[false,true]);
  }
  report.scenes.push({scene:s,text,snapshot});await shot(s);
 });
}
await scene('matched');
await check('actual Rust common and condition provenance / explicit false/null/empty values',async()=>{await run();const text=await preview.innerText();assert.match(text,/模版通用/);assert.match(text,/条件覆盖/);assert.match(text,/明确禁用/);assert.match(text,/明确空集合/);assert.match(await preview.getByRole('row').filter({has:p.getByRole('rowheader',{name:'Fast',exact:true})}).innerText(),/否.*条件覆盖/s);});
await check('actual edit changes backend numeric AND outcome, while original raw ID remains unchanged',async()=>{await scene('matched');await run();assert.equal((await state()).preview.conditions[0].applied,true);await p.getByLabel('覆盖1条件值1',{exact:true}).fill('06-01-001');assert.equal((await state()).preview,null);await run();const snapshot=await state();assert.deepEqual(snapshot.draft.content.conditionOverrides[0].selector.all[0].components,['06','01','001']);assert.equal(snapshot.preview.rawModelId,'gpt-6.1-sol');assert.equal(snapshot.preview.conditions[0].atoms[0].matched,false);assert.equal(snapshot.preview.conditions[0].applied,false);await shot('actual-edit-numeric');});
await check('actual position editing before/after controls changes backend atom outcomes',async()=>{await scene('after');await run();assert.equal((await state()).preview.conditions[0].atoms[1].matched,true);await choose('覆盖1名称位置2','版本前');await run();assert.equal((await state()).preview.conditions[0].atoms[1].matched,false);await choose('覆盖1名称位置2','版本后');await run();assert.equal((await state()).preview.conditions[0].atoms[1].matched,true);await shot('actual-edit-position');});
await check('edit invalidates a pending real Rust reply, cancelled reply remains dismissed',async()=>{await scene('stale');await p.getByRole('button',{name:'预览当前条件',exact:true}).click();await p.waitForFunction(()=>acMatch05.pendingCount()===1);assert.match(await preview.innerText(),/正在核对/);assert.equal(await preview.locator('article').count(),0);await shot('busy');await p.getByLabel('覆盖项名称1',{exact:true}).fill('newer-input');await p.evaluate(()=>acMatch05.release());await p.waitForFunction(()=>!acMatch05.store.previewBusy);assert.equal((await state()).preview,null);assert.match(await preview.innerText(),/尚无当前输入/);await scene('stale');await p.getByRole('button',{name:'预览当前条件',exact:true}).click();await p.waitForFunction(()=>acMatch05.pendingCount()===1);await p.getByRole('button',{name:'取消预览',exact:true}).click();await p.evaluate(()=>acMatch05.release());await p.waitForFunction(()=>!acMatch05.store.previewBusy);assert.equal((await state()).preview,null);await shot('cancelled-late-reply');});
await check('error retry preserves input and exact IPC input, blank raw ID avoids calls',async()=>{await scene('network_error');await run();const before=await state();await preview.getByRole('button',{name:'重新核对',exact:true}).click();await p.waitForFunction(()=>!acMatch05.store.previewBusy);const after=await state();assert.deepEqual(after.draft.content,before.draft.content);assert.deepEqual(after.calls.at(-1).args,before.calls.at(-1).args);await p.getByLabel('预览原始ID',{exact:true}).fill('');const count=(await state()).calls.length;await run();assert.equal((await state()).calls.length,count);assert.match(await preview.innerText(),/请输入完整原始模型 ID/);});
await check('no external traffic; only isolated Rust transport POST; clean console and fit',async()=>{assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);assert.equal(report.requests.filter(r=>!r.url.startsWith('http://127.0.0.1:43130/')||(r.method!=='GET'&&!(r.method==='POST'&&r.url.endsWith('/__rust-preview')))).length,0);assert.deepEqual([...new Set((await state()).calls.map(c=>c.command))].sort(),['model_v2_preview_conditions','model_v2_validate_content']);assert.deepEqual(report.pageErrors,[]);assert.deepEqual(report.console.filter(c=>['error','warning'].includes(c.type)),[]);assert.deepEqual(report.failedRequests,[]);});
}catch(e){report.fatal=e.stack;}finally{report.status=report.fatal||report.checks.some(c=>c.status==='failed')?'failed':'passed';fs.writeFileSync(path.join(out,engine+'-results.json'),JSON.stringify(report,null,2));fs.writeFileSync(path.join(out,'browser-results.json'),JSON.stringify(reports,null,2));await browser.close();}
console.log(JSON.stringify({engine,status:report.status,passed:report.checks.filter(c=>c.status==='passed').length,failed:report.checks.filter(c=>c.status==='failed'),fatal:report.fatal,pageErrors:report.pageErrors,consoleErrors:report.console.filter(c=>c.type==='error')},null,2));
}if(reports.some(r=>r.status==='failed'))process.exitCode=1;})().catch(e=>{console.error(e);process.exitCode=1});
