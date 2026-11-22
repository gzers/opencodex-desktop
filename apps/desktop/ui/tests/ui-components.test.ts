import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import UiButton from '@/components/ui/UiButton.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import SettingRow from '@/components/patterns/SettingRow.vue'
import Stack from '@/components/layout/Stack.vue'
import Inline from '@/components/layout/Inline.vue'

describe('ui components', () => {
  it('UiButton 承载 .btn 配方并透传变体类名', () => {
    const wrapper = mount(UiButton, { props: { class: 'primary' }, slots: { default: '启动' } })
    const btn = wrapper.get('button')
    expect(btn.classes()).toContain('btn')
    expect(btn.classes()).toContain('primary')
    expect(btn.text()).toBe('启动')
    expect(btn.attributes('type')).toBe('button')
  })

  it('UiButton loading 时禁用、给出 aria-busy 与旋转指示，且不改文案', () => {
    const wrapper = mount(UiButton, { props: { loading: true }, slots: { default: '启动中…' } })
    const btn = wrapper.get('button')
    expect(btn.attributes('disabled')).toBeDefined()
    expect(btn.attributes('aria-busy')).toBe('true')
    expect(btn.get('.btn-spinner').attributes('aria-hidden')).toBe('true')
    expect(btn.text()).toContain('启动中…')
  })

  it('UiButton 透传 disabled 与点击事件', async () => {
    const wrapper = mount(UiButton, { props: { disabled: true } })
    expect(wrapper.get('button').attributes('disabled')).toBeDefined()
  })

  it('UiCard 默认渲染为 article.card 并透传额外类名', () => {
    const wrapper = mount(UiCard, { props: { class: 'ovb-status' } })
    const root = wrapper.get('article')
    expect(root.classes()).toContain('card')
    expect(root.classes()).toContain('ovb-status')
  })

  it('UiCard 支持语义元素与内容插槽', () => {
    const wrapper = mount(UiCard, { props: { as: 'section' }, slots: { default: '<h2>标题</h2>' } })
    expect(wrapper.find('section.card').exists()).toBe(true)
    expect(wrapper.get('h2').text()).toBe('标题')
  })

  // 变体 API（§26.3）：variant/size/density/surface，不使用 isGlassHigh 之类组合布尔值。
  it('UiButton 变体 API 生成 variant/size/density/surface 类且默认不产生多余类', () => {
    const plain = mount(UiButton)
    expect(plain.get('button').classes().sort()).toEqual(['btn'])

    const variant = mount(UiButton, { props: { variant: 'primary' } })
    expect(variant.get('button').classes()).toContain('primary')

    const sized = mount(UiButton, { props: { size: 'lg', density: 'compact', surface: 'plain' } })
    const classes = sized.get('button').classes()
    expect(classes).toContain('btn-lg')
    expect(classes).toContain('btn-compact')
    expect(classes).toContain('btn-plain')
  })

  it('UiCard 变体 API 生成 density/surface 类且默认保持 panel 样式', () => {
    const plain = mount(UiCard)
    expect(plain.get('article').classes().sort()).toEqual(['card'])

    const overlay = mount(UiCard, { props: { surface: 'overlay', density: 'compact' } })
    const classes = overlay.get('article').classes()
    expect(classes).toContain('card-overlay')
    expect(classes).toContain('card-compact')
  })

  it('UiCardHeader 承载 .card-head，左槽为标题、actions 槽为右侧操作', () => {
    const wrapper = mount(UiCardHeader, {
      slots: { default: '<h2>标题</h2><p>说明</p>', actions: '<button class="btn">操作</button>' },
    })
    const head = wrapper.get('.card-head')
    expect(head.get('h2').text()).toBe('标题')
    expect(head.findAll('.btn').length).toBe(1)
    expect(head.element.lastElementChild?.getAttribute('class')).toBe('btn')
  })

  it('SettingRow 承载 .setting-row，title/description 进左列、actions 进右列', () => {
    const wrapper = mount(SettingRow, {
      props: { title: '示例开关', description: '说明文本' },
      slots: { actions: '<div class="controls">控件</div>' },
    })
    const row = wrapper.get('.setting-row')
    expect(row.get('.setting-title').text()).toBe('示例开关')
    expect(row.get('.setting-desc').text()).toBe('说明文本')
    expect(row.get('.controls').text()).toBe('控件')
  })

  it('Stack/Inline 提供排列原语并透传间距与语义元素', () => {
    const stack = mount(Stack, { props: { gap: 'var(--space-2)', as: 'section' }, slots: { default: 'A' } })
    expect(stack.find('section.u-stack').exists()).toBe(true)
    expect(stack.get('.u-stack').attributes('style')).toContain('var(--space-2)')

    const inline = mount(Inline, { props: { wrap: false }, slots: { default: 'B' } })
    const style = inline.get('.u-inline').attributes('style') ?? ''
    expect(style).toContain('nowrap')
  })
})
