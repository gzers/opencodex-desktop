import {beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {defineComponent} from 'vue'
import {createPinia} from 'pinia'
import AgentExecutionField from '@/features/models/AgentExecutionField.vue'
import {modelApi,emptyRules,type AgentSettings} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{agentSettings:vi.fn(),executionDirectory:vi.fn(),saveAgentSettings:vi.fn()}}))
const dialog=defineComponent({props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'})
const picker=defineComponent({props:['open','current','rows'],emits:['confirm','close'],template:'<div v-if="open&&rows.length"><span>{{rows[0].modelId}}</span><button @click="$emit(\'confirm\',[{channelId:rows[0].channelId,modelId:rows[0].modelId,reasoningEffort:\'none\',fast:false}])">确认临时选择</button><button @click="$emit(\'close\')">取消临时选择</button></div>'})
const saved:AgentSettings={providerId:'original',modelId:'old',reasoningEffort:null,fast:null,channelId:'old-channel',connectionDigest:'old-digest'}
/** 组件用模型配置 store 做首屏即时呈现，测试需提供 Pinia 实例。 */
function render(){return mount(AgentExecutionField,{global:{plugins:[createPinia()],stubs:{UiDialog:dialog,ModelPicker:picker}}})}
function button(wrapper:ReturnType<typeof render>,name:string){return wrapper.findAll('button').find(b=>b.text()===name)!}
beforeEach(()=>{vi.clearAllMocks();vi.mocked(modelApi.agentSettings).mockResolvedValue({...saved});vi.mocked(modelApi.executionDirectory).mockResolvedValue({channels:[{id:'stable',providerId:'runtime-provider',label:'渠道'}],rows:[{channelId:'stable',providerId:'runtime-provider',modelId:'exact/native-id',label:'名称',enabled:true,selected:false,discovered:true,templateRef:null,rules:emptyRules(),source:{},connectionDigest:'selected-binding'}]});vi.mocked(modelApi.saveAgentSettings).mockImplementation(async value=>value)})
describe('saved common execution preference',()=>{
 it('keeps model picker selection temporary until the shared settings save and returns a stable connection binding',async()=>{
  const wrapper=render();await flushPromises();expect(modelApi.executionDirectory).not.toHaveBeenCalled();await button(wrapper,'Agent 配置').trigger('click');await button(wrapper,'original / old').trigger('click');await flushPromises();await button(wrapper,'确认临时选择').trigger('click');await flushPromises()
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled();expect(wrapper.emitted('update:modelValue')?.at(-1)?.[0]).toEqual(saved)
  await button(wrapper,'保存共用设置').trigger('click');await flushPromises()
  expect(modelApi.saveAgentSettings).toHaveBeenCalledExactlyOnceWith({channelId:'stable',providerId:'runtime-provider',modelId:'exact/native-id',reasoningEffort:'none',fast:false,connectionDigest:'selected-binding'});wrapper.unmount()
 })
 it('cancelling the settings after a temporary choice retains the original committed execution preference',async()=>{
  const wrapper=render();await flushPromises();await button(wrapper,'Agent 配置').trigger('click');await button(wrapper,'original / old').trigger('click');await flushPromises();await button(wrapper,'确认临时选择').trigger('click');await button(wrapper,'取消').trigger('click')
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled();await button(wrapper,'Agent 配置').trigger('click');expect(button(wrapper,'original / old').exists()).toBe(true);wrapper.unmount()
 })
})
