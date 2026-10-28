const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const clean=h=>{assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);};
const node=(h,id)=>h.click('[data-pv="node"][data-id="'+id+'"]');
const tab=(h,id)=>h.click('#modal [data-pv="tab"][data-tab="'+id+'"]');
test('默认聚合显示Desktop出站支路，普通渠道直达，原概览和业务状态独立',async t=>{
 const h=await make('#conversations'),{w,document:d}=h;t.after(()=>h.dispose());const before=JSON.stringify(w.conversationPrototype.state().saved),models=JSON.stringify(w.modelPrototype.state().saved),frame=d.querySelector('.pv-backdrop');assert.equal(frame.hasAttribute('src'),false);
 h.click('[data-launch="connection-map"]');await h.settle();const s=w.connectionMapPrototype.state();assert.ok(s.connections.some(e=>e.from==='proxy'&&e.to==='desktop'));assert.ok(s.connections.some(e=>e.from==='desktop'&&e.to==='krill'&&e.kind==='config'));assert.equal(s.connections.some(e=>e.from==='adapter'||e.to==='adapter'),false);assert.equal(d.querySelector('[data-node="adapter"]'),null);assert.equal(d.querySelector('[data-node="sync"]'),null);assert.equal(s.connections.some(e=>e.from==='proxy'&&e.to==='krill'),false);assert.ok(s.connections.some(e=>e.from==='proxy'&&e.to==='newapi'));assert.equal(d.querySelectorAll('.pv-service').length,1);assert.equal(d.querySelector('#pvFloat'),null);assert.equal(d.querySelector('.pv-node-actions'),null);assert.match(frame.src,/layout=hero/);assert.equal(JSON.stringify(w.conversationPrototype.state().saved),before);assert.equal(JSON.stringify(w.modelPrototype.state().saved),models);clean(h);
});
test('所有节点点击直接开居中Tab弹窗，关闭不会丢失来源',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());
 for(const id of ['codex','claude','opencode','proxy','desktop','newapi','krill']){
  node(h,id);await h.settle();assert.equal(w.connectionMapPrototype.state().dialog.type,'node');assert.equal(w.connectionMapPrototype.state().dialog.id,id);assert.ok(d.querySelector('#modal.pv-dialog'));assert.ok(d.querySelector('#modal [role="tablist"]'));assert.equal(d.querySelector('#modalMask').style.display,'flex');h.click('#modalCancel');await h.settle();assert.equal(w.connectionMapPrototype.state().dialog,null);assert.equal(d.activeElement.dataset.id,id);
 }
 clean(h);
});
test('概览设置跨Tab保留草稿，取消与应用分开，已安装未接入不连线',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());const s=()=>w.connectionMapPrototype.state(),before=JSON.stringify(w.modelPrototype.state().saved);
 h.click('[data-pv="settings"]');assert.equal(d.querySelector('[data-client="cursor"]').disabled,true);h.change('[data-client="codex"]',false);h.change('[data-client="gemini"]',true);tab(h,'channels');h.change('[data-pin="official"]',true);tab(h,'display');h.change('[data-option="sync"]',false);tab(h,'clients');assert.equal(d.querySelector('[data-client="gemini"]').checked,true);h.click('#modalCancel');assert.equal(s().preferences.clients.includes('gemini'),false);assert.equal(s().preferences.sync,true);
 h.click('[data-pv="settings"]');h.change('[data-client="codex"]',false);h.change('[data-client="gemini"]',true);tab(h,'channels');h.change('[data-pin="official"]',true);tab(h,'display');h.change('[data-option="sync"]',false);h.click('#modalConfirm');assert.ok(s().visibleClients.some(c=>c.id==='gemini'));assert.equal(s().connections.some(e=>e.from==='gemini'),false);assert.ok(s().preferences.pins.includes('official'));assert.equal(s().preferences.sync,false);assert.equal(JSON.stringify(w.modelPrototype.state().saved),before);clean(h);
});
test('居中弹窗支持Tab方向键、Esc与遮罩取消，清理专题样式',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());h.click('[data-pv="settings"]');await h.settle();const first=d.querySelector('#pv-tab-clients');first.focus();first.dispatchEvent(new w.KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true}));assert.equal(w.connectionMapPrototype.state().dialog.tab,'channels');assert.equal(d.activeElement.id,'pv-tab-channels');h.change('[data-pin="official"]',true);d.activeElement.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Escape',bubbles:true}));await h.settle();assert.equal(w.connectionMapPrototype.state().dialog,null);assert.equal(w.connectionMapPrototype.state().preferences.pins.includes('official'),false);assert.equal(d.activeElement.dataset.pv,'settings');
 h.click('[data-pv="settings"]');h.change('[data-client="codex"]',false);h.click('#modalMask');assert.ok(w.connectionMapPrototype.state().preferences.clients.includes('codex'));assert.equal(d.querySelector('#modal').classList.contains('pv-dialog'),false);clean(h);
});
test('大量渠道和客户端由居中清单搜索临时展开，不改变聚合目录',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());const s=()=>w.connectionMapPrototype.state();h.click('[data-pv-scene="many"]');assert.equal(s().hiddenChannels.length,27);h.click('[data-pv="channels"]');assert.equal(s().dialog.type,'channels');const input=d.querySelector('#pvChannelSearch');h.input('#pvChannelSearch','渠道 21');assert.equal(d.querySelector('#pvChannelSearch'),input);assert.equal(d.querySelectorAll('#modal [data-pv="expand"]').length,1);h.click('#modal [data-pv="expand"]');assert.equal(s().visibleChannels[0].id,'extra-20');assert.equal(s().hiddenChannels.length,27);assert.equal(s().dialog,null);
 h.click('[data-pv="settings"]');h.change('[data-client="codex-cli"]',true);h.change('[data-client="gemini"]',true);h.click('#modalConfirm');assert.equal(s().hiddenClients.length,2);h.click('[data-pv="clients"]');h.click('#modal [data-pv="expand-client"][data-id="gemini"]');assert.equal(s().visibleClients[0].id,'gemini');assert.equal(s().connections.some(e=>e.from==='gemini'),false);clean(h);
});
test('管理显隐不移除参与出站的Desktop及归属，适配不会声称实际采用',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());h.click('[data-pv="settings"]');tab(h,'display');h.change('[data-option="management"]',false);h.click('#modalConfirm');const s=w.connectionMapPrototype.state();assert.ok(d.querySelector('[data-node="desktop"]'));assert.ok(s.connections.some(e=>e.from==='proxy'&&e.to==='desktop'));assert.ok(s.connections.some(e=>e.from==='desktop'&&e.to==='krill'));assert.equal(s.connections.some(e=>e.kind==='manage'),false);assert.equal(s.connections.some(e=>e.to==='desktop'&&e.kind==='observed'),false);node(h,'desktop');tab(h,'connection');assert.match(d.querySelector('#modalBody').textContent,/未接入生产/);h.click('#modalCancel');h.click('[data-pv-scene="adapter"]');assert.equal(w.connectionMapPrototype.state().connections.some(e=>e.kind==='observed'),false);clean(h);
});
test('直连、配置、停止和过期保持原语义，动作集中在节点操作Tab',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());const s=()=>w.connectionMapPrototype.state();h.click('[data-pv-scene="direct"]');assert.ok(s().connections.some(e=>e.bypass));assert.equal(s().connections.some(e=>e.kind!=='manage'&&e.to==='desktop'),false);h.click('[data-pv-scene="configured"]');assert.equal(s().connections.some(e=>e.kind==='observed'),false);
 node(h,'proxy');tab(h,'actions');h.click('#modal [data-pv="stop"]');assert.equal(s().running,false);assert.equal(s().dialog.tab,'actions');assert.ok(d.querySelector('#modal [data-pv="start"]'));h.click('#modal [data-pv="start"]');assert.equal(s().running,true);h.click('#modalCancel');h.click('[data-pv-scene="unavailable"]');assert.equal(s().motion,'stale');assert.match(d.querySelector('#pvRequest').textContent,/上次采样/);clean(h);
});
test('节点模型Tab往返公共选择器，取消返回原Tab且不改变业务模型',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());const before=JSON.stringify(w.modelPrototype.state().saved);node(h,'newapi');tab(h,'model');h.click('#modal [data-pv="model"]');assert.equal(w.modelPickerPrototype.state().businessIntegration,true);assert.equal(w.connectionMapPrototype.state().dialog,null);assert.ok(d.querySelector('.mpk-table'));h.click('#modalCancel');await h.settle();assert.deepEqual(JSON.parse(JSON.stringify(w.connectionMapPrototype.state().dialog)),{type:'node',id:'newapi',tab:'model'});assert.ok(d.querySelector('#modal.pv-dialog'));assert.equal(JSON.stringify(w.modelPrototype.state().saved),before);h.click('#modalCancel');clean(h);
});
test('具体请求路径与同次回退不混合多客户端，后台适配按目标恢复',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());node(h,'claude');tab(h,'connection');h.click('#modal [data-pv="focus-path"]');h.click('[data-pv="request"]');const s=w.connectionMapPrototype.state();assert.equal(s.connections.filter(e=>e.kind!=='manage'&&e.to==='proxy').length,1);assert.equal(s.connections.find(e=>e.kind!=='manage'&&e.to==='proxy').from,'claude');assert.equal(s.adapter,false);h.click('[data-pv-scene="fallback"]');assert.ok(w.connectionMapPrototype.state().connections.some(e=>e.to==='krill'&&e.kind==='warning'));assert.ok(w.connectionMapPrototype.state().connections.some(e=>e.to==='newapi'&&e.kind==='observed'));clean(h);
});
test('同一Logo生命周期及显示草稿只属本专题，离开关闭居中弹窗',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());const frame=d.querySelector('.pv-backdrop');h.click('[data-pv="settings"]');tab(h,'display');h.change('[data-option="sync"]',false);d.documentElement.dataset.theme='dark';await h.settle();assert.ok(w.connectionMapPrototype.state().draft);assert.equal(d.querySelector('.pv-backdrop'),frame);h.click('[data-route="overview"]');await h.settle();assert.equal(w.connectionMapPrototype.state().dialog,null);assert.equal(d.querySelector('#connectionMapPage').dataset.active,'false');h.click('[data-launch="connection-map"]');await h.settle();assert.equal(w.connectionMapPrototype.state().preferences.sync,true);assert.equal(d.querySelector('.pv-backdrop'),frame);clean(h);
});

