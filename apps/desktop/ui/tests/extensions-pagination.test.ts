import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ExtensionsRoute from '@/routes/ExtensionsRoute.vue'
import type { ExtensionsDto, ExtensionSkillDto } from '@/features/extensions/api'
import { useExtensionsStore } from '@/features/extensions/store'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

function makeSkills(count: number): ExtensionSkillDto[] {
  return Array.from({ length: count }, (_, index) => ({
    id: `skill-${index + 1}`,
    name: `skill-${String(index + 1).padStart(2, '0')}`,
    source: 'agents',
    updatedAt: '2026-09-23T00:00:00Z',
    description: `第 ${index + 1} 个 Skill`,
    // 偶数落 Claude，奇数落 Codex：用于验证客户端筛选真实生效。
    clients: index % 2 === 0 ? ['claude'] : ['codex'],
  }))
}

function extensions(): ExtensionsDto {
  return {
    skills: makeSkills(12),
    servers: [],
    sourceDir: '/fixtures/.agents/skills',
    sourceDirNotice: null,
    sourceDirCustom: false,
    syncMethod: 'symlink',
  }
}

function mountRoute() {
  const pinia = createPinia()
  setActivePinia(pinia)
  useExtensionsStore().discovered = extensions()
  return mount(ExtensionsRoute, { global: { plugins: [pinia] } })
}

describe('extensions pagination and filters', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(null)
  })

  it('paginates the skills list instead of showing a static page count', async () => {
    const wrapper = mountRoute()
    await nextTick()
    expect(wrapper.findAll('.skill-row').length).toBe(8)
    expect(wrapper.find('.ext-pagination span').text()).toContain('12 / 12 条 · 第 1 / 2 页')

    await wrapper.find('[aria-label="下一页"]').trigger('click')
    await nextTick()
    expect(wrapper.findAll('.skill-row').length).toBe(4)
    expect(wrapper.find('.ext-pagination span').text()).toContain('第 2 / 2 页')

    await wrapper.find('[aria-label="首页"]').trigger('click')
    await nextTick()
    expect(wrapper.findAll('.skill-row').length).toBe(8)
    await wrapper.unmount()
  })

  it('actually hides rows that do not match a client filter chip', async () => {
    const wrapper = mountRoute()
    await nextTick()
    const claudeChip = wrapper.findAll('.skills-chip').find(chip => chip.text().includes('Claude'))
    expect(claudeChip).toBeTruthy()

    await claudeChip!.trigger('click')
    await nextTick()
    // 6 个 Skill 落 Claude；过滤后展示这 6 行，而不是 12 行全展示。
    expect(wrapper.findAll('.skill-row').length).toBe(6)
    expect(wrapper.find('.ext-pagination span').text()).toContain('6 / 12 条 · 第 1 / 1 页')

    await claudeChip!.trigger('click')
    await nextTick()
    expect(wrapper.findAll('.skill-row').length).toBe(8)
    await wrapper.unmount()
  })

  it('resets to the first page when the search term changes', async () => {
    const wrapper = mountRoute()
    await nextTick()
    await wrapper.find('[aria-label="下一页"]').trigger('click')
    await nextTick()
    expect(wrapper.find('.ext-pagination span').text()).toContain('第 2 / 2 页')

    await wrapper.find('input[aria-label="搜索 Skills"]').setValue('skill-01')
    await nextTick()
    expect(wrapper.findAll('.skill-row').length).toBe(1)
    expect(wrapper.find('.ext-pagination span').text()).toContain('第 1 / 1 页')
    await wrapper.unmount()
  })
})

// 用户口径（2026-09-24）：源头 Skill 只读，客户端目录里只是链接；
// 行内图标 = 只连/断这一个客户端，绝不能把整条 Skill 卸载掉。
describe('extension row client icons act per client', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(null)
  })

  function rowFor(wrapper: ReturnType<typeof mountRoute>, name: string) {
    return wrapper.findAll('.skill-row').find(row => row.text().includes(name))!
  }

  it('links only the clicked client', async () => {
    const wrapper = mountRoute()
    await nextTick()
    // skill-01 落在 Claude；Gemini 未同步 → 点它应只连 Gemini。
    await rowFor(wrapper, 'skill-01').find('[aria-label="将 skill-01 同步到 Gemini"]').trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'execute_extension_write',
      expect.objectContaining({
        command: expect.objectContaining({
          kind: 'link_skill',
          name: 'skill-01',
          client: 'gemini',
        }),
      }),
    ))
    await wrapper.unmount()
  })

  it('disconnects only the clicked client instead of uninstalling the skill', async () => {
    const wrapper = mountRoute()
    await nextTick()
    // skill-01 已落在 Claude：再点一次 = 断开 Claude 这一个客户端的链接。
    await rowFor(wrapper, 'skill-01').find('[aria-label="将 skill-01 同步到 Claude"]').trigger('click')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'execute_extension_write',
      expect.objectContaining({
        command: expect.objectContaining({
          kind: 'unlink_skill',
          name: 'skill-01',
          client: 'claude',
          confirm: true,
        }),
      }),
    ))
    expect(
      invoke.mock.calls.filter(([, payload]) =>
        (payload?.command as { kind?: string } | undefined)?.kind === 'uninstall_skill',
      ),
    ).toHaveLength(0)
    await wrapper.unmount()
  })
})
