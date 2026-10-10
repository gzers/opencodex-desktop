import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick, ref } from 'vue'
import { expect, it } from 'vitest'
import RuntimeMotionMark from '@/features/runtime/components/RuntimeMotionMark.vue'
import { useEffectsStore } from '@/app/appearance/effects'
import { useGlowRenderStore } from '@/app/appearance/glowRender'

it('keeps the glow at the main boundary while the hero stays in its stage, and removes it on route disposal', async () => {
  setActivePinia(createPinia())
  useEffectsStore().setSetting('mid')
  useGlowRenderStore().setSetting('css')
  const shell = document.createElement('div')
  shell.innerHTML = '<aside class="side"></aside><main class="main"><section class="route-section ov"><div class="motion-stage"></div></section></main>'
  document.body.append(shell)
  const main = shell.querySelector('.main')!
  const stage = shell.querySelector('.motion-stage')!
  const wrapper = mount(RuntimeMotionMark, { attachTo: stage, props: { state: 'running', active: false } })
  try {
    await nextTick()
    const backdrop = main.querySelector('.motion-backdrop')!
    const ambient = backdrop.querySelector('.motion-ambient')!
    expect(backdrop.parentElement).toBe(shell.querySelector('.route-section'))
    expect(backdrop.getAttribute('aria-hidden')).toBe('true')
    expect(stage.querySelector('.motion-backdrop')).toBeNull()
    expect(shell.querySelector('.side .motion-backdrop')).toBeNull()
    expect(stage.querySelectorAll('.motion-hero svg')).toHaveLength(1)

    const colors = ambient.getAttribute('style')
    await wrapper.setProps({ state: 'failed' })
    expect(main.querySelector('.motion-backdrop')).toBe(backdrop)
    expect(ambient.getAttribute('style')).not.toBe(colors)
    expect(stage.querySelectorAll('.motion-hero svg')).toHaveLength(1)
  } finally {
    wrapper.unmount()
    expect(main.querySelector('.motion-backdrop')).toBeNull()
    shell.remove()
    document.documentElement.removeAttribute('data-effects')
    document.documentElement.removeAttribute('data-glow-render')
  }
})

it('can replace the overview route and return without leaving Teleport anchors or duplicate light layers', async () => {
  setActivePinia(createPinia())
  useEffectsStore().setSetting('mid')
  useGlowRenderStore().setSetting('css')
  const overview = ref(true)
  const Host = defineComponent({
    components: { RuntimeMotionMark },
    setup: () => ({ overview }),
    template: '<main class="main"><section v-if="overview" class="route-section ov"><div class="motion-stage"><RuntimeMotionMark state="running" :active="false" /></div></section><section v-else class="other-route">其他页面</section></main>',
  })
  const wrapper = mount(Host)
  try {
    await nextTick()
    expect(wrapper.findAll('.motion-backdrop')).toHaveLength(1)
    overview.value = false
    await nextTick()
    expect(wrapper.find('.other-route').exists()).toBe(true)
    expect(wrapper.findAll('.motion-backdrop')).toHaveLength(0)
    overview.value = true
    await nextTick()
    await nextTick()
    expect(wrapper.findAll('.motion-backdrop')).toHaveLength(1)
    expect(wrapper.findAll('.motion-hero svg')).toHaveLength(1)
  } finally {
    wrapper.unmount()
    document.documentElement.removeAttribute('data-effects')
    document.documentElement.removeAttribute('data-glow-render')
  }
})
