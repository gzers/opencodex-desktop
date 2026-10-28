const assert=require('node:assert/strict');
const labels={none:'无',minimal:'最低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高'};
function selectRow(h,id){
 const [channel,...parts]=id.split('|'),model=parts.join('|');
 const key=channel==='native'?JSON.stringify(['native',model]):JSON.stringify(['channel',channel,model]);
 const row=[...h.document.querySelectorAll('#mpkModels .mpk-model-name')].find(el=>el.dataset.key===key);
 assert.ok(row,'模型目录缺少 '+id);assert.equal(row.disabled,false);row.click();
}
function editOptions(h,effort,fast){
 if(h.document.querySelector('#mpkEffortPopover').hidden){const id=h.w.modelPickerPrototype.state().pending.at(-1),row=[...h.document.querySelectorAll('#mpkModels .mpk-row')].find(row=>row.dataset.key===id);assert.ok(row);row.querySelector('.mpk-effort-trigger').click();}
 if(effort==='model-default')h.click('[data-effort-reset]');
 else if(effort!==undefined){const b=[...h.document.querySelectorAll('[data-effort-step]')].find(b=>b.getAttribute('aria-label')==='选择推理 '+labels[effort]);assert.ok(b,'缺少推理档位 '+effort);b.click();}
 if(typeof fast==='boolean'){const b=h.document.querySelector('[data-effort-fast]');assert.ok(b);if((b.getAttribute('aria-checked')==='true')!==fast)b.click();}
}
function pickModel(h,scope,field,id,effort,fast){
 h.click(scope+' [data-cv-picker="'+field+'"]');assert.equal(h.w.modelPickerPrototype.state().businessIntegration,true);
 selectRow(h,id);editOptions(h,effort,fast);h.click('#modalConfirm');
}
module.exports={pickModel,selectRow,editOptions};
