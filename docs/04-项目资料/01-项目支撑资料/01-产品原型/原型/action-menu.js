/* 原型动作菜单：按触发按钮定位，复用顶层浮层，不建立额外遮罩。 */
(() => {
 'use strict';
 const esc=x=>String(x??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 let current=null,serial=0;
 function close(focus=false){
  if(!current)return;const {menu,anchor}=current;current=null;
  try{if(menu.matches(':popover-open'))menu.hidePopover();}catch(_){}
  menu.remove();anchor?.setAttribute('aria-expanded','false');if(focus&&anchor?.isConnected)anchor.focus({preventScroll:true});
 }
 function open({anchor,label='更多操作',items=[]}){
  if(!anchor?.isConnected||!items.length)return false;
  if(current?.anchor===anchor){close(true);return false;}close();window.prototypeSelect?.close();
  const menu=document.createElement('div');menu.id='prototype-action-menu-'+(++serial);menu.className='prototype-action-menu';menu.setAttribute('role','menu');menu.setAttribute('aria-label',label);
  menu.innerHTML=items.map((item,i)=>(item.separatorBefore?'<div class="action-menu-separator" role="separator"></div>':'')+'<button type="button" role="menuitem" '+(item.danger?'class="danger" ':'')+'data-action-menu-index="'+i+'" '+(item.disabled?'disabled ':'')+'title="'+esc(item.reason||item.label)+'">'+esc(item.label)+'</button>').join('');
  document.body.appendChild(menu);anchor.setAttribute('aria-controls',menu.id);anchor.setAttribute('aria-haspopup','menu');anchor.setAttribute('aria-expanded','true');current={anchor,menu,items,label};
  if(typeof menu.showPopover==='function'){menu.setAttribute('popover','manual');try{menu.showPopover();}catch(_){menu.removeAttribute('popover');}}
  const bounds=document.querySelector('.window')?.getBoundingClientRect()||{left:0,top:0,right:innerWidth,bottom:innerHeight,width:innerWidth,height:innerHeight},rect=anchor.getBoundingClientRect(),width=Math.max(100,Math.min(240,bounds.width-16));
  menu.style.width=width+'px';menu.style.maxHeight=Math.max(60,bounds.bottom-bounds.top-16)+'px';
  const size=menu.getBoundingClientRect(),left=Math.max(bounds.left+8,Math.min(rect.left,bounds.right-width-8)),top=rect.bottom+4+size.height<=bounds.bottom-8?rect.bottom+4:Math.max(bounds.top+8,rect.top-size.height-4);
  menu.style.left=left+'px';menu.style.top=top+'px';menu.querySelector('button:not(:disabled)')?.focus({preventScroll:true});return true;
 }
 document.addEventListener('click',event=>{
  if(!current)return;const {menu,anchor,items}=current,item=event.target.closest('[data-action-menu-index]');
  if(item&&menu.contains(item)){const action=items[Number(item.dataset.actionMenuIndex)];if(!action||action.disabled)return;close(true);action.onSelect?.();}
  else if(!menu.contains(event.target)&&!anchor.contains(event.target))close();
 });
 document.addEventListener('keydown',event=>{
  if(!current)return;
  if(event.key==='Escape'){event.preventDefault();event.stopImmediatePropagation();close(true);return;}
  if(event.key==='Tab'){close(true);return;}
  if(!['ArrowDown','ArrowUp','Home','End'].includes(event.key))return;
  const items=[...current.menu.querySelectorAll('button:not(:disabled)')];if(!items.length)return;event.preventDefault();const index=items.indexOf(document.activeElement),next=event.key==='Home'?0:event.key==='End'?items.length-1:event.key==='ArrowDown'?(index+1)%items.length:(index-1+items.length)%items.length;items[next]?.focus();
 },true);
 document.addEventListener('scroll',event=>{if(current&&!current.menu.contains(event.target))close();},true);window.addEventListener('resize',()=>close());
 window.prototypeActionMenu={open,close,state:()=>current?{label:current.label,items:current.items.map(i=>({label:i.label,disabled:!!i.disabled}))}:null};
})();
