import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs';
import fs from 'node:fs/promises';import path from 'node:path';
const engine=await pickEngine(),root=path.dirname(new URL(import.meta.url).pathname),browser=await engine.launch(),checks=[],errors=[];
function check(name,pass,detail){checks.push({name,passed:!!pass,detail});if(!pass)throw Error(name);}
try{for(const theme of ['light','dark']){
 const page=await browser.newPage({viewport:{width:1280,height:900}});page.on('pageerror',e=>errors.push(e.message));await page.route('**/*',r=>new URL(r.request().url()).hostname==='127.0.0.1'?r.continue():r.abort());await page.goto('http://127.0.0.1:53948/index.html#models');await page.waitForSelector('#mpChannelNode-newapi');await page.evaluate(t=>document.documentElement.dataset.theme=t,theme);await page.locator('#mpChannelNode-newapi').click();await page.locator('#modalMask').waitFor();await page.mouse.move(0,0);
 const styles=await page.evaluate(()=>['primary','backup'].map(id=>{const el=document.getElementById('mpNodePickerChoice-'+id),s=getComputedStyle(el);return {background:s.backgroundColor,weight:s.fontWeight,selected:el.getAttribute('aria-pressed'),text:el.textContent};}));
 check(theme+' no persistent selected background',styles[0].background===styles[1].background&&styles[0].background==='rgba(0, 0, 0, 0)',styles);
 check(theme+' no selected bold',styles[0].weight===styles[1].weight&&Number(styles[0].weight)<=400,styles.map(x=>x.weight));
 check(theme+' checkmark retained',styles[0].selected==='true'&&styles[0].text.includes('✓')&&styles[1].selected==='false');
 await page.screenshot({path:path.join(root,engine.name+'-'+theme+'.png')});
 await page.locator('#mpNodePickerChoice-primary').focus();await page.keyboard.press('ArrowDown');
 check(theme+' keyboard focus visible',await page.locator('#mpNodePickerChoice-backup').evaluate(el=>document.activeElement===el&&getComputedStyle(el).outlineStyle!=='none'));
 await page.keyboard.press('Escape');check(theme+' close works',!await page.locator('#modalMask').isVisible());await page.close();
}check('no page errors',errors.length===0);}finally{await browser.close();await fs.writeFile(path.join(root,engine.name+'.json'),JSON.stringify({engine:engine.name,scope:'prototype CSS only; no external requests',checks,errors},null,2));}
console.log(engine.name,checks.length,'checks passed');
