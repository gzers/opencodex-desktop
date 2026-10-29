import {afterEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {defineComponent,ref} from 'vue'
import DetailWorkspace from '@/components/layout/DetailWorkspace.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
const frames:FrameRequestCallback[]=[]
function drain(){while(frames.length)frames.shift()!(0)}
afterEach(()=>{vi.unstubAllGlobals();document.body.innerHTML='';frames.length=0})
function fixture(zoom=1,reflowShift=0){
 vi.stubGlobal('requestAnimationFrame',(callback:FrameRequestCallback)=>{frames.push(callback);return frames.length})
 vi.stubGlobal('cancelAnimationFrame',vi.fn())
 const main=document.createElement('main');main.className='main';document.body.append(main)
 Object.defineProperties(main,{offsetHeight:{value:600},clientHeight:{value:600},scrollHeight:{value:2400}})
 main.getBoundingClientRect=()=>({top:100,height:600*zoom,bottom:100+600*zoom} as DOMRect)
 const Sample=defineComponent({components:{DetailWorkspace},setup(){return{collapsed:ref(false),active:ref('first')}},template:'<DetailWorkspace v-model:collapsed="collapsed" v-model:active-id="active" label="测试导航" :sections="[{id:\'first\',label:\'第一部分\'},{id:\'second\',label:\'第二部分\'}]"><template #header>标题</template><section id="first" tabindex="-1"><input value="保持人工输入" /></section><section id="second" tabindex="-1">正文</section></DetailWorkspace>'})
 const wrapper=mount(Sample,{attachTo:main});const root=wrapper.get('.detail-workspace').element as HTMLElement
 root.style.setProperty('--page-tabs-height','40px');root.getClientRects=()=>[{}] as unknown as DOMRectList
 const header=wrapper.get('header').element as HTMLElement;Object.defineProperty(header,'offsetHeight',{value:48,configurable:true})
 for(const [id,y] of [['first',200],['second',900]] as const)(wrapper.get('#'+id).element as HTMLElement).getBoundingClientRect=()=>({top:100+(y+(id==='second'&&root.classList.contains('detail-collapsed')?reflowShift:0)-main.scrollTop)*zoom} as DOMRect)
 window.dispatchEvent(new Event('resize'));drain()
 return{wrapper,main,header}
}
describe('shared continuous detail workspace',()=>{
 it('reveals a zoom-constrained workspace without repeatedly scrolling past its sticky boundary',()=>{
  const{wrapper,main}=fixture(2),nav=wrapper.get('.detail-navigation').element as HTMLElement
  Object.defineProperty(wrapper.get('.detail-collapse').element,'offsetHeight',{value:28})
  nav.style.top='50px';nav.getBoundingClientRect=()=>({top:1200-main.scrollTop*2} as DOMRect)
  window.dispatchEvent(new Event('resize'));drain()
  expect(main.scrollTop).toBe(46)
  expect(Number.parseFloat((wrapper.get('.detail-workspace').element as HTMLElement).style.getPropertyValue('--detail-navigation-height'))).toBe(84)
  main.getBoundingClientRect=()=>({top:100,bottom:200,height:1200} as DOMRect)
  nav.getBoundingClientRect=()=>({top:200} as DOMRect)
  const before=main.scrollTop;window.dispatchEvent(new Event('resize'));drain()
  expect(main.scrollTop).toBe(before);wrapper.unmount()
 })
 it('positions an anchor below both fixed layers under UI scaling and tracks manual scroll without changing fields',async()=>{
  const{wrapper,main}=fixture(2)
  await wrapper.get('a[href="#second"]').trigger('click');await flushPromises();drain()
  expect(main.scrollTop).toBe(800);expect(document.activeElement?.id).toBe('second')
  expect(wrapper.get('a[href="#second"]').attributes('aria-current')).toBe('location')
  main.scrollTop=150;main.dispatchEvent(new Event('scroll'));drain();await flushPromises()
  expect(wrapper.get('a[href="#first"]').attributes('aria-current')).toBe('location')
  expect((wrapper.get('input').element as HTMLInputElement).value).toBe('保持人工输入');wrapper.unmount()
 })
 it('preserves a position inside a section when column reflow moves that section',async()=>{
  const{wrapper,main}=fixture(1,220)
  main.scrollTop=1050;main.dispatchEvent(new Event('scroll'));drain();await flushPromises()
  const before=(wrapper.get('#second').element as HTMLElement).getBoundingClientRect().top
  await wrapper.get('button[aria-label="收起测试导航"]').trigger('click');await flushPromises();drain();await flushPromises()
  expect(main.scrollTop).toBe(1270);expect((wrapper.get('#second').element as HTMLElement).getBoundingClientRect().top).toBe(before)
  expect((wrapper.get('input').element as HTMLInputElement).value).toBe('保持人工输入');wrapper.unmount()
 })
 it('keeps the collapse control focused and the active section visible after reflow, and responds to header growth',async()=>{
  const{wrapper,main,header}=fixture()
  await wrapper.get('a[href="#second"]').trigger('click');await flushPromises();drain()
  const toggle=wrapper.get('button[aria-label="收起测试导航"]');(toggle.element as HTMLElement).focus();await toggle.trigger('click');await flushPromises();drain()
  expect(wrapper.get('button[aria-label="展开测试导航"]').element).toBe(document.activeElement)
  expect(wrapper.get('.detail-workspace').classes()).toContain('detail-collapsed');expect(main.scrollTop).toBe(800)
  Object.defineProperty(header,'offsetHeight',{value:96});window.dispatchEvent(new Event('resize'));drain()
  await wrapper.get('a[href="#second"]').trigger('click');await flushPromises();drain();expect(main.scrollTop).toBe(752)
  wrapper.unmount();main.scrollTop=0;main.dispatchEvent(new Event('scroll'));expect(frames).toHaveLength(0)
 })
})

describe('shared page tab geometry',()=>{
 it('measures scrollport insets and wrapping in layout units, then releases parent variables on removal',async()=>{
  let resize=()=>{};const disconnect=vi.fn()
  vi.stubGlobal('ResizeObserver',class {constructor(callback:()=>void){resize=callback}observe(){}disconnect=disconnect})
  const main=document.createElement('main');main.className='main';main.style.padding='20px 24px';document.body.append(main)
  const Sample=defineComponent({components:{StickyTabs},template:'<section><StickyTabs><div role="tablist">内容</div></StickyTabs></section>'})
  const wrapper=mount(Sample,{attachTo:main}),section=wrapper.element as HTMLElement,bar=wrapper.get('.sticky-page-tabs').element
  Object.defineProperty(bar,'offsetHeight',{value:48,configurable:true});resize();await new Promise<void>(resolve=>requestAnimationFrame(()=>resolve()))
  expect(section.style.getPropertyValue('--page-tabs-inset-top')).toBe('20px');expect(section.style.getPropertyValue('--page-tabs-inset-left')).toBe('24px');expect(section.style.getPropertyValue('--page-tabs-height')).toBe('48px')
  Object.defineProperty(bar,'offsetHeight',{value:80});resize();await new Promise<void>(resolve=>requestAnimationFrame(()=>resolve()));expect(section.style.getPropertyValue('--page-tabs-height')).toBe('80px')
  wrapper.unmount();expect(section.style.getPropertyValue('--page-tabs-height')).toBe('');expect(section.style.getPropertyValue('--page-tabs-inset-left')).toBe('');expect(disconnect).toHaveBeenCalled()
 })
})
