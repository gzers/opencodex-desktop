import fs from 'node:fs/promises';
import path from 'node:path';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs';
const engine=await pickEngine(),out=path.join(path.dirname(new URL(import.meta.url).pathname),engine.name);await fs.mkdir(out,{recursive:true});
const browser=await engine.launch(),checks=[],errors=[];
function check(name,value){checks.push({name,passed:!!value});if(!value)throw Error(name);}
try{
 for(const vp of [{width:1440,height:900},{width:1180,height:760},{width:900,height:650}]){
  const page=await browser.newPage({viewport:vp});page.on('pageerror',e=>errors.push(e.message));
  await page.route('**/*',route=>new URL(route.request().url()).hostname==='127.0.0.1'?route.continue():route.abort());
  await page.goto('http://127.0.0.1:53945/index.html#models');await page.waitForSelector('.mp-provider-table');
  if(vp.width===1180)await page.evaluate(()=>document.documentElement.dataset.theme='dark');
  const prefix=vp.width+'-';
  check(prefix+'four named columns',JSON.stringify((await page.locator('.mp-provider-heading').innerText()).split('\n').filter(Boolean))===JSON.stringify(['渠道','模型','启用','操作']));
  check(prefix+'no quota or old node column',await page.locator('.mp-provider-quota-cell,.mp-provider-node-cell,[data-quota-config],[data-mp=quota-refresh]').count()===0);
  check(prefix+'detection remains in operations',await page.locator('.mp-provider-actions #mpChannelNode-newapi').innerText()==='检测');
  check(prefix+'columns do not overlap',await page.locator('.mp-provider').first().evaluate(row=>{const xs=[...row.children].map(x=>x.getBoundingClientRect());return xs.length===5&&xs.every((r,i)=>i===0||r.left>=xs[i-1].right-1);}));
  await page.screenshot({path:path.join(out,prefix+'list.png')});
  await page.locator('#mpChannelNode-newapi').click();await page.locator('#mpNodePicker').waitFor();
  check(prefix+'same node detection window',await page.locator('#mpNodePicker [data-speed-control]').count()===3);
  check(prefix+'detection fits viewport',await page.locator('#mpNodePicker').evaluate(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth+1&&r.top>=0&&r.bottom<=innerHeight+1;}));
  if(vp.width===900)await page.screenshot({path:path.join(out,prefix+'detection.png')});
  await page.keyboard.press('Escape');
  check(prefix+'Escape restores detection focus',await page.locator('#mpChannelNode-newapi').evaluate(el=>document.activeElement===el));
  await page.locator('[data-mp=edit-provider][data-id=newapi]').click();
  const nav=await page.locator('[data-mp=channel-section]').allTextContents();
  check(prefix+'four detail sections no quota',nav.length===4&&!nav.some(x=>x.includes('额度'))&&await page.locator('#mpChannelSection-quota,[data-quota-config]').count()===0);
  check(prefix+'nodes retained',await page.locator('#mpChannelSection-nodes [data-mp=node-speed-one]').count()===2);
  if(vp.width===1440)await page.screenshot({path:path.join(out,prefix+'detail.png')});
  await page.close();
 }
 check('no uncaught browser errors',errors.length===0);
}finally{await browser.close();await fs.writeFile(path.join(out,'result.json'),JSON.stringify({engine:engine.name,scope:'active mock prototype; quota deferred and removed; all external requests blocked; WebDAV records only',checks,errors},null,2));}
console.log(engine.name,checks.length,'checks passed');
