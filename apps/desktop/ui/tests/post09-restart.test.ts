import {afterEach,beforeAll,beforeEach,expect,it,vi} from 'vitest'
import {createPinia,setActivePinia} from 'pinia'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import SaveResultNotice from '@/features/models/SaveResultNotice.vue'
import {modelApi} from '@/features/models/api'
import {installNativeDialogStub} from './helpers/nativeDialog'
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}))
let wrapper:VueWrapper
beforeAll(installNativeDialogStub)
beforeEach(()=>{setActivePinia(createPinia());vi.useFakeTimers({toFake:['setTimeout','clearTimeout','performance','Date']});wrapper=mount(SaveResultNotice,{props:{open:true},attachTo:document.body})})
afterEach(()=>{wrapper.unmount();vi.restoreAllMocks();vi.useRealTimers();document.body.innerHTML=''})
const button=(text:string)=>[...document.querySelectorAll<HTMLButtonElement>('button')].find(node=>node.textContent===text)!
async function attempt(){button('立即重启 Codex').click();await flushPromises();button('确认重启').click();await flushPromises()}

it('keeps saved success, error details and an explicit retry after a restart precondition failure',async()=>{
 const refresh=vi.spyOn(modelApi,'refreshCache').mockRejectedValueOnce({code:'runtime_record_missing',message:'配置已写入，但重启尚未完成：未找到运行记录 runtime-port.json；请先启动服务'})
 await attempt()
 expect(document.querySelector('[role=alert]')?.textContent).toContain('runtime-port.json')
 expect(button('重试重启').disabled).toBe(false)
 expect(wrapper.emitted('close')).toBeUndefined()
 await vi.advanceTimersByTimeAsync(30000)
 expect(wrapper.emitted('close')).toBeUndefined()
 expect(wrapper.text()).toContain('配置已保存；重启尚未完成')
 refresh.mockResolvedValueOnce({success:true,restarted:true,summary:'Codex 已重启'})
 button('重试重启').click();await flushPromises()
 expect(refresh).toHaveBeenCalledTimes(2)
 expect(wrapper.emitted('close')).toHaveLength(1)
})

it('does not dismiss the saved result when the owner reports an incomplete restart',async()=>{
 vi.spyOn(modelApi,'refreshCache').mockResolvedValueOnce({success:false,restarted:false,summary:'未发现正在运行的 Codex 桌面应用，未执行重启'})
 await attempt()
 expect(document.querySelector('[role=alert]')?.textContent).toContain('未执行重启')
 expect(wrapper.emitted('close')).toBeUndefined()
 document.querySelector<HTMLButtonElement>('dialog[open] .modal-close')!.click();await flushPromises()
 expect(document.querySelector('dialog[open]')).toBeNull()
 expect(wrapper.text()).toContain('已写入')
 expect(wrapper.emitted('close')).toBeUndefined()
 button('稍后').click();await flushPromises()
 expect(wrapper.emitted('close')).toHaveLength(1)
})
