/* Existing Skills/MCP detail presenter. Legacy rows keep the original six-target grid;
   multi-platform detail uses a readable left pane and a persistent platform pane. */
(() => {
 'use strict';
 const $=s=>document.querySelector(s),esc=v=>logEsc(String(v??''));
 function sample(kind,name){const data=kind==='mcp'?MCP_DETAIL_MOCK[name]:SKILL_DETAIL_FACTS[name];return data?{...data}:{};}
 function facts(data){
  const isMcp=data.kind==='mcp',mock=data.facts||{},
   fact=(label,value,span2=false,scroll=false)=>'<div'+(span2?' class="span-2"':'')+'><b>'+label+'</b><span'+(scroll?' class="scrollable"':'')+'>'+value+'</span></div>',
   mono=v=>'<span class="mono">'+esc(v||'—')+'</span>';
  return isMcp
   ?fact('类型',esc(data.source||'stdio'))+fact('版本',esc(data.version||'—'))+fact('更新时间',esc(data.updated||'—'))
    +fact('命令',mono(mock.command))+fact('参数',mono(mock.args))+fact('环境变量',mono(mock.env),true,true)+fact('配置落点',mono(mock.landing),true)
   :fact('来源',esc(data.source||'本地'))+fact('版本',esc(data.version||'—'))+fact('更新时间',esc(data.updated||'—'))
    +fact('入口文件',mono('SKILL.md'))+fact('文件数',esc(mock.files||'—'))+fact('体量',esc(mock.size||'—'))+fact('源目录',mono(data.sourcePath||'~/.agents/skills/'+data.name),true);
 }
 function actionHTML(data,options){
  if(options.headActions!==undefined)return options.headActions;
  const actions=data.kind==='mcp'?[['edit','编辑',ICON_EDIT],['delete','删除',ICON_TRASH]]:[['update','检查 / 更新',ICON_REFRESH],['uninstall','卸载',ICON_TRASH]];
  return actions.map(([id,label,icon])=>'<button type="button" class="icon-btn '+(['delete','uninstall'].includes(id)?'danger':'')+'" data-extension-detail-action="'+id+'" aria-label="'+esc(label+' '+data.name)+'" title="'+label+'">'+icon+'</button>').join('');
 }
 function legacyTargets(options){
  return (options.getTargets?.()||[]).map(t=>{const mixed=t.enabled==='mixed',label=t.state||(mixed?'部分启用':t.enabled?'已启用':'未启用');return '<li'+(t.enabled?' class="on"':'')+'><button class="skill-detail-target" type="button" data-extension-detail-target="'+esc(t.id)+'" aria-pressed="'+(mixed?'mixed':!!t.enabled)+'" aria-label="'+esc(t.label+'：'+label+'，点击'+(mixed?'全部启用':t.enabled?'停用':'启用'))+'" title="'+esc(t.reason||label)+'" '+(t.disabled?'disabled':'')+'><span>'+esc(t.label)+'</span><em>'+esc(label)+'</em></button></li>';}).join('');
 }
 function platformRow(t){
  const mixed=t.enabled==='mixed',label=t.state||(mixed?'部分启用':t.enabled?'已启用':'未启用');
  const abnormal=t.state&&!['已启用','未启用'].includes(t.state)?t.state:t.disabled?'待核实':'';
  return '<li class="ext-detail-target'+(t.enabled?' on':'')+(t.disabled?' blocked':'')+'"><button type="button" class="skill-detail-target" data-extension-detail-target="'+esc(t.id)+'" aria-pressed="'+(mixed?'mixed':!!t.enabled)+'" aria-label="'+esc(t.label+'：'+label+'，点击'+(mixed?'全部启用':t.enabled?'停用':'启用'))+'" title="'+esc(t.reason||label)+'" '+(t.disabled?'disabled':'')+'>'
   +'<span class="ext-detail-mark" aria-hidden="true"></span><span class="ext-detail-target-name">'+esc(t.label)+'</span>'+(abnormal?'<em>'+esc(abnormal)+'</em>':'')+'</button></li>';
 }
 function platformPanel(targets,query){
  const on=targets.filter(t=>t.enabled),other=targets.filter(t=>!t.enabled);
  const filter=list=>list.filter(t=>(t.label+' '+(t.state||'')+' '+(t.reason||'')).toLowerCase().includes(query.toLowerCase()));
  const group=(title,list)=>'<div class="ext-detail-target-group"><strong>'+title+' <span>'+list.length+'</span></strong><ul>'+(list.length?list.map(platformRow).join(''):'<li class="ext-detail-empty">'+(query?'没有匹配的平台':title+'暂无平台')+'</li>')+'</ul></div>';
  return '<div class="ext-detail-targets" data-ext-targets><header class="ext-detail-platform-head"><div><strong>平台启用</strong><span>'+on.length+' / '+targets.length+' 已启用</span></div></header><label class="ext-detail-search"><svg viewBox="0 0 20 20" aria-hidden="true"><circle cx="8.5" cy="8.5" r="5.5"/><path d="m13 13 4 4"/></svg><input class="input" type="search" data-ext-target-search aria-label="搜索平台" placeholder="搜索平台" value="'+esc(query)+'"></label><div class="ext-detail-target-groups">'+group('已启用',filter(on))+group('其他平台',filter(other))+'</div></div>';
 }
 function open(data,options={}){
  const usePlatformPanel=typeof options.getTargets==='function'&&options.targetsHTML===undefined;
  const markdown=[data.description,data.markdown??data.facts?.md].filter(Boolean).join('\n\n');
  const doc=data.markdown??data.facts?.md??'';
  const targets=usePlatformPanel?options.getTargets():[];
  const enabledCount=targets.filter(t=>t.enabled).length;
  const legacy=usePlatformPanel?'':'<div class="skill-detail-block"><ul class="skill-detail-targets">'+(options.targetsHTML??legacyTargets(options))+'</ul></div>';
  openModal(data.name,
   usePlatformPanel
    ?'<div class="ext-detail-body" data-pane="main"><div class="ext-detail-switch" role="tablist" aria-label="详情分区"><button type="button" role="tab" id="extDetailTabMain" aria-selected="true" aria-controls="extDetailPaneMain" data-ext-detail-pane="main">详情</button><button type="button" role="tab" id="extDetailTabPlatform" aria-selected="false" aria-controls="extDetailPanePlatform" data-ext-detail-pane="platform">平台启用 <span data-ext-pane-count>'+enabledCount+' / '+targets.length+'</span></button></div><div class="ext-detail-main" id="extDetailPaneMain" role="tabpanel" aria-labelledby="extDetailTabMain"><section class="ext-detail-overview"><div class="ext-detail-lead">'+renderSkillMarkdown(data.description||'')+'</div><div class="skill-detail-facts">'+facts(data)+'</div></section>'+(doc?'<section class="ext-detail-read"><h4 class="ext-detail-section-title">使用说明</h4><div class="skill-detail-md">'+renderSkillMarkdown(doc)+'</div></section>':'')+'</div><aside class="ext-detail-platform" id="extDetailPanePlatform" role="tabpanel" aria-labelledby="extDetailTabPlatform" aria-label="平台启用">'+platformPanel(targets,'')+'</aside></div>'
    :'<div class="ext-detail-legacy"><div class="skill-detail-block skill-detail-doc"><div class="skill-detail-md">'+renderSkillMarkdown(markdown)+'</div></div><div class="skill-detail-block"><div class="skill-detail-facts">'+facts(data)+'</div></div>'+legacy+'</div>',
   '',null,{hideConfirm:true,cancelLabel:'关闭',hideNote:true,headActions:actionHTML(data,options),prototypeAnnotation:options.prototypeAnnotation});
  const box=$('#modal'),head=$('#modalHeadActions'),body=$('#modalBody');
  box.classList.add('modal-wide','modal-tall');
  if(usePlatformPanel)box.classList.add('ext-detail-modal');
  box.setAttribute('role','dialog');box.setAttribute('aria-modal','true');box.setAttribute('aria-labelledby','modalTitle');
  let query='';
  const scrollMain=body.querySelector('.ext-detail-main');if(scrollMain)scrollMain.scrollTop=0;
  function syncPaneCount(){
    const count=body.querySelector('[data-ext-pane-count]');if(!count)return;
    const list=options.getTargets();count.textContent=list.filter(t=>t.enabled).length+' / '+list.length;
  }
  function drawTargets(focusId){
   const aside=body.querySelector('.ext-detail-platform');if(!aside)return;
   const previous=aside.querySelector('.ext-detail-target-groups')?.scrollTop||0;
   aside.innerHTML=platformPanel(options.getTargets(),query);
   const groups=aside.querySelector('.ext-detail-target-groups');if(groups)groups.scrollTop=previous;
   syncPaneCount();
   if(focusId)aside.querySelector('[data-extension-detail-target="'+focusId+'"]')?.focus({preventScroll:true});
   else if(query)aside.querySelector('[data-ext-target-search]')?.focus({preventScroll:true});
  }
  function selectPane(name){
    const pane=body.querySelector('.ext-detail-body');if(!pane)return;
    pane.dataset.pane=name;
    body.querySelectorAll('[data-ext-detail-pane]').forEach(button=>{
      const active=button.dataset.extDetailPane===name;
      button.setAttribute('aria-selected',String(active));
      button.tabIndex=active?0:-1;
    });
    const panel=body.querySelector(name==='platform'?'.ext-detail-platform':'.ext-detail-main');
    if(panel)panel.scrollTop=0;
    body.querySelector('[data-ext-target-search]')?.blur();
  }
  function action(e){const b=e.target.closest('[data-extension-detail-action]');if(b&&options.onAction)options.onAction(b.dataset.extensionDetailAction,b);}
  function click(e){
   const tab=e.target.closest('[data-ext-detail-pane]');if(tab){selectPane(tab.dataset.extDetailPane);return;}
   const target=e.target.closest('[data-extension-detail-target]');if(!target||target.disabled||!options.onToggleTarget)return;options.onToggleTarget(target.dataset.extensionDetailTarget,target.getAttribute('aria-pressed')!=='true');if(usePlatformPanel)drawTargets(target.dataset.extensionDetailTarget);
  }
  function search(e){if(!usePlatformPanel||!e.target.matches('[data-ext-target-search]'))return;query=e.target.value;const pos=e.target.selectionStart;drawTargets();const input=body.querySelector('[data-ext-target-search]');if(input){input.focus({preventScroll:true});input.setSelectionRange(pos,pos);}}
  function keydown(e){
   const tab=e.target.closest('[data-ext-detail-pane]');if(!tab)return;
   const tabs=[...body.querySelectorAll('[data-ext-detail-pane]')],index=tabs.indexOf(tab);
   if(!['ArrowLeft','ArrowRight','Home','End'].includes(e.key))return;
   e.preventDefault();
   const next=e.key==='Home'?tabs[0]:e.key==='End'?tabs.at(-1):tabs[(index+(e.key==='ArrowRight'?1:tabs.length-1))%tabs.length];
   selectPane(next.dataset.extDetailPane);next.focus({preventScroll:true});
  }
  head.addEventListener('click',action);body.addEventListener('click',click);body.addEventListener('input',search);body.addEventListener('keydown',keydown);
  let cleaned=false;
  function cleanup(){if(cleaned)return;cleaned=true;head.removeEventListener('click',action);body.removeEventListener('click',click);body.removeEventListener('input',search);body.removeEventListener('keydown',keydown);box.classList.remove('ext-detail-modal');box.removeAttribute('role');box.removeAttribute('aria-modal');box.removeAttribute('aria-labelledby');options.onClose?.();}
  window.__modalOnClose=cleanup;
  return {cleanup};
 }
 window.PrototypeExtensionDetail={open,sample};
})();
