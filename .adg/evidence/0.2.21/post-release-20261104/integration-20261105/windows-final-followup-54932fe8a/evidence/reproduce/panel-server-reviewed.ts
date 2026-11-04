import {writeFileSync,readFileSync} from 'node:fs'
import {join} from 'node:path'
const base=import.meta.dir,root=join(base,'first-scope-reviewed'),launch=JSON.parse(readFileSync(join(root,'identity.json'),'utf8'))
for(const [key,value]of Object.entries(launch.environment))process.env[key]=String(value)
if(!process.env.OPENCODEX_HOME?.includes('post021-windows-final'))throw Error('isolated home required')
const official=join(base,'../post021-restart-panel/tools/node_modules/@bitkyc08/opencodex')
const configPath=join(process.env.OPENCODEX_HOME,'config.json'),config=JSON.parse(readFileSync(configPath,'utf8'))
Object.assign(config,{bindHost:'127.0.0.1',port:60444,codexAutoStart:false,oauthOpenBrowser:false,syncOnStart:false,autoUpdate:false,claudeCodeEnabled:false,clientIntegrations:{codex:false,grok:false,claude:false}})
writeFileSync(configPath,JSON.stringify(config,null,2))
const {startServer}=await import(join(official,'src/server/index.ts'))
const server=startServer(Number(process.argv[2]??60444),{managementApi:{codexRestartService:{readState:async()=>({state:'not_running',runningCount:0}),performRestart:async()=>{throw Error('desktop restart is forbidden by panel-only isolation fixture')}}}})
writeFileSync(join(process.env.OPENCODEX_HOME,'runtime-port.json'),JSON.stringify({pid:process.pid,port:server.port}))
writeFileSync(join(base,'panel-reviewed-service-identity.json'),JSON.stringify({pid:process.pid,port:server.port,home:process.env.OPENCODEX_HOME,officialPackage:'2.84.0',restartDisabledByFixture:true}))
console.log('owned official GUI service '+server.port)
