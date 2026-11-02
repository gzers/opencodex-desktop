import fs from 'node:fs/promises';
import {spawn} from 'node:child_process';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs';
const ui='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui', out='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/.adg/evidence/0.2.19-macos-closeout-20261101';
const frozen='/tmp/ocx0219-frozen-prototype';
const vite=spawn(process.execPath,[ui+'/node_modules/vite/bin/vite.js','--host','127.0.0.1','--port','52419','--strictPort'],{cwd:ui,stdio:'ignore'});
const server=spawn('python3',['-m','http.server','52420','--bind','127.0.0.1','--directory',frozen],{stdio:'ignore'});
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
const checks=[],errors=[],states=[];let browser;
function check(name,ok,details){checks.push({name,ok,details})}
try {
 for(let i=0;i<60;i++){try{await fetch('http://127.0.0.1:52419');await fetch('http://127.0.0.1:52420');break}catch{await sleep(100)}}
 browser=await (await pickEngine()).launch();
 const app=await browser.newPage({viewport:{width:1180,height:760}}),proto=await browser.newPage({viewport:{width:1700,height:1000}});
 for(const [label,page] of [['app',app],['prototype',proto]]) page.on('pageerror',e=>errors.push(label+': '+e.message));
 await app.goto('http://127.0.0.1:52419/qa/overview-harness/index.html?theme=light&effects=high');
 await app.waitForSelector('.mods');
 await proto.goto('http://127.0.0.1:52420/'+encodeURI('docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html')+'#overview');
 await proto.waitForSelector('.window');
 for(const theme of ['light','dark'])for(const effects of ['high','mid','low']) {
  const read=async(page,prototype)=>{
   await page.evaluate(({theme,effects,prototype})=>{document.documentElement.dataset.theme=theme;document.documentElement.dataset.effects=effects;if(prototype){const w=document.querySelector('.window');Object.assign(w.style,{width:'1180px',height:'760px',flex:'none'})}},{theme,effects,prototype});
   await sleep(150);
   return page.evaluate(()=>{
    const value=s=>{const e=document.querySelector(s);if(!e)return null;const c=getComputedStyle(e);return {bg:c.backgroundColor,blur:c.backdropFilter||c.webkitBackdropFilter||'none'}};
    return {page:value('.main'),card:value('.card'),blurSupport:CSS.supports('backdrop-filter','blur(1px)')||CSS.supports('-webkit-backdrop-filter','blur(1px)')};
   });
  };
  const a=await read(app,false),p=await read(proto,true);states.push({theme,effects,app:a,prototype:p});
  for(const key of ['page','card'])for(const property of ['bg','blur'])check(theme+' '+effects+' '+key+' '+property,a[key]?.[property]===p[key]?.[property],{app:a[key]?.[property],prototype:p[key]?.[property]});
  check(theme+' '+effects+' blur supported',a.blurSupport&&p.blurSupport);
  await app.screenshot({path:out+'/'+theme+'-'+effects+'-webkit.png'});
  await proto.locator('.window').screenshot({path:out+'/'+theme+'-'+effects+'-prototype.png'});
 }
 check('zero page errors',errors.length===0,errors);
 await fs.writeFile(out+'/materials-webkit.json',JSON.stringify({baseline:'cfbd90c4b',engine:'WebKit',states,errors,checks},null,2));
 console.log(checks.filter(c=>c.ok).length+'/'+checks.length,'passed');
 if(checks.some(c=>!c.ok)){console.log(JSON.stringify(checks.filter(c=>!c.ok)));process.exitCode=1}
}finally{await browser?.close();vite.kill();server.kill()}
