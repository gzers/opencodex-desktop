import { describe,it,expect,vi,beforeEach } from 'vitest'
import { receiveOfflineDrop } from '@/features/runtime/api'
const invoke=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke}))
beforeEach(()=>invoke.mockReset())
function file(bytes:number):File {
 const data=new Uint8Array(bytes).map((_,n)=>n%251)
 return {name:'opencodex-2.91.0.tgz',size:bytes,slice:(a:number,b:number)=>({arrayBuffer:async()=>data.slice(a,b).buffer})} as unknown as File
}
describe('bounded local file reception',()=>{
 it('streams sequential binary chunks and returns only the inspected temporary path',async()=>{
  invoke.mockImplementation(async(_command,args)=>{
   if(args?.action==='begin')return {id:'owned'}
   if(args?.action==='finish')return {path:'/owned/opencodex-2.91.0.tgz',preview:{ok:true}}
   return {}
  })
  const lease=vi.fn(),result=await receiveOfflineDrop(file(2*1024*1024+1),lease,()=>false)
  expect(lease).toHaveBeenCalledWith('owned');expect(result.path).toBe('/owned/opencodex-2.91.0.tgz')
  const chunks=invoke.mock.calls.filter(c=>c[1] instanceof Uint8Array)
  expect(chunks.map(c=>c[1].length)).toEqual([1024*1024,1024*1024,1])
  expect(chunks.map(c=>c[2].headers['x-ocx-drop-offset'])).toEqual(['0','1048576','2097152'])
  expect(chunks.every(c=>c[2].headers['x-ocx-drop-id']==='owned')).toBe(true)
  expect(invoke.mock.calls.some(c=>c[1]?.action==='clear')).toBe(false)
 })
 it('releases only its lease on cancellation and never publishes a partial path',async()=>{
  invoke.mockResolvedValue({id:'cancelled-copy'})
  await expect(receiveOfflineDrop(file(2),()=>{},()=>true)).rejects.toThrow('取消')
  expect(invoke).toHaveBeenLastCalledWith('stage_offline_package',{action:'clear',id:'cancelled-copy'})
  expect(invoke.mock.calls.some(c=>c[1]?.action==='finish')).toBe(false)
 })
 it('cleans a transport failure and rejects an oversized file before opening a receiver',async()=>{
  invoke.mockResolvedValueOnce({id:'failed-copy'}).mockRejectedValueOnce(new Error('disk full')).mockResolvedValue({})
  await expect(receiveOfflineDrop(file(2),()=>{},()=>false)).rejects.toThrow('disk full')
  expect(invoke).toHaveBeenLastCalledWith('stage_offline_package',{action:'clear',id:'failed-copy'})
  invoke.mockClear()
  await expect(receiveOfflineDrop({name:'opencodex-2.91.0.tgz',size:512*1024*1024+1} as File,()=>{},()=>false)).rejects.toThrow('体积')
  expect(invoke).not.toHaveBeenCalled()
 })
})
