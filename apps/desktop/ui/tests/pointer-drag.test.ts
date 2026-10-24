import {afterEach,describe,expect,it} from 'vitest'
import {mount} from '@vue/test-utils'
import {defineComponent,ref} from 'vue'
import {usePointerDrag} from '@/features/models/pointerDrag'

/**
 * 指针拖拽的单元覆盖。
 *
 * 背景：WKWebView 里 HTML5 拖放的 drop 常不触发（排序静默失败）并会框选文字，
 * 因此行排序改成指针事件。这里锁住四条关键语义：阈值、起手排除、Esc 收尾、禁选类。
 */
function harness(options:{enabled?:boolean}={}) {
  const order=['a','b','c']
  const events:string[]=[]
  const Comp=defineComponent({
    setup(){
      const enabled=ref(options.enabled??true)
      const drag=usePointerDrag({
        rowSelector:'tr[data-item]',itemAttr:'data-item',
        enabled:()=>enabled.value,
        onReorder:(id,target)=>{events.push(id+'>'+target)},
        onActive:id=>{events.push('active:'+id)},
        onOver:id=>{events.push('over:'+id)},
      })
      return {drag,enabled,order}
    },
    template:'<table><tbody><tr v-for="id in order" :key="id" :data-item="id" @pointerdown="drag.start($event,id)"><td>{{ id }}</td></tr></tbody></table>',
  })
  const wrapper=mount(Comp,{attachTo:document.body})
  return {wrapper,events}
}
function pointer(type:string,x:number,y:number){
  const event=new Event(type,{bubbles:true,cancelable:true}) as PointerEvent
  Object.defineProperties(event,{clientX:{value:x},clientY:{value:y},button:{value:0},pointerId:{value:1}})
  return event
}
const move=(x:number,y:number)=>window.dispatchEvent(pointer('pointermove',x,y))
const up=()=>window.dispatchEvent(pointer('pointerup',0,0))
afterEach(()=>{document.body.innerHTML='';document.documentElement.classList.remove('ocx-pointer-drag')})

describe('pointer drag ordering',()=>{
  it('starts only after passing the threshold and drops onto the hovered row',()=>{
    const {wrapper,events}=harness()
    const row=wrapper.get('tr[data-item="a"]').element as HTMLElement
    row.dispatchEvent(pointer('pointerdown',100,100))
    move(102,101); expect(events).toEqual([]); expect(document.documentElement.classList.contains('ocx-pointer-drag')).toBe(false)
    move(100,160)
    expect(events[0]).toBe('active:a')
    expect(document.documentElement.classList.contains('ocx-pointer-drag')).toBe(true)
    const target=wrapper.get('tr[data-item="c"]').element as HTMLElement
    const box=target.getBoundingClientRect()
    Object.defineProperty(document,'elementFromPoint',{configurable:true,value:()=>target.querySelector('td')})
    move(box.left+2,box.top+2)
    expect(events).toContain('over:c')
    up()
    expect(events).toContain('a>c')
    expect(document.documentElement.classList.contains('ocx-pointer-drag')).toBe(false)
    wrapper.unmount()
  })

  it('ignores presses that begin on a control so normal interaction still works',()=>{
    const {wrapper,events}=harness()
    const cell=wrapper.get('tr[data-item="a"] td').element as HTMLElement
    const button=document.createElement('button'); cell.appendChild(button)
    button.dispatchEvent(pointer('pointerdown',10,10))
    move(10,80)
    expect(events).toEqual([])
    expect(document.documentElement.classList.contains('ocx-pointer-drag')).toBe(false)
    wrapper.unmount()
  })

  it('cancels cleanly on Escape without reordering',()=>{
    const {wrapper,events}=harness()
    const row=wrapper.get('tr[data-item="a"]').element as HTMLElement
    row.dispatchEvent(pointer('pointerdown',0,0))
    move(0,60)
    expect(events[0]).toBe('active:a')
    window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))
    expect(document.documentElement.classList.contains('ocx-pointer-drag')).toBe(false)
    expect(events).toContain('active:null')
    up()
    expect(events.some(e=>e.includes('>'))).toBe(false)
    wrapper.unmount()
  })

  it('does not arm when reordering is not allowed',()=>{
    const {wrapper,events}=harness({enabled:false})
    const row=wrapper.get('tr[data-item="a"]').element as HTMLElement
    row.dispatchEvent(pointer('pointerdown',0,0))
    move(0,90)
    expect(events).toEqual([])
    expect(document.documentElement.classList.contains('ocx-pointer-drag')).toBe(false)
    wrapper.unmount()
  })
})

