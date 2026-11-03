// Baseline 0.2.19 only needs its own saved-file API contract. Never impose new
// 0.2.20 dialog/adoption behavior before the updater has installed that version.
import assert from 'node:assert/strict'
import {randomUUID,createHash} from 'node:crypto'
import {writeFile,readFile} from 'node:fs/promises'
import {join} from 'node:path'
import {modelConfigPath,modelFileEvidence,assertModelFileWrite} from './windows-model-ui.mjs'
export async function seedOtaBaseline(invoke,output,facts,sandbox){
 const config=modelConfigPath(facts,sandbox)
 const channelId=randomUUID(),nodeId=randomUUID(),providerId='ota-fixture-'+randomUUID().slice(0,8)
 const rules=Object.fromEntries(['protocol','contextWindow','inputModalities','reasoningEfforts','defaultEffort','fast','pinnedEffort','effortMap','noReasoning','reasoningAuthoritative','fastWire'].map(k=>[k,{mode:'inherit'}]))
 const initial=await invoke('model_file_snapshot')
 const modelIds=['ota-fixture-alpha','ota-fixture-beta']
 const channel={id:channelId,label:'Owned OTA baseline',templateRefs:[],modelRefs:{},selection:{mode:'whitelist',models:modelIds},modelOrder:modelIds,discovered:modelIds.map(id=>({id,label:id,rules})),defaults:rules,overrides:{},familyOverrides:{},connection:{providerId,protocol:'openai-chat',nodes:[{id:nodeId,name:'Never requested',url:'https://example.test/v1'}],activeNodeId:nodeId,credentialRef:null,enabled:false,allowPrivateNetwork:false,authMode:'key'}}
 const seeded=await invoke('model_file_save',{request:{expectedFile:initial.policyDigest,policy:{...initial.policy,channels:[...initial.policy.channels,channel]},credentials:[{channelId,action:'replace',value:'synthetic-ota-fixture-key'}]}})
 assert(seeded.changed,'baseline seed was not saved')
 const before=await modelFileEvidence(config,sandbox),raw=JSON.parse(before.bytes)
 assert.equal(raw.providers[providerId].apiKey,'synthetic-ota-fixture-key')
 const policy=structuredClone(seeded.snapshot.policy),target=policy.channels.find(c=>c.id===channelId)
 assert(target,'saved baseline fixture channel missing');target.modelOrder=[...modelIds].reverse()
 const saved=await invoke('model_file_save',{request:{expectedFile:seeded.snapshot.policyDigest,policy,credentials:[]}})
 assert(saved.changed,'baseline second real file save was not committed')
 const after=await modelFileEvidence(config,sandbox);await assertModelFileWrite(before,after,sandbox)
 const final=JSON.parse(after.bytes);assert.equal(final.providers[providerId].apiKey,'synthetic-ota-fixture-key')
 for(const [id,provider] of Object.entries(raw.providers))assert.deepEqual(final.providers[id],provider,'baseline edit changed provider '+id)
 const record={configSha256:createHash('sha256').update(after.bytes).digest('hex'),fromVersion:facts.about.version,providerId,fixture:'two production saved-file commits and exact prewrite backup; synthetic key; no daily config, generation, sync, or new-version UI assertions'}
 await writeFile(join(output,'ota-data-before.json'),JSON.stringify(record,null,2)+'\n')
 assert.deepEqual(await readFile(config),after.bytes)
 return record
}
