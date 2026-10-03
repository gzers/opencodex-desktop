import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ClientIcon from '@/features/extensions/components/ClientIcon.vue'
import { clientIcons } from '@/features/extensions/clientIcons'
import { skillClientOrder } from '@/features/extensions/presentation'

describe('client brand icons', () => {
  it('covers every sync target with non-empty path geometry', () => {
    for (const client of skillClientOrder) {
      const icon = clientIcons[client]
      expect(icon, `${client} 缺少图标`).toBeTruthy()
      expect(icon.viewBox).toBe('0 0 24 24')
      expect(icon.paths.length).toBeGreaterThan(0)
      for (const path of icon.paths) {
        expect(path.d.length).toBeGreaterThan(0)
      }
    }
  })

  it('renders a distinct shape per client instead of a shared placeholder circle', () => {
    // 回归：此前所有目标图标都是同一个占位圆，原型要求显示各客户端品牌图标。
    const shapes = skillClientOrder.map((client) => clientIcons[client].paths.map((p) => p.d).join('|'))
    expect(new Set(shapes).size).toBe(skillClientOrder.length)
    for (const shape of shapes) {
      expect(shape).not.toContain('M12 12')
    }
  })

  it('renders an inline svg with the client path data', () => {
    const wrapper = mount(ClientIcon, { props: { client: 'claude' } })
    const svg = wrapper.find('svg.target-svg')
    expect(svg.exists()).toBe(true)
    expect(svg.attributes('viewBox')).toBe('0 0 24 24')
    expect(svg.findAll('path').length).toBe(clientIcons.claude.paths.length)
    expect(svg.find('path').attributes('d')).toBe(clientIcons.claude.paths[0].d)
  })
})
