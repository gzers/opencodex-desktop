import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {createPinia,setActivePinia} from 'pinia'
import {mount,flushPromises,type VueWrapper} from '@vue/test-utils'
import LogsRoute from '@/routes/LogsRoute.vue'
import {useAppStore} from '@/stores/app'
import {useNotificationsStore} from '@/features/notifications/store'
const invoke=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke}))
let wrapper:VueWrapper|undefined
beforeEach(()=>{invoke.mockReset();window.location.hash='#logs?tab=notifications';setActivePinia(createPinia());Object.defineProperty(window,'matchMedia',{configurable:true,value:()=>({matches:false,addEventListener(){},removeEventListener(){}})})})
afterEach(()=>wrapper?.unmount())
const cases=[{name:'all',button:'清理已解决',resolved:false,command:'clear_notifications',success:'全部通知已清理。'},{name:'resolved',button:'清理已解决',resolved:true,command:'clear_resolved_notifications',success:'已清理已解决通知。'},{name:'read',button:'清理已读',resolved:false,command:'clear_read_notifications',success:'已读通知已清理。'}]
describe('notification history cleanup receipts',()=>{
 for(const entry of cases)for(const outcome of ['success','failure','pending'] as const)it(entry.name+' '+outcome+' follows the actual backend result',async()=>{
  let finish!:(value:unknown)=>void
  invoke.mockImplementation(async(command:string)=>{if(command===entry.command){if(outcome==='pending')return new Promise(resolve=>{finish=resolve});if(outcome==='failure')throw Error('isolated cleanup failure');return{items:[],aggregate:{total:0,unread:0,unresolved:0,dangerUnread:0}}}if(command==='read_logs')return{kind:'app',lines:[],redactedLineCount:0,truncated:false,fileMissing:false,fallbackFile:null};return null})
  const notifications=useNotificationsStore();notifications.list=[{id:'owned-fixture',kind:'info',category:'system',title:'Owned fixture',detail:'History cleanup',time:'2026-11-03T00:00:00Z',read:true,resolved:entry.resolved}]
  wrapper=mount(LogsRoute,{global:{stubs:{AppTopbar:true}}});await flushPromises();const app=useAppStore(),before=JSON.stringify(notifications.list),toast=vi.spyOn(app,'showToast');await wrapper.findAll('button').find(b=>b.text()===entry.button)!.trigger('click');expect(app.modal?.onConfirm).toBeTypeOf('function');const result=app.modal!.onConfirm!();await flushPromises();expect(invoke).toHaveBeenCalledWith(entry.command)
  if(outcome==='pending'){expect(toast).not.toHaveBeenCalled();expect(JSON.stringify(notifications.list)).toBe(before);finish({items:[],aggregate:{total:0,unread:0,unresolved:0,dangerUnread:0}});await result;await flushPromises();expect(toast).toHaveBeenCalledWith(entry.success)}
  else if(outcome==='failure'){expect(JSON.stringify(notifications.list)).toBe(before);expect(toast).toHaveBeenCalledWith(expect.stringContaining('失败'));expect(toast).not.toHaveBeenCalledWith(entry.success)}
  else{expect(notifications.list).toEqual([]);expect(toast).toHaveBeenCalledWith(entry.success)}
 })
})
