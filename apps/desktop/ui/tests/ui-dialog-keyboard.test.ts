import {afterEach,beforeEach,describe,expect,it} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {defineComponent,h,ref} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
const wrappers:VueWrapper[]=[]
beforeEach(()=>{
 Object.assign(HTMLDialogElement.prototype,{showModal(this:HTMLDialogElement){this.setAttribute('open','')},close(this:HTMLDialogElement){this.removeAttribute('open')}})
})
afterEach(()=>{wrappers.splice(0).forEach(w=>w.unmount());document.body.innerHTML=''})
async function render(busy=false){const w=mount(UiDialog,{props:{open:true,title:'输入窗口',busy},slots:{default:'<input type="search" value="typed query" />'},attachTo:document.body});wrappers.push(w);await flushPromises();return w}
function escape(target:Element){const e=new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true});target.dispatchEvent(e);return e}
describe('native dialog keyboard ownership',()=>{
 it('closes from a populated input before native search cancellation can consume Escape',async()=>{const w=await render();const e=escape(document.querySelector('dialog input')!);expect(w.emitted('close')).toHaveLength(1);expect(e.defaultPrevented).toBe(true)})
 it('keeps busy dialogs open and prevents the browser default close',async()=>{const w=await render(true);const e=escape(document.querySelector('dialog input')!);expect(w.emitted('close')).toBeUndefined();expect(e.defaultPrevented).toBe(true)})
 it('respects a child that already consumed Escape',async()=>{const w=await render();const input=document.querySelector('dialog input')!;input.addEventListener('keydown',e=>e.preventDefault(),{once:true});escape(input);expect(w.emitted('close')).toBeUndefined()})
 it('lets an open select menu consume the first Escape, then closes its owner',async()=>{
  const host=defineComponent({setup:()=>()=>h(UiDialog,{open:true,title:'菜单窗口'},()=>h(UiSelectField,{modelValue:'a',label:'目录选择',options:[{value:'a',label:'条目A'}]}))})
  const w=mount(host,{attachTo:document.body});wrappers.push(w);await flushPromises();(document.querySelector('button.select-trigger') as HTMLButtonElement).click();await flushPromises();
  const option=document.querySelector('[role=option]')!;escape(option);await flushPromises();expect(document.querySelector('[role=option]')).toBeNull();const dialog=w.findComponent(UiDialog);expect(dialog.emitted('close')).toBeUndefined();escape(document.querySelector('button.select-trigger')!);expect(dialog.emitted('close')).toHaveLength(1)
 })
 it('only closes the nested dialog containing the event target',async()=>{
  const host=defineComponent({setup:()=>()=>h(UiDialog,{open:true,title:'外层'},()=>h(UiDialog,{open:true,title:'内层'},()=>h('input',{id:'nested-dialog-input'})))})
  const w=mount(host,{attachTo:document.body});wrappers.push(w);await flushPromises();escape(document.querySelector('#nested-dialog-input')!);const dialogs=w.findAllComponents(UiDialog);expect(dialogs[0]!.emitted('close')).toBeUndefined();expect(dialogs[1]!.emitted('close')).toHaveLength(1)
 })
 it('preserves menu priority when its content mounts after the owning dialog',async()=>{
  const show=ref(false),host=defineComponent({setup:()=>()=>h(UiDialog,{open:true,title:'异步内容'},()=>show.value?h(UiSelectField,{modelValue:'a',label:'后到目录',options:[{value:'a',label:'条目A'}]}):h('p','读取中'))})
  const w=mount(host,{attachTo:document.body});wrappers.push(w);await flushPromises();show.value=true;await flushPromises();const trigger=document.querySelector('button.select-trigger') as HTMLButtonElement;trigger.click();await flushPromises();escape(trigger);await flushPromises();expect(document.querySelector('[role=option]')).toBeNull();expect(w.findComponent(UiDialog).emitted('close')).toBeUndefined();escape(trigger);expect(w.findComponent(UiDialog).emitted('close')).toHaveLength(1)
 })
 it('closes an open child menu first when native AX activation kept focus on the dialog',async()=>{
  const host=defineComponent({setup:()=>()=>h(UiDialog,{open:true,title:'原生菜单'},()=>h(UiSelectField,{modelValue:'a',label:'目录',options:[{value:'a',label:'条目A'}]}))})
  const w=mount(host,{attachTo:document.body});wrappers.push(w);await flushPromises();(document.querySelector('button.select-trigger') as HTMLButtonElement).click();await flushPromises();escape(document.querySelector('dialog')!);await flushPromises();expect(document.querySelector('[role=option]')).toBeNull();expect(w.findComponent(UiDialog).emitted('close')).toBeUndefined();escape(document.querySelector('dialog')!);expect(w.findComponent(UiDialog).emitted('close')).toHaveLength(1)
 })
})
