import fs from 'node:fs/promises';
import path from 'node:path';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs';
const engine=await pickEngine(),root=path.dirname(new URL(import.meta.url).pathname),checks=[],errors=[];
const browser=await engine.launch();
const check=(name,passed,detail)=>{checks.push({name,passed:!!passed,detail});if(!passed)throw Error(name);};
try{for(const width of [1440,1180,900]){
 const page=await browser.newPage({viewport:{width,height:width===900?650:900}});
 page.on('pageerror',e=>errors.push(e.message));await page.route('**/*',r=>new URL(r.request().url()).hostname==='127.0.0.1'?r.continue():r.abort());
 await page.goto('http://127.0.0.1:53946/index.html#models');await page.waitForSelector('.mp-provider');
 const geometry=await page.locator('.mp-provider').first().evaluate(el=>({columns:[...el.children].map(x=>{const r=x.getBoundingClientRect();return {left:r.left,right:r.right,width:r.width};}),header:[...document.querySelector('.mp-provider-heading').children].map(x=>x.getBoundingClientRect().left)}));
 check(width+' channel bounded',geometry.columns[1].width<=220.5&&geometry.columns[1].width>=160,geometry.columns[1].width);
 check(width+' header aligned',geometry.columns.every((c,i)=>Math.abs(c.left-geometry.header[i])<1));
 check(width+' cells not overlapping',geometry.columns.every((c,i)=>!i||c.left>=geometry.columns[i-1].right));
 check(width+' all actions retained',await page.locator('.mp-provider').first().locator('.mp-provider-actions button').count()===6);
 await page.screenshot({path:path.join(root,engine.name+'-'+width+'.png')});
 await page.locator('#mpChannelNode-newapi').click();await page.locator('#mpNodePicker').waitFor();
 check(width+' detection reachable',await page.locator('#mpNodePicker').isVisible());await page.keyboard.press('Escape');await page.close();
}check('no browser errors',!errors.length);}finally{await browser.close();await fs.writeFile(path.join(root,engine.name+'.json'),JSON.stringify({scope:'prototype CSS adjustment; external requests blocked',engine:engine.name,checks,errors},null,2));}
console.log(engine.name,checks.length,'checks passed');