test('公共选择器确认只回节点预览，保持居中模型Tab与独立业务草稿',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());const before=JSON.stringify(w.modelPrototype.state().saved);node(h,'newapi');tab(h,'model');h.click('#modal [data-pv="model"]');const key=JSON.stringify(['channel','newapi','gpt-5.4']);const row=[...d.querySelectorAll('.mpk-model-name')].find(el=>el.dataset.key===key);assert.ok(row);row.click();h.click('#modalConfirm');await h.settle();assert.equal(w.connectionMapPrototype.state().dialog.tab,'model');assert.equal(w.connectionMapPrototype.state().modelPreviews.newapi.model,'gpt-5.4');assert.match(d.querySelector('#modalBody').textContent,/gpt-5.4/);assert.equal(JSON.stringify(w.modelPrototype.state().saved),before);h.click('#modalCancel');clean(h);
});

test('Desktop动态品牌与OpenCodex服务标识各归其节点，详情版本不串用',async t=>{
 const h=await make('#connection-map'),{document:d}=h;t.after(()=>h.dispose());
 const desktop=d.querySelector('[data-node="desktop"]'),proxy=d.querySelector('[data-node="proxy"]');
 assert.ok(desktop.querySelector('.pv-desktop-logo-hit'));assert.match(desktop.textContent,/OpenCodex Desktop/);assert.match(desktop.textContent,/v0\.2\.17/);assert.equal(proxy.querySelector('.pv-desktop-logo-hit'),null);
 assert.match(proxy.querySelector('.pv-service-logo-light').getAttribute('src'),/opencodex-mark-light\.svg$/);assert.match(proxy.querySelector('.pv-service-logo-dark').getAttribute('src'),/opencodex-mark-dark\.svg$/);assert.match(proxy.textContent,/v2\.84\.0/);
 node(h,'desktop');assert.equal(d.querySelector('#modalTitle').textContent,'OpenCodex Desktop');tab(h,'sync');assert.match(d.querySelector('#modalBody').textContent,/WebDAV/);assert.ok(d.querySelector('#modal [data-pv="sync"][data-tab="file"]'));h.click('#modalCancel');node(h,'proxy');assert.equal(d.querySelector('#modalTitle').textContent,'OpenCodex');h.click('#modalCancel');clean(h);
});


