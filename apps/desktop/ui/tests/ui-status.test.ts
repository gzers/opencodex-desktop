import {mount} from '@vue/test-utils'
import {describe,expect,it} from 'vitest'
import UiStatus from '@/components/ui/UiStatus.vue'
import {saveStateOf} from '@/lib/saveStates'

const render = (props: Record<string, unknown>) => mount(UiStatus, {props})

describe('UiStatus：只接语义 tone，不接颜色名', () => {
  it('五类 tone 各自落在既有 .state-* 类名上', () => {
    for (const [tone, cls] of [
      ['running','state-running'],['starting','state-starting'],['warn','state-warn'],
      ['bad','state-bad'],['muted','state-muted'],
    ] as const) {
      const wrapper = render({tone, label: '示例'})
      expect(wrapper.classes()).toContain('state-pill')
      expect(wrapper.classes()).toContain(cls)
      wrapper.unmount()
    }
  })

  it('不产生 data-tone 别名', () => {
    const wrapper = render({tone: 'warn', label: '待重启'})
    expect(wrapper.attributes('data-tone')).toBeUndefined()
    wrapper.unmount()
  })

  it('直接接 SaveState 时用其 label 与 tone', () => {
    const wrapper = render({state: saveStateOf('committed')})
    expect(wrapper.text()).toContain('已保存 · 待重启')
    expect(wrapper.classes()).toContain('state-warn')
    wrapper.unmount()
  })

  it('阶段名走 title，不占常驻宽度（裁决 #1）', () => {
    const wrapper = render({state: saveStateOf('writing_credentials'), hint: '写入凭据'})
    expect(wrapper.text().trim()).toBe('正在保存')
    expect(wrapper.attributes('title')).toBe('写入凭据')
    wrapper.unmount()
  })

  it('子标签与计数分别渲染，子标签带 · 前缀类', () => {
    const wrapper = render({state: saveStateOf('committed'), sub: '上次保存待重启', count: 12})
    expect(wrapper.text()).toContain('已保存 · 待重启')
    expect(wrapper.find('.ui-status-sub').text()).toBe('上次保存待重启')
    expect(wrapper.find('.ui-status-count').text()).toBe('12')
    wrapper.unmount()
  })

  it('role=status 让读屏能在状态变化时播报', () => {
    expect(render({tone: 'warn', label: 'x'}).attributes('role')).toBe('status')
  })
})

