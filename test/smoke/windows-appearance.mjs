// Fixed-source native WebView2 checks. The PowerShell driver owns focus, token,
// window state and isolated process lifetime; this script never changes system settings.
import assert from 'node:assert/strict'
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import { join } from 'node:path'
import { discoverMainTarget } from './windows-cdp.mjs'
import { pngStripeContrast } from './windows-png.mjs'

const [endpoint, output, phase = 'visual', expectedScale = '1'] = process.argv.slice(2)
await mkdir(output, { recursive: true })
const diagnostics = {}
const target = await discoverMainTarget(endpoint, { diagnostics,
  saveDiagnostics: value => writeFile(join(output, `cdp-${phase}.json`), JSON.stringify(value, null, 2)) })
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => { ws.addEventListener('open', resolve, { once: true }); ws.addEventListener('error', reject, { once: true }) })
let id = 0
const pending = new Map()
ws.addEventListener('message', event => {
  const value = JSON.parse(event.data), request = pending.get(value.id)
  if (request) { pending.delete(value.id); value.error ? request.reject(Error(value.error.message)) : request.resolve(value.result) }
})
function call(method, params = {}) {
  return new Promise((resolve, reject) => {
    const current = ++id, timer = setTimeout(() => { pending.delete(current); reject(Error(method + ' timeout')) }, phase === 'performance' ? 45_000 : 15_000)
    pending.set(current, { resolve: value => { clearTimeout(timer); resolve(value) }, reject: error => { clearTimeout(timer); reject(error) } })
    ws.send(JSON.stringify({ id: current, method, params }))
  })
}
async function evaluate(expression) {
  const response = await call('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true })
  assert(!response.exceptionDetails, JSON.stringify(response.exceptionDetails))
  return response.result.value
}
async function capture(name) {
  const data = Buffer.from((await call('Page.captureScreenshot', { format: 'png' })).data, 'base64')
  await writeFile(join(output, name + '.png'), data)
  return data
}
async function configure(tier, renderer, theme = 'dark') {
  return evaluate(`(async()=>{
    const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;
    const prefs=p._s.get('preferences');
    if(!await prefs.setVisualEffects(${JSON.stringify(tier)}))throw Error('tier save failed');
    if(!await prefs.setGlowRender(${JSON.stringify(renderer)}))throw Error('renderer save failed');
    p._s.get('theme').apply(${JSON.stringify(theme)});
    p._s.get('routes').go('overview');
    await new Promise(r=>setTimeout(r,180));
    return {tier:document.documentElement.dataset.effects,renderer:document.documentElement.dataset.glowRender};
  })()`)
}
async function sample(staticResize = false) {
  return evaluate(`(async()=>{
    const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia,e=p._s.get('effects');
    const hero=document.querySelector('.motion-hero'),canvas=document.querySelector('.motion-mesh-host canvas');
    const gl=canvas?.getContext('webgl'),original=gl?.drawArrays;
    let draws=0,alpha=0;
    if(gl)gl.drawArrays=function(...args){const r=original.apply(this,args);draws++;const a=new Uint8Array(4);
      gl.readPixels(Math.floor(canvas.width*.5),Math.floor(canvas.height*.55),1,1,gl.RGBA,gl.UNSIGNED_BYTE,a);alpha=Math.max(alpha,a[3]);return r};
    const snapshots=[hero.innerHTML],intervals=[];let last=0;
    let raf=0,active=true;
    const tick=t=>{if(!active)return;if(last)intervals.push(t-last);last=t;raf=requestAnimationFrame(tick)};
    raf=requestAnimationFrame(tick);
    try {
      ${staticResize ? "const host=canvas?.parentElement;if(host)host.style.width=(host.clientWidth-2)+'px';await new Promise(r=>setTimeout(r,100));if(host)host.style.removeProperty('width');" : ''}
      for(let i=0;i<4;i++){await new Promise(r=>setTimeout(r,160));snapshots.push(hero.innerHTML)}
      intervals.sort((a,b)=>a-b);
      return {tier:e.setting,effective:e.effective,foreground:await window.__TAURI_INTERNALS__.invoke('app_foreground'),
        docFocus:document.hasFocus(),visible:document.visibilityState,strategy:JSON.parse(JSON.stringify(e.strategy)),
        reducedMedia:matchMedia('(prefers-reduced-motion: reduce)').matches,
        svgFrames:new Set(snapshots).size,draws,alpha,canvas:!!canvas,
        rafP95:intervals[Math.floor(intervals.length*.95)]??0,frameSamples:intervals.length,DPR:devicePixelRatio,
        svgTransform:hero.querySelector('svg')?.style.transform};
    }finally{active=false;cancelAnimationFrame(raf);if(gl)gl.drawArrays=original}
  })()`)
}
async function performanceProbe(renderer) {
  await configure('high', renderer)
  await new Promise(resolve => setTimeout(resolve, 1000))
  return evaluate(`(async()=>{
    const intervals=[],costs=[],longTasks=[],native=requestAnimationFrame;let last=0,active=true,ownFrame=0;
    const observed=new PerformanceObserver(list=>longTasks.push(...list.getEntries().map(e=>({start:e.startTime,duration:e.duration}))));
    observed.observe({type:'longtask'});
    window.requestAnimationFrame=callback=>native.call(window,time=>{
      const start=performance.now();try{callback(time)}finally{costs.push(performance.now()-start)}
    });
    const tick=time=>{if(!active)return;if(last)intervals.push(time-last);last=time;ownFrame=native.call(window,tick)};
    ownFrame=native.call(window,tick);
    try{
      await new Promise(r=>setTimeout(r,30000));
      const ordered=[...intervals].sort((a,b)=>a-b),work=[...costs].sort((a,b)=>a-b),B=1000/60;
      return {renderer:${JSON.stringify(renderer)},durationMs:30000,refreshHz:60,B,samples:ordered.length,
        p50:ordered[Math.floor(ordered.length*.5)],p95:ordered[Math.floor(ordered.length*.95)],
        max:Math.max(...ordered),over6B:intervals.filter(v=>v>6*B).length,
        scriptP95:work[Math.floor(work.length*.95)],scriptMax:Math.max(...work),longTasks,intervals,costs,
        foreground:await window.__TAURI_INTERNALS__.invoke('app_foreground')};
    }finally{active=false;cancelAnimationFrame(ownFrame);window.requestAnimationFrame=native;observed.disconnect()}
  })()`)
}
const results = { phase, cases: [] }
try {
  const about = await evaluate(`window.__TAURI_INTERNALS__.invoke('app_about')`)
  const config = JSON.parse(await readFile(new URL('../../apps/desktop/tauri/tauri.conf.json', import.meta.url)))
  assert.equal(about.version, config.version)
  results.about = about
  results.environment = await evaluate(`(()=>{
    const canvas=document.createElement('canvas'),gl=canvas.getContext('webgl'),debug=gl?.getExtension('WEBGL_debug_renderer_info');
    const result={userAgent:navigator.userAgent,DPR:devicePixelRatio,viewport:{width:innerWidth,height:innerHeight},
      hardwareConcurrency:navigator.hardwareConcurrency,webgl:!!gl,
      renderer:debug?gl.getParameter(debug.UNMASKED_RENDERER_WEBGL):null,
      vendor:debug?gl.getParameter(debug.UNMASKED_VENDOR_WEBGL):null};
    gl?.getExtension('WEBGL_lose_context')?.loseContext();return result;
  })()`)
  if (phase === 'visual') {
    for (const media of ['reduce', 'no-preference']) {
      await call('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-motion', value: media }] })
      for (const renderer of ['mesh', 'css']) {
        for (const tier of ['high', 'mid', 'low']) {
          await configure(tier, renderer)
          const result = await sample(tier === 'mid' && renderer === 'mesh')
          results.cases.push({ media, renderer, ...result })
          assert.equal(result.tier, tier); assert.equal(result.effective, tier)
          assert(result.foreground, 'Native foreground signal missing during visual sample')
          assert(Math.abs(result.DPR - Number(expectedScale)) < .01, 'Actual DPR differs from requested diagnostic scale')
          if (tier === 'high') {
            assert(result.svgFrames > 1, 'High tier SVG did not advance')
            if (renderer === 'mesh') assert(result.draws > 1 && result.alpha > 0, 'High WebGL did not render light')
          } else {
            assert.equal(result.svgFrames, 1, 'Static tier SVG runs continuously')
            if (tier === 'mid' && renderer === 'mesh') {
              assert(result.draws > 0 && result.draws <= 4 && result.alpha > 0, 'Mid WebGL failed static resize paint')
            } else assert.equal(result.draws, 0)
          }
          if (tier === 'low') assert.equal(result.canvas, false)
          if (media === 'reduce') await capture(`${renderer}-${tier}`)
        }
      }
    }
    // Nine state targets × both renderers in both themes; testing only the decoration's
    // state projection, never manufacturing a business runtime status.
    for (const theme of ['dark', 'light']) {
      for (const renderer of ['mesh', 'css']) {
        await configure('mid', renderer, theme)
        const states = await evaluate(`(async()=>{
          const node=document.querySelector('.motion-mark');
          // Production Vue omits __vueParentComponent. Traverse the mounted vnode
          // tree rather than requiring dev hooks or altering business runtime data.
          const find=vnode=>{
            if(!vnode)return null;
            if(vnode.component?.type.__name==='RuntimeMotionMark')return vnode.component;
            const subtree=find(vnode.component?.subTree);if(subtree)return subtree;
            for(const child of Array.isArray(vnode.children)?vnode.children:[]){
              const result=find(child);if(result)return result;
            }
            return null;
          };
          const component=find(document.querySelector('#app').__vue_app__._container._vnode);
          if(!component)throw Error('motion component missing');
          const original=component.props.state,results=[];
          for(const state of ['not_ready','stopped','starting','running','stopping','failed','problem','confirming','stale']){
            component.props.state=state;await new Promise(r=>setTimeout(r,40));
            results.push({state,path:node.querySelector('[data-layer=core]').innerHTML,background:getComputedStyle(node.querySelector('.cloud-a')).backgroundImage});
          }
          component.props.state=original;return results;
        })()`)
        results.cases.push({ type: 'state-targets', theme, renderer, states })
        await capture(`${renderer}-${theme}-soft`)
      }
    }
    await configure('high', 'mesh')
  } else if (phase === 'performance') {
    for (const renderer of ['mesh', 'css']) {
      const result = await performanceProbe(renderer)
      results.cases.push(result)
      assert(result.foreground, 'Performance sample was not foreground')
      assert(result.samples > 900, 'Stable 30s performance sample was too short')
      assert(result.p95 <= 2 * result.B, 'High-tier frame interval exceeded 2B')
      assert.equal(result.over6B, 0, 'Long frame requires analysis')
      assert(result.scriptP95 <= result.B / 2, 'Script rendering exceeded B/2')
    }
    await configure('high', 'mesh')
  } else if (phase === 'prepare-high' || phase === 'prepare-mid') {
    results.cases.push(await configure(phase.slice(8), 'mesh'))
  } else if (phase === 'background' || phase === 'recovery') {
    const tier = await evaluate(`document.documentElement.dataset.effects`)
    const result = await sample(phase === 'recovery' && tier === 'mid')
    results.cases.push(result)
    if (phase === 'background') {
      assert.equal(result.foreground, false); assert.equal(result.draws, 0); assert.equal(result.svgFrames, 1)
    } else {
      assert(result.foreground)
      if (tier === 'high') { assert(result.draws > 1); assert(result.svgFrames > 1) }
      else { assert.equal(result.svgFrames, 1); assert(result.draws > 0 && result.draws <= 4 && result.alpha > 0) }
    }
  } else if (phase === 'reload') {
    for (const tier of ['high', 'mid', 'low']) {
      await configure(tier, 'mesh')
      await call('Page.reload')
      let result
      for (let attempt = 0; attempt < 40; attempt++) {
        result = await evaluate(`(async()=>{
          const p=document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia;
          if(!p?._s.get('preferences')?.data)return null;
          return {tier:document.documentElement.dataset.effects,
            saved:(await window.__TAURI_INTERNALS__.invoke('get_preferences')).visualEffects};
        })()`)
        if (result) break
        await new Promise(resolve => setTimeout(resolve, 100))
      }
      results.cases.push(result)
      assert(result, 'Preferences did not initialize after reload')
      assert.equal(result.tier, tier); assert.equal(result.saved, tier)
    }
    await configure('high', 'mesh')
  } else if (phase === 'menu') {
    for (const theme of ['dark', 'light']) {
      for (const tier of ['high', 'mid', 'low']) {
        await configure(tier, 'mesh', theme)
        const geometry = await evaluate(`(async()=>{
          const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;
          p._s.get('routes').go('settings',{section:'general'});await new Promise(r=>setTimeout(r,100));
          const row=document.querySelector('[data-testid=setting-visual-effects]');row.scrollIntoView({block:'center'});
          const trigger=row.querySelector('.select-trigger');trigger.click();await new Promise(r=>setTimeout(r,100));
          const menu=document.querySelector('.select-menu[aria-label="界面特效"]'),r=menu.getBoundingClientRect();
          if(!menu)throw Error('menu missing');
          const style=e=>{const c=getComputedStyle(e);return {class:e.className,blur:c.backdropFilter,
            background:c.backgroundColor,filter:c.filter,opacity:c.opacity,transform:c.transform,
            willChange:c.willChange,isolation:c.isolation,zIndex:c.zIndex,overflow:c.overflow,mask:c.maskImage}};
          const parents=[];for(let e=menu.parentElement;e;e=e.parentElement)parents.push(style(e));
          window._appearanceMenu=menu;
          const test=document.createElement('div');test.id='appearance-stripes';
          const shell=document.querySelector('.app-window'),zoom=Number(getComputedStyle(shell).zoom)||1;
          test.style.cssText='position:fixed;pointer-events:none;z-index:40;background:repeating-linear-gradient(90deg,#fff 0px,#fff 2px,#000 2px,#000 4px);'
            +'left:'+r.left/zoom+'px;top:'+r.top/zoom+'px;width:'+r.width/zoom+'px;height:'+r.height/zoom+'px;';
          shell.insertBefore(test,menu);
          // Keep the fixture's inline positioning measurable; global application
          // rules also apply to direct shell children.
          test.style.setProperty('position','fixed','important');
          test.style.setProperty('z-index','40','important');
          const fixture={rect:test.getBoundingClientRect().toJSON(),style:test.style.cssText,
            computed:style(test)};
          return {rect:r.toJSON(),...style(menu),parents,
            trigger:trigger.getBoundingClientRect().toJSON(),viewport:{width:innerWidth,height:innerHeight},DPR:devicePixelRatio,zoom,fixture};
        })()`)
        assert(geometry.parents.every(parent => parent.blur === 'none'), 'Menu is still nested in an ancestor backdrop root')
        assert(geometry.rect.left >= 0 && geometry.rect.right <= geometry.viewport.width + 1 && geometry.rect.bottom <= geometry.viewport.height + 1)
        const filtered = await capture(`menu-${theme}-${tier}-filtered`)
        await evaluate(`window._appearanceMenu.style.backdropFilter='none';window._appearanceMenu.style.webkitBackdropFilter='none';new Promise(r=>setTimeout(r,50))`)
        const sharp = await capture(`menu-${theme}-${tier}-no-filter`)
        const filteredContrast = pngStripeContrast(filtered, geometry.rect, geometry.DPR)
        const sharpContrast = pngStripeContrast(sharp, geometry.rect, geometry.DPR)
        results.cases.push({ theme, tier, ...geometry, filteredContrast, sharpContrast })
        if (tier !== 'low') {
          assert(sharpContrast > 10 / geometry.DPR, 'Unfiltered fixture pixels do not contain the expected stripe signal')
          assert(filteredContrast < sharpContrast * .35, `Menu pixels are not blurred (${filteredContrast}/${sharpContrast})`)
        }
        else assert.equal(geometry.blur, 'none')
        await evaluate(`document.querySelector('#appearance-stripes').remove();window._appearanceMenu.style.removeProperty('backdrop-filter');window._appearanceMenu.style.removeProperty('-webkit-backdrop-filter');`)
        await capture(`menu-${theme}-${tier}-actual`)
        // Keyboard selects real option and persists; Escape/outside clicks close.
        const keyboard = await evaluate(`(async()=>{
          const menu=window._appearanceMenu;
          const options=[...menu.querySelectorAll('[role=option]')];
          options[0].focus();
          options[0].dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));
          const moved=document.activeElement===options[1];
          options[1].click();await new Promise(r=>setTimeout(r,120));
          const saved=await window.__TAURI_INTERNALS__.invoke('get_preferences');
          const closed=!document.querySelector('.select-menu-portal');
          const trigger=document.querySelector('[data-testid=setting-visual-effects] .select-trigger');
          trigger.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));await new Promise(r=>setTimeout(r,40));
          document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));await new Promise(r=>setTimeout(r,40));
          const escaped=!document.querySelector('.select-menu-portal')&&document.activeElement===trigger;
          trigger.click();await new Promise(r=>setTimeout(r,40));
          document.body.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));await new Promise(r=>setTimeout(r,40));
          return {moved,saved:saved.visualEffects,closed,escaped,outsideClosed:!document.querySelector('.select-menu-portal')};
        })()`)
        assert(keyboard.moved && keyboard.closed && keyboard.escaped && keyboard.outsideClosed)
        assert.equal(keyboard.saved, 'mid'); results.cases.at(-1).keyboard = keyboard
      }
    }
    // Application zoom remains independent of native DPR, including menu edge placement.
    for (const zoom of [75, 150, 200]) {
      const result = await evaluate(`(async()=>{
        const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;
        await p._s.get('preferences').setScale(${zoom});await new Promise(r=>setTimeout(r,50));
        const row=document.querySelector('[data-testid=setting-visual-effects]');row.scrollIntoView({block:'center'});
        row.querySelector('.select-trigger').click();await new Promise(r=>setTimeout(r,60));
        const menu=document.querySelector('.select-menu-portal'),r=menu.getBoundingClientRect();
        const result={zoom:${zoom},rect:r.toJSON(),viewport:{width:innerWidth,height:innerHeight}};
        document.body.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));return result;
      })()`)
      assert(result.rect.left >= 0 && result.rect.right <= result.viewport.width + 1 && result.rect.bottom <= result.viewport.height + 1)
      results.cases.push(result)
    }
    await evaluate(`document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('preferences').setScale(100)`)
    await configure('high', 'mesh')
  } else throw Error('unknown phase')
  results.result = 'pass'
  console.log(`PASS: ${phase}, ${results.cases.length} cases`)
} catch (error) {
  results.result = 'fail'; results.error = error.stack
  await capture(`failure-${phase}`).catch(() => {})
  throw error
} finally {
  await writeFile(join(output, `${phase}.json`), JSON.stringify(results, null, 2) + '\n')
  await call('Emulation.setEmulatedMedia', { features: [] }).catch(() => {})
  ws.close()
}
