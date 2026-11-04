import {join,resolve} from 'node:path'
import {writeFileSync,mkdirSync,readFileSync,existsSync} from 'node:fs'
import {randomBytes,createHash} from 'node:crypto'
const base=import.meta.dir,label=process.argv[2],port=Number(process.argv[3]),root=join(base,label),home=join(root,'sandbox','opencodex-home')
if(!['baseline','candidate'].includes(label)||!port||resolve(process.env.OPENCODEX_HOME??'')!==resolve(home))throw Error('exact isolated home required')
for(const key of ['HOME','USERPROFILE','APPDATA','LOCALAPPDATA','CODEX_HOME','TEMP','TMP'])if(!resolve(process.env[key]??'').startsWith(resolve(root)))throw Error('outside isolated root: '+key)
mkdirSync(home,{recursive:true})
const official=join(base,'tools/node_modules/@bitkyc08/opencodex'),path=join(home,'config.json')
const configModule=await import(join(official,'src/config.ts'))
if(!existsSync(path)){
 const config=configModule.getDefaultConfig();config.port=port;config.bindHost='127.0.0.1';config.codexAutoStart=false;config.oauthOpenBrowser=false;config.syncOnStart=false;config.autoUpdate=false;config.providers={};
 writeFileSync(path,JSON.stringify(config,null,2))
}
const {startServer}=await import(join(official,'src/server/index.ts'))
const secret=randomBytes(32).toString('base64url')
const owned=label==='candidate'?await (await import(join(base,'../../post021-windows-materials/test/smoke/post021-restart/owned-desktop.ts'))).ownedDesktop(root,official):null
const server=startServer(port,{localAttestationSecret:secret,managementApi:{codexRestartService:{readState:async()=>({state:'not_running',runningCount:0}),performRestart:async()=>{
 if(!owned)throw Error('Global desktop restart is forbidden by this isolated panel fixture')
 const {performCodexRestart}=await import(join(official,'src/codex/app-server-restart-service.ts'))
 return performCodexRestart({restartDesktopApp:owned.restart,collectState:()=>({state:'not_running',processes:[]}),syncCatalog:async()=>false,resetStateCache:()=>{}})
}}}})
writeFileSync(join(home,'runtime-port.json'),JSON.stringify({pid:process.pid,port:server.port,attestationSecret:secret}))
writeFileSync(join(root,'server-identity.json'),JSON.stringify({pid:process.pid,port:server.port,package:JSON.parse(readFileSync(join(official,'package.json'),'utf8')).version,home,mode:'unmodified official startServer and packaged GUI; restart hook refuses global desktop operations',sourceSha256:createHash('sha256').update(readFileSync(join(official,'src/server/index.ts'))).digest('hex')}))
console.log('READY '+JSON.stringify({pid:process.pid,port:server.port,home}))
process.on('SIGTERM',()=>{owned?.stop();server.stop(true);process.exit(0)})
