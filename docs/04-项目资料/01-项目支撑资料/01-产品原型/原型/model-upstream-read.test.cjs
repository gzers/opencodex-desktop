/* 读取现有渠道与软件同口径：读取成功按已保存模版匹配到草稿；人工单模型覆盖保留，保存前不写基线。 */
const {test} = require('node:test'), assert = require('node:assert/strict')
const make = require('./model-prototype-test-harness.cjs')
const policy = h => JSON.stringify({draft: h.state().draft, saved: h.state().saved, pending: h.state().pending, records: h.state().records})

test('列表工具栏与软件一致：读取现有渠道在新增渠道之前，详情不重复出现', () => {
  const h = make(), page = h.page()
  const toolbar = page.slice(page.indexOf('class="mp-list-toolbar"'), page.indexOf('class="mp-provider-table"'))
  assert.match(toolbar, /data-mp="read-upstream"/)
  assert.match(toolbar, /读取现有渠道/)
  assert.ok(toolbar.indexOf('read-upstream') < toolbar.indexOf('new-channel'))
  h.click('edit-provider', { id: 'newapi' })
  assert.doesNotMatch(h.page(), /data-mp="read-upstream"/)
})

test('读取中禁用入口，完成后自动匹配草稿且保留已选与保存基线', () => {
  const h = make(), before = policy(h)
  const selected = h.state().draft.providers.map(p => p.models.filter(m => m.selected).length),saved=JSON.stringify(h.state().saved)
  h.click('read-upstream')
  assert.match(h.page(), /读取中…/)
  assert.match(h.page(), /data-mp="read-upstream" aria-busy="true"[^>]*disabled/)
  assert.equal(policy(h), before)
  h.drain()
  assert.deepEqual(h.state().draft.providers.map(p => p.models.filter(m => m.selected).length), selected)
  assert.match(h.page(), /来源：Codex 配置（演示读取） · 已读取到草稿/)
  assert.notEqual(policy(h), before)
  assert.equal(JSON.stringify(h.state().saved),saved)
  assert.match(h.page(),/自动匹配/)
  assert.ok(h.state().upstreamMatchResult.matched>0)
  assert.doesNotMatch(h.page(), /data-mp="read-upstream"[^>]*disabled/)
})

test('自动匹配产生未保存草稿，重复读取无额外变化，正式保存仍由用户确认',()=>{
 const h=make(),saved=JSON.stringify(h.state().saved);h.click('read-upstream');h.drain();assert.match(h.page(),/data-channel-state="dirty"/);assert.equal(JSON.stringify(h.state().saved),saved);
 const matched=JSON.stringify(h.state().draft);h.click('read-upstream');h.drain();assert.equal(JSON.stringify(h.state().draft),matched);assert.equal(h.state().upstreamMatchResult.updated,0);
 h.click('save');assert.equal(h.nodes.get('modalMask').style.display,'flex');assert.equal(h.state().operation,null);assert.equal(JSON.stringify(h.state().saved),saved);
});

test('读取失败 / 无权限 / 无接口只提示，不写来源也不改草稿', () => {
  for (const scene of ['discovery-fail', 'discovery-unauthorized', 'discovery-unsupported']) {
    const h = make(), before = policy(h)
    h.proto.scene(scene)
    h.click('read-upstream'); h.drain()
    assert.doesNotMatch(h.page(), /读取中…/)
    assert.doesNotMatch(h.page(), /来源：Codex 配置（演示读取）/) 
    assert.equal(policy(h), before)
  }
})

test('保存弹窗不再出现操作说明句，说明只留在原型标签', () => {
  const h = make()
  h.click('save')
  assert.doesNotMatch(h.dialog(), /预览固定当前草稿|这里可直接快速启停|没有未保存修改；可在下方/)
  h.context.closeModal()
  h.click('toggle-provider', { id: 'aimami' })
  h.click('save')
  assert.doesNotMatch(h.dialog(), /预览固定当前草稿|这里可直接快速启停/)
  assert.match(h.dialog(), /is-changed/)
})

