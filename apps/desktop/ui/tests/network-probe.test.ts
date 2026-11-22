import {mount,flushPromises} from '@vue/test-utils'
import {expect,it,vi,beforeEach} from 'vitest'
import NetworkProbe from '@/features/preferences/NetworkProbe.vue'
const invoke=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke}))
const response=()=>({ok:false,detail:'部分连接未通过',results:[{target:'github',state:'rate_limited',httpStatus:429,retryAfterSeconds:12,checkedAt:'2026-11-11T10:00:00Z'},{target:'npm',state:'ready',httpStatus:200,retryAfterSeconds:null,checkedAt:'2026-11-11T10:00:00Z'}]})
beforeEach(()=>{invoke.mockReset()})
it('explicit check reads both destinations without changing preferences or versions',async()=>{
 invoke.mockResolvedValue(response());const w=mount(NetworkProbe,{props:{policyKey:'saved-policy'}});expect(invoke).not.toHaveBeenCalled();await w.get('button').trigger('click');await flushPromises();expect(w.text()).toContain('GitHub');expect(w.text()).toContain('npm');expect(w.text()).toContain('限流');expect(w.text()).toContain('HTTP 429');expect(w.text()).toContain('12 秒');expect(w.text()).toContain('来源核验通过');expect(w.findAll('time')).toHaveLength(2);expect(invoke.mock.calls).toEqual([['check_network_proxy']]);w.unmount()
})
it('policy changes invalidate late results',async()=>{
 let finish!:(v:unknown)=>void;invoke.mockImplementation(()=>new Promise(resolve=>{finish=resolve}));const w=mount(NetworkProbe,{props:{policyKey:'old'}});await w.get('button').trigger('click');await flushPromises();await w.setProps({policyKey:'new'});finish(response());await flushPromises();expect(w.find('[aria-label="网络连接检查结果"]').exists()).toBe(false);expect(w.get('button').attributes('disabled')).toBeUndefined();w.unmount()
})
it('partial receipts remain unconfirmed',async()=>{
 invoke.mockResolvedValue({...response(),results:[response().results[0]]});const w=mount(NetworkProbe,{props:{policyKey:'saved'}});await w.get('button').trigger('click');await flushPromises();expect(w.get('[role="alert"]').text()).toContain('尚未确认');expect(w.find('time').exists()).toBe(false);w.unmount()
})
