/* Shared interactive list/table renderer. Data and business changes stay with its caller. */
(function(root,factory){const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;else root.PrototypeInteractiveCollection=api;})(typeof window==='undefined'?globalThis:window,function(){
 'use strict';
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 function validate(rows){const ids=new Set();for(const r of rows){if(!r.id||ids.has(r.id))throw Error('交互集合需要唯一标识');ids.add(r.id);}}
 function mount(host,initial){
  let config={kind:'list',rows:[],current:null,selection:true,...initial},disposed=false;const doc=host.ownerDocument;validate(config.rows);
  function row(id){return config.rows.find(r=>r.id===id);}
  function check(r){return config.selection?'<input type="checkbox" class="ic-check" data-ic-check="'+esc(r.id)+'" aria-label="'+esc((config.selectionLabel||'选择')+' '+r.label)+'" '+(r.selected===true?'checked':'')+' '+(r.disabled?'disabled':'')+' aria-checked="'+(r.selected==='mixed'?'mixed':r.selected===true)+'">':'';}
  function leadingIcon(r){if(!r.icon)return '';return '<span class="ic-leading-icon" aria-hidden="true">'+(r.icon.symbol?'<svg viewBox="0 0 24 24" focusable="false"><use href="#'+esc(r.icon.symbol)+'"/></svg>':'<span class="ic-icon-fallback">'+esc(r.icon.text||r.label.slice(0,2))+'</span>')+'</span>';}
  function description(r){if(!r.description)return '';return r.descriptionAction?'<button type="button" class="ic-description ic-path" data-ic-action="'+esc(r.descriptionAction)+'" data-id="'+esc(r.id)+'" aria-label="'+esc('打开路径：'+r.description)+'" title="'+esc(r.description)+'">'+esc(r.description)+'</button>':'<span class="ic-description" title="'+esc(r.description)+'">'+esc(r.description)+'</span>';}
  function name(r){return '<div class="ic-name"'+(r.icon?' data-icon="true"':'')+'>'+leadingIcon(r)+'<button type="button" class="ic-title" data-ic-'+(config.onDetail?'detail':'activate')+'="'+esc(r.id)+'" title="'+esc(r.label)+'" '+(r.detailDisabled?'disabled aria-disabled="true"':'')+'>'+esc(r.label)+'</button>'+description(r)+(config.kind==='table'&&!config.pathColumn?'<span class="ic-inline-state" data-tone="'+esc(r.tone||'neutral')+'">'+stateContent(r)+'</span>':'')+'</div>';}
  function actions(r){return '<div class="ic-actions">'+(r.actions||[]).map(a=>'<button type="button" class="btn ghost btn-sm ic-action '+(a.icon?'ic-icon-action':'')+'" data-ic-action="'+esc(a.id)+'" data-id="'+esc(r.id)+'" '+(a.kind==='toggle'?'aria-pressed="'+(r.selected==='mixed'?'mixed':r.selected===true)+'" ':'')+'aria-label="'+esc(a.label+' · '+r.label)+'" title="'+esc(a.reason||a.label)+'" '+(a.disabled?'disabled':'')+'>'+esc(a.icon||a.label)+'</button>').join('')+'</div>';}
  function attrs(r){return 'data-ic-row="'+esc(r.id)+'"'+(r.id===config.current?' aria-current="true"':'')+(config.onActivate?' tabindex="0"':'')+(r.disabled?' data-disabled="true"':'')+(r.pending?' data-pending="true"':'')+(r.unavailable?' data-unavailable="true"':'');}
  function stateContent(r){return r.statusAction?'<button type="button" class="btn ghost btn-sm ic-status-action" data-ic-action="'+esc(r.statusAction)+'" data-id="'+esc(r.id)+'" aria-label="'+esc((r.status||'')+' · '+r.label)+'" title="'+esc(r.statusReason||r.status||'')+'" '+(r.statusDisabled?'disabled':'')+'>'+esc(r.status||'')+'</button>':esc(r.status||'');}
  function status(r){return '<span class="ic-state" data-tone="'+esc(r.tone||'neutral')+'">'+stateContent(r)+'</span>';}
  function render(){
   if(disposed)return;const active=doc.activeElement;let focus=null;
   if(host.contains(active)){const attr=['data-ic-check','data-ic-detail','data-ic-action','data-ic-activate','data-ic-row'].find(k=>active.hasAttribute(k));if(attr)focus={attr,value:active.getAttribute(attr),id:active.dataset.id};}
   const scroll=host.scrollTop;host.classList.add('ic-collection');host.dataset.kind=config.kind;host.dataset.selection=String(config.selection);host.dataset.pathColumn=String(!!config.pathColumn);
   host.setAttribute('aria-busy',String(config.status==='loading'));
   const message=config.status==='loading'?'正在读取…':config.status==='failed'?'读取失败，已有选择保留。':!config.rows.length?(config.emptyLabel||'没有匹配内容。'):'';
   const rows=config.rows;
   const empty=message?'<div class="ic-message" role="'+(config.status==='failed'?'alert':'status')+'">'+message+(config.status==='failed'?'<button type="button" class="btn ghost btn-sm" data-ic-retry>重试</button>':'')+'</div>':'';

   if(config.kind==='table'){
    const pathCol=config.pathColumn?'<col class="ic-path-col">':'',pathHead=config.pathColumn?'<th scope="col">'+esc(config.pathLabel||'原始路径')+'</th>':'';
    const pathCell=r=>config.pathColumn?'<td class="ic-path-cell">'+(r.path?description({...r,description:r.path,descriptionAction:r.pathAction}):'<span class="ic-description">—</span>')+'</td>':'';
    host.innerHTML='<table class="ic-table" aria-label="'+esc(config.label)+'"><colgroup>'+(config.selection?'<col class="ic-check-col">':'')+'<col>'+pathCol+'<col class="ic-status-col"><col class="ic-action-col"></colgroup><thead><tr>'+(config.selection?'<th scope="col">'+(rows.some(r=>!r.disabled)?'<input type="checkbox" class="ic-check" data-ic-all aria-label="'+esc(config.selectionAllLabel||'选择当前结果的全部可用项')+'">':'')+'</th>':'')+'<th scope="col">'+esc(config.nameLabel||'名称')+'</th>'+pathHead+'<th scope="col" class="ic-status-cell">'+esc(config.statusLabel||'状态')+'</th><th scope="col">操作</th></tr></thead><tbody>'+(!message?rows.map(r=>'<tr '+attrs(r)+'>'+(config.selection?'<td>'+check(r)+'</td>':'')+'<th scope="row">'+name(r)+'</th>'+pathCell(r)+'<td class="ic-status-cell">'+status(r)+'</td><td>'+actions(r)+'</td></tr>').join(''):'<tr><td colspan="'+((config.selection?4:3)+(config.pathColumn?1:0))+'">'+empty+'</td></tr>')+'</tbody></table>';
   }else host.innerHTML=message?empty:'<ul class="ic-list" aria-label="'+esc(config.label)+'">'+rows.map(r=>'<li class="ic-list-row" '+attrs(r)+'>'+check(r)+name(r)+(r.status?status(r):'')+actions(r)+'</li>').join('')+'</ul>';
   host.querySelectorAll('[data-ic-check]').forEach(el=>el.indeterminate=row(el.dataset.icCheck)?.selected==='mixed');
   const all=host.querySelector('[data-ic-all]');if(all){const allowed=rows.filter(r=>!r.disabled);all.checked=allowed.length>0&&allowed.every(r=>r.selected===true);all.indeterminate=allowed.some(r=>r.selected===true||r.selected==='mixed')&&!all.checked;}
   host.scrollTop=scroll;
   if(focus){const el=[...host.querySelectorAll('['+focus.attr+']')].find(e=>e.getAttribute(focus.attr)===focus.value&&(!focus.id||e.dataset.id===focus.id));el?.focus({preventScroll:true});}
  }
  function click(e){
   if(e.target.closest('[data-ic-retry]')){config.onRetry?.();return;}
   const a=e.target.closest('[data-ic-action]');if(a){config.onAction?.(row(a.dataset.id),a.dataset.icAction,a);return;}
   const d=e.target.closest('[data-ic-detail]');if(d){config.onDetail?.(row(d.dataset.icDetail),d);return;}
   if(e.target.closest('input,button,a,select,textarea,[contenteditable]')&&!e.target.closest('[data-ic-activate]'))return;
   const el=e.target.closest('[data-ic-row]'),r=el&&row(el.dataset.icRow);if(r&&!r.disabled)config.onActivate?.(r);
  }
  function change(e){if(e.target.matches('[data-ic-all]')){config.onSelectMany?.(config.rows.filter(r=>!r.disabled).map(r=>r.id),e.target.checked);return;}const id=e.target.dataset.icCheck;if(id)config.onSelect?.(row(id),e.target.checked);}
  function key(e){
   if(!e.target.matches('[data-ic-row]'))return;const r=row(e.target.dataset.icRow);
   if(['Enter',' '].includes(e.key)){e.preventDefault();if(!r.disabled)config.onActivate?.(r);return;}
   const elements=[...host.querySelectorAll('[data-ic-row]')].filter(el=>el.dataset.disabled!=='true'),i=elements.indexOf(e.target);
   if(!['ArrowDown','ArrowUp','Home','End'].includes(e.key))return;e.preventDefault();const j=e.key==='Home'?0:e.key==='End'?elements.length-1:Math.max(0,Math.min(elements.length-1,i+(e.key==='ArrowDown'?1:-1)));const next=elements[j];next?.focus({preventScroll:true});if(next){const bounds=host.getBoundingClientRect(),box=next.getBoundingClientRect();if(box.top<bounds.top)host.scrollTop-=bounds.top-box.top;else if(box.bottom>bounds.bottom)host.scrollTop+=box.bottom-bounds.bottom;}
  }
  host.addEventListener('click',click);host.addEventListener('change',change);host.addEventListener('keydown',key);render();
  return {update(next){if(next.rows)validate(next.rows);config={...config,...next};render();},render,destroy(){disposed=true;host.removeEventListener('click',click);host.removeEventListener('change',change);host.removeEventListener('keydown',key);},state:()=>({kind:config.kind,current:config.current,rows:config.rows.map(r=>({...r}))})};
 }
 return {mount};
});
