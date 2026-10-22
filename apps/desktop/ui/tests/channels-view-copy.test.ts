import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {describe,expect,it} from 'vitest'

const source=readFileSync(resolve(__dirname,'../src/features/models/ChannelsView.vue'),'utf8')

describe('渠道列表与原型一致的入口口径', () => {
  it('列表上没有原型不存在的“默认渠道”选择器', () => {
    // 原型只有预览选择 / 状态 / 还原修改 / 保存；默认渠道会替用户改官方默认，超出本页职责。
    expect(source).not.toContain('>默认渠道<')
    expect(source).not.toContain('保持官方当前默认')
  })

  it('读取现有渠道不会顺手写入默认渠道', () => {
    expect(source).not.toContain('models.channelDraft.defaultChannelId=defaultChannel.id')
  })

  it('“预览选择器”打开只读预览，而不是再给一个可选项列表', () => {
    expect(source).toContain("import SelectorPreviewDialog from './SelectorPreviewDialog.vue'")
    expect(source).toContain('<SelectorPreviewDialog')
    // 预览弹窗不再复用公共选择器的确认写入口。
    expect(source).not.toContain('<ModelPicker :open="pickerOpen"')
  })
})

describe('查看草稿入口已移除 (WP-28)', () => {
  it('顶部操作行只剩预览选择器 / 状态 / 还原修改 / 保存', () => {
    expect(source).not.toContain('查看草稿')
    expect(source).not.toContain('PolicyDraftDialog')
  })
})
