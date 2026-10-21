import {beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {defineComponent} from 'vue'
import ClientLoadStatus from '@/features/models/ClientLoadStatus.vue'
import {modelApi,type ModelClientState} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{clientStatus:vi.fn(),checkClient:vi.fn()}}))
const dialog=defineComponent({props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'})
const state=(status:ModelClientState['status']='restart_required'):ModelClientState=>({planId:'committed-target',status,checkedAt:null,checkId:null,confirmedAt:null,runtimeMatches:false,automaticClientVerification:false})
function render(){return mount(ClientLoadStatus,{global:{stubs:{UiDialog:dialog}}})}
function button(wrapper:ReturnType<typeof render>,text:string){return wrapper.findAll('button').find(b=>b.text()===text)!}
beforeEach(()=>{vi.clearAllMocks();vi.mocked(modelApi.clientStatus).mockResolvedValue(state())})
describe('Codex client-loaded evidence boundary',()=>{
 it('requires runtime check and separate explicit user confirmation and does not claim automatic loading',async()=>{
  vi.mocked(modelApi.checkClient).mockResolvedValueOnce({...state('pending_verification'),checkId:'fresh-check',runtimeMatches:true}).mockResolvedValueOnce({...state('user_confirmed'),confirmedAt:'2026-10-19T00:00:00Z',runtimeMatches:true})
  const wrapper=render();await flushPromises();expect(modelApi.checkClient).not.toHaveBeenCalled();await button(wrapper,'客户端加载核对').trigger('click')
  await button(wrapper,'我已重启，重新检查').trigger('click');await flushPromises();expect(modelApi.checkClient).toHaveBeenCalledWith('committed-target',undefined)
  expect(button(wrapper,'记录用户确认').attributes('disabled')).toBeDefined();expect(wrapper.text()).toContain('待客户端验证')
  await wrapper.get('input[type=checkbox]').setValue(true);await button(wrapper,'记录用户确认').trigger('click');await flushPromises()
  expect(modelApi.checkClient).toHaveBeenLastCalledWith('committed-target','fresh-check');expect(wrapper.text()).toContain('用户已确认客户端结果');expect(wrapper.text()).not.toContain('客户端已生效');wrapper.unmount()
 })
 it('cannot confirm when recheck finds drift and leaves the saved target visible',async()=>{
  vi.mocked(modelApi.checkClient).mockResolvedValue({...state('runtime_changed'),checkedAt:'2026-10-19T00:00:00Z'})
  const wrapper=render();await flushPromises();await button(wrapper,'客户端加载核对').trigger('click');await button(wrapper,'我已重启，重新检查').trigger('click');await flushPromises()
  expect(wrapper.find('input[type=checkbox]').exists()).toBe(false);expect(wrapper.text()).toContain('运行配置已变化');expect(wrapper.findAll('button').some(b=>b.text()==='记录用户确认')).toBe(false);wrapper.unmount()
 })
})
