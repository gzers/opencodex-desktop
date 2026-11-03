import fs from 'node:fs/promises'
import path from 'node:path'
import {pickEngine} from './engine.mjs'
const out=process.env.AUDIT_OUT;await fs.mkdir(out,{recursive:true})
const e=await pickEngine(),b=await e.launch(),rows=[]
try{
 const p=await b.newPage({viewport:{width:1180,height:760}})
 await p.goto('http://127.0.0.1:52422/?__audit=1&route=models&env=ready&runtime=running&effects=low');await p.locator('.channel-table').waitFor()
 await p.evaluate(async()=>{(await import('/src/stores/routes.ts')).useRouteStore().go('settings',{section:'upgrade'})})
 await p.getByRole('button',{name:'查看管理器更新',exact:true}).waitFor();rows.push({state:'settings',text:await p.locator('body').innerText()})
 await p.getByRole('button',{name:'查看管理器更新',exact:true}).click();await p.getByTestId('update-center').waitFor();rows.push({state:'manager',text:await p.getByTestId('update-center').innerText()});await p.screenshot({path:path.join(out,'manager.png')});await p.close()
 const q=await b.newPage({viewport:{width:1180,height:760}})
 await q.goto('http://127.0.0.1:52429/'+encodeURI('docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html')+'#models');await q.waitForFunction(()=>!!window.maintenancePrototype&&!!document.querySelector('[data-mp=tab][data-tab=templates]'))
 await q.locator('[data-mp=tab][data-tab=templates]').click();rows.push({state:'prototype templates',buttons:await q.locator('#route-models button').evaluateAll(xs=>xs.map(x=>({text:x.textContent,action:x.dataset.mp,label:x.getAttribute('aria-label')})))})
 await q.locator('#route-models').getByRole('button',{name:'编辑',exact:true}).first().click();await q.locator('#mpTemplateAgentAction').click();await q.locator('#mpAssistantPanel').waitFor();rows.push({state:'prototype assistant',text:await q.locator('#mpAssistantPanel').innerText()});await q.screenshot({path:path.join(out,'prototype-agent.png')})
 await q.evaluate(()=>window.maintenancePrototype.updates());await q.locator('#modalMask').waitFor({state:'visible'});rows.push({state:'prototype updates',text:await q.locator('#modal').innerText()});await q.close()
}finally{await b.close();await fs.writeFile(path.join(out,'probe.json'),JSON.stringify(rows,null,2)+'\n')}
