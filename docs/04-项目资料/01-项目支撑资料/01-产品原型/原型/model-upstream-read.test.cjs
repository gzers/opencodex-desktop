/* 读取现有渠道与软件同口径：官方快照只补发现清单，本地顺序 / 已选 / 配置保留；不进保存差异。 */
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

test('读取中禁用入口并提示，读取完成后显示来源但不动草稿与已选', () => {
  const h = make(), before = policy(h)
  const selected = h.state().draft.providers.map(p => p.models.filter(m => m.selected).length)
  h.click('read-upstream')
  assert.match(h.page(), /读取中…/)
  assert.match(h.page(), /data-mp="read-upstream" aria-busy="true"[^>]*disabled/)
  assert.equal(policy(h), before)
  h.drain()
  assert.deepEqual(h.state().draft.providers.map(p => p.models.filter(m => m.selected).length), selected)
  assert.match(h.page(), /来源：Codex 配置（演示读取） · 已读取到草稿/)
  assert.equal(policy(h), before)
  assert.doesNotMatch(h.page(), /data-mp="read-upstream"[^>]*disabled/)
})

test('读取不建立未保存修改，仍可直接打开保存核对且确认禁用', () => {
  const h = make()
  h.click('read-upstream'); h.drain()
  assert.match(h.page(), /data-channel-state="clean"/)
  h.click('save')
  assert.doesNotMatch(h.dialog(), /本次变更与写入|协议写入计划/)
  assert.equal(h.nodes.get('modalConfirm').disabled, false)
  h.runConfirm(); assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
})

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
