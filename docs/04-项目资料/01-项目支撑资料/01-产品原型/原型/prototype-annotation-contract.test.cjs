/* 原型专属说明/独立注释层的源码护栏；真正像素关系需实屏运行右侧检查。 */
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const read=name=>fs.readFileSync(path.join(__dirname,name),'utf8');
test('共享弹窗不再向正文添加原型脚注或注释占位，说明只走共享注释参数',()=>{
 const html=read('index.html'),code=html.slice(html.indexOf('function openModal('),html.indexOf('\nfunction closeModal()'));
 assert.match(code,/modalBody.innerHTML=body;/);assert.doesNotMatch(code,/modalBody.innerHTML=body\+|modal-prototype-note/);
 assert.match(code,/prototypeAnnotation\?\.blocks/);assert.match(code,/annotationHost.innerHTML=modalNote/);
 assert.match(html,/<div id="modalAnnotationHost" class="modal-note-anchor" data-prototype-layer="modal" hidden>/);
 const model=read('model-config.js'),dialog=model.slice(model.indexOf(' function externalAgentDialog('),model.indexOf(' function buildExternalPreview('));
 assert.doesNotMatch(dialog,/<(?:p|div)[^>]+id="(?:mpCopyStatus|mpExternalSceneHint|mpExternalError)"/);
 assert.doesNotMatch(dialog,/mpAnn\(/);assert.match(dialog,/prototypeAnnotation:/);
 for(const id of ['mpCopyStatus','mpExternalSceneHint','mpExternalError'])assert.match(dialog,new RegExp("id:'"+id+"'"));
});
test('所有原型CSS禁止标记改回static/relative，专用宿主不参与布局',()=>{
 for(const file of fs.readdirSync(__dirname).filter(f=>f.endsWith('.css'))){
  for(const match of read(file).matchAll(/([^{}]+)\{([^{}]+)\}/g)){
   const selectors=match[1].split(',').map(s=>s.trim()),position=/(?:^|;)\s*position\s*:\s*([a-z]+)/.exec(match[2])?.[1];
   if(position&&selectors.some(s=>/\.proto-annotation(?:\[[^\]]+\])?$/.test(s)))assert.ok(['absolute','fixed'].includes(position),file+': '+match[1]);
  }
 }
 const css=read('annotation-cards.css');assert.match(css,/#modalAnnotationHost\{position:absolute;[^}]*width:0;height:0/);assert.doesNotMatch(css,/#modalBody>\.modal-note-anchor\{position:sticky/);
 assert.match(read('annotation-cards.js'),/function audit\(/);assert.match(read('annotation-cards.js'),/annotation-layout-check/);
});
