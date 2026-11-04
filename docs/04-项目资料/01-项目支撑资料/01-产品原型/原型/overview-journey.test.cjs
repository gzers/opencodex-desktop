const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
async function setup(t){const h=await make('#overview');h.w.eval("setState('running')");await h.settle();t.after(()=>{try{assert.deepEqual(h.errors,[]);assert.deepEqual(h.network,[])}finally{h.dispose()}});return h}

test('渠道、模型、请求逐级展开并返回，原有节点与全局时间保持',async t=>{
 const h=await setup(t),d=h.document,j=h.w.overviewHeroPrototype.journey(),stage=d.querySelector('#phStage'),source=d.querySelector('#phCh-newapi');
 h.click('#phCh-newapi .ph-orb');assert.equal(j.state().level,'channel');assert.equal(j.state().channel,'newapi');
 assert.equal(d.querySelector('#phCh-newapi'),source);assert.equal(source.classList.contains('pj-selected'),true);
 assert.equal(source.classList.contains('pj-logo-clear'),false,'当前渠道属于前景，不套用后景净空遮罩');
 assert.equal(d.querySelector('.ph-clients').classList.contains('pj-logo-clear'),true,'旧客户端在 Logo 位置局部退让');
 const first=d.querySelector('.pj-model');h.click('.pj-model');assert.equal(j.state().level,'model');
 assert.equal(d.querySelector('.pj-model'),first,'模型对象在深入时保留');
 const request=d.querySelector('.pj-record');h.click('.pj-record');assert.equal(j.state().level,'request');
 assert.equal(d.querySelector('.pj-record'),request);assert.equal(j.state().playing,true);
 assert.equal(d.documentElement.dataset.route,'overview');assert.equal(d.querySelector('#phStage'),stage);
 j.back();assert.equal(j.state().level,'model');assert.equal(j.state().playing,false);assert.equal(d.activeElement,request);
 j.back();assert.equal(j.state().level,'channel');assert.equal(d.activeElement,first);
 j.back();assert.equal(j.state().level,'overview');assert.equal(d.querySelector('#phCh-newapi'),source);
 assert.equal(source.style.translate,'');assert.equal(stage.classList.contains('pj-active'),false);
 assert.equal(d.querySelectorAll('.pj-logo-clear').length,0,'返回后清理后景遮罩，恢复完整概览');
 assert.equal(h.w.overviewHeroPrototype.state().win,'1 天');
});

test('同名模型不会串渠道，时间切换清除单次请求并刷新样例范围',async t=>{
 const h=await setup(t),j=h.w.overviewHeroPrototype.journey();
 j.enter('newapi');j.model('gpt-6.1-sol');let s=j.state();
 assert.equal(s.requests[0].tps,78);assert.equal(s.requests[0].channel,'newapi');
 j.request(s.requests[0].id);h.click('[data-ph-win="1 小时"]');s=j.state();
 assert.equal(s.channel,'newapi');assert.equal(s.model,'gpt-6.1-sol');assert.equal(s.level,'model');assert.equal(s.playing,false);
 assert.ok(s.window.startsWith('1 小时|'));
 assert.equal(h.document.querySelectorAll('.pj-wire.is-replaying').length,0,'范围变化清理旧回放样式');
 const end=new Date('2026-11-02T00:00').getTime();
 assert.ok(s.requests.every(r=>new Date(r.at).getTime()>=end-3600000&&new Date(r.at).getTime()<end));
 j.request(s.requests[0].id);h.w.overviewHeroPrototype.scene('slow');
 assert.equal(j.state().requests[0].tps,27);
 assert.match(h.document.querySelector('.pj-record.is-visible').textContent,/27 t\/s/);
 assert.match(h.document.querySelector('.pj-phases').textContent,/18.42s/);
 assert.equal(j.state().playing,false);
 h.w.overviewHeroPrototype.scene('normal');
 j.enter('krill');j.model('gpt-6.1-sol');s=j.state();
 assert.equal(s.requests[0].channel,'krill');assert.equal(s.requests[0].tps,63);assert.notEqual(s.requests[0].id,'req_newapi_gpt61sol_1');
 const failure=s.requests.find(r=>r.status==='failed');assert.equal(failure.output,0);assert.equal(failure.ttft,null);assert.equal(failure.tps,null);
 assert.equal(j.request('nonexistent-request'),false);assert.equal(j.model('nonexistent-model'),false);
});

