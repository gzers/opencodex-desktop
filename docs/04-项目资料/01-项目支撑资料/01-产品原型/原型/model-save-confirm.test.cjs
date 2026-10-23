/* 确认保存即关闭：通过则直接开始写入；被门禁 / 基准变化拦下也关闭，改为登记已注册通知。 */
const {test} = require('node:test'), assert = require('node:assert/strict')
const make = require('./model-prototype-test-harness.cjs')
const open = h => { h.click('save'); assert.equal(h.nodes.get('modalMask').style.display, 'flex') }

test('确认保存后弹窗立即关闭并开始写入，不再停留在弹窗里看进度', () => {
  const h = make()
  h.click('toggle-provider', { id: 'aimami' })
  open(h)
  assert.equal(h.nodes.get('modalConfirm').disabled, false)
  h.runConfirm()
  assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
  assert.equal(h.state().operation.status, 'running')
})

test('没有可写入修改时确认仍可点，点了关闭并登记“模型配置未保存”通知', () => {
  const h = make(), blocked = []
  h.context.window.modelSaveBlockedBridge = payload => blocked.push(payload)
  open(h)
  assert.equal(h.nodes.get('modalConfirm').disabled, false)
  assert.match(h.nodes.get('modalConfirm').title, /模型配置未保存/)
  h.runConfirm()
  assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
  assert.equal(blocked.length, 1)
  assert.match(blocked[0].reason, /没有未保存的修改/)
  assert.equal(blocked[0].key, 'gated')
  assert.equal(h.state().operation, null)
})

test('待确认编辑被拦下时关闭弹窗、保留草稿，并给出具体原因', () => {
  const h = make(), blocked = []
  h.context.window.modelSaveBlockedBridge = payload => blocked.push(payload)
  h.click('edit-provider', { id: 'newapi' })
  h.event('input', '[data-channel-config]', { channelConfig: 'name' }, '未确认名字')
  h.click('back-channels')
  open(h)
  h.runConfirm()
  assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
  assert.equal(blocked.length, 1)
  assert.match(blocked[0].reason, /连接编辑/)
  assert.equal(h.state().channelConnections.newapi.name, '未确认名字')
  assert.equal(h.state().operation, null)
})

test('超出激活预算时同样关闭并登记通知，不写入运行配置', () => {
  const h = make(), blocked = []
  h.context.window.modelSaveBlockedBridge = payload => blocked.push(payload)
  h.proto.scene('over-budget')
  open(h)
  h.runConfirm()
  assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
  assert.match(blocked[0].reason, /超出 100/)
  assert.equal(h.state().operation, null)
})

test('弹窗正文不再含内联错误块，失败原因只走通知', () => {
  const h = make()
  open(h)
  assert.doesNotMatch(h.dialog(), /mpSaveError/)
  assert.doesNotMatch(h.dialog(), /确认后会先备份/)
})
