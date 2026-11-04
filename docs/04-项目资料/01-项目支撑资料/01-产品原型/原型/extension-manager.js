/* COL-LOCAL-20261014-01 · Extensions business view, memory mock only. */
(() => {
 'use strict';
 const $=(s,r=document)=>r.querySelector(s),$$=(s,r=document)=>[...r.querySelectorAll(s)];
 const clone=v=>JSON.parse(JSON.stringify(v));
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const route=$('#route-extensions');if(!route)return;
 const legacy=document.createElement('div');legacy.id='extensionLegacyContent';
 while(route.firstChild)legacy.append(route.firstChild);route.append(legacy);
 const host=document.createElement('div');host.id='extensionManagerContent';host.className='xp3-page';host.hidden=true;route.append(host);
 const button=(label,action,attrs='',cls='ghost')=>'<button type="button" class="btn '+cls+'" data-xp3="'+action+'" '+attrs+'>'+label+'</button>';
 const SHARED='~/.agents/skills';
 const PLATFORM_SYMBOLS={codex:'codex',claude:'claude',gemini:'gemini',grok:'grok',opencode:'opencode',hermes:'hermesagent','claude-desktop':'claude',cursor:'cursor',qoder:'qoder','qoder-cn':'qoder',openclaw:'openclaw',kimi:'kimi',dsh:'deepseek',mcode:'minimax',cline:'cline',kilo:'kilocode'};
 const PLATFORM_INITIALS={workbuddy:'WB','workbuddy-ai':'WA',pi:'Pi',omp:'OM',gajae:'GJ',zcode:'ZC',commandcode:'CC',prime:'PA',aside:'AS',raycast:'RC',omo:'om',droid:'FD','gentle-shell':'GS',t3code:'T3'};
 function platformIcon(t){const key=t.iconKey||t.id,symbol=PLATFORM_SYMBOLS[key];return symbol?{symbol:'ext-icon-'+symbol}:{text:PLATFORM_INITIALS[key]||t.name.slice(0,2)};}
 const P=(id,name,skillMode,skillPath,mcpMode,mcpPath,mcpKey,cowrite,note,evidence,extra={})=>({id,iconKey:id,name,installed:true,skillMode,skillPath,mcpMode,mcpPath,mcpKey,cowrite,note,evidence,...extra});
 const catalog=[
  P('codex','Codex','shared','~/.codex/skills（契约，待核实）','file','~/.codex/config.toml','[mcp_servers.<n>]','MCP 与 Skills 都与模型同文件','契约写 ~/.codex/skills，官方文档写 ~/.agents/skills，目录待核实','官方文档；Skills 目录待核实'),
  P('claude','Claude Code','own','~/.claude/skills','file','~/.claude.json','顶层 mcpServers','不同文件','不读 .agents；新增目录后 /reload-skills','官方文档'),
  P('gemini','Gemini CLI','shared','~/.gemini/skills','file','~/.gemini/settings.json','mcpServers','不共写（OpenCodex 不写 Gemini）','另有 enablement 文件；不在 OpenCodex 连接概览','官方文档'),
  P('grok','Grok Build','shared','~/.grok/skills','file','~/.grok/config.toml','[mcp_servers.<n>]','MCP 与模型同文件','用户级；兼容读取 ~/.claude.json、.mcp.json','官方文档'),
  P('opencode','OpenCode','shared','~/.config/opencode/skills','file','~/.config/opencode/opencode.json','mcp.<n>（V1）/ mcp.servers.<n>（V2）','MCP 与模型同文件','','官方文档'),
  P('hermes','Hermes Agent','own','~/.hermes/skills','file','~/.hermes/config.yaml','mcp_servers','MCP 与 Skills 都与模型同文件','只在项目级读 .agents/skills，需 hermes skills trust','官方文档'),
  P('claude-desktop','Claude Desktop','guide','Customize > Skills 上传 ZIP','file','~/Library/Application Support/Claude/claude_desktop_config.json','mcpServers','待核实（3p 模式推断写 Claude-3p/）','改 MCP 后要完全退出再打开','官方文档；3p 路径推断'),
  P('cursor','Cursor','shared','~/.cursor/skills','file','~/.cursor/mcp.json','mcpServers','双重管理：OpenCodex provider.mcpServers','以哪份为准待评审','官方文档、源码核实'),
  P('qoder','Qoder','own','~/.qoder/skills','file','~/.qoder/settings.json','mcpServers','MCP 与模型同文件','','官方文档'),
  P('qoder-cn','Qoder CN','unknown','推断 ~/.qoder-cn/skills','file','~/.qoder-cn/settings.json','mcpServers','MCP 与模型同文件','~/.qoderwork 属于另一个产品','MCP 官方文档；Skills 推断'),
  P('workbuddy','WorkBuddy','unknown','~/.workbuddy/skills（二手来源）','file','~/.workbuddy/mcp.json','mcpServers','同目录不同文件','启用状态随账号同步','MCP 官方文档；Skills 二手来源'),
  P('workbuddy-ai','WorkBuddy AI','unknown','推断 ~/.workbuddy-ai/skills','unknown','推断 ~/.workbuddy-ai/mcp.json','mcpServers','同目录不同文件','路径全部推断','推断'),
  P('pi','Pi','shared','~/.pi/agent/skills','file','~/.pi/agent/mcp.json','mcpServers','同目录不同文件','MCP 0.99.0 起内置；!command 读到时标风险，Desktop 不生成','官方文档'),
  P('omp','OMP / Oh My Pi','own','~/.omp/agent/skills','file','~/.omp/agent/mcp.json','mcpServers','同目录不同文件','读 agents 目录受 enable 开关控制；覆盖变量与 Pi 同名','官方文档、源码核实'),
  P('openclaw','OpenClaw','shared','~/.openclaw/skills','file','~/.openclaw/openclaw.json','mcp.servers（JSON5）','MCP 与 Skills 都与模型同文件','共享目录只在默认 state 下读','官方文档'),
  P('kimi','Kimi Code','shared','$KIMI_CODE_HOME/skills','file','$KIMI_CODE_HOME/mcp.json','mcpServers','同目录不同文件','home 默认推断为 ~/.kimi-code','官方文档'),
  P('gajae','Gajae Code / gjc','own','~/.gjc/agent/skills','file','~/.gjc/agent/mcp.json','mcpServers','同目录不同文件','不读 agents 目录，只能 /extensions 导入','官方文档',{installed:false}),
  P('dsh','DeepSeek Harness','shared','~/.dsh/skills','file','$DSH_HOME/cordis.patch.yml','插件条目数组','MCP 与模型同文件（同一 patch 文件）','没有开关字段，停用即删除，需保存原条目；平台自己加锁','官方文档'),
  P('mcode','MiniMax Code','shared','~/.minimax/skills','file','~/.minimax/mcp.json','mcpServers','Skills 与模型同文件（config.yaml skills.external）','项目级 .mcp.json 只读','源码核实'),
  P('zcode','ZCode','own','~/.zcode/skills','file','~/.zcode/cli/config.json','mcp.servers','同目录不同文件','改完要在设置页手动刷新','官方文档',{installed:false}),
  P('commandcode','Command Code','shared','~/.commandcode/skills','file','~/.commandcode/mcp.json','mcpServers','同目录不同文件','平台自己加锁','官方文档'),
  P('prime','Prime Agent','own','~/.prime/agent/skills','file','~/.prime/agent/settings.json','mcpServers','同目录不同文件','没有项目级；平台自己加锁；Skill 可能是 Python 包','源码核实'),
  P('aside','Aside','unknown','待核实','unknown','待核实','—','待核实','app 运行时会改写配置，写入前要完全退出','待核实'),
  P('raycast','Raycast','shared','~/.config/raycast/skills','guide','只能在 Raycast 内配置','—','不同文件；只写 Skills 目录','目录名必须等于 frontmatter name，只扫顶层','官方文档'),
  P('omo','omo','unknown','待核实','unknown','待核实','—','待核实','一个名字对应 senpi、LazyCodex、oh-my-opencode 三个产品','待核实'),
  P('cline','Cline CLI','own','~/.cline/skills','file','~/.cline/data/settings/cline_mcp_settings.json','mcpServers','同目录不同文件','不读 .agents','官方文档'),
  P('kilo','Kilo','shared','~/.kilo/skills','file','~/.config/kilo/kilo.json[c]','mcp.<n>','MCP 与模型同文件','新会话生效','官方文档'),
  P('droid','Factory Droid','shared','~/.factory/skills','file','~/.factory/mcp.json','mcpServers','Skills 与模型同文件（settings.json disabledSkills）','切换项目级 server 会在用户级留副本','官方文档'),
  P('gentle-shell','Gentle Shell','unknown','推断 ~/.gentle-shell/agent/skills','unknown','所选 home 下 mcp.json','mcpServers','link 模式与 Pi 共用 home','link 模式下是 Pi 的别名，不能当两个目标','官方文档；路径待核实'),
  P('t3code','T3 Code','follow','跟随 Claude Code / Codex','follow','跟随 Claude Code / Codex','—','跟随 provider，本包不写','用 -c 注入的条目不落文件','官方文档')
 ];
 const seed=[
  {id:'design',kind:'skills',name:'design-studio',description:'产品设计、界面原型与设计检查；按任务加载必要参考。',source:'本地源库',version:'v1.8.0',updated:'2026-09-13',selected:['claude','hermes'],projections:{claude:'synced',hermes:'failed'},shared:true,sharedState:'synced'},
  {id:'development',kind:'skills',name:'development-knowledge',description:'编程语言、框架与实现模式的开发参考。',source:'本地源库',version:'v2.3.1',updated:'2026-09-12',selected:['claude'],projections:{claude:'synced'},shared:true,sharedState:'stale'},
  {id:'governance',kind:'skills',name:'project-governance',description:'已绑定项目的需求、事实、协作资料与回写管理。',source:'本地源库',version:'v1.5.0',updated:'2026-09-09',selected:['claude','hermes'],projections:{claude:'synced',hermes:'synced'},shared:false,sharedState:'synced'},
  {id:'management',kind:'skills',name:'project-management',description:'推进项目、协调阶段、验证并整理交付状态。',source:'本地源库',version:'v1.2.4',updated:'2026-09-08',selected:['hermes'],projections:{hermes:'synced'},shared:true,sharedState:'synced'},
  {id:'audit',kind:'skills',name:'skill-audit',description:'检查 Skill 的触发范围、重复措辞和维护成本。',source:'Git 仓库',version:'v1.1.3',updated:'2026-09-07',selected:[],projections:{},shared:true,sharedState:'synced'},
  {id:'docs',kind:'skills',name:'document-review',description:'整理文档结构与评审意见，保留来源和决定。',source:'Git 仓库',version:'v0.3.0',updated:'2026-09-05',selected:[],projections:{},shared:false,sharedState:'synced'},
  {id:'pets',kind:'skills',name:'hatch-pet',description:'制作和检查 Codex 动画宠物资源。',source:'本地源库',version:'v0.4.2',updated:'2026-09-10',selected:['claude'],projections:{claude:'synced'},shared:true,sharedState:'synced'},
  {id:'filesystem',kind:'mcp',name:'文件系统',description:'为 Agent 提供限定目录的文件读取工具。',source:'受管定义',version:'stdio',updated:'2026-09-22',selected:['codex','claude','gemini'],projections:{codex:'synced',claude:'synced',gemini:'failed'},definition:{command:'node',args:['/example/mcp-filesystem.js','/example/workspace'],env:['WORKSPACE_ROOT']}},
  {id:'documentation',kind:'mcp',name:'开发文档',description:'查询框架与库的参考文档。',source:'受管定义',version:'HTTP',updated:'2026-09-21',selected:['codex'],projections:{codex:'synced'},definition:{url:'https://example.com/mcp',headers:['Authorization（环境变量引用）']}},
  {id:'browser',kind:'mcp',name:'浏览器工具',description:'检查网页内容与界面状态。',source:'受管定义',version:'stdio',updated:'2026-09-20',selected:['codex'],projections:{codex:'synced'},definition:{command:'node',args:['/example/mcp-browser.js'],env:[]}},
  {id:'git',kind:'mcp',name:'Git 只读查询',description:'查看仓库状态与提交记录。',source:'受管定义',version:'stdio',updated:'2026-09-19',selected:[],projections:{},definition:{command:'node',args:['/example/mcp-git.js'],env:[]}}
 ];
 seed.push({id:'node_repl',kind:'mcp',name:'node_repl',description:'由客户端应用维护的 Node 运行环境。',source:'Codex 桌面应用',version:'stdio',updated:'2026-11-04',selected:['codex'],projections:{codex:'synced'},appMaintained:true,definition:{command:'node',args:['/example/app/node-repl.js'],env:[]}});

 let resources=clone(seed),kind='skills',mode='resource',current={resource:'design',client:'codex'},query={left:'',right:''},capacity='30',scene='normal',sceneNotice='',actionNotice='';
 let directoryOS='mac',directoryScene='occupied',versionScene='normal',platformDialog=null,dialogEpoch=0,discoveryDialog=false,detectionScene='normal',manualScene='normal';
 const tabs={skills:'Skills',mcp:'MCP'},initialMapped=new Set(['codex','gemini','grok','opencode','cursor','openclaw','kimi']);
 const mappings={mac:{},windows:{}},versions={},installations={mac:{},windows:{}},detection={state:'idle',message:'',completed:false};
 let detectionEpoch=0;
 const localBindings={mac:{filesystem:{entry:'/example/mcp-filesystem.js',workspace:'/example/workspace'},browser:{entry:'/example/mcp-browser.js'},git:{entry:'/example/mcp-git.js'},node_repl:{entry:'/example/app/node-repl.js'}},windows:{}};
 const syncedContent={},receivedExtensions=new Set();
 const active=()=>document.documentElement.dataset.route==='extensions';
 const isResource=()=>mode==='resource';
 const compareNames=(a,b)=>a.name.localeCompare(b.name,'zh-CN',{numeric:true,sensitivity:'base'})||a.id.localeCompare(b.id);
 function sortPlatforms(list){return [...list].sort((a,b)=>Number(installed(b))-Number(installed(a))||compareNames(a,b));}
 function targets(){const base=capacity==='6'?catalog.slice(0,6):catalog;return sortPlatforms(capacity==='50'?[...base,...Array.from({length:20},(_,i)=>({...catalog[i%6],id:'sample-'+i,name:catalog[i%6].name+' · 容量样例 '+(i+1),skillMode:'unknown',mcpMode:'unknown',skillPath:'模拟实例',mcpPath:'模拟实例',mcpKey:'—',cowrite:'—',note:'容量样例',evidence:'容量样例 · 未核实真实实例',sample:true}))]:base);}
 const target=id=>targets().find(t=>t.id===id)||catalog.find(t=>t.id===id);
 const resource=id=>resources.find(r=>r.id===id);
 const sourcePath=()=>directoryOS==='windows'?'%USERPROFILE%\\.agents\\skills':SHARED;
 const pathForOS=p=>directoryOS==='windows'?p.replace(/^~\//,'%USERPROFILE%\\').replace(/\$([A-Z_]+)/g,'%$1%').replaceAll('/','\\'):p;
 function originalPath(t,k='skills'){
  if(!t||t.sample)return '';
  let p=(k==='skills'?t.skillPath:t.mcpPath).replace(/^推断\s*/,'').replace(/（.*?）/g,'');
  if(!/^(~\/|\$)/.test(p))return '';
  if(k==='skills'&&t.id==='kimi')p='~/.kimi-code/skills';
  if(directoryOS==='windows'&&t.id==='claude-desktop'&&k==='mcp')return '%APPDATA%\\Claude\\claude_desktop_config.json';
  return pathForOS(p);
 }

 const absentFixtures=new Set(['gajae','zcode','workbuddy-ai','aside','omo','gentle-shell']);
 function defaultInstallPath(t){
  if(directoryOS==='windows')return '%LOCALAPPDATA%\\Programs\\'+t.name.replace(/[^a-zA-Z0-9]/g,'')+'\\'+t.id+'.exe';
  return ['claude-desktop','cursor','qoder','workbuddy','zcode'].includes(t.id)?'/Applications/'+t.name+'.app':'/usr/local/bin/'+t.id;
 }
 function installation(t){
  if(!t)return {installed:false,path:'',source:'none',draft:'',state:'idle',message:'',verified:false};
  const store=installations[directoryOS];
  if(!store[t.id]){const found=t.installed&&!t.sample&&!absentFixtures.has(t.id);store[t.id]={installed:found,path:found?defaultInstallPath(t):'',source:found?'auto':'none',draft:'',state:'idle',message:'',verified:false,editing:false};}
  return store[t.id];
 }
 const installed=t=>!!t&&installation(t).installed;
 function installationLabel(t){const i=installation(t);return i.installed?(i.source==='manual'?'手动加入':'已发现'):'未发现';}
 function installationSection(t){
  const i=installation(t),busy=i.state==='validating',editing=i.editing||!i.installed;
  return '<section class="xp3-platform-section" id="xp3InstallSection"><div class="xp3-section-heading"><h3>安装与检测</h3><span>'+installationLabel(t)+'</span></div>'
   +(i.installed?'<div class="xp3-install-current"><span>'+esc(i.path)+'</span><button class="btn ghost" type="button" data-platform-action="edit-install">更改路径</button></div>':'<p class="xp3-feedback">未检测到 '+esc(t.name)+'，暂不参与 Skills 和 MCP 管理。若已安装在其他位置，可手动指定路径。</p>')
   +(editing?'<label class="xp3-install-label" for="xp3InstallPath">'+(directoryOS==='windows'?'安装程序路径':'应用或可执行文件路径')+'</label><div class="xp3-install-input"><input class="input" id="xp3InstallPath" type="text" autocomplete="off" spellcheck="false" value="'+esc(i.draft)+'" placeholder="'+(directoryOS==='windows'?'C:\\Users\\用户名\\AppData\\Local\\Programs\\平台\\平台.exe':'/Applications/平台.app 或 /usr/local/bin/平台')+'" aria-describedby="xp3InstallFeedback" '+(busy?'disabled':'')+'><button type="button" class="btn ghost" data-platform-action="browse-install" '+(busy?'disabled':'')+'>浏览…</button></div><p class="xp3-feedback" id="xp3InstallFeedback" role="status" data-tone="'+(i.state==='error'?'danger':'neutral')+'">'+esc(i.message||'选择已安装的应用或程序，验证平台身份后再加入管理。')+'</p><div class="xp3-detail-actions"><button type="button" class="btn ghost" data-platform-action="validate-install" '+(busy?'disabled':'')+'>'+(busy?'验证中…':'验证路径')+'</button>'+(i.verified?'<button type="button" class="btn" data-platform-action="save-install">'+(i.installed?'保存路径':'加入管理')+'</button>':'')+(i.installed?'<button type="button" class="btn ghost" data-platform-action="cancel-install">取消</button>':'')+'</div>':'')+'</section>';
 }
 function validInstallPath(value){return directoryOS==='windows'?/^(?:[A-Za-z]:\\|\\\\[^\\]+\\[^\\]+)/.test(value):/^\/(?!\/)/.test(value);}
 function validateInstall(t){
  const i=installation(t);if(i.state==='validating')return;
  i.draft=$('#xp3InstallPath')?.value.trim()||i.draft.trim();i.verified=false;
  if(!validInstallPath(i.draft)){i.state='error';i.message=directoryOS==='windows'?'请输入 Windows 绝对路径，例如 C:\\Programs\\平台\\平台.exe。':'请输入 macOS 绝对路径，例如 /Applications/平台.app。';redrawPlatform();return;}
  const epoch=dialogEpoch,os=directoryOS,value=i.draft;i.state='validating';i.message='正在验证程序与平台身份…';redrawPlatform();
  setTimeout(()=>{if(epoch!==dialogEpoch||os!==directoryOS){i.state='idle';return;}i.state=manualScene==='normal'?'verified':'error';i.verified=manualScene==='normal';i.message=manualScene==='missing'?'找不到该路径，请确认平台已安装。':manualScene==='denied'?'没有访问该路径的权限，请更换路径或检查权限。':manualScene==='mismatch'?'程序身份与 '+t.name+' 不一致，请选择正确的平台。':'已验证为 '+t.name+'，版本 1.2.0。点击“'+(i.installed?'保存路径':'加入管理')+'”后生效。';i.verifiedPath=value;redrawPlatform();},380);
 }
 function saveInstall(t){
  const i=installation(t);if(!i.verified||i.draft!==i.verifiedPath||manualScene!=='normal')return;
  const previous=i.path,wasInstalled=i.installed;i.installed=true;i.path=i.verifiedPath;i.source='manual';i.editing=false;i.verified=false;i.state='idle';i.message='';
  if(!wasInstalled||previous!==i.path){const m=mapping(t);m.mapped=false;m.checked=false;m.problem='';}
  version(t).current='1.2.0';actionNotice=t.name+' 已加入管理。'+(capabilityBlock(t,'skills')?'':mapping(t).mapped?'':'请继续完成 Skills 目录映射。');render();redrawPlatform();document.dispatchEvent(new CustomEvent('prototype:platform-availability'));
 }
 function detectionHTML(){
  const busy=detection.state==='running';return '<div class="xp3-detection"><p class="xp3-feedback" role="status" data-tone="'+(detection.state==='error'?'danger':'neutral')+'">'+esc(detection.message||'检测常见安装位置与命令路径。未发现的平台可手动指定安装路径。')+'</p><div class="xp3-detail-actions"><button type="button" class="btn ghost" data-detect-action="scan" '+(busy?'disabled':'')+'>'+(busy?'检测中…':'重新检测')+'</button></div><ul class="xp3-detection-list">'+sortPlatforms(catalog).map(t=>{const i=installation(t);return '<li><div><strong>'+esc(t.name)+'</strong><span>'+esc(i.path||'未发现安装位置')+'</span></div><span class="xp3-detected-status" data-found="'+i.installed+'">'+installationLabel(t)+'</span><button type="button" class="btn ghost" data-detect-action="configure" data-id="'+esc(t.id)+'" '+(busy?'disabled':'')+'>'+(i.installed?'查看':'配置路径')+'</button></li>';}).join('')+'</ul></div>';
 }
 function redrawDetection(){if(discoveryDialog&&$('.xp3-detection')){const top=$('#modalBody').scrollTop;$('#modalBody').innerHTML=detectionHTML();$('#modalBody').scrollTop=top;}}
 function scanPlatforms(){
  if(detection.state==='running')return;const epoch=++detectionEpoch,os=directoryOS;detection.state='running';detection.message='正在检测平台安装位置…';render();redrawDetection();
  setTimeout(()=>{
   if(epoch!==detectionEpoch||os!==directoryOS)return;
   if(detectionScene==='failed'){detection.state='error';detection.message='检测失败，已保留上次结果。请重试，或打开平台详情手动配置路径。';}
   else{let found=0;for(const t of catalog){const i=installation(t);if(i.source==='manual'){found++;continue;}i.installed=!absentFixtures.has(t.id)&&t.installed&&!(detectionScene==='codex-missing'&&t.id==='codex');i.path=i.installed?defaultInstallPath(t):'';if(i.installed)found++;}detection.state='ready';detection.completed=true;detection.message='检测完成：发现 '+found+' 个平台，'+(catalog.length-found)+' 个未发现。未发现的平台暂不参与管理。';}
   actionNotice=detection.message;render();redrawDetection();document.dispatchEvent(new CustomEvent('prototype:platform-availability'));
  },650);
 }
 function openDetection(){
  if($('#modalMask').style.display==='flex')closeModal();discoveryDialog=true;
  openModal('检测平台',detectionHTML(),'',null,{hideConfirm:true,prototypeAnnotation:{title:'平台检测 · 内存样例',body:'检测结果、安装路径、程序身份与版本均为虚构数据，不扫描真实文件、不执行程序。真实检测应使用各平台适配器并验证程序身份，失败保留上次结果。手动路径须验证并确认后加入管理，不安装软件。'}});$('#modal').classList.add('xp3-platform-modal','modal-wide');$('#modalBody').scrollTop=0;
  window.__modalOnClose=()=>{discoveryDialog=false;detectionEpoch++;if(detection.state==='running'){detection.state='idle';detection.message='检测已取消，保留上次结果。';}$('#modal').classList.remove('xp3-platform-modal');render();};scanPlatforms();
 }
 document.addEventListener('click',e=>{const b=e.target.closest('.xp3-detection [data-detect-action]');if(!b||b.disabled)return;if(b.dataset.detectAction==='scan')scanPlatforms();else{const t=target(b.dataset.id);openClient(t);installation(t).editing=!installed(t);redrawPlatform();$('#xp3InstallSection')?.scrollIntoView({block:'nearest'});}});
 document.addEventListener('input',e=>{if(e.target.id!=='xp3InstallPath'||!platformDialog)return;const i=installation(target(platformDialog));i.draft=e.target.value;i.verified=false;i.state='idle';i.message='路径已更改，请重新验证。';$('.xp3-platform-detail [data-platform-action="save-install"]')?.remove();const feedback=$('#xp3InstallFeedback');if(feedback)feedback.textContent=i.message;});
 function capabilityBlock(t,k=kind){
  if(!t)return {short:'未选择平台',full:'请先选择平台'};
  if(t.sample)return {short:'待核实',full:t.evidence};
  const m=k==='skills'?t.skillMode:t.mcpMode,p=k==='skills'?t.skillPath:t.mcpPath;
  if(!installed(t))return {short:'未发现',full:'本机未发现 '+t.name};
  if(m==='unknown')return {short:'待核实',full:'平台目录与支持情况待核实，暂不能操作'};
  if(m==='guide')return {short:'应用内配置',full:k==='skills'?'请在 '+t.name+' 的 '+p+' 配置':p};
  if(m==='follow')return {short:'跟随来源',full:p+'，请管理对应来源平台'};
  return null;
 }
 function mapping(t){
  const store=mappings[directoryOS];
  if(!store[t.id])store[t.id]={mapped:!t.sample&&initialMapped.has(t.id),checked:false,checking:false,problem:'',busy:false};
  return store[t.id];
 }
 function block(t,k=kind){const reason=capabilityBlock(t,k);if(reason)return reason;return k==='skills'&&!mapping(t).mapped?{short:'待映射',full:'请先在平台详情中映射 Skills 目录'}:null;}
 function landing(t,k=kind){return originalPath(t,k)||'—';}
 function modeLabel(t,k=kind){if(!installed(t))return '未发现';if(k==='skills')return mapping(t).mapped?'已映射':'待映射';return capabilityBlock(t,k)?.short||'配置文件';}
 const viaShared=(r,t)=>r.kind==='skills'&&mapping(t).mapped;
 const isOn=(r,t)=>!installed(t)?false:r.kind==='skills'?(mapping(t).mapped&&!!r.shared):r.selected.includes(t.id);
 function setOn(r,t,v){
  if(block(t,r.kind)||(r.kind==='mcp'&&missingBindings(r)))return '';
  if(r.kind==='skills'){if(!!r.shared!==v){r.shared=v;r.sharedState='synced';return 'shared';}return '';}
  if(v&&!r.selected.includes(t.id)){r.selected.push(t.id);r.projections[t.id]='synced';}
  if(!v&&r.selected.includes(t.id)){r.selected=r.selected.filter(id=>id!==t.id);delete r.projections[t.id];}
  return '';
 }
 const sharedTargets=()=>catalog.filter(t=>mapping(t).mapped&&!capabilityBlock(t,'skills'));
 function sharedNotice(names,v){return names.length?(v?'已启用 ':'已停用 ')+names.join('、')+'。已映射的 '+sharedTargets().length+' 个平台共用 '+sourcePath()+'，同步生效。':'';}
 const relStale=(r,t)=>viaShared(r,t)?['stale','failed'].includes(r.sharedState):['stale','failed'].includes(r.projections[t.id]);
 function objects(){return isResource()?[resource(current.resource)].filter(Boolean):[target(current.client)].filter(Boolean);}
 function leftItems(){const q=query.left.toLowerCase(),list=isResource()?resources.filter(r=>r.kind===kind).sort(compareNames):targets().map(t=>({...t,description:landing(t)}));return list.filter(r=>(r.name+' '+(r.description||'')).toLowerCase().includes(q));}
 function rightItems(){
  if(scene==='empty')return [];
  const chosen=objects();if(!isResource()&&block(chosen[0]))return [];
  const q=query.right.toLowerCase(),list=isResource()?targets().map(t=>({...t,description:landing(t)})):resources.filter(r=>r.kind===kind).sort(compareNames);
  return list.filter(t=>(t.name+' '+(t.description||'')).toLowerCase().includes(q)).map(t=>{
   const r=isResource()?chosen[0]:t,c=isResource()?t:chosen[0],b=block(c)||(r?.kind==='mcp'&&missingBindings(r)?{short:'路径待配置',full:'请先配置本机 MCP 入口与工作目录'}:null),on=r&&c?isOn(r,c):false;
   return {...t,selected:on,status:isResource()&&!installed(t)?'未发现':isResource()&&kind==='skills'?(mapping(t).mapped?'已映射':'待映射'):b?b.short:relStale(r,c)?'待更新':on?'已启用':'未启用',tone:b?'warning':'neutral',blocked:b?.full||''};
  });
 }
 const noticeText=()=>[sceneNotice,actionNotice].filter(Boolean).join(' ');
 function render(){
  if(!active())return;
  const focus=document.activeElement,id=focus?.id,pos=focus?.selectionStart,scrolls=[$('#xp3LeftRows')?.scrollTop||0,$('#xp3RightRows')?.scrollTop||0];
  host.innerHTML='<div class="xp3-mode"><div class="xp3-types" role="tablist" aria-label="扩展类型">'+Object.entries(tabs).map(([id,text])=>'<button type="button" role="tab" aria-selected="'+(id===kind)+'" tabindex="'+(id===kind?'0':'-1')+'" class="xp3-kind-btn" data-xp3="kind" data-value="'+id+'" aria-pressed="'+(kind===id)+'">'+text+'</button>').join('')+'</div><div class="xp3-tools">'+button(detection.state==='running'?'检测中…':'检测平台','detect',detection.state==='running'?'disabled':'')+button('资源工具','tools')+button('设置','settings')+'</div><div class="xp3-switch" role="tablist" aria-label="扩展管理视图">'+[['resource','按扩展'],['client','按平台']].map(([m,t])=>'<button type="button" role="tab" aria-selected="'+(m===mode)+'" tabindex="'+(m===mode?'0':'-1')+'" class="xp3-mode-btn" data-xp3="mode" data-value="'+m+'" aria-pressed="'+(mode===m)+'">'+t+'</button>').join('')+'</div></div>'+annotation()+(noticeText()?'<div class="xp3-note" role="status">'+esc(noticeText())+'</div>':'')+pendingHTML()+'<div class="ic-workbench xp3-workbench"><div class="ic-panes"><section class="ic-pane" aria-label="左侧对象列表"><header class="ic-pane-head"><div class="ic-pane-caption"><h4>'+(isResource()?tabs[kind]:'平台')+'</h4><span>'+leftItems().length+' 项</span></div><input class="input" id="xp3LeftSearch" type="search" aria-label="搜索左侧对象" placeholder="搜索名称" value="'+esc(query.left)+'"><div class="ic-context" id="xp3LeftContext"></div></header><div id="xp3LeftRows" class="ic-collection"></div></section><section class="ic-pane" aria-label="右侧关系表格"><header class="ic-pane-head"><div class="ic-pane-caption"><h4>'+(isResource()?'平台':tabs[kind])+'</h4><span>'+rightItems().length+' 项</span></div><input class="input" id="xp3RightSearch" type="search" aria-label="'+(isResource()?'搜索平台':'搜索 '+tabs[kind])+'" placeholder="搜索名称或路径" value="'+esc(query.right)+'"><div class="ic-context" id="xp3RightContext"></div></header><div id="xp3RightRows" class="ic-collection"></div></section></div><footer class="ic-footer"><div class="ic-footer-status" id="xp3Status" role="status"><strong>'+(kind==='skills'?'Skills 源目录':'启用关系')+'</strong> · '+(kind==='skills'?esc(sourcePath())+' · '+sharedTargets().length+' 个平台已映射':'列表与详情状态同步')+'</div><div class="ic-footer-actions">'+button('设置','settings')+'</div></footer></div>';
  mountCollections();$('#xp3LeftRows').scrollTop=scrolls[0];$('#xp3RightRows').scrollTop=scrolls[1];window.prototypeSelect?.enhance();window.prototypeStickyTabs?.refresh();window.prototypePanel?.refresh();
  if(id){const el=$('#'+id);if(el){el.focus({preventScroll:true});if(pos!=null&&el.type==='search')el.setSelectionRange(pos,pos);}}
 }
 function mountCollections(){
  const left=leftItems(),right=rightItems(),chosen=objects(),mappingTable=isResource()&&kind==='skills';
  $('#xp3LeftContext').textContent=isResource()?'单击切换，名称查看详情':kind==='skills'?'未映射平台仅可查看详情':'单击切换平台';
  $('#xp3RightContext').innerHTML='<span>'+(isResource()?'当前扩展：':'当前平台：')+'</span><strong>'+esc(chosen[0]?.name||'尚未选择')+'</strong>'+bulkButtons(right,chosen);
  window.PrototypeInteractiveCollection.mount($('#xp3LeftRows'),{kind:'list',label:isResource()?'扩展对象列表':'平台列表',selection:false,current:isResource()?current.resource:current.client,rows:left.map(r=>({id:r.id,label:r.name,description:r.description,descriptionAction:!isResource()&&originalPath(r,kind)?'path':null,status:isResource()?r.version:modeLabel(r),tone:!isResource()&&block(r)?'warning':'neutral',disabled:!isResource()&&!!block(r),actions:[]})),onActivate(r){if(isResource())current.resource=r.id;else if(!block(target(r.id)))current.client=r.id;render();},onDetail(r){isResource()?openResource(resource(r.id)):openClient(target(r.id));},onAction(r,a){if(a==='path')openPath(target(r.id),kind);else if(a==='detail')openClient(target(r.id));}});
  const locked=!isResource()&&block(chosen[0]);
  window.PrototypeInteractiveCollection.mount($('#xp3RightRows'),{kind:'table',label:isResource()?'平台关系':tabs[kind]+' 启用关系',selection:false,pathColumn:isResource(),statusLabel:mappingTable?'目录映射':'状态',emptyLabel:locked?(!installed(chosen[0])?'平台未发现，请打开平台详情配置安装路径。':locked.full):undefined,rows:right.map(r=>{
   const pending=mappingTable&&!mapping(r).mapped,unsupported=mappingTable&&capabilityBlock(r,'skills');
   return {id:r.id,label:r.name,icon:isResource()?platformIcon(r):null,description:isResource()?installationLabel(r):r.description,path:isResource()?originalPath(r,kind):null,pathAction:'path',selected:r.selected,unavailable:isResource()&&!installed(r),disabled:!!r.blocked,status:r.status,statusAction:pending&&installed(r)?'map':null,statusReason:unsupported?.full||'将共享 Skills 源映射到平台原目录',statusDisabled:!!unsupported&&installed(r),tone:r.tone,actions:isResource()&&!installed(r)?[{id:'install',label:'配置路径'}]:[{id:'detail',label:'查看详情',icon:'ⓘ'},...(!pending?[{id:'toggle',kind:'toggle',label:r.selected===true?'停用':'启用',disabled:!!r.blocked,reason:r.blocked}]:[])]};
  }),onDetail(r){if(isResource())openClient(target(r.id));else if(!block(chosen[0]))openResource(resource(r.id));},onAction(r,a){if(a==='path'){openPath(target(r.id),kind);return;}if(a==='map'){openClient(target(r.id),true);return;}if(a==='install'){openClient(target(r.id));return;}if(a==='detail'){isResource()?openClient(target(r.id)):!block(chosen[0])&&openResource(resource(r.id));return;}if(a==='toggle')toggleRelationship([r.id],r.selected!==true);},onRetry(){setScene('normal');}});
 }
 function bulkIds(rows,value){return rows.filter(r=>!r.blocked&&(value?r.selected!==true:r.selected!==false)).map(r=>r.id);}
 function bulkButtons(rows,chosen){
  const one=(value,id,text)=>{const ids=bulkIds(rows,value),reason=!chosen.length?'尚未选择对象':!ids.length?'当前没有可'+(value?'启用':'停用')+'的项':'',tip=reason||(kind==='skills'?'更改共享源，对所有已映射平台生效':'作用于当前搜索结果');return '<span class="xp3-bulk-item" title="'+esc(tip)+'">'+button(text,id,'id="xp3'+(value?'EnableAll':'DisableAll')+'" aria-label="'+esc(text+'：'+tip)+'"'+(reason?' disabled':''))+'</span>';};
  return '<span class="xp3-bulk" role="group" aria-label="批量启用关系">'+one(true,'enable-all','全部启用')+one(false,'disable-all','全部停用')+'</span>';
 }
 function toggleRelationship(ids,value){const changed=[];for(const item of rightItems().filter(t=>ids.includes(t.id)&&!t.blocked))for(const o of objects()){const r=isResource()?o:resource(item.id),c=isResource()?target(item.id):o;if(!r||!c)continue;if(setOn(r,c,value)==='shared'&&!changed.includes(r.name))changed.push(r.name);}actionNotice=sharedNotice(changed,value);render();}
 function annotation(){return '<div class="proto-annotation" id="ann-xp3-scope" data-annotation-number="01" data-annotation-title="多平台扩展 · 映射管理" data-annotation-category="COL-LOCAL-20261014-01"><button type="button" class="annotation-marker" aria-expanded="false" aria-controls="annotationPopover" aria-label="原型注释 01：多平台扩展 · 映射管理"><span>原型</span><span>01</span><span>＋</span></button><div class="annotation-source" hidden><p>左侧单选，无复选框和更多按钮，标题进入详情。未发现的平台置灰，先检测或验证安装路径后加入管理。按扩展时右表平台图标跨名称与安装状态两行，原始路径单独一列；按平台时左侧纯文字。Skills 必须先映射目录，原目录有文件、其他链接或无权限时均阻止映射，先备份并清空后重新检查。所有映射平台共用源目录，启停同步生效。版本、目录和文件管理器均为内存样例，Windows 路径是演示表示，未核实的平台保持不可操作；不会执行命令或读写真实配置。</p></div></div>';}
 function sampleData(role){const sample=PrototypeExtensionDetail.sample(role.kind,role.name)||{},def=resolvedDefinition(role);if(role.kind==='mcp')return {...sample,command:def.command||def.url||'—',args:Array.isArray(def.args)?def.args.join(' '):'—',env:(def.env||def.headers||[]).join('、')||'—',landing:mcpLanding(role),md:'## 完整配置\n\n'+String.fromCharCode(96).repeat(3)+'json\n'+JSON.stringify(def,null,2)+'\n'+String.fromCharCode(96).repeat(3)};return syncedContent[role.id]?{...sample,md:syncedContent[role.id].find(f=>f.path==='SKILL.md')?.text||''}:sample;}
 function mcpLanding(role){const t=!isResource()?target(current.client):(targets().find(x=>!x.sample&&role.selected.includes(x.id)&&!block(x,'mcp'))||target('codex'));return block(t,'mcp')?.full||originalPath(t,'mcp')+' · '+t.mcpKey+'（'+t.name+'）';}
 function openResource(role){
  if(!role||(!isResource()&&block(target(current.client),role.kind)))return;
  const returnFocus=document.activeElement;
  window.PrototypeExtensionDetail.open({kind:role.kind,name:role.name,description:role.description,source:role.source,version:role.version,updated:role.updated,sourcePath:role.kind==='skills'?sourcePath()+(directoryOS==='windows'?'\\':'/')+role.name:undefined,facts:sampleData(role)},{getTargets:()=>targets().map(t=>{const b=block(t,role.kind)||(role.kind==='mcp'&&missingBindings(role)?{short:'路径待配置',full:'请先配置本机 MCP 路径'}:null),on=isOn(role,t);return {id:t.id,label:t.name,enabled:on,state:b?b.short:on?'已启用':'未启用',disabled:!!b,reason:b?.full||(role.kind==='skills'?'从 '+sourcePath()+' 读取':'')};}),onToggleTarget(id,v){const t=target(id);if(!t||block(t,role.kind))return;actionNotice=setOn(role,t,v)==='shared'?sharedNotice([role.name],v):'';render();},onAction(action){legacyAction(role.kind,action,role.name);},onClose(){if(returnFocus?.isConnected)returnFocus.focus({preventScroll:true});}});
 }
 function legacyAction(kind,action,name){const list=kind==='mcp'?'#mcpList':'#skillsList',row=[...legacy.querySelectorAll(list+' .skill-row')].find(r=>r.querySelector('.skill-title h3')?.textContent.trim()===name),selector=kind==='mcp'?(action==='edit'?'[data-prototype-action="mcp-edit"]':'[data-prototype-action="mcp-delete"]'):(action==='update'?'[data-prototype-action="skills-update"]':'[data-prototype-action="skills-uninstall"]');row?.querySelector(selector)?.click();}
 function iconHTML(t){const i=platformIcon(t);return '<span class="xp3-brand" aria-hidden="true">'+(i.symbol?'<svg viewBox="0 0 24 24"><use href="#'+i.symbol+'"/></svg>':'<span>'+esc(i.text)+'</span>')+'</span>';}
 function pathButton(path,part='skills',label){return path?'<button type="button" class="xp3-path" data-platform-action="path" data-part="'+part+'" title="'+esc(path)+'" aria-label="打开路径：'+esc(path)+'">'+esc(label||path)+'<span aria-hidden="true"> ↗</span></button>':'<span class="xp3-muted">目录待核实</span>';}
 function version(t){if(!versions[t.id])versions[t.id]={current:'',latest:'',state:'idle',message:'',confirm:false};return versions[t.id];}
 function directoryProblem(t){if(t.id==='claude')return directoryScene;if(t.id==='qoder')return 'other-link';return 'empty';}
 function platformHTML(t){
  const v=version(t),m=mapping(t),unsupported=capabilityBlock(t,'skills'),available=installed(t)&&!t.sample,busy=['reading','checking','updating'].includes(v.state);
  const vb=(label,a,disabled=false)=>'<button type="button" class="btn ghost" data-platform-action="'+a+'" '+(disabled?'disabled':'')+'>'+esc(label)+'</button>';
  const versionBody='<section class="xp3-platform-section"><div class="xp3-section-heading"><h3>平台版本</h3><span>'+(!available?'未发现':v.current?'已读取':'待读取')+'</span></div><div class="xp3-version-row"><strong>'+esc(v.current||'—')+'</strong><div class="xp3-detail-actions">'+vb(v.state==='reading'?'读取中…':'读取版本','read-version',!available||busy)+vb(v.state==='checking'?'检查中…':'检查更新','check-version',!available||busy)+(v.latest&&v.latest!==v.current?vb(v.state==='updating'?'更新中…':'更新至 '+v.latest,'update',busy):'')+'</div></div>'+(v.message?'<p class="xp3-feedback" role="status" data-tone="'+(v.state==='error'?'danger':'neutral')+'">'+esc(v.message)+'</p>':'')+(v.confirm?'<div class="xp3-update-confirm"><p>将 '+esc(t.name)+' 从 '+esc(v.current)+' 更新到 '+esc(v.latest)+'？</p><div class="xp3-detail-actions">'+vb('取消','cancel-update')+vb('确认更新','confirm-update')+'</div></div>':'')+'</section>';
  let mappingMessage=m.mapped?'已连接共享源目录。Skills 的启停对所有已映射平台同步生效。':unsupported?unsupported.full:m.problem==='occupied'?'原目录存在文件。请先备份文件并清空原目录，再重新检查；当前不能映射。':m.problem==='other-link'?'原目录已链接到其他位置。请先备份并移走原链接，保持原目录为空，再重新检查。':m.problem==='denied'?'无法读取原目录。请检查访问权限后重试；当前不能映射。':m.checked?'原目录为空，可以映射。':'映射前会检查原目录，目录为空时才能建立映射。';
  const mapAction=m.mapped?'':unsupported?'':m.checking?vb('检查中…','inspect',true):m.busy?vb('映射中…','map',true):m.checked&&!m.problem?vb('映射 Skills','map'):vb(m.problem?'重新检查':'检查并映射','inspect');
  const mapBody='<section class="xp3-platform-section" id="xp3MappingSection"><div class="xp3-section-heading"><h3>Skills 目录映射</h3><span class="xp3-mapping-status" data-mapped="'+m.mapped+'">'+(m.mapped?'已映射':'待映射')+'</span></div><div class="xp3-mapping-paths"><div><span>共享源目录</span>'+pathButton(sourcePath(),'source')+'</div><span class="xp3-map-arrow" aria-hidden="true">↓</span><div><span>平台原目录</span>'+pathButton(originalPath(t),'skills')+'</div></div><p class="xp3-feedback" role="status" data-tone="'+(m.problem?'warning':'neutral')+'">'+esc(mappingMessage)+'</p><div class="xp3-detail-actions">'+mapAction+'</div></section>';
  const config='<section class="xp3-platform-section"><div class="xp3-section-heading"><h3>平台配置</h3></div><dl class="xp3-platform-facts"><dt>MCP 配置</dt><dd>'+pathButton(originalPath(t,'mcp'),'mcp')+'</dd><dt>配置入口</dt><dd>'+esc(t.mcpKey||'—')+'</dd><dt>说明</dt><dd>'+esc(t.note||'—')+'</dd><dt>来源</dt><dd>'+esc(t.evidence)+'</dd></dl></section>';
  return '<div class="xp3-platform-detail" data-platform="'+esc(t.id)+'"><header class="xp3-platform-hero">'+iconHTML(t)+'<div><strong>'+esc(t.name)+'</strong><p>'+esc(t.id)+' · '+installationLabel(t)+' · '+(directoryOS==='windows'?'Windows':'macOS')+'</p></div></header>'+installationSection(t)+versionBody+mapBody+config+'</div>';
 }
 function redrawPlatform(){if(!platformDialog)return;const root=$('.xp3-platform-detail');if(!root)return;const scroll=$('#modalBody').scrollTop,action=document.activeElement?.dataset.platformAction;$('#modalBody').innerHTML=platformHTML(target(platformDialog));$('#modalBody').scrollTop=scroll;if(action)$('#modalBody [data-platform-action="'+action+'"]')?.focus({preventScroll:true});}
 function openClient(t,inspect=false){
  if(!t)return;if($('#modalMask').style.display==='flex')closeModal();platformDialog=t.id;dialogEpoch++;
  openModal(t.name+' · 平台',platformHTML(t),'',null,{hideConfirm:true,prototypeAnnotation:{title:'平台管理 · 内存演示',body:'版本号为虚构 fixture。读取版本、更新、目录检查、映射和打开路径均只改变当前页面内存，不读取本机文件、不执行系统命令。macOS 使用目录符号链接、Windows 使用目录联接是待实施的设计方向；未知平台与 Windows 实际路径仍需实施前核实。'}});
  $('#modal').classList.add('xp3-platform-modal','modal-wide');$('#modalBody').scrollTop=0;
  window.__modalOnClose=()=>{$('#modal').classList.remove('xp3-platform-modal');platformDialog=null;dialogEpoch++;};
  if(inspect){$('#xp3MappingSection')?.scrollIntoView({block:'nearest'});if(!capabilityBlock(t,'skills'))inspectDirectory(t);}
 }
 function inspectDirectory(t){
  const m=mapping(t);if(m.mapped||m.checking||capabilityBlock(t,'skills'))return;
  const epoch=dialogEpoch,os=directoryOS;m.checking=true;m.problem='';m.checked=false;redrawPlatform();
  setTimeout(()=>{m.checking=false;if(epoch!==dialogEpoch||os!==directoryOS)return;m.checked=true;const condition=directoryProblem(t);m.problem=condition==='empty'?'':condition;redrawPlatform();},320);
 }
 function mapDirectory(t){
  const m=mapping(t);if(m.mapped||m.busy||m.checking||capabilityBlock(t,'skills'))return;
  // Recheck the actual fixture at commit time, including a changed directory after the first check.
  const condition=directoryProblem(t);if(!m.checked||condition!=='empty'){m.checked=true;m.problem=condition==='empty'?'':condition;redrawPlatform();return;}
  const epoch=dialogEpoch,os=directoryOS;m.busy=true;redrawPlatform();
  setTimeout(()=>{m.busy=false;if(epoch!==dialogEpoch||os!==directoryOS)return;const latest=directoryProblem(t);if(latest!=='empty'){m.problem=latest;redrawPlatform();return;}m.mapped=true;m.problem='';if(mode==='client')current.client=t.id;actionNotice=t.name+' 已映射到共享 Skills 目录。';render();redrawPlatform();},400);
 }
 function runVersion(t,action){
  const v=version(t);if(!installed(t)||t.sample||['reading','checking','updating'].includes(v.state))return;
  const epoch=dialogEpoch;v.confirm=false;v.state=action==='read-version'?'reading':action==='confirm-update'?'updating':'checking';v.message=v.state==='reading'?'正在读取已安装版本…':v.state==='updating'?'正在更新平台…':'正在检查可用版本…';redrawPlatform();
  setTimeout(()=>{
   if(epoch!==dialogEpoch){v.state='idle';v.message='操作已取消，请重新检查。';return;}
   if(versionScene==='read-error'||(versionScene==='check-error'&&action==='check-version')||(versionScene==='update-error'&&action==='confirm-update')){v.state='error';v.message='操作失败，请重试。已安装版本保持不变。';redrawPlatform();return;}
   v.current=action==='confirm-update'?v.latest:v.current||'1.2.0';
   if(action==='check-version')v.latest=v.current==='1.3.0'?v.current:'1.3.0';
   v.state='ready';v.message=action==='read-version'?'已读取已安装版本。':action==='confirm-update'?'平台更新完成。':v.latest===v.current?'已是最新版本。':'发现可用版本 '+v.latest+'。';redrawPlatform();
  },420);
 }
 let lastOpenedPath=null;
 function openPath(t,k){const path=k==='source'?sourcePath():originalPath(t,k);if(!path)return;const manager=directoryOS==='windows'?'文件资源管理器':'访达';lastOpenedPath={os:directoryOS,path,manager};toast('原型演示：在'+manager+'中打开 '+path+'；未调用系统。');}
 document.addEventListener('click',e=>{
  const b=e.target.closest('.xp3-platform-detail [data-platform-action]');if(!b||b.disabled||!platformDialog)return;
  const t=target(platformDialog),a=b.dataset.platformAction,v=version(t);
  if(a==='path')openPath(t,b.dataset.part);
  if(a==='edit-install'){installation(t).editing=true;installation(t).draft=installation(t).path;redrawPlatform();}
  if(a==='cancel-install'){installation(t).editing=false;installation(t).verified=false;redrawPlatform();}
  if(a==='browse-install'){const i=installation(t);i.draft=directoryOS==='windows'?'C:\\Programs\\'+t.id+'\\'+t.id+'.exe':defaultInstallPath(t);i.verified=false;i.state='idle';i.message='已选择路径，请验证平台身份。';redrawPlatform();}
  if(a==='validate-install')validateInstall(t);if(a==='save-install')saveInstall(t);
  if(a==='inspect')inspectDirectory(t);
  if(a==='map')mapDirectory(t);
  if(a==='read-version'||a==='check-version'||a==='confirm-update')runVersion(t,a);
  if(a==='update'){v.confirm=true;redrawPlatform();}
  if(a==='cancel-update'){v.confirm=false;redrawPlatform();}
 });
 function openTools(){const actions=kind==='skills'?[['skills-rescan','检查更新'],['skills-discover','发现'],['skills-repos','仓库管理'],['skills-restore','恢复'],['skills-zip','ZIP'],['skills-import','导入']]:[['mcp-add','新增 MCP'],['mcp-import','导入已有 MCP']];openModal(tabs[kind]+' 资源工具','<div class="xp3-tool-grid">'+actions.map(([a,l])=>'<button type="button" class="btn ghost" data-xp3="legacy" data-action="'+a+'">'+l+'</button>').join('')+'</div>','',null,{hideConfirm:true});}
 function setMode(m){mode=m;actionNotice='';query.right='';render();}
 function setKind(k){kind=k;actionNotice='';if(!resources.some(r=>r.id===current.resource&&r.kind===kind))current.resource=resources.find(r=>r.kind===kind).id;render();}
 function setScene(v){scene=v;sceneNotice=v==='error'?'读取失败；已有选择保留。':v==='empty'?'源库暂时没有资源。':'';const sel=$('#xp3Scene');if(sel)sel.value=v;render();}
 function setCapacity(v){capacity=v;if(!targets().some(t=>t.id===current.client))current.client=targets()[0].id;actionNotice='';render();}
 const tool=document.createElement('div');tool.className='proto-card xp3-demo';tool.id='extensionManagerDemoCard';tool.dataset.protoContexts='extensions';
 tool.innerHTML='<div class="proto-note-title">多平台扩展 · 映射管理</div><label class="proto-mock-field">平台数量<select id="xp3Capacity"><option value="6">六个平台</option><option value="30">30 项平台目录</option><option value="50">50 个模拟目标</option></select></label><label class="proto-mock-field">数据状态<select id="xp3Scene"><option value="normal">正常</option><option value="empty">空结果</option><option value="error">读取失败</option></select></label><label class="proto-mock-field">目录系统<select id="xp3DirectoryOS"><option value="mac">macOS</option><option value="windows">Windows</option></select></label><label class="proto-mock-field">Claude 原目录<select id="xp3DirectoryScene"><option value="occupied">存在文件，需备份</option><option value="empty">已备份并清空</option><option value="other-link">链接到其他位置</option><option value="denied">无读取权限</option></select></label><label class="proto-mock-field">平台版本服务<select id="xp3VersionScene"><option value="normal">正常</option><option value="read-error">读取失败</option><option value="check-error">检查失败</option><option value="update-error">更新失败</option></select></label><label class="proto-mock-field">平台检测结果<select id="xp3DetectionScene"><option value="normal">正常</option><option value="failed">检测失败</option><option value="codex-missing">Codex 已卸载</option></select></label><label class="proto-mock-field">手动路径验证<select id="xp3ManualScene"><option value="normal">身份匹配</option><option value="missing">路径不存在</option><option value="denied">无访问权限</option><option value="mismatch">程序身份不匹配</option></select></label><p class="pctl-hint">版本号和目录均为虚构样例。Claude 默认目录非空，Hermes 目录为空；在这里模拟备份清空后可重试映射。Windows 路径只作演示。映射、文件管理器与版本命令不会真实执行。</p>';
 ($('#protoStdCards .proto-group-body')||$('#protoStdCards')).append(tool);
 tool.addEventListener('change',e=>{
  if(e.target.id==='xp3Capacity')setCapacity(e.target.value);if(e.target.id==='xp3Scene')setScene(e.target.value);
  if(e.target.id==='xp3DirectoryOS'){if($('#modalMask').style.display==='flex')closeModal();detectionEpoch++;detection.state='idle';directoryOS=e.target.value;actionNotice='';render();redrawPlatform();document.dispatchEvent(new CustomEvent('prototype:platform-availability'));}
  if(e.target.id==='xp3DirectoryScene'){directoryScene=e.target.value;const m=mappings[directoryOS].claude;if(m&&!m.mapped){m.checked=false;m.problem='';}redrawPlatform();}
  if(e.target.id==='xp3VersionScene')versionScene=e.target.value;
  if(e.target.id==='xp3DetectionScene')detectionScene=e.target.value;if(e.target.id==='xp3ManualScene')manualScene=e.target.value;
 });
 host.addEventListener('input',e=>{if(e.target.id==='xp3LeftSearch'){query.left=e.target.value;render();}if(e.target.id==='xp3RightSearch'){query.right=e.target.value;render();}});
 document.addEventListener('click',e=>{
  const b=e.target.closest('[data-xp3]');if(!b||b.disabled)return;const a=b.dataset.xp3;
  if(a==='resolve-sync'){if(b.dataset.reason==='路径待配置')openLocalBindings(b.dataset.resource);else openClient(target(b.dataset.id));return;}
  if(a==='mode')setMode(b.dataset.value);if(a==='kind')setKind(b.dataset.value);
  if(a==='settings'){setRoute('settings?section=extensions');showSettingsSection('extensions');}
  if(a==='detect')openDetection();if(a==='tools')openTools();if(a==='enable-all'||a==='disable-all'){toggleRelationship(bulkIds(rightItems(),a==='enable-all'),a==='enable-all');const next=[$('#'+b.id),$(a==='enable-all'?'#xp3DisableAll':'#xp3EnableAll'),$('#xp3RightSearch')].find(el=>el&&!el.disabled);next?.focus({preventScroll:true});}
  if(a==='legacy'){const btn=legacy.querySelector('[data-prototype-action="'+b.dataset.action+'"]');closeModal();btn?.click();}
 });

 const resourceSyncId=r=>(r.kind==='skills'?'skill:':'mcp:')+r.id;
 function resolvedDefinition(r){if(!r.portableDefinition)return r.definition||{};const d=r.portableDefinition;return {command:d.command,url:d.url,args:d.args?.map(a=>typeof a==='object'&&a.binding?localBindings[directoryOS][r.id]?.[a.binding]||('待配置：'+a.binding):a),env:d.envRefs,headers:d.headerRefs};}
 function portableResource(r){
  const base={schemaVersion:2,id:r.id,kind:r.kind,name:r.name,description:r.description,source:r.source,version:r.version,updated:r.updated};
  if(r.kind==='skills')return {...base,enabled:!!r.shared,files:clone(syncedContent[r.id]||[{path:'SKILL.md',text:'# '+r.name+'\n\n'+r.description},{path:'references/README.md',text:'扩展资源 · '+r.name}])};
  const d=r.definition||{},definition={transport:d.url?'http':'stdio'};
  if(d.url)definition.url=d.url;if(d.command)definition.command=d.command;
  if(d.args)definition.args=d.args.map((arg,index)=>/^(?:\/|[A-Za-z]:\\|~\/)/.test(arg)?{binding:index===0?'entry':'workspace'}:arg);
  if(d.env)definition.envRefs=clone(d.env);if(d.headers)definition.headerRefs=clone(d.headers);
  return {...base,definition:clone(r.portableDefinition||definition),platformIds:[...r.selected].sort(),appMaintained:!!r.appMaintained};
 }
 function syncObjects(){return resources.map(r=>({id:resourceSyncId(r),group:r.kind,name:r.name,summary:r.kind==='skills'?(r.shared?'已启用':'已停用')+' · 内容与资源文件':(r.appMaintained?'客户端维护 · 默认不同步':r.version+' · '+r.selected.length+' 个目标平台'),source:r.source,fields:r.kind==='skills'?'Skill 内容、资源文件、来源、版本与共享源启用状态；共享源只打包一次':'MCP 定义、目标平台与启用关系；本机路径以绑定名传输，实际路径保留本机，凭据独立选择',files:r.kind==='skills'?'共享拓展库 / '+r.name:'受管 MCP 定义',defaultExcluded:!!r.appMaintained,extensionSecret:!!r.definition?.headers?.length,payload:portableResource(r),extensionObject:true})).concat(sortPlatforms(catalog.filter(installed)).map(t=>({id:'skill-platform:'+t.id,group:'extension-platforms',name:t.name,summary:platformSyncInfo(t.id).condition,source:'Agent 平台关联',fields:'决定该 Agent 平台是否参与本次 Skills / MCP 同步，不要求预先映射。原始路径与映射结果仅在本机显示，不进入同步包，不创建或解除映射',files:'单个平台关联',defaultExcluded:false,extensionObject:true,platformAssociation:true,payload:platformPayload(t.id)})));}
 function platformSyncInfo(id){
  const t=catalog.find(t=>t.id===id);if(!t)return null;
  const m=mapping(t),found=installed(t),supported=!capabilityBlock(t,'skills');
  return {id:t.id,name:t.name,installed:found,mapped:m.mapped,supported,path:originalPath(t,'skills'),condition:!found?'平台未发现':!supported?'暂不支持':m.mapped?'已映射':'未映射'};
 }
 function platformPayload(id){return catalog.some(t=>t.id===id)?{schemaVersion:2,kind:'skill-platform',platformId:id}:null;}
 function syncObject(id){if(id.startsWith('skill-platform:'))return platformPayload(id.slice(15));return syncObjects().find(x=>x.id===id)?.payload;}
 function syncFixture(id){
  const value=clone(syncObject(id));if(!value)return null;
  if(value.kind==='skill-platform')return value;
  if(value.kind==='skills'){value.description+=' 同步内容示例。';value.version=value.version+'-sync';value.enabled=false;value.files=[{path:'SKILL.md',text:'# '+value.name+'\n\n来自另一台设备的内容。'},{path:'references/checklist.md',text:'同步后的检查清单'}];}
  else{value.description+=' 同步定义示例。';value.platformIds=['codex','claude','gajae'];}
  value.updated='2026-11-04';return value;
 }
 function applySyncObject(id,payload,options={}){
  if(id==='extension-platforms:skills'||payload?.kind==='platform-intent'||payload?.kind==='skill-mapping-target')return {ok:false,message:'旧版平台目标配置需转换为平台关联，本机映射保持。'};
  if(!payload||payload.schemaVersion!==2)return {ok:false,message:'旧版拓展启用矩阵需先转换，当前未覆盖任何内容。'};


  if(id.startsWith('skill-platform:')){
   if(payload.kind!=='skill-platform'||payload.platformId!==id.slice(15)||!catalog.some(t=>t.id===payload.platformId)||Object.keys(payload).some(k=>!['schemaVersion','kind','platformId'].includes(k)))return {ok:false,message:'Agent 平台关联无效，本机映射保持'};
   receivedExtensions.add(id);render();return {ok:true};
  }
  const r=resources.find(r=>resourceSyncId(r)===id);if(!r||payload.id!==r.id||payload.kind!==r.kind)return {ok:false,message:'拓展身份不匹配'};
  if(r.kind==='skills'){
   if(typeof payload.enabled!=='boolean'||!Array.isArray(payload.files)||payload.files.some(f=>!f||typeof f.path!=='string'||typeof f.text!=='string'||f.path.startsWith('/')||f.path.includes('..')||f.path.includes('\\')))return {ok:false,message:'Skill 文件清单无效，原内容保留'};
   r.shared=payload.enabled;r.sharedState='synced';syncedContent[r.id]=clone(payload.files);
  }else{
   if(!Array.isArray(payload.platformIds)||!payload.definition||payload.platformIds.some(x=>typeof x!=='string'))return {ok:false,message:'MCP 定义或平台关联无效'};
   const d=payload.definition;if(!['stdio','http'].includes(d.transport)||(d.args&&!Array.isArray(d.args))||(d.args||[]).some(a=>typeof a!=='string'&&(!a||typeof a.binding!=='string'))){return {ok:false,message:'MCP 配置格式无效，原定义保留'};}r.definition={command:d.command,url:d.url,args:d.args?.map(a=>typeof a==='object'&&a.binding?'$'+'{local:'+a.binding+'}':a),env:clone(d.envRefs||[]),headers:clone(d.headerRefs||[])};r.portableDefinition=clone(d);const scope=options.platformIds?new Set(options.platformIds):null;const next=scope?[...r.selected.filter(id=>!scope.has(id)),...payload.platformIds.filter(id=>scope.has(id))]:payload.platformIds;r.selected=[...new Set(next)];r.projections=Object.fromEntries(r.selected.map(id=>[id,'synced']));
  }
  for(const key of ['name','description','source','version','updated'])if(typeof payload[key]==='string')r[key]=payload[key];
  receivedExtensions.add(id);render();document.dispatchEvent(new CustomEvent('prototype:extensions-updated'));return {ok:true};
 }
 function missingBindings(r){const d=r.portableDefinition||portableResource(r).definition;return (d?.args||[]).some(a=>a&&typeof a==='object'&&a.binding&&!localBindings[directoryOS][r.id]?.[a.binding]);}
 function syncReadiness(ids,incoming={}){
  const chosen=new Set(ids||[]),platforms=new Set(),pending=new Map();
  for(const id of chosen)if(id.startsWith('skill-platform:')){const value=incoming[id]||syncObject(id);if(value?.kind==='skill-platform')platforms.add(value.platformId);}
  const add=(id,reason,resourceId)=>{const t=target(id),key=id+'|'+reason+'|'+(resourceId||'');pending.set(key,{id,name:t?.name||id,reason,resourceId});};
  for(const id of platforms){const t=target(id);if(!t){add(id,'平台暂不支持');continue;}if(!installed(t))add(id,'平台未发现');else if(capabilityBlock(t,'skills'))add(id,'平台暂不支持');else if(!mapping(t).mapped)add(id,'待映射');}
  for(const r of resources.filter(r=>r.kind==='mcp'&&chosen.has(resourceSyncId(r)))){
   const projected=incoming[resourceSyncId(r)],definition=projected?.definition||r.portableDefinition||portableResource(r).definition,missing=(definition.args||[]).some(a=>a&&typeof a==='object'&&a.binding&&!localBindings[directoryOS][r.id]?.[a.binding]);
   for(const id of projected?.platformIds||r.selected){const t=target(id);if(!t||!installed(t))add(id,'平台未发现',r.id);else if(capabilityBlock(t,'mcp'))add(id,'平台暂不支持',r.id);else if(missing)add(id,'路径待配置',r.id);}
  }
  return [...pending.values()];
 }
 function openLocalBindings(id){
  const r=resource(id);if(!r)return;const d=r.portableDefinition||portableResource(r).definition,keys=[...new Set((d.args||[]).filter(a=>a&&typeof a==='object'&&a.binding).map(a=>a.binding))];if(!keys.length)return;
  openModal(r.name+' · 本机路径',keys.map(key=>'<label class="xp3-install-label" for="xp3Bind-'+key+'">'+(key==='entry'?'服务器入口文件':'工作目录')+'</label><input class="input" id="xp3Bind-'+key+'" value="'+esc(localBindings[directoryOS][id]?.[key]||'')+'" placeholder="'+(directoryOS==='windows'?'C:\\Tools\\…':'/opt/…')+'">').join('')+'<p class="xp3-feedback" id="xp3BindingError" role="alert">只保存在本机，不随同步传输。</p>','保存路径',()=>{
   const values=Object.fromEntries(keys.map(key=>[key,$('#xp3Bind-'+key).value.trim()]));if(Object.values(values).some(value=>!validInstallPath(value))){$('#xp3BindingError').textContent='请填写当前系统的绝对路径。';return;}
   localBindings[directoryOS][id]=values;closeModal();render();
  },{confirmKeepsOpen:true});
 }
 function pendingHTML(){
  const pending=syncReadiness([...receivedExtensions]);if(!pending.length)return '';
  return '<details class="xp3-sync-pending"><summary>拓展已保存 · '+pending.length+' 项平台接入待处理</summary><ul>'+pending.map(x=>'<li><span>'+esc(x.name)+(x.resourceId?' · '+esc(resource(x.resourceId)?.name):'')+'</span><span>'+esc(x.reason)+'</span><button class="btn ghost" type="button" data-xp3="resolve-sync" data-id="'+esc(x.id)+'" data-resource="'+esc(x.resourceId||'')+'" data-reason="'+esc(x.reason)+'">'+(x.reason==='路径待配置'?'配置路径':'查看平台')+'</button></li>').join('')+'</ul></details>';
 }
 function refreshScope(){const yes=active();legacy.hidden=yes;host.hidden=!yes;if(yes){$('#routeTitle').textContent='拓展';$('#routeSubtitle').textContent='维护 Skills 与 MCP，按扩展或平台管理。';render();}tool.querySelector('#xp3Capacity').value=capacity;tool.querySelector('#xp3Scene').value=scene;window.prototypeSelect?.enhance(tool);window.prototypePanel?.refresh();}
 new MutationObserver(refreshScope).observe(document.documentElement,{attributes:true,attributeFilter:['data-design-scope','data-route']});
 window.extensionPrototype={platformSyncInfo,syncObjects,syncObject,syncFixture,applySyncObject,syncReadiness,openLocalBindings,portableSnapshot:ids=>({schemaVersion:2,objects:syncObjects().filter(x=>ids.includes(x.id)).map(x=>({id:x.id,payload:clone(x.payload)}))}),active,refresh:render,tab:()=>kind,setKind,setMode,state:()=>clone({mode,kind,capacity,scene,directoryOS,directoryScene,versionScene,notice:noticeText(),current,resources,mappings,versions,installations,detection,detectionScene,manualScene,lastOpenedPath,picks:{resource:[],client:[]}})};
 refreshScope();document.dispatchEvent(new CustomEvent('prototype:extensions-ready'));
})();