test('保存前直接标在渠道与型号上：渠道“改”标记、型号新增 / 停用标记，不再单列变更清单', () => {
  const h = make()
  h.click('edit-provider', { id: 'newapi' })
  h.click('toggle-model', { id: 'newapi::0' })
  h.click('toggle-model', { id: 'newapi::17' })
  h.click('back-channels')
  h.click('save')
  const dialog = h.dialog()
  assert.match(dialog, /mp-preview-delta-tag/)
  assert.match(dialog, /is-changed/)
  assert.match(dialog, /mp-preview-item is-off[^>]*is-removed/)
  assert.match(dialog, /mp-preview-item is-on[^>]*is-added/)
  assert.match(dialog, /本次停用 1 个型号|新启用 1 个型号/)
  assert.doesNotMatch(dialog, /本次变更与写入/)
  assert.doesNotMatch(dialog, /确认后会先备份/)
})

test('默认保存版本自动用于未采用族，单模型明确自定义和人工指定保留',()=>{
 const h=make(),policy=h.state().draft,p=policy.providers.find(p=>p.id==='newapi');policy.globalRefs={};p.refs={};p.models[0].overrides={protocol:'Chat',context:777};p.models[1].templateRef='gpt:2';
 const before=JSON.stringify(policy),result=h.proto.previewReadTemplateMatch(policy),next=result.policy.providers.find(p=>p.id==='newapi');
 assert.equal(JSON.stringify(policy),before);assert.equal(next.models[0].templateRef,'gpt:1');assert.equal(next.models[0].templateAssignment,'auto-read');assert.deepEqual(next.models[0].overrides,{protocol:'Chat',context:777});assert.equal(next.models[1].templateRef,'gpt:2');assert.equal(next.models[1].templateAssignment,undefined);
 assert.equal(next.models.find(m=>m.family==='Claude').templateRef,'claude:1');assert.ok(result.report.explicit>0);
});
test('未知型号与无效模版引用保留原配置，不自动修名字或填能力',()=>{
 const h=make(),policy=h.state().draft,p=policy.providers[0];p.models[0].id='unknown-custom';p.models[0].overrides={protocol:'Chat'};p.models[1].templateRef='missing:9';
 const result=h.proto.previewReadTemplateMatch(policy),next=result.policy.providers[0];assert.deepEqual(next.models[0],p.models[0]);assert.deepEqual(next.models[1],p.models[1]);assert.ok(result.report.unmatched>0);assert.ok(result.report.conflicts>0);
});
test('读取期间新编辑和空/失败结果不触发自动匹配或覆盖',()=>{
 const h=make();h.click('read-upstream');h.click('toggle-provider',{id:'aimami'});const after=JSON.stringify(h.state().draft);h.drain();assert.equal(JSON.stringify(h.state().draft),after);assert.equal(h.state().upstreamMatchResult,null);
 const empty=make(),before=policy(empty);empty.proto.scene('discovery-empty');empty.click('read-upstream');empty.drain();assert.equal(policy(empty),before);assert.equal(empty.state().upstreamMatchResult,null);
});
test('同优先级已保存规则重叠时保留原模型，不任意选择',()=>{
 const h=make();h.open('claude');h.match('include','prefix:gpt-');h.click('save-template');h.runConfirm();
 const policy=h.state().draft,p=policy.providers.find(p=>p.id==='newapi');p.refs.claude='claude:2';const model=JSON.stringify(p.models[0]);
 const r=h.proto.previewReadTemplateMatch(policy);assert.equal(JSON.stringify(r.policy.providers.find(p=>p.id==='newapi').models[0]),model);assert.ok(r.report.conflicts>0);assert.ok(r.report.details.some(x=>x.reason==='同优先级模版冲突'));
});
test('未指定默认版本的新模版不会被读取动作擅自选最新版',()=>{
 const h=make();h.open('gpt');h.click('copy-template');h.match('include','prefix:future-');h.click('save-template');h.runConfirm();
 const t=h.state().templates.at(-1);assert.equal(t.main,0);const policy=h.state().draft,p=policy.providers[0];p.models[0].id='future-model';
 const r=h.proto.previewReadTemplateMatch(policy);assert.equal(r.policy.providers[0].models[0].templateRef,undefined);assert.ok(r.report.unmatched>0);
});
