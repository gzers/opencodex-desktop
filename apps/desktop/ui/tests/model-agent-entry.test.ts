import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ModelsRoute from '@/routes/ModelsRoute.vue'
import {useModelsV2Store} from '@/features/models/store-v2'
import {agentLibrary} from './helpers/modelAgentV2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn(async()=>()=>{})}))
let frames:Map<number,FrameRequestCallback>;let serial:number
beforeEach(()=>{setActivePinia(createPinia());frames=new Map();serial=0;vi.stubGlobal('requestAnimationFrame',(cb:FrameRequestCallback)=>{frames.set(++serial,cb);return serial});vi.stubGlobal('cancelAnimationFrame',(id:number)=>frames.delete(id));ipc.mockReset();ipc.mockResolvedValue(agentLibrary())})
afterEach(()=>{vi.unstubAllGlobals();document.body.innerHTML=''})
describe('v2 final sidebar assistant entry',()=>{
 it('opens from sidebar only, restores focus and follows editor/tab visibility without a v1 library command',async()=>{const pinia=createPinia();setActivePinia(pinia);const store=useModelsV2Store();store.view.tab='templates';const w=mount(ModelsRoute,{attachTo:document.body,global:{plugins:[pinia],stubs:{AppTopbar:true,ChannelsView:true,AgentExecutionField:true,BatchAgentDialog:true,ModelRuleFields:true,UiDialog:true,UiSelectField:true,AgentPanel:{props:['active'],emits:['close'],template:'<aside id="model-agent-panel"><button @click="$emit(\'close\')">收起助手</button></aside>'}}}});try{await flushPromises();store.start(store.snapshot!.library.templates[0]);store.view.templateEditor=true;await flushPromises();expect(w.find('.model-agent-launcher').exists()).toBe(false);const trigger=w.get<HTMLButtonElement>('[aria-label="Agent自动填充"]');trigger.element.focus();await trigger.trigger('click');expect(w.findAll('#model-agent-panel')).toHaveLength(1);expect(w.find('dialog[open]').exists()).toBe(false);await w.findAll('button').find(b=>b.text()==='收起助手')!.trigger('click');await flushPromises();expect(document.activeElement).toBe(trigger.element);await trigger.trigger('click');expect(trigger.attributes('aria-expanded')).toBe('true');store.view.tab='channels';await flushPromises();expect(w.find('#model-agent-panel').exists()).toBe(false);store.view.tab='templates';await flushPromises();expect(w.get('#model-agent-panel').isVisible()).toBe(true);expect(ipc.mock.calls.every(c=>c[0]==='model_v2_library')).toBe(true)}finally{w.unmount()}expect(frames.size).toBe(0)})
})
