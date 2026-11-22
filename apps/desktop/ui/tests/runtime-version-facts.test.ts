import {createPinia,setActivePinia} from 'pinia'
import {mount} from '@vue/test-utils'
import {expect,it,beforeEach,vi} from 'vitest'
import RuntimeVersionFacts from '@/features/updates/RuntimeVersionFacts.vue'
import {useUpdatesStore} from '@/features/updates/store'
const invoke=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke}))
const facts=()=>({disk:{diskVersion:'2.99.0',selectedVersion:'2.98.0',recordedVersion:'2.97.0',recordedAt:'2026-11-01',integrity:'changed',checkedAt:'2026-11-11T00:00:00Z',confirmationDigest:'disk-review'},running:{version:'2.98.0',pid:42,port:55581},runningState:'attested'})
beforeEach(()=>{setActivePinia(createPinia());invoke.mockReset()})
it('renders independent disk running and recorded versions and never enables update for changed source',async()=>{
 invoke.mockResolvedValue(facts());const s=useUpdatesStore();s.officialRemote={tag:'latest',version:'3.0.0',integrity:null};await s.loadVersions();const w=mount(RuntimeVersionFacts);expect(w.text()).toContain('2.99.0');expect(w.text()).toContain('2.98.0');expect(w.text()).toContain('2.97.0');expect(w.text()).toContain('当前进程尚未采用');expect(w.text()).toContain('来源变化');expect(s.officialUpdateAvailable()).toBeNull();expect(invoke.mock.calls).toEqual([['runtime_version_facts']]);w.unmount()
})
it('failed read retains old facts only with an explicit stale warning and refuses comparison',async()=>{
 const s=useUpdatesStore();invoke.mockResolvedValueOnce(facts()).mockRejectedValueOnce(Error('offline'));await s.loadVersions();await s.loadVersions();const w=mount(RuntimeVersionFacts);expect(w.text()).toContain('上次观察');expect(s.officialUpdateAvailable()).toBeNull();w.unmount()
})
it('late old observation cannot restore a source fact after a newer failure',async()=>{
 let finish!:(v:unknown)=>void;invoke.mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve})).mockRejectedValueOnce(Error('unconfirmed'));const s=useUpdatesStore();const first=s.loadVersions();await s.loadVersions();finish(facts());await first;expect(s.versionFacts).toBeNull();expect(s.versionFactsError).not.toBe('')
})
it('no runtime record is displayed as unverified rather than stopped',async()=>{
 invoke.mockResolvedValue({...facts(),running:null,runningState:'no_record'});const s=useUpdatesStore();await s.loadVersions();const w=mount(RuntimeVersionFacts);expect(w.text()).toContain('运行实例版本未核实');expect(w.text()).not.toContain('已停止');w.unmount()
})
