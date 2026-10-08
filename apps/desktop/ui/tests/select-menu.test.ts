import { mount } from '@vue/test-utils'
import { defineComponent, nextTick, ref } from 'vue'
import { expect, it, vi } from 'vitest'
import UiSelectMenu from '@/components/ui/UiSelectMenu.vue'

it('escapes the filtered card, positions in scaled coordinates and supports keyboard/outside dismissal', async () => {
  const shell = document.createElement('div')
  shell.className = 'app-window'
  shell.style.zoom = '1.5'
  document.body.append(shell)
  // jsdom does not calculate CSS zoom; native positioning has its own pixel/DPI gate.
  const computedStyle = window.getComputedStyle.bind(window)
  vi.spyOn(window, 'getComputedStyle').mockImplementation(node =>
    node === shell ? { zoom: '1.5' } as CSSStyleDeclaration : computedStyle(node))
  const Host = defineComponent({
    components: { UiSelectMenu },
    setup() { return { open: ref(false) } },
    template: `<div class="setting-row" style="backdrop-filter:blur(8px)"><div class="select">
      <button class="select-trigger" @click="open=!open">Choose</button>
      <UiSelectMenu :open="open" @close="open=false" role="listbox">
        <button role="option">One</button><button role="option">Two</button>
      </UiSelectMenu></div></div>`,
  })
  const wrapper = mount(Host, { attachTo: shell })
  try {
    const trigger = wrapper.find('.select-trigger')
    vi.spyOn(trigger.element, 'getBoundingClientRect').mockReturnValue({
      x: 300, y: 150, left: 300, top: 150, right: 525, bottom: 201, width: 225, height: 51,
    } as DOMRect)
    await trigger.trigger('keydown', { key: 'ArrowDown' })
    await nextTick(); await nextTick()
    const menu = shell.querySelector<HTMLElement>('.select-menu')!
    expect(menu.parentElement).toBe(shell)
    expect(wrapper.find('.setting-row .select-menu').exists()).toBe(false)
    expect(menu.style.minWidth).toBe('150px')
    expect(menu.style.top).toBe('140px')
    expect(document.activeElement?.textContent).toBe('One')
    document.activeElement!.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement?.textContent).toBe('Two')
    document.activeElement!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await nextTick()
    expect(shell.querySelector('.select-menu')).toBeNull()
    expect(document.activeElement).toBe(trigger.element)
    await trigger.trigger('click'); await nextTick()
    document.body.dispatchEvent(new Event('pointerdown', { bubbles: true })); await nextTick()
    expect(shell.querySelector('.select-menu')).toBeNull()
  } finally { wrapper.unmount(); shell.remove(); vi.restoreAllMocks() }
})
