/* 公共树形导航原型。选择、焦点和展开分别保存；不承担业务保存或网络读取。 */
(function(root,factory){
 const api=factory(typeof module==='object'&&module.exports?require('./tree-table.js'):root.PrototypeTreeTable);
 if(typeof module==='object'&&module.exports)module.exports=api;else root.PrototypeTreeNavigation=api;
})(typeof window==='undefined'?globalThis:window,function(treeTable){
 'use strict';
 const escape=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 let sequence=0;
 function create(options={}){
  let nodes=[],byId=new Map(),children=new Map(),query='',filter=null,filterReveals=false,selected=options.selected||null,active=selected;
  let selectedLabel='',expanded=new Set(options.expanded||[]),searchCollapsed=new Set();
  function setNodes(input){
   treeTable.visible(input,new Set());
   nodes=input.map(n=>({...n,id:String(n.id),parent:n.parent==null?null:String(n.parent)}));byId=new Map(nodes.map(n=>[n.id,n]));children=new Map();
   for(const n of nodes){const list=children.get(n.parent)||[];list.push(n);children.set(n.parent,list);}
   if(byId.has(selected))selectedLabel=byId.get(selected).label;
  }
  setNodes(options.nodes||[]);
  const parents=id=>{const result=[];let p=byId.get(id)?.parent;while(p){result.unshift(p);p=byId.get(p)?.parent;}return result;};
  function project(){
   const filtering=!!query.trim()||!!filterReveals,q=query.trim().toLocaleLowerCase(),included=new Set(),matches=new Set();
   const allowed=n=>!filter||filter(n);
   const descendants=id=>{for(const n of children.get(id)||[]){if(allowed(n))included.add(n.id);descendants(n.id);}};
   for(const n of nodes){if(!allowed(n))continue;if(!q||(n.label+' '+(n.description||'')+' '+(n.keywords||'')).toLocaleLowerCase().includes(q)){matches.add(n.id);included.add(n.id);if(q&&children.has(n.id))descendants(n.id);}}
   for(const id of [...included])for(const parent of parents(id))included.add(parent);
   const rows=[];
   function walk(parent=null,depth=0){
    const siblings=(children.get(parent)||[]).filter(n=>included.has(n.id));
    siblings.forEach((node,i)=>{const childCount=(children.get(node.id)||[]).filter(n=>included.has(n.id)).length,hasChildren=childCount>0;const open=hasChildren&&(filtering?!searchCollapsed.has(node.id):expanded.has(node.id));rows.push({node,id:node.id,depth,hasChildren,childCount,expanded:open,match:!!q&&matches.has(node.id),context:filtering&&!matches.has(node.id),position:i+1,size:siblings.length});if(open)walk(node.id,depth+1);});
   }
   walk();return {rows,filtering,matches:matches.size};
  }
  function visible(){return project().rows;}
  function focus(id){if(visible().some(n=>n.id===id))active=id;}
  function toggle(id,open){const view=project(),row=view.rows.find(n=>n.id===id);if(!row?.hasChildren)return false;const next=open??!row.expanded;if(view.filtering){next?searchCollapsed.delete(id):searchCollapsed.add(id);}else next?expanded.add(id):expanded.delete(id);if(!next&&parents(active).includes(id))active=id;return true;}
  function select(id){const n=byId.get(id);if(!n||n.disabled||n.selectable===false)return false;active=id;if(selected===id)return false;selected=id;selectedLabel=n.label;return true;}
  function reveal(){if(!byId.has(selected))return false;query='';filter=null;filterReveals=false;searchCollapsed.clear();for(const p of parents(selected))expanded.add(p);active=selected;return true;}
  function key(value){
   const rows=visible();if(!rows.length)return null;let i=rows.findIndex(n=>n.id===active);if(i<0)i=0;active=rows[i].id;const row=rows[i];
   if(value==='ArrowDown')active=rows[Math.min(i+1,rows.length-1)].id;
   else if(value==='ArrowUp')active=rows[Math.max(i-1,0)].id;
   else if(value==='Home')active=rows[0].id;else if(value==='End')active=rows.at(-1).id;
   else if(value==='ArrowRight'){if(row.hasChildren&&!row.expanded)toggle(row.id,true);else if(row.expanded&&rows[i+1])active=rows[i+1].id;}
   else if(value==='ArrowLeft'){if(row.expanded)toggle(row.id,false);else if(row.node.parent&&rows.some(n=>n.id===row.node.parent))active=row.node.parent;}
   else if(value==='Enter'||value===' '){if(row.node.selectable===false){toggle(row.id);return {type:'toggle',id:row.id};}return select(row.id)?{type:'select',id:row.id}:null;}
   return null;
  }
  function state(){const view=project();if(!view.rows.some(r=>r.id===active))active=view.rows.find(r=>r.id===selected)?.id||view.rows[0]?.id||null;return {nodes:nodes.map(n=>({...n})),query,selected,selectedLabel,active,expanded:[...expanded],filtered:view.filtering,matches:view.matches,rows:view.rows,selectionOutside:!!selected&&!view.rows.some(r=>r.id===selected),selectionMissing:!!selected&&!byId.has(selected)};}
  return {state,setNodes,parents,focus,toggle,select,reveal,key,node:id=>byId.get(id),setSelected(id){selected=id;if(byId.has(id))selectedLabel=byId.get(id).label;},setQuery(value){query=String(value);searchCollapsed.clear();},setFilter(value,settings={}){filter=typeof value==='function'?value:null;filterReveals=!!filter&&settings.reveal!==false;searchCollapsed.clear();},expandAll(){if(!project().filtering)for(const id of children.keys())if(id)expanded.add(id);searchCollapsed.clear();},collapseAll(){if(project().filtering){for(const id of children.keys())if(id)searchCollapsed.add(id);}else expanded.clear();}};
 }
 const icons={folder:'M3 5h6l2 2h10v12H3z',chat:'M3 4h18v12H9l-6 4z',agent:'M6 5h12v12H6z M10 2v3 M10 17v3 M2 10h4 M18 10h4 M9 10h.01 M15 10h.01',group:'M4 6h16 M4 12h16 M4 18h16'};
 function icon(kind){return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="'+(icons[kind]||icons.chat)+'"/></svg>';}
 function highlighted(text,query){text=String(text);const q=query.trim(),i=q?text.toLocaleLowerCase().indexOf(q.toLocaleLowerCase()):-1;return i<0?escape(text):escape(text.slice(0,i))+'<mark>'+escape(text.slice(i,i+q.length))+'</mark>'+escape(text.slice(i+q.length));}
 function mount(host,options={}){
  const doc=host.ownerDocument,store=create(options),id='tn-'+(++sequence);let density=options.density||'normal',status='ready',destroyed=false;
  host.classList.add('tn-widget');host.innerHTML='<div class="tn-tree" role="tree" tabindex="0" aria-label="'+escape(options.label||'对象树形导航')+'"></div><div class="tn-outside" role="status" hidden></div>';
  const tree=host.querySelector('.tn-tree'),outside=host.querySelector('.tn-outside');
  const rowId=value=>id+'-'+encodeURIComponent(value);
  function render(){
   if(destroyed)return;const s=store.state(),scroll=tree.scrollTop;host.dataset.density=density;tree.setAttribute('aria-busy',String(status==='loading'));
   if(status!=='ready'||!s.rows.length){
    const text=status==='loading'?'正在读取对象…':status==='failed'?'暂时无法读取对象':s.filtered?'没有匹配的对象':'暂无对象';
    tree.innerHTML='<div class="tn-empty" role="status"><span>'+icon(status==='failed'?'group':'folder')+'</span><strong>'+text+'</strong>'+(status==='failed'?'<button type="button" class="btn ghost btn-sm" data-tn-retry>重试</button>':s.filtered?'<button type="button" class="btn ghost btn-sm" data-tn-clear>清除筛选</button>':'')+'</div>';tree.removeAttribute('aria-activedescendant');
   }else{
    tree.innerHTML=s.rows.map(r=>{const n=r.node,picked=r.id===s.selected,focused=r.id===s.active;return '<div class="tn-row'+(picked?' is-selected':'')+(focused?' is-focused':'')+(r.context?' is-context':'')+'" role="treeitem" id="'+escape(rowId(r.id))+'" data-tn-id="'+escape(r.id)+'" aria-label="'+escape(n.label+(n.status?' · '+n.status:''))+'" aria-level="'+(r.depth+1)+'" aria-posinset="'+r.position+'" aria-setsize="'+r.size+'" '+(n.selectable===false?'':'aria-selected="'+picked+'"')+(r.hasChildren?' aria-expanded="'+r.expanded+'"':'')+(n.disabled?' aria-disabled="true"':'')+' style="--tn-level:'+r.depth+'"><span class="tn-arrow" '+(r.hasChildren?'data-tn-toggle aria-hidden="true"':'aria-hidden="true"')+'>'+(r.hasChildren?'<svg viewBox="0 0 16 16"><path d="m6 4 4 4-4 4"/></svg>':'')+'</span><span class="tn-icon">'+icon(n.kind)+'</span><span class="tn-copy"><span class="tn-label" title="'+escape(n.label)+'">'+highlighted(n.label,s.query)+'</span>'+(n.description?'<span class="tn-description" title="'+escape(n.description)+'">'+escape(n.description)+'</span>':'')+'</span>'+(n.status?'<span class="tn-state"'+(n.tone?' data-tone="'+escape(n.tone)+'"':'')+'>'+escape(n.status)+'</span>':'')+(r.hasChildren?'<span class="tn-count">'+r.childCount+'</span>':'')+'</div>';}).join('');tree.setAttribute('aria-activedescendant',rowId(s.active));
   }
   outside.hidden=!(s.selected&&(s.selectionOutside||status!=='ready'));
   outside.innerHTML=outside.hidden?'':'<span><small>当前选择'+(status==='loading'?' · 读取中':status==='failed'?' · 读取失败':s.selectionMissing?' · 已不在目录':s.filtered?' · 筛选范围外':' · 已收起')+'</small><strong title="'+escape(s.selectedLabel)+'">'+escape(s.selectedLabel||s.selected)+'</strong></span>'+(!s.selectionMissing&&status==='ready'?'<button type="button" class="btn ghost btn-sm" data-tn-reveal>定位</button>':'');
   tree.scrollTop=scroll;options.onUpdate?.(s);
  }
  function emit(result){if(result?.type==='select')options.onSelect?.({...store.node(result.id)});}
  function activate(key){const node=store.node(key);if(!node||node.disabled||node.selectable===false)return;if(store.select(key))emit({type:'select',id:key});options.onActivate?.({...node});}
  function click(event){const row=event.target.closest('[data-tn-id]');if(event.target.closest('[data-tn-reveal]')){store.reveal();render();tree.focus({preventScroll:true});scrollActive();options.onReveal?.();return;}if(event.target.closest('[data-tn-clear]')){store.setQuery('');store.setFilter(null);render();options.onClear?.();return;}if(event.target.closest('[data-tn-retry]')){options.onRetry?.();return;}if(!row||status!=='ready')return;const key=row.dataset.tnId;store.focus(key);if(event.target.closest('[data-tn-toggle]')||store.node(key)?.selectable===false)store.toggle(key);else activate(key);render();tree.focus({preventScroll:true});}
  function scrollActive(){
   if(status!=='ready')return;const active=doc.getElementById(rowId(store.state().active));if(!active)return;
   const item=active.getBoundingClientRect(),box=tree.getBoundingClientRect(),scale=tree.offsetHeight?box.height/tree.offsetHeight:1;
   // Move only the tree's own viewport; a long list must not scroll the containing page.
   if(item.top<box.top)tree.scrollTop-=(box.top-item.top)/scale+4;
   else if(item.bottom>box.bottom)tree.scrollTop+=(item.bottom-box.bottom)/scale+4;
  }
  function keydown(event){if(status!=='ready'||!['ArrowDown','ArrowUp','ArrowRight','ArrowLeft','Home','End','Enter',' '].includes(event.key))return;event.preventDefault();event.stopPropagation();emit(store.key(event.key));if(['Enter',' '].includes(event.key)){const node=store.node(store.state().active);if(node&&!node.disabled&&node.selectable!==false)options.onActivate?.({...node});}render();scrollActive();}
  host.addEventListener('click',click);tree.addEventListener('keydown',keydown);render();
  return {state:store.state,render,store,setQuery(value){store.setQuery(value);render();},setFilter(value,settings){store.setFilter(value,settings);render();},setNodes(value){store.setNodes(value);render();},setSelected(value){store.setSelected(value);render();},setStatus(value){status=value;render();},setDensity(value){density=value;render();},expandAll(){store.expandAll();render();},collapseAll(){store.collapseAll();render();},reveal(){store.reveal();render();scrollActive();},destroy(){destroyed=true;host.removeEventListener('click',click);tree.removeEventListener('keydown',keydown);host.replaceChildren();}};
 }
 return {create,mount};
});
