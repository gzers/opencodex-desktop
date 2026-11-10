import {beforeEach,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ChannelsView from '@/features/models/ChannelsView.vue'
import ModelPicker from '@/features/models/ModelPicker.vue'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import ChannelModelTable from '@/features/models/ChannelModelTable.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {useModelsV2Store} from '@/features/models/store-v2'
import {emptyRulesV2,type ChannelV2,type FileSnapshotV2,type FileSaveRequestV2,type StageConnectionRequestV2,type ReadBaseV2} from '@/features/models/api-v2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
const id=(n:number)=>'00000000-0000-4000-8000-'+String(n).padStart(12,'0'),digest=(n:number)=>'sha256:'+String(n).repeat(64)
const copy=<T>(v:T):T=>JSON.parse(JSON.stringify(v)) as T
function channel():ChannelV2{return {id:id(1),label:'已有中文 渠道',connection:{providerId:'stable-provider',nodes:[{id:id(2),name:'主站',url:'https://primary.example.test'},{id:id(3),name:'备用',url:'https://backup.example.test'}],activeNodeId:id(2),credentialRef:'opaque-key',enabled:true,allowPrivateNetwork:false,authMode:'key',sourceReadId:null,importedConfig:null},selection:{mode:'whitelist',models:['raw/model']},modelOrder:['raw/model'],models:[{rawModelId:'raw/model',label:'Raw model',observed:false,source:{rawModelId:'raw/model',fields:{protocol:{value:'openai-chat',evidence:{kind:'imported_file',sourceDigest:digest(1),providerId:'stable-provider',pointer:'/providers/stable/adapter'}}}},explicitOverrides:emptyRulesV2(),binding:null}],templateRefs:[],defaults:emptyRulesV2(),familyOverrides:{},naming:{},namespace:null}}
function base(source=digest(1)):ReadBaseV2{return {dtoVersion:2,draftId:id(10),draftGeneration:1,sourceDigest:source,policyDigest:digest(2),libraryDigest:digest(3),rootRevision:'root-v2'}}
function fixture(withDraft=false):FileSnapshotV2{const policy={schemaVersion:2 as const,channels:[channel()],templateRefs:[],defaultChannelId:null,pendingRemovals:[],recoveryId:null};return {dtoVersion:2,library:{schemaVersion:2,templates:[],submissionReceipts:{}},libraryDigest:digest(3),savedPolicy:policy,draft:withDraft?{schemaVersion:2,base:base(),policy:copy(policy)}:null,sourceDigest:digest(1),rootRevision:'root-v2',effective:[{channelId:id(1),rawModelId:'raw/model',selected:true,enabled:true,observed:false,binding:null,rules:{...emptyRulesV2(),protocol:{mode:'custom',value:'openai-chat'}},protocol:{kind:'resolved',value:'openai-chat',origin:{kind:'source',evidence:{kind:'imported_file',sourceDigest:digest(1),providerId:'stable-provider',pointer:'/providers/stable/adapter'}}},fieldOrigins:{protocol:{kind:'source',evidence:{kind:'imported_file',sourceDigest:digest(1),providerId:'stable-provider',pointer:'/providers/stable/adapter'}}},matchedConditionId:null,notices:[]}]}}
let backend:FileSnapshotV2
beforeEach(()=>{setActivePinia(createPinia());backend=fixture();ipc.mockReset();ipc.mockImplementation(async(command,args)=>{
 if(command==='model_v2_file_snapshot')return copy(backend)
 if(command==='model_v2_begin_draft'){backend.draft??={schemaVersion:2,base:base(backend.sourceDigest),policy:copy(backend.savedPolicy)};return copy(backend)}
 if(command==='model_v2_read_existing'){backend.draft??={schemaVersion:2,base:base(),policy:copy(backend.savedPolicy)};return {snapshot:copy(backend),results:[{dtoVersion:2,readId:id(20),base:base(),nextDraftGeneration:1,changed:false,models:[],counts:{total:1,matched:1,manualPreserved:0,unmatched:0,missingDefault:0,missingDependency:0,conflict:0,changed:0,protocolPending:0}}]}}
 if(command==='model_v2_file_save'){const request=args.request as FileSaveRequestV2;for(const intent of request.channelIntents){const c=backend.draft!.policy.channels.find(c=>c.id===intent.channelId)!;if(intent.kind==='rename')c.label=intent.name;else if(intent.kind==='set_enabled')c.connection.enabled=intent.enabled;else if(intent.kind==='set_selection')c.selection=intent.selection;else if(intent.kind==='set_order')c.modelOrder=intent.rawModelIds}backend.savedPolicy=copy(backend.draft!.policy);backend.draft=null;backend.sourceDigest=digest(4);return copy(backend)}
 if(command==='model_v2_stage_connection'){const request=args.request as StageConnectionRequestV2;const c=channel();c.id=request.channelId??id(90);c.label=request.name;c.connection.nodes=request.nodes!;c.connection.activeNodeId=request.activeNodeId!;backend.draft??={schemaVersion:2,base:base(),policy:copy(backend.savedPolicy)};backend.draft.policy.channels.push(c);return copy(backend)}
 if(command==='model_v2_discover')return {dtoVersion:2,models:[{rawModelId:'raw/exact',label:'Exact'}],elapsedMs:1}
 throw new Error('Unexpected command '+command)
})})
