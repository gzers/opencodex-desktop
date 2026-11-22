import {describe,it,expect} from 'vitest'
import {compactShellFor} from '@/components/layout/useShellLayout'
describe('effective shell width at saved scale',()=>{
 it('keeps reading space at native 900/1180 windows when enlarged and preserves normal navigation when space is available',()=>{
  expect(compactShellFor(1180,1)).toBe(false);expect(compactShellFor(900,1)).toBe(false)
  expect(compactShellFor(1180,2)).toBe(true);expect(compactShellFor(900,1.5)).toBe(true)
  expect(compactShellFor(900,.5)).toBe(false);expect(compactShellFor(1520,2)).toBe(true)
  expect(compactShellFor(1522,2)).toBe(false)
 })
})
