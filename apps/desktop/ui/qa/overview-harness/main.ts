// QA-only explicit IPC fixture. Production components and handlers render unchanged.
const params = new URLSearchParams(location.search)
const gate=params.get('env') || 'ready'
const found={found:true,path:'/fixtures/node',version:null}, absent={found:false,path:null,version:null}
const source={kind:'unresolved',path:null,version:null,resolvedAt:null,managedEntry:'/fixtures/data/runtime/bin/ocx',managedPrefix:'/fixtures/data/runtime/opencodex',defaultPrefix:'/fixtures/data/runtime/opencodex',explicitPath:null,insideDataRoot:true,history:[]}
const environment={gate:gate==='checking'?'ready':gate,node:gate==='missing_node'?absent:found,npm:['missing_node','missing_npm'].includes(gate)?absent:found,ocx:gate==='ready'?found:absent,shortCircuited:gate!=='ready',brewFound:params.get('brew')!=='0'}
const status={channel:'stable',currentVersion:'0.1.9',availableVersion:'0.1.10',lastCheckedAt:'2026-10-28',signatureVerified:true,error:null,notes:'概览更新入口\n\n检查后集中查看管理器与面板更新，分别确认每个对象的更新范围。',pendingRestart:null}
let updateModule: typeof import('/src/features/updates/update')
const api={invoke:async(command:string,args:any={})=>{
 if(command==='get_update_status')return status
 if(command==='check_for_update')return {status:'available',update:status}
 if(command==='runtime_source')return source
 if(command==='runtime_protection_status')return {state:'none',version:null,backupId:null}
 if(command==='discover_environment')return environment
 if(command==='get_data_root_config')return {currentDataRoot:'/fixtures/data',activeDataRoot:'/fixtures/data',currentOpencodexHome:'/fixtures/home',runtimeActive:true}
 if(command==='update_release_details')return {target:args.target,channel:args.channel,candidateVersion:args.candidateVersion,notesState:'ready',notes:status.notes,notesSource:'updater',releaseUrl:'https://github.com/gzers/opencodex-desktop/releases/tag/v'+args.candidateVersion,fetchedAt:1,publishedAt:null,cached:false}
 if(command==='install_update'){
  updateModule.managerInstallProgress.value={operationId:'qa',target:'manager',channel:'stable',candidateVersion:'0.1.10',generation:1,sequence:1,stage:'downloading',downloadedBytes:100,totalBytes:200}
  await new Promise(resolve=>setTimeout(resolve,400))
  status.pendingRestart='0.1.10' as any
  updateModule.managerInstallProgress.value={...updateModule.managerInstallProgress.value!,stage:'pending_restart',sequence:2}
  return null
 }
 if(command==='plugin:dialog|open')return null
 if(command==='official_project_facts')return {displayName:'OpenCodex',version:'2.50.0',rawVersion:'2.50.0',truncated:false}
 if(command==='official_remote_latest')return {tag:'latest',version:'2.51.0',integrity:null}
 return null
},transformCallback:()=>1,metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},convertFileSrc:(x:string)=>x}
;(window as any).__TAURI_INTERNALS__=api
async function boot(){
 const [{createApp,h},{createPinia,setActivePinia},{default:Overview},{default:Install},{useAppStore},{useRuntimeStore},{useEnvironmentStore},{useAboutStore},{useDataRootStore},{useRouteStore},{useThemeStore},{useEffectsStore},updates,{default:Sidebar}]=await Promise.all([
 import('vue'),import('pinia'),import('/src/routes/OverviewRoute.vue'),import('/src/features/runtime/components/RuntimeInstallDialog.vue'),import('/src/stores/app'),import('/src/features/runtime/store'),import('/src/features/environment/store'),import('/src/features/about/store'),import('/src/features/data-root/store'),import('/src/stores/routes'),import('/src/app/appearance/theme'),import('/src/app/appearance/effects'),import('/src/features/updates/update'),import('/src/components/AppSidebar.vue')])
 updateModule=updates
 const pinia=createPinia();setActivePinia(pinia)
 useAboutStore().app={name:'OpenCodeX Desktop',version:'0.1.9',platform:params.get('platform') || 'macOS',identifier:'qa',framework:'Tauri v2',license:'MIT'} as any
 useRuntimeStore().source=source as any
 useEnvironmentStore().report=environment as any;useEnvironmentStore().loading=gate==='checking'
 useDataRootStore().config={currentDataRoot:'/fixtures/data',activeDataRoot:'/fixtures/data',currentOpencodexHome:'/fixtures/home',runtimeActive:true} as any
 useRouteStore().go('overview')
 updates.managerUpdateStatus.value=status as any
 const app=useAppStore();app.setRuntimeState('stopped' as any)
 useThemeStore().apply((params.get('theme') || 'light') as any)
 document.documentElement.dataset.theme=params.get('theme') || 'light'
 document.documentElement.dataset.effects='low'
 document.documentElement.style.setProperty('--ui-zoom','1')
 createApp({render:()=>h('div',{class:'app-window'},[h('div',{class:'desktop-stage'},h('div',{class:'app-shell'},[h(Sidebar),h('main',{class:'main'},h(Overview))])),h(Install)])}).use(pinia).mount('#harness')
}
void boot()
