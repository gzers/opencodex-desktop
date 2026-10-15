const fs=require('node:fs'),vm=require('node:vm'),path=require('node:path'),assert=require('node:assert/strict');
const harnessFile=path.resolve(__dirname,'../../../03-项目协作资料/01-协作包/2026-10-04-模型配置管理与同步/01-需求分析/05-原型/qa/template-workflow.source.mjs');
// 复用既有最小 DOM 的交互能力，不执行它的历史断言，不代表浏览器布局。
function make(){
 const legacy=fs.readFileSync(harnessFile,'utf8');
 let source=legacy.slice(legacy.indexOf('function make(){'),legacy.indexOf('const results=[];'));
 source=source.replace('headerHeight=72,mainPadding=0','headerHeight=72,tabBarHeight=40,mainPadding=0');
 source=source.replace(' getBoundingClientRect(){',' getBoundingClientRect(){if(this.id===\'mpTabsBar\'){const top=Math.max(100,180-nodes.get(\'mpMain\').scrollTop);return {top,bottom:top+tabBarHeight,height:tabBarHeight};}');
 source=source.replace("const document={activeElement:null,","const document={body:new Node('body'),createElement:()=>new Node(),activeElement:null,");
 source=source.replace(" select(){this.selected=true;}"," appendChild(child){nodes.set(child.id,child);child.parent=this;} remove(){nodes.delete(this.id);} select(){this.selected=true;}");
 source=source.replace('\\bid=','\\sid=');
 source=source.replace("['modalBody','mpContent','mpAssistantHost'].includes(this.id)","['modalBody','mpContent','mpAssistantHost','mpNodePicker'].includes(this.id)");
 source=source.replace('nodes.set(node.id,node);bodyOwner.get(this.id).push(node.id);','node.parent=this;nodes.set(node.id,node);bodyOwner.get(this.id).push(node.id);');
 source=source.replace(' querySelector(q){',' contains(node){for(let n=node;n;n=n.parent)if(n===this)return true;return false;} querySelector(q){if(this.id===\'mpNodePicker\'&&q.startsWith(\'[data-mp="node-picker-choose"]\'))return (bodyOwner.get(this.id)||[]).map(id=>nodes.get(id)).find(n=>n?.dataset.mp===\'node-picker-choose\'&&(!q.includes(\'aria-pressed\')||n.getAttribute(\'aria-pressed\')===\'true\'))||null;');
 source=source.replace(' querySelectorAll(q){',' querySelectorAll(q){if(this.id===\'mpNodePicker\')return (bodyOwner.get(this.id)||[]).map(id=>nodes.get(id)).filter(n=>n&&(q===\'[data-speed-control]\'?n.dataset.speedControl:q.includes(\'node-picker-choose\')?n.dataset.mp===\'node-picker-choose\'&&!n.disabled:false));');

 source=source.replace("vm.createContext(context);vm.runInContext(fs.readFileSync(source,'utf8'),context,{filename:source});","vm.createContext(context);for(const file of ['component-views.js','tree-table.js','model-template-updates.js','model-template-seeds.js','model-template-summary.js'])vm.runInContext(fs.readFileSync(path.join(path.dirname(source),file),'utf8'),context);vm.runInContext(fs.readFileSync(source,'utf8'),context,{filename:source});");
 source=source.replace('return {nodes,context,state,page,dialog,click,','return {resizeTabs:height=>{tabBarHeight=height;listeners[\'window:resize\']?.();},key:(key,target)=>listeners.keydown({key,target,preventDefault(){},stopImmediatePropagation(){}}),pointer:target=>listeners.pointerdown({target}),focusTarget:target=>listeners.focusin({target}),clickTarget:target=>listeners.click({target}),nodes,context,state,page,dialog,click,');
 source=source.replace("for(const file of ['tree-table.js','model-template-updates.js','model-template-seeds.js'])","for(const file of ['component-views.js','tree-table.js','model-template-updates.js','model-template-seeds.js','model-template-summary.js'])");
 source=source.replace("const startExternal=(range='current')=>{click('external-agent');", "const startExternal=(range='current')=>{click('external-new');");
 source=source.replace("nodes.get('mpExternalScope').value=range;runConfirm();", "nodes.get('mpExternalScope').value=range;nodes.get('mpExternalScene').value=range==='create'?'docs':'update';context.window.modelPrototype.updateExternalPreview();runConfirm();");
 source=source.replace("nodes.get('modalBody').innerHTML=body;", "nodes.get('modalBody').innerHTML=body+((opts?.prototypeAnnotation?.blocks||[]).length?'<div class=\"annotation-source\" hidden>'+opts.prototypeAnnotation.blocks.map(b=>'<span id=\"'+b.id+'\">'+(b.text||'')+'</span>').join('')+'</div>':'');");
 source=source.replace(" select(){this.selected=true;}", " removeAttribute(key){this.attributes.delete(key);} select(){this.selected=true;}");
 const scope={fs,vm,assert,path,console,URL,JSON,Date,Set,Object,String,Number,Array,source:path.join(__dirname,'model-config.js'),sectionNames:['basic','match','common','variants','sources']};
 vm.runInNewContext(source+'globalThis.create=make;',scope);return scope.create();
}
module.exports=make;
