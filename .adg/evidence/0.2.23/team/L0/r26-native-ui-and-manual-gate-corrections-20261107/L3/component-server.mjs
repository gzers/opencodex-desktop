import fs from 'node:fs';
import {createServer} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/node_modules/vite/dist/node/index.js';
const root='/Users/ezio/.codex/worktrees/ocxd-023-ui/ocx-0219-fixes/apps/desktop/ui';
const entry=fs.readFileSync(new URL('./component-entry.mjs',import.meta.url),'utf8');
const server=await createServer({root,cacheDir:'/Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-f5de-7c53-89a3-cad653af6685/native-models-parity/vite-cache',configFile:root+'/vite.config.ts',plugins:[{name:'l3-component-evidence',resolveId(id){if(id==='virtual:l3-evidence')return '\0l3-evidence';},load(id){if(id==='\0l3-evidence')return entry;},configureServer(server){server.middlewares.use('/__l3-component',async(req,res,next)=>{if(req.url?.includes('html-proxy'))return next();res.setHeader('Content-Type','text/html');res.end(await server.transformIndexHtml('/__l3-component','<!doctype html><html lang="zh-CN"><meta charset="UTF-8"><title>L3 actual component evidence</title><div id="app"></div><script type="module">import "virtual:l3-evidence"</script></html>'));});}}],server:{port:43133,host:'127.0.0.1',strictPort:true}});
await server.listen();console.log('L3 actual component fixture on 127.0.0.1:43133');
process.on('SIGTERM',async()=>{await server.close();process.exit(0)});
