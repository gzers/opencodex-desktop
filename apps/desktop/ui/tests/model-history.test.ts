import {beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {defineComponent} from 'vue'
import ModelSaveHistory from '@/features/backup/ModelSaveHistory.vue'
import {modelApi,type ModelSnapshot,type ModelRecoveryPreview} from '@/features/models/api'
import {useModelsStore} from '@/features/models/store'
const go=vi.hoisted(()=>vi.fn())
vi.mock('@/stores/routes',()=>({useRouteStore:()=>({go})}))
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{policyHistory:vi.fn(),prepareRecovery:vi.fn(),acceptRecovery:vi.fn(),loadRecoveryBaseline:vi.fn(),load:vi.fn()}}))
const dialog=defineComponent({props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'})
const snapshot=():ModelSnapshot=>({library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'actual-file',workspaceDigest:'empty-workspace',workspace:null})
const baseline=():ModelSnapshot=>({...snapshot(),policyDigest:'legacy-policy',workspaceDigest:'legacy-workspace'})
const recovered=():ModelSnapshot=>({...baseline(),workspaceDigest:'recovered',workspace:{basePolicy:'legacy-policy',policy:{...snapshot().policy,recoveryId:'restore-plan'}}})
const preview=():ModelRecoveryPreview=>({id:'restore-plan',sourceId:'successful-save',side:'before',scope:'affected',restoreDefaults:false,changes:[{field:'channel/a/defaults',before:1,after:2}],defaultChanges:[],channelLabels:['测试渠道'],notices:[]})
function render(){return mount(ModelSaveHistory,{global:{stubs:{UiDialog:dialog,UiSelectField:true}}})}
function button(wrapper:ReturnType<typeof render>,text:string){return wrapper.findAll('button').find(b=>b.text()===text)!}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();useModelsStore().snapshot=snapshot();vi.mocked(modelApi.load).mockResolvedValue(snapshot());vi.mocked(modelApi.loadRecoveryBaseline).mockResolvedValue(baseline());vi.mocked(modelApi.policyHistory).mockResolvedValue([{id:'successful-save',createdAt:'2026-10-19T00:00:00Z',phase:'committed',channelLabels:['测试渠道'],recoverable:true}]);vi.mocked(modelApi.prepareRecovery).mockResolvedValue(preview());vi.mocked(modelApi.acceptRecovery).mockResolvedValue(recovered())})
describe('successful model save history',()=>{
 it('only reads summaries on mount and stages exactly the reviewed recovery after confirmation',async()=>{
  const wrapper=render();await flushPromises();expect(modelApi.prepareRecovery).not.toHaveBeenCalled();expect(modelApi.acceptRecovery).not.toHaveBeenCalled();expect(modelApi.loadRecoveryBaseline).not.toHaveBeenCalled()
  await button(wrapper,'生成恢复草稿').trigger('click');expect(modelApi.prepareRecovery).not.toHaveBeenCalled()
  await button(wrapper,'比较恢复差异').trigger('click');await flushPromises()
  expect(modelApi.prepareRecovery).toHaveBeenCalledExactlyOnceWith({sourceId:'successful-save',side:'before',scope:'affected',restoreDefaults:false,expectedLibrary:'library',expectedPolicy:'legacy-policy',expectedWorkspace:'legacy-workspace'})
  await button(wrapper,'放入恢复草稿').trigger('click');await flushPromises();expect(modelApi.acceptRecovery).toHaveBeenCalledExactlyOnceWith('restore-plan')
  expect(useModelsStore().snapshot).toEqual(snapshot());expect(useModelsStore().historicalRecovery?.workspaceDigest).toBe('recovered');expect(useModelsStore().channelDraft?.recoveryId).toBe('restore-plan')
  expect(wrapper.text()).toContain('运行配置尚未改变');await button(wrapper,'前往模型页保存').trigger('click');expect(go).toHaveBeenCalledWith('models');wrapper.unmount()
 })
 it('keeps an uncertain original recovery ID for retry instead of preparing another candidate',async()=>{
  vi.mocked(modelApi.acceptRecovery).mockRejectedValueOnce({message:'草稿结果需核对'}).mockResolvedValue(recovered())
  const wrapper=render();await flushPromises();await button(wrapper,'生成恢复草稿').trigger('click');await button(wrapper,'比较恢复差异').trigger('click');await flushPromises()
  await button(wrapper,'放入恢复草稿').trigger('click');await flushPromises();expect(wrapper.text()).toContain('草稿结果需核对');await button(wrapper,'放入恢复草稿').trigger('click');await flushPromises()
  expect(vi.mocked(modelApi.acceptRecovery).mock.calls).toEqual([['restore-plan'],['restore-plan']]);expect(modelApi.prepareRecovery).toHaveBeenCalledTimes(1);wrapper.unmount()
 })
 it('leaves incomplete or legacy snapshots in history without granting successful restore',async()=>{
  vi.mocked(modelApi.policyHistory).mockResolvedValue([{id:'partial',createdAt:'2026-10-19T00:00:00Z',phase:'providers_written',channelLabels:[],recoverable:false}]);const wrapper=render();await flushPromises()
  expect(button(wrapper,'生成恢复草稿').attributes('disabled')).toBeDefined();expect(modelApi.prepareRecovery).not.toHaveBeenCalled();wrapper.unmount()
 })
 it('retains Key edits and explains the conflict before comparing a historical recovery',async()=>{
  const models=useModelsStore();models.editCredential('channel','replacement-value','original-value')
  const wrapper=render();await flushPromises();await button(wrapper,'生成恢复草稿').trigger('click');await button(wrapper,'比较恢复差异').trigger('click');await flushPromises()
  expect(wrapper.text()).toContain('当前输入已保留');expect(modelApi.loadRecoveryBaseline).not.toHaveBeenCalled();expect(modelApi.acceptRecovery).not.toHaveBeenCalled();expect(models.credentialEdits.channel?.value).toBe('replacement-value');expect(models.snapshot).toEqual(snapshot());wrapper.unmount()
 })
 it('rechecks new input between comparison and acceptance without clearing it',async()=>{
  const wrapper=render();await flushPromises();await button(wrapper,'生成恢复草稿').trigger('click');await button(wrapper,'比较恢复差异').trigger('click');await flushPromises()
  const models=useModelsStore();models.clearCredential('channel');await button(wrapper,'放入恢复草稿').trigger('click');await flushPromises()
  expect(wrapper.text()).toContain('当前输入已保留');expect(modelApi.acceptRecovery).not.toHaveBeenCalled();expect(models.credentialEdits.channel?.action).toBe('clear');expect(models.historicalRecovery).toBeNull();wrapper.unmount()
 })
 it('rejects an acceptance response without matching historical provenance',async()=>{
  vi.mocked(modelApi.acceptRecovery).mockResolvedValue(baseline())
  const wrapper=render();await flushPromises();await button(wrapper,'生成恢复草稿').trigger('click');await button(wrapper,'比较恢复差异').trigger('click');await flushPromises();await button(wrapper,'放入恢复草稿').trigger('click');await flushPromises()
  expect(wrapper.text()).toContain('恢复草稿与所选记录不符');expect(useModelsStore().snapshot).toEqual(snapshot());expect(useModelsStore().historicalRecovery).toBeNull();wrapper.unmount()
 })
})
