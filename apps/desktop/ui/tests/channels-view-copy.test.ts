import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {describe,expect,it} from 'vitest'

const source=readFileSync(resolve(__dirname,'../src/features/models/ChannelsView.vue'),'utf8')
const editor=readFileSync(resolve(__dirname,'../src/features/models/ChannelEditor.vue'),'utf8')

describe('渠道列表与原型一致的入口口径', () => {
  it('列表上没有原型不存在的“默认渠道”选择器', () => {
    expect(source).not.toContain('>默认渠道<')
    expect(source).not.toContain('保持官方当前默认')
  })

  it('读取现有渠道不会顺手写入默认渠道', () => {
    expect(source).not.toContain('models.channelDraft.defaultChannelId=defaultChannel.id')
  })

  it('保存即预览：列表与详情都只有一个保存入口', () => {
    // UI 规范 §38.1：删除独立预览按钮与激活数胶囊，Tab 右侧为 状态 → 还原修改 → 保存。
    expect(source).not.toContain('预览选择器')
    expect(editor).not.toContain('预览选择器')
    expect(editor).not.toContain("emit('preview')")
    expect(source).not.toContain("previewSource")
    expect(source).toContain('<SelectorPreviewDialog')
    expect(source).toContain('@confirm="confirmSave"')
  })

  it('预览不提供来源切换与数量折叠', () => {
    const preview=readFileSync(resolve(__dirname,'../src/features/models/SelectorPreviewDialog.vue'),'utf8')
    expect(preview).toContain('保存 · 选择器预览')
    expect(preview).not.toContain('预览来源')
    expect(preview).not.toContain('按渠道查看数量')
    expect(preview).not.toContain('UiSelectField')
  })

  it('确认即关闭；弹窗内不再显示保存错误块或过程说明', () => {
    expect(source).toContain('previewOpen.value=false;previewProjection.value=null')
    expect(source).not.toContain('确认后会先备份')
    expect(source).not.toContain('基准已变化，请关闭并重新打开保存预览')
  })

  it('仍保留软件已有的“读取现有渠道”按钮', () => {
    // 该按钮是软件正确实现，原型后补；两侧都要有。
    expect(source).toContain('读取现有渠道')
  })
})

describe('查看草稿入口已移除 (WP-28)', () => {
  it('没有独立的草稿查看弹窗', () => {
    expect(source).not.toContain('查看草稿')
    expect(source).not.toContain('PolicyDraftDialog')
  })
})

