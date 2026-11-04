import fs from 'node:fs/promises';
import path from 'node:path';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs';
const engine=await pickEngine(),out=path.join(path.dirname(new URL(import.meta.url).pathname),engine.name);await fs.mkdir(out,{recursive:true});
const checks=[],errors=[],browser=await engine.launch();const check=(name,value,detail)=>{checks.push({name,passed:!!value,detail});if(!value)throw Error(name);};
try{for(const width of [1440,1180,900]){
 const page=await browser.newPage({viewport:{width,height:width===900?650:900}});page.on('pageerror',e=>errors.push(e.message));await page.route('**/*',r=>new URL(r.request().url()).hostname==='127.0.0.1'?r.continue():r.abort());await page.goto('http://127.0.0.1:53947/index.html#models');await page.waitForSelector('.mp-provider');
 if(width===1180)await page.evaluate(()=>document.documentElement.dataset.theme='dark');
 const before=await page.evaluate(()=>JSON.stringify({draft:window.modelPrototype.state().draft,saved:window.modelPrototype.state().saved}));
 const trigger=page.locator('#mpChannelNode-newapi'),mask=page.locator('#modalMask'),sheet=page.locator('#modal');await trigger.click();await sheet.getByRole('button',{name:'关闭',exact:true}).waitFor();
 check(width+'shared modal host',await page.locator('#modalBody #mpNodePicker').count()===1&&await sheet.getAttribute('aria-modal')==='true'&&await sheet.getAttribute('role')==='dialog');
 check(width+'no anchored popover',await page.locator('#mpNodePicker').evaluate(el=>!el.hasAttribute('popover')&&!el.style.left&&!el.style.top));
 const box=await sheet.evaluate(el=>{const r=el.getBoundingClientRect(),m=document.getElementById('modalMask').getBoundingClientRect();return {dx:(r.left+r.right-m.left-m.right)/2,dy:(r.top+r.bottom-m.top-m.bottom)/2,left:r.left,right:r.right,top:r.top,bottom:r.bottom,w:innerWidth,h:innerHeight};});
 check(width+'centered on product',Math.abs(box.dx)<1&&Math.abs(box.dy)<1,box);check(width+'dialog fits viewport',box.left>=0&&box.right<=box.w+1&&box.top>=0&&box.bottom<=box.h+1);
 await page.screenshot({path:path.join(out,width+'-centered.png')});
 await page.locator('#mpNodePickerTest-backup').focus();await page.keyboard.press('Enter');await page.locator('#mpNodePickerSpeed-backup [data-speed-status=success]').waitFor();
 check(width+'measurement retains dialog and focus',await mask.isVisible()&&await page.locator('#mpNodePickerTest-backup').evaluate(el=>document.activeElement===el));
 check(width+'measurement preserves config',before===await page.evaluate(()=>JSON.stringify({draft:window.modelPrototype.state().draft,saved:window.modelPrototype.state().saved})));
 await page.locator('#mpNodePickerAll').click();await page.locator('#mpNodePickerAll[data-mp=node-speed-stop]').click();check(width+'stop remains in dialog',await mask.isVisible()&&await page.locator('#mpNodePickerAll').innerText()==='一键检测');
 await page.locator('#modalCancel').focus();await page.keyboard.press('Tab');check(width+'Tab wraps within modal',await page.locator('#mpNodePickerAll').evaluate(el=>document.activeElement===el));await page.keyboard.press('Shift+Tab');check(width+'ShiftTab wraps within modal',await page.locator('#modalCancel').evaluate(el=>document.activeElement===el));
 await page.evaluate(()=>document.querySelector('.main').scrollTop=100);check(width+'scroll does not dismiss',await mask.isVisible());
 await page.keyboard.press('Escape');check(width+'Escape dismisses and restores focus',!await mask.isVisible()&&await trigger.evaluate(el=>document.activeElement===el));check(width+'modal attrs cleaned',await sheet.getAttribute('aria-modal')===null);
 await trigger.click();await mask.waitFor();await page.locator('#modalCancel').click();check(width+'shared close button restores trigger',!await mask.isVisible()&&await trigger.evaluate(el=>document.activeElement===el));
 await trigger.click();await mask.waitFor();await page.locator('#mpNodePickerChoice-backup').click();check(width+'selection remains draft-only',!await mask.isVisible()&&await page.evaluate(()=>{const x=window.modelPrototype.state();return x.draft.providers[0].activeNodeId==='backup'&&x.saved.providers[0].activeNodeId==='primary';}));
 await trigger.click();await mask.waitFor();const m=await mask.boundingBox();await mask.click({position:{x:Math.max(8,-m.x+8),y:Math.max(8,-m.y+8)}});check(width+'shared mask dismisses',!await mask.isVisible());
 await page.close();
}check('no browser errors',errors.length===0);}finally{await browser.close();await fs.writeFile(path.join(out,'result.json'),JSON.stringify({engine:engine.name,scope:'mock-only prototype; shared modal, no external requests',checks,errors},null,2));}
console.log(engine.name,checks.length,'checks passed');
