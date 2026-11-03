import fs from 'node:fs/promises'
import path from 'node:path'
import {pickEngine as originalPickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs'
export {Blocked, engineLabel} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs'
const ROOT='/tmp/ocx020-final-review-agent/final-current-version-details/'
const allowed=url=>{try{const u=new URL(url);return ['about:','data:','blob:'].includes(u.protocol)||(['http:','ws:'].includes(u.protocol)&&u.hostname==='127.0.0.1'&&['52422'].includes(u.port))}catch{return false}}
export async function pickEngine(){
 const e=await originalPickEngine()
 return {...e,launch:async()=>{
  const browser=await e.launch(),observations=[],requests=[],blocked=[],pageErrors=[],resourceErrors=[]
  const seenPages=new WeakSet(),seenContexts=new WeakSet()
  async function preparePage(page){
   if(seenPages.has(page))return page;seenPages.add(page)
   page.setDefaultTimeout(15000)
   page.on('pageerror',x=>pageErrors.push({url:page.url(),message:x.message}))
   page.on('request',r=>requests.push({url:r.url(),method:r.method(),type:r.resourceType()}))
   page.on('response',r=>{if(r.status()>=400)resourceErrors.push({url:r.url(),status:r.status()})})
   const rawRoute=page.route.bind(page)
   page.route=(pattern,handler,options)=>rawRoute(pattern,async(route,request)=>{if(!allowed(route.request().url())){blocked.push(route.request().url());return route.abort('blockedbyclient')}return handler(route,request)},options)
   const screenshot=page.screenshot.bind(page)
   page.screenshot=async(options={})=>{
    if(options.path&&!path.resolve(options.path).startsWith(ROOT))throw Error('Screenshot outside authorized directory: '+options.path)
    const sample=await page.evaluate(()=>{
     const css=getComputedStyle(document.documentElement),body=getComputedStyle(document.body),zoom=parseFloat(css.getPropertyValue('--ui-zoom'))||1
     const box=el=>{const r=el.getBoundingClientRect();return {x:r.x,y:r.y,right:r.right,bottom:r.bottom,width:r.width,height:r.height}}
     const tokens=Object.fromEntries(['micro','caption','label','control','body','subtitle','title','display'].map(k=>[k,css.getPropertyValue('--text-'+k).trim()]))
     const visible=el=>el.getBoundingClientRect().width>0&&el.getBoundingClientRect().height>0&&getComputedStyle(el).visibility!=='hidden'
     const surfaces=[...document.querySelectorAll('dialog[open],.ext-detail,[data-testid=update-center] .update-center,#model-agent-panel,#mpAssistantPanel')].filter(visible).map(el=>({selector:el.tagName+'.'+el.className,title:(el.querySelector('h2,h3,strong')?.textContent||'').slice(0,100),rect:box(el),headings:[...el.querySelectorAll('h2,h3')].map(h=>({text:h.textContent,fontSize:getComputedStyle(h).fontSize})),controls:[...el.querySelectorAll('header button,footer button,.update-center-foot button,.modal-actions button')].filter(visible).map(b=>({text:b.textContent.trim(),rect:box(b),fontSize:getComputedStyle(b).fontSize}))}))
     return {url:location.href,viewport:{width:innerWidth,height:innerHeight},zoom,bodyFont:body.fontSize,fontFamily:body.fontFamily,tokens,surfaces}
    }).catch(error=>({sampleError:error.message}))
    const expected={micro:'10px',caption:'11px',label:'12px',control:'12px',body:'13px',subtitle:'15px',title:'18px',display:'22px'}
    sample.fontContractPass=sample.bodyFont==='13px'&&Object.entries(expected).every(([k,v])=>sample.tokens?.[k]===v)
    observations.push({screenshot:options.path??null,...sample})
    return screenshot(options)
   }
   return page
  }
  async function prepareContext(context){
   if(seenContexts.has(context))return context;seenContexts.add(context)
   await context.route('**/*',route=>{if(allowed(route.request().url()))return route.continue();blocked.push(route.request().url());return route.abort('blockedbyclient')})
   await context.addInitScript(()=>{window.__reviewAgentBrowserOnly=true})
   context.on('page',page=>{void preparePage(page)})
   return context
  }
  const newContext=browser.newContext.bind(browser),newPage=browser.newPage.bind(browser)
  browser.newContext=async options=>prepareContext(await newContext(options))
  browser.newPage=async options=>{const p=await newPage(options);await prepareContext(p.context());return preparePage(p)}
  const close=browser.close.bind(browser)
  browser.close=async()=>{
   const out=process.env.AUDIT_OUT
   if(!out||!path.resolve(out).startsWith(ROOT))throw Error('AUDIT_OUT outside authorized directory')
   await fs.mkdir(out,{recursive:true})
   await fs.writeFile(path.join(out,'browser-boundary-and-fonts.json'),JSON.stringify({engine:e.name,productSource:process.env.AUDIT_SOURCE,
    allowedOrigins:['http://127.0.0.1:52422','http://127.0.0.1:52424','http://127.0.0.1:52429'],
    scope:'Explicit isolated browser adapters only; no native IPC, real model/remote server operations or daily source. Font samples are computed values at captured actual states.',
    blockedExternal:blocked,pageErrors,resourceErrors,requests,observations},null,2)+'\n')
   return close()
  }
  return browser
 }}
}
