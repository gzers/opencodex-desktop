import {describe,it,expect} from 'vitest'
import {savePlanErrorText} from '@/features/models/api'
describe('saved-operation error messages',()=>{
 it('keeps uncertain results actionable without exposing internal plan error codes',()=>{
  expect(savePlanErrorText('catalog_write')).toContain('目录刷新结果尚未确认')
  expect(savePlanErrorText('credential_drift')).toContain('后来的配置修改')
  expect(savePlanErrorText('provider_auth_keychain')).toContain('系统凭据')
  expect(savePlanErrorText('future_backend_code')).toContain('已保留执行记录')
  expect(savePlanErrorText(null)).toBe('')
 })
})
