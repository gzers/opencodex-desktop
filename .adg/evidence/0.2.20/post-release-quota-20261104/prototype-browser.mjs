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
  await page.goto('http://127.0.0.1:53944/index.html#models');await page.waitForSelector('.mp-provider-table');
  if(vp.width===1180)await page.evaluate(()=>document.documentElement.dataset.theme='dark');
  const prefix=vp.width+'-';
  check(prefix+'quota column',!(await page.locator('.mp-provider-heading').innerText()).includes('服务节点')&&(await page.locator('.mp-provider-heading').innerText()).includes('额度'));
  check(prefix+'old list node selector removed',await page.locator('.mp-provider-node-cell,.mp-node-picker-trigger').count()===0);
  check(prefix+'detection is two-character action',await page.locator('#mpChannelNode-newapi').innerText()==='检测');
  check(prefix+'row columns do not overlap',await page.locator('.mp-provider').first().evaluate(row=>{const xs=[...row.children].map(x=>x.getBoundingClientRect());return xs.every((r,i)=>i===0||r.left>=xs[i-1].right-1);}));
  await page.screenshot({path:path.join(out,prefix+'list.png')});
  await page.locator('#mpChannelNode-newapi').click();await page.locator('#mpNodePicker').waitFor();
  check(prefix+'same node test window',await page.locator('#mpNodePicker [data-speed-control]').count()===3);
  check(prefix+'test window fits product',await page.locator('#mpNodePicker').evaluate(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth+1&&r.top>=0&&r.bottom<=innerHeight+1;}));
  await page.screenshot({path:path.join(out,prefix+'detection.png')});await page.keyboard.press('Escape');
  check(prefix+'Escape restores action focus',await page.locator('#mpChannelNode-newapi').evaluate(el=>document.activeElement===el));
  await page.locator('[data-mp=edit-provider][data-id=newapi]').click();await page.locator('[data-mp=channel-section][data-id=quota]').click();
  const section=page.locator('#mpChannelSection-quota');await section.scrollIntoViewIfNeeded();
  check(prefix+'independent config default empty',await section.locator('[data-quota-config=url]').inputValue()==='');
  await page.screenshot({path:path.join(out,prefix+'quota-config.png')});
  const connection=await page.evaluate(()=>JSON.stringify(window.modelPrototype.state().draft.providers[0].connection));
  await section.locator('[data-quota-config=url]').fill('https://balance.example.test/quota');
  await section.locator('[data-quota-config=key]').fill('mock-independent-quota-key');
  await section.locator('[data-quota-config=remainingPath]').locator('..').isVisible().catch(()=>false);
  await section.locator('details summary').click();await section.locator('[data-quota-config=remainingPath]').fill('data.balance');
  await section.locator('[data-mp=quota-refresh]').click();await section.locator('[data-quota-state=success]').waitFor();
  check(prefix+'query does not change model connection',connection===await page.evaluate(()=>JSON.stringify(window.modelPrototype.state().draft.providers[0].connection)));
  check(prefix+'raw mock secret absent from public state',!JSON.stringify(await page.evaluate(()=>window.modelPrototype.state())).includes('mock-independent-quota-key'));
  await section.locator('[data-quota-config=url]').fill('https://fail.example.test/quota');
  await section.locator('[data-mp=quota-refresh]').click();await section.locator('[data-quota-state=failed]').waitFor();
  check(prefix+'failure distinct from zero',await section.locator('[data-quota-state=failed]').innerText()==='查询失败');
  await page.screenshot({path:path.join(out,prefix+'quota-failure.png')});
  await page.close();
 }
 check('no uncaught browser errors',errors.length===0);
}finally{await browser.close();await fs.writeFile(path.join(out,'result.json'),JSON.stringify({engine:engine.name,scope:'mock-only prototype, external requests blocked',checks,errors},null,2));}
console.log(engine.name,checks.length,'checks passed');
