/* 双平台目录评审：内存 fixture，权限与迁移不执行。 */
(() => {
  const $=selector=>document.querySelector(selector);
  const esc=value=>String(value).replace(/[&<>"']/g,char=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
  let windowsMode=document.body.classList.contains('windows-mode');
  let platform=windowsMode?'windows':'macos', scene='default';
  const current=()=>window.DataPaths.paths(platform,scene==='protected'&&platform==='windows'?{installation:'C:\\Program Files\\OpenCodex Desktop'}:scene==='custom'?{
    root:platform==='windows'?'F:\\OpenCodexData':'~/Documents/OpenCodexData',
    home:platform==='windows'?'F:\\OfficialHome':'~/Documents/OfficialHome',
    prefix:platform==='windows'?'F:\\ManagedOpenCodex':'~/Documents/ManagedOpenCodex'
  }:{});
  const card=document.createElement('section');card.className='proto-card';card.id='dataPathsDemo';
  card.setAttribute('data-proto-contexts','overview settings:installation settings:backup');
  card.innerHTML='<div class="proto-note-title">0.1.10 · 数据目录规则</div><div class="pctl-grid"><button type="button" data-path-platform="macos">macOS</button><button type="button" data-path-platform="windows">Windows</button></div><div class="pctl-sub">目录情景</div><div class="pctl-grid"><button type="button" data-path-scene="default">默认目录</button><button type="button" data-path-scene="custom">已有自定义</button><button type="button" data-path-scene="protected">受保护安装目录</button></div><p class="pctl-hint">Windows 默认以实际安装目录派生 data，盘符仅为示例；macOS 使用 Application Support。已有自定义目录、前缀及外置 HOME 保留。这里只投影路径，不检测权限、不迁移、不卸载；独立备份目录待设计。</p>';
  ($('#protoStdCards .proto-group-body') || $('#protoStdCards')).appendChild(card);
  const pathButton=(path,action,label)=>'<button type="button" class="'+(action==='copy-path'?'cli-copy-btn':'btn ghost')+'" data-prototype-action="'+action+'" data-path="'+esc(path)+'" title="'+esc(path)+'">'+(action==='copy-path'?'<code>'+esc(path)+'</code>':label)+'</button>';
  const row=(path,purpose,edit)=>'<tr><td>'+pathButton(path,'copy-path')+'</td><td>'+purpose+'</td><td>'+pathButton(path,'open-path','打开')+(edit?'<button type="button" class="btn" data-prototype-action="'+edit+'">修改</button>':'')+'</td></tr>';
  function render(){
    const p=current();
    $('#card-install-paths tbody').innerHTML=row(p.root,'数据目录','choose-root')+row(p.home,'OpenCodex 配置与运行数据','choose-home');
    $('#card-install-partitions tbody').innerHTML=[
      [p.manager,'管理器配置与运行记录'],[p.backups,'备份（各事务声明范围）'],[p.prefix,'托管 OpenCodex 包体 / 面板'],[p.entry,'托管启动入口'],
      [p.logs,'日志'],[p.exports,'导出文件'],[p.cache,'可清理缓存'],[p.sync,'同步与冲突摘要']
    ].map(([path,purpose])=>row(path,purpose)).join('');
    let note=$('#directoryRuleNote');
    if(!note){note=document.createElement('div');note.id='directoryRuleNote';note.className='setting-list';$('#card-install-paths').appendChild(note);}
    note.innerHTML='<div class="setting-row"><div><div class="setting-title">'+(p.platform==='windows'?'Windows · 随安装位置':'macOS · 用户数据目录')+'</div><p class="mnt-path">'+esc(p.installation)+'</p><p class="setting-desc">'+(p.custom?'使用自定义目录；升级保留当前位置。':p.platform==='windows'?'默认保存到安装目录内的 data。':'默认保存到当前用户的 Application Support。')+'</p><p class="setting-desc">备份保存到 '+esc(p.backups)+'。托管凭据保留在 '+p.credentials+'。</p></div></div>'+(scene==='protected'&&platform==='windows'?'<p role="alert">安装位置受系统权限保护。请先检查普通用户写入权限；无法写入时需选择可写数据目录，不自动改回其他位置。</p>':'');
    card.querySelectorAll('button').forEach(button=>{
      button.classList.toggle('active',button.dataset.pathPlatform===platform||button.dataset.pathScene===scene);
      button.disabled=button.dataset.pathScene==='protected'&&platform!=='windows';
    });
    window.dispatchEvent(new CustomEvent('prototype:data-paths',{detail:p}));
    window.prototypePanel?.refresh();
  }
  function preview(kind){
    const p=current(), root=kind==='root';
    openModal(root?'修改数据目录':'修改 OpenCodex 数据目录',
      '<p class="modal-lead">当前位置</p><p class="mnt-path">'+esc(root?p.root:p.home)+'</p><div class="setting-list"><div class="setting-row"><div><div class="setting-title">选择目标并检查</div><p>检查目录可写、剩余空间及已有数据；运行中先停止相关代理，不自动覆盖已有文件。</p></div></div><div class="setting-row"><div><div class="setting-title">确认变更范围</div><p>'+(root?'仅切换引用不搬旧数据；迁移数据需列明管理器配置、备份、托管包体和内置 HOME。外置 HOME 与自定义托管前缀保留，另行确认。':'可沿用数据目录内的 opencodex-home，或选择外置目录；外置目录的已有配置先展示差异，不直接覆盖。')+'</p></div></div><div class="setting-row"><div><div class="setting-title">保护与核验</div><p>备份、复制与校验完成后再切换；失败保留旧目录与引用，成功后重启核验，旧目录清理由用户确认。</p></div></div></div>',
      '关闭',()=>{}, {cancelLabel:'取消'});
  }
  card.addEventListener('click',event=>{
    const button=event.target.closest('button');if(!button||button.disabled)return;
    if(button.dataset.pathPlatform){platform=button.dataset.pathPlatform;if(platform==='macos'&&scene==='protected')scene='default';}
    if(button.dataset.pathScene)scene=button.dataset.pathScene;
    render();
  });
  new MutationObserver(()=>{
    const next=document.body.classList.contains('windows-mode');
    if(next===windowsMode)return;
    windowsMode=next;platform=next?'windows':'macos';if(platform==='macos'&&scene==='protected')scene='default';render();
  }).observe(document.body,{attributes:true,attributeFilter:['class']});
  window.dataPathsPrototype={current,preview};render();
})();