test('生成中的响应流只沿当前请求返回，适配目标仍保留自己的配置路径',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());
 h.click('[data-pv-scene="streaming"]');await h.settle();const s=w.connectionMapPrototype.state();
 assert.equal(s.livePhase,'streaming');assert.equal(s.usage.input,12840);
 assert.deepEqual(JSON.parse(JSON.stringify(s.connections.filter(e=>e.live).map(e=>[e.from,e.to]))),[['codex','proxy'],['proxy','newapi']]);
 assert.ok(s.connections.filter(e=>e.live).every(e=>e.reverse));
 assert.ok(s.connections.some(e=>e.from==='desktop'&&e.to==='krill'&&e.kind==='config'&&!e.live));
 assert.equal(s.connections.some(e=>e.from==='proxy'&&e.to==='krill'),false);
 assert.equal(d.querySelectorAll('.pv-flow').length,2);assert.equal(d.querySelectorAll('.pv-wire[marker-start]').length,2);
 assert.match(d.querySelector('#pvActivity').textContent,/正在生成.*12,840/);clean(h);
});
test('发送待用量，完成取消失败和历史场景均停止流动，数值不被当作最终计费',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());
 h.click('[data-pv-scene="requesting"]');await h.settle();assert.equal(w.connectionMapPrototype.state().usage.input,null);
 assert.ok(w.connectionMapPrototype.state().connections.filter(e=>e.live).every(e=>!e.reverse));assert.match(d.querySelector('#pvActivity').textContent,/用量待返回/);
 for(const scene of ['completed','cancelled','failed','recent','configured','adapter','stopped','unavailable']){
  h.click('[data-pv-scene="'+scene+'"]');await h.settle();assert.equal(w.connectionMapPrototype.state().flowing,false);assert.equal(d.querySelectorAll('.pv-flow').length,0);
  if(['cancelled','failed'].includes(scene))assert.match(d.querySelector('#pvActivity').textContent,/用量待核对/);
 }
 clean(h);
});
test('流动和用量的展示偏好取消不生效，应用只改变专题投影',async t=>{
 const h=await make('#connection-map'),{w,document:d}=h;t.after(()=>h.dispose());const saved=JSON.stringify(w.modelPrototype.state().saved);
 h.click('[data-pv-scene="streaming"]');h.click('[data-pv="settings"]');tab(h,'display');h.change('[data-option="flow"]',false);h.change('[data-option="usage"]',false);h.click('#modalCancel');
 assert.equal(w.connectionMapPrototype.state().preferences.flow,true);
 h.click('[data-pv="settings"]');tab(h,'display');h.change('[data-option="flow"]',false);h.change('[data-option="usage"]',false);h.click('#modalConfirm');await h.settle();
 assert.equal(d.querySelectorAll('.pv-flow').length,0);assert.equal(d.querySelector('#pvActivity .pv-token-count'),null);assert.equal(w.connectionMapPrototype.state().flowing,true);assert.equal(JSON.stringify(w.modelPrototype.state().saved),saved);clean(h);
});
