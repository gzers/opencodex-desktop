/* 共用 Select：保留 native value/change 契约；菜单在 body/top layer，退出后归还原属主。 */
(()=>{
  'use strict';
  const records=new Set(), owners=new WeakMap();
  let active=null, sequence=0, queued=false, typeahead='', typedAt=0;
  const text=node=>(node?.textContent||'').trim();
  const options=record=>[...record.menu.querySelectorAll('.select-option')];
  const available=record=>options(record).filter(option=>!option.disabled&&!option.hidden);
  const number=(style,name,fallback)=>parseFloat(style.getPropertyValue(name))||fallback;
  // 输入 / 输出均为 viewport 物理 CSS px；内部菜单再按 trigger 实际缩放换算。
  function placement(rect,viewport,settings={}){
    const scale=settings.scale||1,gap=(settings.gap??8)*scale,gutter=settings.gutter??12;
    const width=Math.max(0,Math.min(Math.max(rect.width,(settings.width||320)*scale),viewport.width-gutter*2));
    const below=Math.max(0,viewport.height-gutter-rect.bottom-gap),above=Math.max(0,rect.top-gutter-gap);
    const wanted=Math.min((settings.height||260)*scale,(settings.maxHeight||260)*scale);
    const up=below<wanted&&above>below,height=Math.min(wanted,up?above:below);
    return {left:Math.max(gutter,Math.min(rect.left,viewport.width-gutter-width)),top:up?rect.top-gap-height:rect.bottom+gap,width,height,up};
  }
  function labelFor(source){
    if(source.getAttribute('aria-label'))return source.getAttribute('aria-label');
    const labelled=source.getAttribute('aria-labelledby');
    if(labelled)return labelled.split(/\s+/).map(id=>text(document.getElementById(id))).join(' ').trim();
    const label=source.labels?.[0]||source.closest('label');
    if(label){const clone=label.cloneNode(true);clone.querySelectorAll('select,.select').forEach(node=>node.remove());return text(clone);}
    return text(source.previousElementSibling)||'选择选项';
  }
  function register(root,source=null){
    if(owners.has(root))return owners.get(root);
    const trigger=root.querySelector('.select-trigger'),menu=root.querySelector('.select-menu');
    if(!trigger||!menu)return null;
    sequence++;
    trigger.id=trigger.id||'prototype-select-trigger-'+sequence;
    menu.id=menu.id||'prototype-select-menu-'+sequence;
    trigger.setAttribute('aria-controls',menu.id);trigger.setAttribute('aria-haspopup','listbox');trigger.setAttribute('aria-expanded','false');
    menu.setAttribute('role','listbox');menu.setAttribute('aria-labelledby',trigger.id);
    const record={root,source,trigger,menu,signature:'',label:source?labelFor(source):trigger.getAttribute('aria-label')||'选择选项'};
    records.add(record);owners.set(root,record);owners.set(trigger,record);owners.set(menu,record);
    if(source)owners.set(source,record);
    return record;
  }
  function enhanceSource(source){
    if(owners.has(source)||(!source.multiple&&source.size>1))return;
    const label=labelFor(source);
    const root=document.createElement('div');root.className='select select-native';root.dataset.select='';
    if(source.classList.contains('mp-select'))root.classList.add('mp-select');
    root.style.cssText=source.style.cssText;
    source.before(root);root.appendChild(source);
    source.hidden=true;source.classList.add('select-backing');source.tabIndex=-1;source.setAttribute('aria-hidden','true');
    const trigger=document.createElement('button');trigger.type='button';trigger.className='select-trigger';trigger.dataset.selectUi='';
    const value=document.createElement('span');value.className='select-value';trigger.appendChild(value);
    const menu=document.createElement('div');menu.className='select-menu';menu.dataset.selectUi='';
    root.appendChild(trigger);root.appendChild(menu);
    const record=register(root,source);
    record.label=label;
    // 显式 label 的激活目标是可见 trigger；隐式 label 内的按钮天然可点击。
    for(const label of source.labels||[])if(label.htmlFor===source.id)label.htmlFor=trigger.id;
    sync(record);
  }
  function sync(record){
    const {source,trigger,menu}=record;
    if(!source){
      options(record).forEach(option=>{option.tabIndex=-1;option.setAttribute('aria-selected',String(option.classList.contains('selected')));});
      const value=text(trigger.querySelector('.select-value'));trigger.setAttribute('aria-label',record.label+'：'+value);trigger.title=value;return;
    }
    const name=source.getAttribute('aria-label')||record.label;
    for(const attribute of ['aria-describedby','aria-invalid']){
      const value=source.getAttribute(attribute);if(value!==null)trigger.setAttribute(attribute,value);else trigger.removeAttribute(attribute);
    }
    trigger.disabled=source.disabled;
    const list=[...source.options],selected=source.multiple?list.filter(option=>option.selected):[list[source.selectedIndex]].filter(Boolean);
    if(source.multiple)menu.setAttribute('aria-multiselectable','true');else menu.removeAttribute('aria-multiselectable');
    const value=selected.map(option=>option.textContent).join(' / ')||(source.multiple?'未选择':'选择选项');
    trigger.setAttribute('aria-label',name+'：'+value);
    if(trigger.querySelector('.select-value').textContent!==value)trigger.querySelector('.select-value').textContent=value;
    trigger.title=source.title?source.title+'\n'+value:value;
    const signature=JSON.stringify(list.map(option=>[option.value,option.textContent,option.disabled,option.hidden,option.parentElement?.disabled,option.parentElement?.label]));
    if(signature!==record.signature){
      menu.replaceChildren();let group=null;
      list.forEach((option,index)=>{
        const parent=option.parentElement;
        if(parent?.tagName==='OPTGROUP'&&parent!==group){
          const caption=document.createElement('div');caption.className='select-group-label';caption.setAttribute('role','presentation');caption.textContent=parent.label;menu.appendChild(caption);group=parent;
        }
        const item=document.createElement('button');item.type='button';item.className='select-option';item.setAttribute('role','option');item.dataset.optionIndex=String(index);item.tabIndex=-1;
        item.textContent=option.textContent;item.disabled=!!(option.disabled||parent?.disabled);item.hidden=option.hidden;menu.appendChild(item);
      });record.signature=signature;
    }
    options(record).forEach(option=>{const index=Number(option.dataset.optionIndex),chosen=source.multiple?!!list[index]?.selected:index===source.selectedIndex;option.classList.toggle('selected',chosen);option.setAttribute('aria-selected',String(chosen));});
    if(active===record&&trigger.disabled)close();
  }
  function enhance(container=document){
    container.querySelectorAll('select').forEach(enhanceSource);
    container.querySelectorAll('.select[data-select]').forEach(root=>register(root));
    for(const record of records){
      if(!record.root.isConnected){if(active===record)close();records.delete(record);continue;}
      sync(record);
    }
    if(active&&!visible(active))close();
  }
  function visible(record){return record.root.isConnected&&!record.trigger.disabled&&record.trigger.getClientRects().length>0;}
  function position(record){
    const {trigger,menu}=record,rect=trigger.getBoundingClientRect(),style=getComputedStyle(trigger);
    const scale=trigger.offsetWidth?rect.width/trigger.offsetWidth:1;
    const viewport={width:window.innerWidth,height:window.innerHeight};
    const preferred=number(style,'--select-menu-max-width',320),maxHeight=number(style,'--select-menu-max-height',260);
    const gutter=number(style,'--space-3',12),limit=Math.max(0,(viewport.width-gutter*2)/scale);
    menu.style.transform='scale('+scale+')';menu.style.width='max-content';menu.style.minWidth=Math.min(rect.width/scale,limit)+'px';menu.style.maxWidth=Math.min(Math.max(rect.width/scale,preferred),limit)+'px';
    menu.style.maxHeight='none';
    const placed=placement(rect,viewport,{scale,width:menu.offsetWidth||preferred,height:menu.scrollHeight,maxHeight,gap:number(style,'--space-2',8),gutter});
    menu.style.left=placed.left+'px';menu.style.top=placed.top+'px';menu.style.width=placed.width/scale+'px';menu.style.maxHeight=placed.height/scale+'px';
    record.root.classList.toggle('up',placed.up);
  }
  function close(returnFocus=false){
    if(!active)return;
    const record=active;active=null;
    try{if(record.menu.matches(':popover-open'))record.menu.hidePopover();}catch(_){/* 无 Popover 的宿主走 body portal。 */}
    record.menu.removeAttribute('data-floating');record.menu.removeAttribute('popover');record.menu.style.cssText='';
    record.root.appendChild(record.menu);record.root.classList.remove('open','up');record.trigger.setAttribute('aria-expanded','false');
    typeahead='';
    if(returnFocus&&record.trigger.isConnected)record.trigger.focus({preventScroll:true});
  }
  function open(record,last=false){
    close();sync(record);if(!visible(record))return;
    active=record;record.root.classList.add('open');record.trigger.setAttribute('aria-expanded','true');
    document.body.appendChild(record.menu);record.menu.dataset.floating='true';
    if(typeof record.menu.showPopover==='function'){
      record.menu.setAttribute('popover','manual');try{record.menu.showPopover();}catch(_){record.menu.removeAttribute('popover');}
    }
    position(record);
    const list=available(record),chosen=list.find(option=>option.getAttribute('aria-selected')==='true');
    (chosen||(last?list[list.length-1]:list[0]))?.focus({preventScroll:true});
  }
  function choose(record,option){
    if(option.disabled||option.hidden||record.trigger.disabled)return;
    if(record.source){
      const source=record.source,index=Number(option.dataset.optionIndex);
      if(source.multiple){
        source.options[index].selected=!source.options[index].selected;sync(record);
        source.dispatchEvent(new Event('input',{bubbles:true}));source.dispatchEvent(new Event('change',{bubbles:true}));
        enhance();
        if(active===record&&option.isConnected)option.focus({preventScroll:true});
        return;
      }
      const changed=source.selectedIndex!==index;
      source.selectedIndex=index;sync(record);close(true);
      if(changed){source.dispatchEvent(new Event('input',{bubbles:true}));source.dispatchEvent(new Event('change',{bubbles:true}));}
      enhance();
      // 页面 change 可同步重绘自身；优先用稳定 id / data-* 把焦点送回同一字段。
      if(!record.root.isConnected){
        const id=source.id,attributes=[...source.attributes].filter(attribute=>attribute.name.startsWith('data-'));
        const next=[...records].find(item=>item.source&&(id?item.source.id===id:attributes.length&&attributes.every(attribute=>item.source.getAttribute(attribute.name)===attribute.value)));
        next?.trigger.focus({preventScroll:true});
      }
    }else{
      options(record).forEach(item=>{const selected=item===option;item.classList.toggle('selected',selected);item.setAttribute('aria-selected',String(selected));});
      record.trigger.querySelector('.select-value').textContent=text(option);record.trigger.title=text(option);
      sync(record);
      close(true);record.root.dispatchEvent(new CustomEvent('selectchange',{bubbles:true,detail:{value:option.dataset.value??text(option),option}}));
    }
  }
  function recordFor(target){return owners.get(target)||owners.get(target.closest?.('.select-menu'))||owners.get(target.closest?.('.select'));}
  document.addEventListener('click',event=>{
    const trigger=event.target.closest?.('.select-trigger');
    if(trigger){const record=recordFor(trigger);if(record){if(active===record)close(true);else open(record);}return;}
    const option=event.target.closest?.('.select-option');
    if(option&&active&&active.menu.contains(option)){choose(active,option);return;}
    if(active&&!active.menu.contains(event.target))close();
  });
  document.addEventListener('keydown',event=>{
    if(event.key==='Escape'&&active){event.preventDefault();event.stopImmediatePropagation();close(true);return;}
    if(event.key==='Tab'&&active){close(true);return;}
    const record=recordFor(event.target);if(!record||record.trigger.disabled)return;
    if(['Enter',' ','ArrowDown','ArrowUp','Home','End'].includes(event.key)){
      event.preventDefault();event.stopImmediatePropagation();
      if(active!==record){open(record,event.key==='ArrowUp'||event.key==='End');if(event.key!=='Home'&&event.key!=='End')return;}
      else if(event.key==='Enter'||event.key===' '){const option=event.target.closest?.('.select-option');if(option)choose(record,option);else close(true);return;}
      const list=available(record);if(!list.length)return;
      const index=list.indexOf(document.activeElement);
      const next=event.key==='Home'?0:event.key==='End'?list.length-1:event.key==='ArrowUp'?(index-1+list.length)%list.length:(index+1)%list.length;
      list[next].focus();return;
    }
    if(active===record&&event.key.length===1&&!event.ctrlKey&&!event.metaKey&&!event.altKey){
      event.preventDefault();const now=Date.now();typeahead=now-typedAt>700?event.key:typeahead+event.key;typedAt=now;
      const list=available(record),start=list.indexOf(document.activeElement);
      const ordered=list.slice(start+1).concat(list.slice(0,start+1));
      ordered.find(option=>text(option).toLocaleLowerCase().startsWith(typeahead.toLocaleLowerCase()))?.focus();
    }
  },true);
  document.addEventListener('change',event=>{const record=owners.get(event.target);if(record)sync(record);});
  document.addEventListener('scroll',event=>{if(active&&!active.menu.contains(event.target))close();},true);
  window.addEventListener('resize',()=>{if(active)position(active);});
  window.addEventListener('blur',()=>close());
  window.addEventListener('hashchange',()=>close());
  function schedule(){if(queued)return;queued=true;queueMicrotask(()=>{queued=false;enhance();});}
  const observer=new MutationObserver(mutations=>{
    if(mutations.some(mutation=>{
      if(mutation.target.closest?.('[data-select-ui]'))return false;
      if(mutation.type==='attributes')return mutation.target.tagName==='SELECT'||mutation.target.tagName==='OPTION'||mutation.target.tagName==='OPTGROUP'||(active&&(mutation.target.contains(active.root)||mutation.target===document.documentElement));
      return [...mutation.addedNodes,...mutation.removedNodes].some(node=>node.nodeType===1&&!node.matches?.('[data-select-ui]'))||mutation.target.closest?.('select')||mutation.target.parentElement?.closest('select');
    }))schedule();
  });
  observer.observe(document.body,{childList:true,subtree:true,attributes:true,characterData:true,attributeFilter:['disabled','selected','multiple','hidden','class','style','value','label','aria-label','aria-describedby','aria-invalid']});
  window.prototypeSelect={enhance,close,placement};
  enhance();
})();