test('Escape 每次退一级，切页或环境门禁清理回放和探索状态',async t=>{
 const h=await setup(t),d=h.document,j=h.w.overviewHeroPrototype.journey();
 j.enter('newapi');j.model('gpt-6.1-sol');j.request(j.state().requests[0].id);
 const esc=()=>d.dispatchEvent(new h.w.KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
 h.w.openModal('确认','<p>检查返回优先级</p>','确定',()=>{});
 esc();assert.notEqual(d.querySelector('#modalMask').style.display,'flex');assert.equal(j.state().level,'request','关闭上层 Modal 不推进探索返回');
 h.click('[data-ph-range]');esc();assert.equal(d.querySelector('.ph-rng-host'),null);assert.equal(j.state().level,'request','关闭日历不推进探索返回');
 esc();assert.equal(j.state().level,'model');esc();assert.equal(j.state().level,'channel');esc();assert.equal(j.state().level,'overview');
 j.enter('newapi');j.model('gpt-6.1-sol');j.request(j.state().requests[0].id);
 h.click('.nav button[data-route="models"]');await h.settle();
 assert.equal(j.state().level,'overview');assert.equal(j.state().playing,false);assert.equal(d.querySelector('.pj-scene').getAttribute('aria-hidden'),'true');
 h.click('.nav button[data-route="overview"]');await h.settle();j.enter('newapi');
 h.w.setEnv('missing_ocx');await h.settle();assert.equal(j.state().level,'overview');
});

test('低特效可以完整选择请求，回放直接呈现终态而不启动循环',async t=>{
 const h=await setup(t),d=h.document,j=h.w.overviewHeroPrototype.journey();d.documentElement.dataset.effects='low';
 j.enter('newapi');j.model('gpt-6.1-sol');j.request(j.state().requests[0].id);
 assert.equal(j.state().playing,false);assert.equal(d.querySelector('.pj-inspector').style.getPropertyValue('--pj-progress'),'1');
 assert.match(d.querySelector('.pj-play-status').textContent,/回放结束/);
 j.back();j.back();j.back();assert.equal(j.state().level,'overview');
});

test('详情光点随运行、回放、返回、低动态与退出切换，不残留循环',async t=>{
 const h=await setup(t),w=h.w,d=h.document,j=w.overviewHeroPrototype.journey();
 // jsdom 不测 SVG 几何；这里用确定长度验证光点调度和生命周期，真实沿线坐标另用浏览器验证。
 w.SVGElement.prototype.getTotalLength=()=>200;
 w.SVGElement.prototype.getPointAtLength=d=>({x:d,y:20});
 w.eval("setState('stopped')");assert.equal(j.enter('newapi'),false,'停止状态不开放具体渠道');assert.equal(j.traffic().moving,false);
 w.eval("setState('running')");await h.settle();
 j.enter('newapi');
 assert.equal(j.traffic().moving,true);assert.ok(j.traffic().particles>=3);
 assert.equal(j.traffic().speeds.find(s=>s.path==='model-0').speed,120,'保留 80px/s 基准并按模型速度换算');
 const head=d.querySelector('.pj-ambient-particle .pj-particle-head'),x=head.getAttribute('cx');await h.wait(30);assert.notEqual(head.getAttribute('cx'),x);
 j.model('gpt-6.1-sol');assert.equal(j.traffic().speeds.some(s=>s.path==='request-2'),false,'失败请求不循环表现为成功流量');
 j.request(j.state().requests[0].id);await h.wait(30);
 assert.equal(j.traffic().moving,false);assert.equal(j.traffic().replay,true);
 j.back();assert.equal(j.traffic().replay,false);assert.equal(j.traffic().moving,true);
 d.documentElement.dataset.effects='low';await h.settle();assert.equal(j.traffic().moving,false);assert.equal(j.traffic().particles,0);
 d.documentElement.dataset.effects='high';await h.settle();assert.equal(j.traffic().moving,true);
 w.eval("setState('stopped')");await h.settle();assert.equal(j.traffic().moving,false);
 assert.equal(j.state().level,'overview');assert.equal(j.enter('newapi'),false,'停止后只能查看整体');
 assert.equal(j.request('req_newapi_gpt61sol_1'),false);assert.equal(j.traffic().replay,false);
 j.exit();assert.equal(j.traffic().moving,false);assert.equal(j.traffic().replay,false);assert.equal(j.traffic().particles,0);
});

test('适配渠道各详情层保留原适配节点，流光经过两段链路，直连和返回无残留',async t=>{
 const h=await setup(t),w=h.w,d=h.document,j=w.overviewHeroPrototype.journey(),adapter=d.querySelector('#phAdapter');
 w.SVGElement.prototype.getTotalLength=()=>100;
 w.SVGElement.prototype.getPointAtLength=d=>({x:d,y:20});
 w.eval("setState('running')");j.enter('krill');
 assert.equal(d.querySelector('#phAdapter'),adapter);assert.equal(adapter.classList.contains('pj-adapter-active'),true);
 assert.equal(adapter.getAttribute('role'),'button');assert.equal(adapter.tabIndex,0);assert.match(adapter.getAttribute('aria-label'),/协议转换/);
 assert.deepEqual([...d.querySelectorAll('.pj-wire.is-visible')].map(e=>e.dataset.path),['hub-adapter','adapter-channel','model-0','model-1']);
 assert.ok(j.traffic().speeds.some(p=>p.path==='hub-adapter'));assert.ok(j.traffic().speeds.some(p=>p.path==='adapter-channel'));
 j.model('gpt-6.1-sol');assert.equal(adapter.classList.contains('pj-adapter-active'),true);
 const r=j.state().requests[0];assert.equal(r.via,true);assert.equal(r.adapterMs,18);
 assert.equal(r.duration,+(r.ttft+r.output/r.tps).toFixed(2),'适配已包含在首字时间内，不能重复加到总时长');
 j.request(r.id);assert.equal(adapter.classList.contains('pj-adapter-active'),true);
 assert.equal(d.querySelector('.pj-phases').firstElementChild.querySelector('b').textContent,'适配');
 assert.match(d.querySelector('.pj-phases').textContent,/18ms/);assert.equal(d.querySelector('.pj-phases').children.length,4);
 await h.wait(30);assert.equal(j.traffic().replay,true);
 assert.deepEqual([...d.querySelectorAll('.pj-wire.is-replaying')].slice(0,2).map(e=>e.dataset.path),['hub-adapter','adapter-channel']);
 j.enter('newapi');assert.equal(adapter.classList.contains('pj-adapter-active'),false);assert.equal(adapter.style.translate,'');
 assert.equal(d.querySelectorAll('.pj-wire.is-visible[data-path="hub-channel"]').length,1);
 assert.equal(d.querySelectorAll('.pj-wire.is-visible[data-path="hub-adapter"],.pj-wire.is-visible[data-path="adapter-channel"]').length,0);
 j.model('gpt-6.1-sol');j.request(j.state().requests[0].id);assert.equal(d.querySelector('.pj-phases').children.length,3);
 assert.equal(j.state().requests[0].via,false);assert.equal(j.state().requests[0].adapterMs,null);
 j.enter('krill');j.exit();assert.equal(adapter.classList.contains('pj-adapter-active'),false);assert.equal(adapter.getAttribute('role'),'button');assert.equal(adapter.style.translate,'');
});

test('点击实时适配展开同场景详情，关闭或 Escape 保留原渠道模型请求层级',async t=>{
 const h=await setup(t),d=h.document,j=h.w.overviewHeroPrototype.journey();
 h.click('#phAdapter');assert.equal(j.state().channel,'krill');assert.equal(j.state().adapterOpen,true);
 const panel=d.querySelector('.pj-adapter-detail');assert.equal(panel.inert,false);assert.match(panel.textContent,/协议转换/);
 assert.equal(panel.querySelector('[data-pj-adapter-target]').textContent,'Krill');assert.match(panel.textContent,/18ms/);
 assert.equal(d.querySelector('#phAdapter').getAttribute('aria-expanded'),'true');
 h.click('[data-pj="adapter-close"]');assert.equal(j.state().adapterOpen,false);assert.equal(j.state().level,'channel');
 j.model('gpt-6.1-sol');j.request(j.state().requests[0].id);const id=j.state().request;
 h.click('#phAdapter');assert.equal(j.state().adapterOpen,true);assert.equal(j.state().playing,false);
 d.dispatchEvent(new h.w.KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
 assert.equal(j.state().adapterOpen,false);assert.equal(j.state().request,id);assert.equal(j.state().level,'request');
 h.click('#phAdapter');h.click('[data-ph-win="1 小时"]');assert.equal(j.state().adapterOpen,true);assert.match(panel.querySelector('.pj-adapter-window').textContent,/1 小时/);
 assert.match(panel.querySelector('[data-pj-adapter-requests]').textContent,/21/);assert.equal(j.state().request,null);
});

test('未运行仅整体汇总，所有细节入口关闭，启动恢复，停止时清理正在查看的详情',async t=>{
 const h=await setup(t),w=h.w,d=h.document,api=w.overviewHeroPrototype,j=api.journey();
 w.eval("setState('stopped')");await h.settle();
 assert.equal(d.querySelector('#phStage').classList.contains('ph-overview-only'),true);
 for(const selector of ['.ph-clients','.ph-channels','#phAdapter'])assert.equal(d.querySelector(selector).inert,true);
 assert.equal(j.enter('krill'),false);assert.equal(j.adapter(true),false);assert.equal(j.request('req_krill_gpt61sol_1'),false);
 api.setFocus('krill');assert.equal(api.state().focus,null);
 api.setFocus('desktop');const pop=d.querySelector('#phPop');assert.match(pop.textContent,/整体汇总/);assert.equal(pop.querySelector('.ph-mt'),null);assert.doesNotMatch(pop.textContent,/Krill|NewApi|gpt-6/);
 h.click('[data-ph-win="1 小时"]');assert.match(pop.textContent,/0.13M/);
 w.eval("setState('running')");await h.settle();assert.equal(d.querySelector('#phStage').classList.contains('ph-overview-only'),false);
 assert.equal(d.querySelector('.ph-channels').inert,false);j.enter('krill');j.model('gpt-6.1-sol');j.request(j.state().requests[0].id);j.adapter(true);
 w.eval("setState('stopped')");await h.settle();assert.equal(j.state().level,'overview');assert.equal(j.state().adapterOpen,false);assert.equal(j.state().playing,false);assert.equal(j.traffic().particles,0);
 for(const next of ['starting','starting_failed']){w.eval("setState('"+next+"')");await h.settle();assert.equal(j.enter('krill'),false);assert.equal(d.querySelector('#phStage').classList.contains('ph-overview-only'),true)}
});

test('四个 Agent 平台各自深入，样例归属独立，不复用 codex 入口模型或混合速度',async t=>{
 const h=await setup(t),d=h.document,j=h.w.overviewHeroPrototype.journey();
 for(const key of ['app','opencode','cli','claude']){
  const source=d.querySelector('[data-client="'+key+'"]');h.click('[data-client="'+key+'"] .ph-orb');
  assert.equal(j.state().level,'platform');assert.equal(j.state().platform,key);assert.equal(source.classList.contains('pj-source'),true);assert.equal(d.querySelector('[data-client="'+key+'"]'),source);
  assert.equal(source.classList.contains('pj-logo-clear'),false);assert.doesNotMatch(d.querySelector('.pj-model').textContent,/t\/s/);
  h.click('.pj-model.is-visible');assert.equal(j.state().level,'model');assert.ok(j.state().requests.every(r=>r.platform===key));
  const r=j.state().requests[0];j.request(r.id);assert.equal(j.state().channel,r.channel);assert.equal(d.querySelector('.pj-selected').dataset.k,r.channel);
  j.back();assert.equal(j.state().channel,null);assert.equal(j.state().level,'model');
  j.back();assert.equal(j.state().level,'platform');j.back();assert.equal(j.state().level,'overview');assert.equal(source.style.translate,'');assert.equal(d.querySelectorAll('.pj-source').length,0);
 }
});

test('同一请求可在平台与渠道双向切入，ID、速度和适配阶段一致',async t=>{
 const h=await setup(t),d=h.document,j=h.w.overviewHeroPrototype.journey();j.platform('app');j.model('gpt-6.1-sol');
 const r=j.state().requests.find(r=>r.channel==='krill');j.request(r.id);
 assert.match(d.querySelector('.pj-request-origin').textContent,/Codex App → Krill/);
 assert.equal(d.querySelector('.pj-phases').children.length,4);assert.ok(d.querySelector('.pj-wire.is-visible[data-path="platform-hub"]'));
 h.click('[data-pj="counterpart"]');assert.equal(j.state().entry,'channel');assert.equal(j.state().channel,'krill');assert.equal(j.state().request,r.id);assert.match(d.querySelector('.pj-counterpart').textContent,/从平台查看/);
 assert.equal(j.state().requests.find(x=>x.id===r.id).tps,r.tps);
 h.click('[data-pj="counterpart"]');assert.equal(j.state().entry,'platform');assert.equal(j.state().platform,'app');assert.equal(j.state().request,r.id);assert.match(d.querySelector('.pj-counterpart').textContent,/从渠道查看/);
 j.adapter(true);j.back();assert.equal(j.state().request,r.id);assert.equal(j.state().platform,'app');
 h.click('[data-ph-win="1 小时"]');assert.equal(j.state().platform,'app');assert.equal(j.state().model,'gpt-6.1-sol');assert.equal(j.state().request,null);assert.equal(j.state().channel,null);
 h.w.eval("setState('stopped')");await h.settle();assert.equal(j.state().level,'overview');assert.equal(j.platform('app'),false);assert.equal(j.traffic().particles,0);
});

test('平台模型和请求分组可继续查看，选中后返回保留分组位置',async t=>{
 const h=await setup(t),d=h.document,j=h.w.overviewHeroPrototype.journey();j.platform('opencode');
 assert.equal(d.querySelectorAll('.pj-model.is-visible').length,3);h.click('[data-pj="models-next"]');assert.equal(d.querySelector('.pj-model-pages span').textContent,'2 / 2');
 const name=d.querySelector('.pj-model.is-visible').dataset.model;j.model(name);j.back();assert.equal(d.querySelector('.pj-model.is-visible').dataset.model,name);
 j.platform('app');j.model('gpt-6.1-sol');assert.equal(d.querySelectorAll('.pj-record.is-visible').length,3);h.click('[data-pj="requests-next"]');
 const record=d.querySelector('.pj-record.is-visible');j.request(record.dataset.request);j.back();assert.equal(d.querySelector('.pj-record.is-visible'),record);
});
