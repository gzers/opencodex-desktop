/* 0.2.23 declared mock DTOs. Memory only; no filesystem, clipboard or network access. */
(function (root, factory) {
  const api = factory(root);
  if (typeof module === 'object' && module.exports) module.exports = api;
  else { root.directoryRuntime023 = api; root.OCXD023 = api.ui; api.mount(); }
})(typeof window === 'undefined' ? globalThis : window, function (root) {
  'use strict';
  const clone = value => JSON.parse(JSON.stringify(value));
  const esc = value => String(value ?? '').replace(/[&<>"']/g, char => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
  const button = (label, action, disabled = false, primary = false) => '<button type="button" class="btn ' + (primary ? 'primary' : 'ghost') + '" data-o23="' + esc(action) + '"' + (disabled ? ' disabled' : '') + '>' + esc(label) + '</button>';
  const options = (items, value) => items.map(item => '<option value="' + esc(item[0]) + '"' + (item[0] === value ? ' selected' : '') + '>' + esc(item[1]) + '</option>').join('');
  const select = (label, action, items, value) => '<label>' + esc(label) + '<select class="mp-select" aria-label="' + esc(label) + '" data-o23-select="' + esc(action) + '">' + options(items, value) + '</select></label>';
  const facts = rows => '<dl class="sx-detail-list">' + rows.map(row => '<dt>' + esc(row[0]) + '</dt><dd class="local-path" style="overflow-wrap:anywhere;white-space:pre-wrap">' + esc(row[1]) + '</dd>').join('') + '</dl>';
  let dialogSerial = 0;
  function dialog(title, body, primary, onConfirm, extra = {}) {
    const serial = ++dialogSerial;
    root.openModal(title, '<div class="mp-dialog" data-o23-dialog="' + serial + '">' + body + '</div>', primary || '', onConfirm || null, {
      mode: primary ? 'confirm' : 'view', hideConfirm: !primary, confirmKeepsOpen: true,
      prototypeAnnotation: {title:'0.2.23 · 声明式内存演示', body:'固定 mock DTO 演示，页面不访问网络、剪贴板、配置或文件系统；不代表生产匹配、安装或传输通过。'}, ...extra
    });
    const modal = root.document.getElementById('modal');
    modal.classList.add('mp-modal', 'mp-external-modal');
    root.document.getElementById('modalConfirm').disabled = !!extra.disabled;
    root.__modalOnClose = () => { dialogSerial++; modal.classList.remove('mp-modal','mp-external-modal'); root.document.getElementById('modalConfirm').disabled = false; extra.closed?.(); };
    return serial;
  }
  const ui = {clone, esc, button, options, select, facts, dialog, token:()=>dialogSerial, close:()=>root.closeModal()};
  const snapshots = {
    macos: {schemaVersion:1, aggregateRoot:'/Users/demo/.codex-desktop', desktopRoot:'/Users/demo/.codex-desktop/desktop', opencodexHome:'/Users/demo/.codex-desktop/opencodex/home', configPath:'/Users/demo/.codex-desktop/opencodex/home/config.json', installationRoot:'/Users/demo/Applications/OpenCodeX Desktop', runtimePrefix:'/Users/demo/Applications/OpenCodeX Desktop/runtime/opencodex', runtimeEntry:'/Users/demo/Applications/OpenCodeX Desktop/runtime/bin/ocx', managerExecutable:'/Users/demo/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app', packageVersion:'演示运行包修订', manifestVersion:'演示运行包修订', sourceVersion:'演示运行包修订', consistency:'valid'},
    windows: {"schemaVersion": 1, "aggregateRoot": "C:\\Users\\demo\\.codex-desktop", "desktopRoot": "C:\\Users\\demo\\.codex-desktop\\desktop", "opencodexHome": "C:\\Users\\demo\\.codex-desktop\\opencodex\\home", "configPath": "C:\\Users\\demo\\.codex-desktop\\opencodex\\home\\config.json", "installationRoot": "C:\\Users\\demo\\AppData\\Local\\Programs\\OpenCodeX Desktop", "runtimePrefix": "C:\\Users\\demo\\AppData\\Local\\Programs\\OpenCodeX Desktop\\runtime\\opencodex", "runtimeEntry": "C:\\Users\\demo\\AppData\\Local\\Programs\\OpenCodeX Desktop\\runtime\\bin\\ocx.cmd", "managerExecutable": "C:\\Users\\demo\\AppData\\Local\\Programs\\OpenCodeX Desktop\\opencodex-desktop.exe", "packageVersion": "演示运行包修订", "manifestVersion": "演示运行包修订", "sourceVersion": "演示运行包修订", "consistency": "valid"}
  };
  const scenes = [['valid','位置有效'],['loading','读取中'],['missing','尚未初始化'],['unreadable','无读取权限'],['corrupted','记录损坏'],['future','未知结构版本'],['pending','切换后待重启'],['port','端口被其它实例占用']];
  function create(snapshot = snapshots.macos) {
    let state = {snapshot:clone(snapshot), scene:'valid', pending:null, revision:1, reads:0, writes:0, copied:null};
    return {
      state:()=>clone(state),
      useSnapshot(value) { state.snapshot = clone(value); state.pending = null; state.scene = 'valid'; state.revision++; },
      scene(value) { if (!scenes.some(row=>row[0]===value)) return; state.scene=value; state.pending=null; if(value==='pending')state.pending={aggregateRoot:state.snapshot.aggregateRoot+'-selected',revision:state.revision+1}; },
      prepare(path) { state.reads++; return {path, expectedRevision:state.revision, valid:path.startsWith('/') || path.length>3 && path[1]===':' && [String.fromCharCode(92),'/'].includes(path[2]), writes:0}; },
      confirm(plan) { if(!plan.valid||plan.expectedRevision!==state.revision)return false;state.pending={aggregateRoot:plan.path,revision:state.revision+1};state.scene='pending';state.writes++;return true; },
      cancel() { return clone(state); },
      restart() { if(!state.pending)return false;const before=state.snapshot.aggregateRoot,next=state.pending.aggregateRoot;for(const k of ['aggregateRoot','desktopRoot','opencodexHome','configPath'])state.snapshot[k]=state.snapshot[k].replace(before,next);state.revision=state.pending.revision;state.pending=null;state.scene='valid';return true; }
    };
  }
  const controller = create();
  const validity = {loading:'正在读取实际位置…',missing:'用户目录尚未初始化。确认初始化前不会创建文件。',unreadable:'当前用户无法读取该位置。现有内容已保留，请核对访问权限。',corrupted:'定位或安装记录损坏，未当作空目录覆盖。',future:'结构版本暂不支持，未按旧格式继续。',port:'代理端口被其它实例占用。目录有效性不受影响，未连接或停止其它用户服务。'};
  function render() {
    const host = root.document.getElementById('ocxd023Settings'); if (!host) return;
    const s=controller.state(),p=s.snapshot;
    const paths = rows => facts(rows.map(([key,label])=>[label,s.scene==='loading'?'正在读取…':p[key]]));
    host.innerHTML = '<div class="card-head"><div><h2>用户目录与程序安装</h2><p>用户配置与程序分开存放；路径显示当前实际生效位置。</p></div></div>' +
      (validity[s.scene]?'<p class="mp-alert" role="status">'+esc(validity[s.scene])+'</p>':'') +
      '<h3>用户数据与配置</h3>'+paths([['aggregateRoot','用户集合根'],['desktopRoot','Desktop 工作根'],['opencodexHome','OpenCodex HOME'],['configPath','渠道配置文件']])+
      '<div class="mp-actions">'+button('完整查看与复制路径','path-details',s.scene==='loading')+button('选择新版目录','path-select',!['valid','pending'].includes(s.scene))+button('手动恢复渠道说明','path-json')+(s.scene==='missing'?button('初始化用户目录','path-init',false,true):'')+'</div>'+
      (s.pending?'<h3>等待重启生效</h3>'+facts([['当前位置',p.aggregateRoot],['待生效位置',s.pending.aggregateRoot]])+button('模拟重启并核对','path-restart',false,true):'')+
      '<h3>程序安装</h3>'+paths([['installationRoot','Desktop 安装根'],['managerExecutable','Desktop 本体'],['runtimePrefix','运行包前缀'],['runtimeEntry','稳定程序入口']])+facts([['运行状态','代理已停止（演示）'],['包 / 清单 / 来源',s.scene==='corrupted'?'一致性待核对':p.packageVersion+' / 一致'],['程序有效性',s.scene==='missing'?'尚未安装':s.scene==='loading'?'检查中':'位置已记录，身份另行核验']])+
      '<div class="mp-actions">'+button('诊断与恢复安装','install-open',false,true)+button('刷新位置','path-refresh')+'</div><p class="mp-muted">Desktop 与 OpenCodex 分别更新；本体更新保留签名 .app 之外的 runtime。程序路径不从配置根推导。</p>';
  }
  function pathDetails() {
    const p=controller.state().snapshot;
    dialog('完整路径',facts([['用户集合根',p.aggregateRoot],['Desktop 工作根',p.desktopRoot],['OpenCodex HOME',p.opencodexHome],['渠道配置文件',p.configPath],['安装根',p.installationRoot],['程序入口',p.runtimeEntry]])+'<label>选择文本手动复制<textarea class="input" readonly aria-label="完整路径文本">'+esc(Object.values(p).filter(v=>typeof v==='string'&&(/[\/]/.test(v))).join('\n'))+'</textarea></label><p id="o23CopyStatus" role="status">原型不访问剪贴板；可选择文本手动复制。</p>');
  }
  function selectPath() {
    const s=controller.state();
    dialog('选择新版用户目录','<label>明确的目标绝对路径<input id="o23RootTarget" class="input" value="'+esc(s.snapshot.aggregateRoot+'-selected')+'"></label><p>仅切换到已校验的新结构，不复制旧资料；确认前只读检查。</p><p id="o23PathError" role="alert"></p>','校验并预览',()=>{
      const plan=controller.prepare(root.document.getElementById('o23RootTarget').value.trim());
      if(!plan.valid){root.document.getElementById('o23PathError').textContent='请输入明确的绝对路径。';return;}
      dialog('确认使用此位置',facts([['当前生效',s.snapshot.aggregateRoot],['待生效',plan.path]])+'<p>新结构校验通过（声明 mock）。确认后记录切换意图，重启核对后生效；安装根保持。</p>','确认使用此位置',()=>{controller.confirm(plan);ui.close();render();});
    });
  }
  function mount() {
    const panel=root.document.getElementById('settings-installation'); if(!panel)return;
    const host=root.document.createElement('article');host.id='ocxd023Settings';host.className='card mp-page';panel.prepend(host);
    for(const id of ['card-install-runtime','card-runtime-source','card-install-paths','card-install-partitions','dataPathsDemo']){const old=root.document.getElementById(id);if(old){old.hidden=true;if(id==='dataPathsDemo')old.dataset.protoContexts='legacy-path-demo';}}
    const rail=root.document.getElementById('protoStdCards');if(rail){const tools=root.document.createElement('section');tools.className='proto-card';tools.dataset.protoContexts='settings:installation';tools.innerHTML='<div class="proto-note-title">0.2.23 · 目录状态演示</div>'+select('演示平台','path-platform',[['macos','macOS'],['windows','Windows']],'macos')+select('目录状态','path-scene',scenes,'valid');rail.append(tools);}
    render();
    root.document.addEventListener('change',event=>{
      const key=event.target.dataset.o23Select;
      if(key==='path-platform'){controller.useSnapshot(snapshots[event.target.value]);render();}
      if(key==='path-scene'){controller.scene(event.target.value);render();}
    });
    root.document.addEventListener('click',event=>{
      const el=event.target.closest('[data-o23]');if(!el||el.disabled)return;
      const action=el.dataset.o23;
      if(action==='path-details')pathDetails();
      if(action==='path-select')selectPath();
      if(action==='path-restart'){controller.restart();render();}
      if(action==='path-refresh')render();
      if(action==='path-init')dialog('初始化用户目录','<p>将仅初始化新版用户目录结构；不读取或搬移旧目录。配置及程序安装分别核对。</p>','确认初始化',()=>{controller.scene('valid');ui.close();render();});
      if(action==='path-json')dialog('手动恢复渠道',facts([['放入位置',controller.state().snapshot.configPath]])+'<p>将当前支持格式的 config.json 放入新 HOME，再使用“读取现有渠道”。格式错误、认证引用缺失或空结果均保留原草稿。</p>','前往读取渠道',()=>{ui.close();root.setRoute('models');});
      if(action==='install-open')root.runtimeRecovery023?.open();
    });
  }
  return {ui,snapshots,scenes,create,controller,render,mount};
});
