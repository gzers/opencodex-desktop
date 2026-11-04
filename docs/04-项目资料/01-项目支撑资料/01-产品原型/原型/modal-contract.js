/* 公共模态退出合同：仅用于原型内存交互，不执行外部操作。 */
(() => {
 'use strict';
 const esc=value=>String(value??'').replace(/[&<>"']/g,char=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
 const exitLabels=new Set(['关闭','关闭窗口','知道了','返回编辑','完成']);
 function resolve(confirmText,hasHandler,options={}){
  const primary=!!hasHandler&&!options.hideConfirm&&!options.closeOnly;
  const mode=options.mode||(!primary||exitLabels.has(options.cancelLabel)?'view':'confirm');
  return {mode,primary,cancel:mode!=='view',cancelLabel:exitLabels.has(options.cancelLabel)?'取消':options.cancelLabel||'取消'};
 }
 function closeButton(attributes='',label='关闭'){
  return '<button type="button" class="icon-btn modal-close" aria-label="'+esc(label)+'" title="'+esc(label)+'" '+attributes+'><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18"/></svg></button>';
 }
 const contract=Object.freeze({resolve,closeButton});
 if(typeof window!=='undefined')window.PrototypeModal=contract;
 if(typeof module!=='undefined'&&module.exports)module.exports=contract;
})();
