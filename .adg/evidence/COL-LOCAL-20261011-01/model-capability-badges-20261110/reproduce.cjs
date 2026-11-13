const {chromium}=require('/Users/ezio/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core');const assert=require('node:assert/strict'),fs=require('node:fs');
const root='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main',out=root+'/.adg/evidence/COL-LOCAL-20261011-01/model-capability-badges-20261110';fs.mkdirSync(out,{recursive:true});
const url='http://127.0.0.1:57285/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html?design=conversations&channel=krill-demo&review=model-badges5#models';
const checks=[],errors=[];let browser,page;const check=(id,v)=>{assert.ok(v,id);checks.push(id);};
(async()=>{browser=await chromium.launch({executablePath:'/Users/ezio/Library/Caches/ms-playwright/chromium_headless_shell-1248/chrome-headless-shell-mac-arm64/chrome-headless-shell'});
for(const [w,h,theme] of [[1180,760,'light'],[900,600,'dark']]){
 const tag=w+'-'+theme;page=await browser.newPage({viewport:{width:1600,height:1000}});page.on('pageerror',e=>errors.push(String(e)));await page.route('**/*',r=>{const u=r.request().url();return u.startsWith('http://127.0.0.1:57285/')||u.startsWith('data:')?r.continue():r.abort();});
 await page.goto(url);await page.waitForFunction(()=>document.querySelector('.cc-model-capabilities'));await page.evaluate(({w,h,theme})=>{setWinSize(w,h);document.documentElement.dataset.theme=theme;},{w,h,theme});await page.locator('[data-mp=channel-section][data-id=models]').click();await page.waitForTimeout(400);
 const rows=page.locator('#ccModelPanel-list .mp-model-row');
 check(tag+'-three-models',await rows.count()===3);
 const sol=rows.nth(0),terra=rows.nth(2);
 check(tag+'-sol-supported-only',JSON.stringify(await sol.locator('.cc-model-capability').allTextContents())===JSON.stringify(['Fast','Ultrafast']));
 check(tag+'-terra-unsupported-omitted',JSON.stringify(await terra.locator('.cc-model-capability').allTextContents())===JSON.stringify(['Fast']));
 check(tag+'-no-unknown-compact-badges',await rows.locator('[data-supported-capability=compact]').count()===0);
 check(tag+'-badges-in-first-identity-cell',await sol.locator('.cc-model-capabilities').evaluate(el=>el.parentElement.matches('.mp-model-identity > strong')));
 check(tag+'-alias-remains-second-grid-child',await sol.locator('.mp-model-identity').evaluate(el=>el.children[1].matches('.mp-model-alias')));
 check(tag+'-badge-text-has-no-status-repetition',!(await rows.locator('.cc-model-capabilities').allTextContents()).some(t=>/支持|待核对|不支持/.test(t)));
 check(tag+'-no-extra-identity-column',await sol.locator('.mp-model-identity').evaluate(el=>{const badge=el.querySelector('.cc-model-capabilities').getBoundingClientRect(),name=el.querySelector('strong').getBoundingClientRect(),alias=el.querySelector('.mp-model-alias').getBoundingClientRect();return badge.left>=name.left-1&&badge.right<=name.right+1&&(name.right<=alias.left+1||alias.top>=name.bottom-1)&&el.scrollWidth<=el.clientWidth+1;}));
 await sol.evaluate(el=>el.scrollIntoView({block:'center'}));await page.waitForTimeout(100);await page.screenshot({path:out+'/'+tag+'.png'});
 await page.locator('#channelCapabilityDemoCard [data-scene=ready]').click();await page.locator('[data-mp=channel-section][data-id=models]').click();await page.waitForTimeout(350);
 check(tag+'-confirmed-compact-can-appear',await page.locator('#ccModelPanel-list [data-supported-capability=compact]').count()===1);
 await page.evaluate(()=>conversationPrototype.setScope('models'));await page.waitForTimeout(100);check(tag+'-other-scope-no-badges',await page.locator('.cc-model-capabilities').count()===0);await page.close();
}
check('no-page-errors',errors.length===0);fs.writeFileSync(out+'/browser-checks.json',JSON.stringify({status:'passed',checks,errors,url},null,2));console.log(JSON.stringify({status:'passed',checks:checks.length,errors:errors.length}));await browser.close();
})().catch(async e=>{console.error(e);fs.writeFileSync(out+'/failure.json',JSON.stringify({error:String(e),checks,errors},null,2));try{await page.screenshot({path:out+'/failure.png'})}catch{}await browser?.close();process.exit(1)});
