import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import MarkdownContent from '@/components/patterns/MarkdownContent.vue'

// IMP-04 §7.11 / §13.6：MarkdownContent 是唯一的 v-html 边界，且保持受限渲染契约。
describe('MarkdownContent', () => {
  it('渲染受限 Markdown，但不执行内联 HTML', () => {
    const wrapper = mount(MarkdownContent, {
      props: { source: '# 标题\n\n<script>alert(1)</script>\n\n正文 <b>粗</b>' },
    })
    const root = wrapper.get('.markdown-content')
    expect(root.get('h1').text()).toBe('标题')
    expect(root.find('script').exists()).toBe(false)
    expect(root.find('b').exists()).toBe(false)
    expect(root.text()).toContain('<script>')
  })

  it('链接只保留文字，不生成可跳转的 href', () => {
    const wrapper = mount(MarkdownContent, { props: { source: '[官方文档](https://example.com/doc)' } })
    const root = wrapper.get('.markdown-content')
    expect(root.find('a').exists()).toBe(false)
    expect(root.text()).toContain('官方文档')
    expect(root.text()).not.toContain('example.com')
  })

  it('空源渲染为空且不报错', () => {
    const wrapper = mount(MarkdownContent, { props: { source: null } })
    expect(wrapper.get('.markdown-content').text()).toBe('')
  })
})
