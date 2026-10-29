import {describe,it,expect,vi,afterEach} from 'vitest'
import {layoutObserver} from '@/lib/layoutObserver'
afterEach(()=>vi.unstubAllGlobals())
describe('resize layout delivery',()=>{
 it('never writes layout during the observer callback and coalesces the next frame',()=>{
  let delivery!:()=>void,frame!:FrameRequestCallback
  const disconnect=vi.fn(),request=vi.fn((fn:FrameRequestCallback)=>{frame=fn;return 9})
  vi.stubGlobal('ResizeObserver',class{constructor(fn:()=>void){delivery=fn} observe(){} disconnect=disconnect})
  vi.stubGlobal('requestAnimationFrame',request);vi.stubGlobal('cancelAnimationFrame',vi.fn())
  const update=vi.fn(),observer=layoutObserver(update)!
  observer.observe(document.body);delivery();delivery()
  expect(update).not.toHaveBeenCalled();expect(request).toHaveBeenCalledTimes(1)
  frame(0);expect(update).toHaveBeenCalledTimes(1)
  delivery();expect(request).toHaveBeenCalledTimes(2)
 })
 it('cancels queued layout and rejects a late delivery when unmounted',()=>{
  let delivery!:()=>void,frame!:FrameRequestCallback
  const cancel=vi.fn(),disconnect=vi.fn()
  vi.stubGlobal('ResizeObserver',class{constructor(fn:()=>void){delivery=fn}observe(){}disconnect=disconnect})
  vi.stubGlobal('requestAnimationFrame',(fn:FrameRequestCallback)=>{frame=fn;return 4});vi.stubGlobal('cancelAnimationFrame',cancel)
  const update=vi.fn(),observer=layoutObserver(update)!
  delivery();observer.disconnect();frame(0);delivery()
  expect(cancel).toHaveBeenCalledWith(4);expect(disconnect).toHaveBeenCalledOnce();expect(update).not.toHaveBeenCalled()
 })
})
