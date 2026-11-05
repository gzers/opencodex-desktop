import {readFileSync,writeFileSync} from 'node:fs'
import {join} from 'node:path'
const base=import.meta.dir,root=join(base,'../post021-windows-final-msvc-41913003716/perf'),launch=JSON.parse(readFileSync(join(root,'launch.json'),'utf8'))
for(const name of Object.keys(process.env))delete process.env[name]
Object.assign(process.env,launch.environment)
if(!process.env.OPENCODEX_HOME?.includes('post021-windows-final-msvc-41913003716'))throw Error('owned isolated home required')
const official=join(base,'../post021-restart-panel/tools/node_modules/@bitkyc08/opencodex')
const packageInfo=JSON.parse(readFileSync(join(official,'package.json'),'utf8'));if(packageInfo.version!=='2.84.0')throw Error('pinned package required')
const port=Number(process.argv[2]);if(![60444,60446].includes(port))throw Error('fixed owned fixture port required')
const configPath=join(process.env.OPENCODEX_HOME,'config.json'),config=JSON.parse(readFileSync(configPath,'utf8'))
Object.assign(config,{bindHost:'127.0.0.1',port,codexAutoStart:false,oauthOpenBrowser:false,syncOnStart:false,autoUpdate:false,claudeCodeEnabled:false,clientIntegrations:{codex:false,grok:false,claude:false}})
for(const provider of Object.values(config.providers??{}))if(String(provider.baseUrl??'').startsWith('http://127.0.0.1:60443'))provider.allowPrivateNetwork=true
writeFileSync(configPath,JSON.stringify(config,null,2))
const {startServer}=await import(join(official,'src/server/index.ts'))
const server=startServer(port,{managementApi:{codexRestartService:{readState:async()=>({state:'not_running',runningCount:0}),performRestart:async()=>{throw Error('restart forbidden in panel fixture')}}}})
const identity={pid:process.pid,port:server.port,home:process.env.OPENCODEX_HOME,officialPackage:'2.84.0',restartDisabledByFixture:true,at:new Date().toISOString()}
writeFileSync(join(process.env.OPENCODEX_HOME,'runtime-port.json'),JSON.stringify({pid:process.pid,port:server.port}))
writeFileSync(join(base,'connected-native/panel-service-'+port+'.json'),JSON.stringify(identity,null,2));console.log(JSON.stringify(identity))
