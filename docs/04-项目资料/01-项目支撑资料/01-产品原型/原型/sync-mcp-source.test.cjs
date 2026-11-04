/* 总原型的内存交互回归：同名 MCP 定义不同时在预览里选择来源，node_repl 默认不同步。不读取真实客户端配置、不连接 WebDAV。 */
const test=require('node:test');
const assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');

const plain=value=>JSON.parse(JSON.stringify(value));
const sync=h=>plain(h.w.syncPrototype.state());
const scope=['skill:design','mcp:documentation'];

async function setup(t){
 const h=await make('#overview');
 h.w.confirm=()=>{throw Error('不得追加原生确认');};
 t.after(()=>{try{
  assert.deepEqual(h.network,[]);
  assert.deepEqual(h.errors,[]);
  assert.ok(h.writes.every(([key])=>['ocx-theme','ocx-route'].includes(key)),'仅允许壳层既有缓存');
 }finally{h.dispose();}});
 return h;
}

async function finished(h){
 for(let i=0;i<100&&sync(h).running;i++)await h.wait(50);
 const s=sync(h);assert.equal(s.running,false);assert.ok(s.result);return s.result;
}

function previewConflict(h){
 h.w.syncPrototype.open({tab:'webdav',scope});
 h.w.syncPrototype.setScenario('mcp-source');
 assert.equal(sync(h).scenario,'mcp-source');
 h.click('[data-sx=check]');
 assert.equal(sync(h).dialogKind,'sync-preview');
 const body=h.document.querySelector('#modalBody').textContent;
 for(const word of ['Codex','Claude Code','env','type','选择来源'])assert.match(body,new RegExp(word));
 const picker=h.document.querySelector('[data-sx-mcp-source="mcp:documentation"]');
 assert.ok(picker,'冲突行提供来源选择');assert.equal(picker.value,'','默认暂不同步');
 assert.deepEqual([...picker.options].map(o=>o.value),['','Codex','Claude Code']);
 assert.equal(h.document.querySelectorAll('[data-sx-mcp-source]').length,1,'只有同名定义不同的 MCP 需要选择');
 return picker;
}

test('node_repl 默认不在同步范围，可手动勾选',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav'});
 assert.equal(sync(h).selected.includes('mcp:node_repl'),false);
 assert.ok(sync(h).selected.includes('mcp:documentation'));
 h.click('[data-sx=expand][data-id=mcp]');
 h.change('[data-sx-object="mcp:node_repl"]',true);
 assert.ok(sync(h).selected.includes('mcp:node_repl'));
});

test('未选来源：其余对象照常更新，冲突 MCP 本次跳过并在结果中写明',async t=>{
 const h=await setup(t);previewConflict(h);
 h.click('#modalConfirm');const result=await finished(h);
 const skill=result.items.find(x=>x.id==='skill:design'),mcp=result.items.find(x=>x.id==='mcp:documentation');
 assert.equal(skill.status,'completed');
 assert.equal(mcp.direction,'skip');assert.equal(mcp.deferred,true);assert.match(mcp.reason,/未选择来源/);
 assert.match(result.detail,/待选来源 1 项/);
 assert.equal(sync(h).managedMcp['mcp:documentation'],undefined);
});

test('选定来源后记为受管定义，再次预览不再要求选择',async t=>{
 const h=await setup(t);const picker=previewConflict(h);
 picker.value='Codex';
 h.click('#modalConfirm');const result=await finished(h);
 const mcp=result.items.find(x=>x.id==='mcp:documentation');
 assert.equal(mcp.status,'completed');assert.equal(mcp.mcpSource,'Codex');
 assert.equal(sync(h).managedMcp['mcp:documentation'].source,'Codex');
 h.click('[data-sx=check]');
 assert.equal(h.document.querySelector('[data-sx-mcp-source]'),null);
});

test('导出：未选来源的 MCP 不进包，只剩冲突项时提示选择来源，选定后记录来源',async t=>{
 const h=await setup(t);h.w.syncPrototype.setScenario('mcp-source');
 h.w.syncPrototype.open({tab:'file',scope:['mcp:documentation','skill:design'],action:'export'});
 assert.equal(sync(h).dialogKind,'export');
 assert.match(h.document.querySelector('#modalBody').textContent,/同名 MCP 定义不同/);
 assert.equal(h.document.querySelector('[data-sx-mcp-source="mcp:documentation"]').value,'');
 h.change('#sxEncrypt',false);h.click('#modalConfirm');
 let exp=sync(h).lastExport;assert.ok(exp);assert.deepEqual(exp.objects.filter(id=>!id.startsWith('skill-platform:')),['skill:design']);assert.deepEqual(exp.mcpSources,{});
 const receipt=plain(h.w.prototypeNotificationPublisher.deliveries()).filter(x=>x.eventId==='sync.export.complete').at(-1);
 assert.ok(receipt);

 h.w.syncPrototype.open({tab:'file',scope:['mcp:documentation'],action:'export'});
 h.change('#sxEncrypt',false);h.click('#modalConfirm');
 assert.ok(sync(h).lastExport.objects.every(id=>id.startsWith('skill-platform:')),'冲突MCP不进包，已选平台关联仍可导出');
 h.w.syncPrototype.open({tab:'file',scope:['mcp:documentation'],action:'export'});

 h.document.querySelector('[data-sx-mcp-source="mcp:documentation"]').value='Claude Code';
 h.click('#modalConfirm');
 exp=sync(h).lastExport;assert.deepEqual(exp.objects.filter(id=>!id.startsWith('skill-platform:')),['mcp:documentation']);assert.deepEqual(exp.mcpSources,{'mcp:documentation':'Claude Code'});
 assert.equal(sync(h).managedMcp['mcp:documentation'].source,'Claude Code');
 h.w.syncPrototype.open({tab:'file',scope:['mcp:documentation'],action:'export'});
 assert.equal(h.document.querySelector('[data-sx-mcp-source]'),null,'受管定义不再要求选择');
});

test('详情写明定义来源：冲突与默认排除',async t=>{
 const h=await setup(t);h.w.syncPrototype.setScenario('mcp-source');h.w.syncPrototype.open({tab:'webdav'});
 h.click('[data-sx=expand][data-id=mcp]');
 const detail=id=>{h.click('[data-sx=detail][data-id="'+id+'"]');const text=h.document.querySelector('#modalBody').textContent;h.click('#modalCancel');return text;};
 assert.match(detail('mcp:documentation'),/定义来源.*不同/);
 assert.match(detail('mcp:node_repl'),/默认不同步/);
});
