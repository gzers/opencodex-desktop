import {expect,it,vi} from 'vitest'
import {invoke} from '@tauri-apps/api/core'
import {modelTransferApi,type TransferScope} from '@/features/sync/modelTransfer'
import type {ModelPolicy} from '@/features/models/api'

vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}))

it('passes the optional actual in-memory policy to draft export IPC',async()=>{
  const scope:TransferScope={includePolicy:true,channelIds:[],templateRefs:[],includeGlobal:true}
  const draft={expectedLibrary:'library',expectedPolicy:'file-digest',expectedWorkspace:'workspace'}
  const policy:ModelPolicy={schemaVersion:1,templateRefs:[],channels:[],defaultChannelId:null}
  await modelTransferApi.prepareDraft(scope,draft,'fixture-passphrase',policy)
  expect(invoke).toHaveBeenCalledExactlyOnceWith('model_transfer_prepare_draft',{scope,draft,passphrase:'fixture-passphrase',policy})
  await modelTransferApi.prepareDraft(scope,draft)
  expect(invoke).toHaveBeenLastCalledWith('model_transfer_prepare_draft',{scope,draft,passphrase:null,policy:null})
})
