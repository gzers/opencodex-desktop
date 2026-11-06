/* L2-sync-v1, digest 8f082326…ded5267. Declared DTO/state transitions only, no transport. */
(function(root,factory){const api=factory(root);if(typeof module==='object'&&module.exports)module.exports=api;else{root.syncRecovery023=api;api.mount();}})(typeof window==='undefined'?globalThis:window,function(root){
  'use strict';
  const copy=v=>JSON.parse(JSON.stringify(v)),id=n=>'00000000-0000-4000-8000-'+String(n).padStart(12,'0'),hash=n=>'sha256:'+String(n).repeat(64),time='2026-11-06T09:00:00Z';
  const generation=n=>({generationId:id(n),digest:hash(n)}),head=n=>({operationId:id(n),digest:hash(n%10)});
  const action=(kind,extra={})=>({kind,...extra});
  const payloadSchema={envelope:{format:'OCXDCONF',version:3},document:{format:'OCXD_CONFIG_OBJECTS',version:2},model:{format:'OCXD_MODEL_PACKAGE',formatVersion:6,schemaVersion:2}};
  const scenes=[['ready','云端有效 · 默认下载'],['upload','本机较新 · 上传'],['unknown-time','时间未知 · 云端有配置'],['empty','目录确空 · 首次确认初始化'],['generation_empty','唯一代次尚无提交'],['same','内容一致'],['no-content','没有所选内容'],['incomplete','列举不完整'],['invalid','远端损坏'],['auth','认证失败'],['network','网络中断'],['capacity','远端容量不足'],['expired','计划已过期'],['conflict','同代次并发分叉'],['initialization_ambiguous','多个初始化代次'],['mcp-source','同名 MCP 定义不同'],['unknown','原上传可能已发送'],['local-changed','原上传已存储 · 本机后改'],['descendant','原上传已存储 · 云端后继'],['collision','原操作字节身份冲突'],['endpoint','原连接已切换'],['journal','本地日记损坏'],['backup-failed','下载保护备份失败'],['download-stale','下载目标已变化']];
  const failure=(code,step='observe',recoveryActions=[])=>({code,step,retryability:['sync_remote_invalid','sync_operation_collision','sync_journal_invalid'].includes(code)?'never':'after_refresh',recoveryActions});
  function observation(scene){
    const state=['empty','generation_empty','conflict','initialization_ambiguous','incomplete','invalid'].includes(scene)?scene:'ready';
    const o={dtoVersion:1,bindingId:hash(1),state,generations:state==='empty'?[]:state==='initialization_ambiguous'?[generation(1),generation(2)]:[generation(1)],heads:['ready','conflict'].includes(state)?state==='conflict'?[head(101),head(102)]:[head(101)]:[],orphanCount:state==='generation_empty'?1:0,staleCount:0,viewDigest:hash(3),observedAt:time,problems:[],recoveryCandidates:[],recoveryActions:[]};
    if(['generation_empty','ready','conflict'].includes(state))o.generation=generation(1);
    if(state==='initialization_ambiguous'){o.recoveryCandidates=[{generation:generation(1),head:head(101)},{generation:generation(2),head:head(102)}];o.recoveryActions=[action('prepare_recovery_export'),action('prepare_recovery_download'),action('select_empty_directory')];o.problems=[failure('sync_generation_ambiguous','observe',o.recoveryActions)];}
    if(state==='conflict')o.recoveryActions=[action('choose_conflict_source'),action('refresh_remote')];
    if(['incomplete','invalid'].includes(state))o.problems=[failure(state==='invalid'?'sync_remote_invalid':'sync_listing_incomplete','observe',[action(state==='invalid'?'inspect_error_details':'refresh_remote')])];
    return o;
  }
  function originalFact(scene='unknown'){
    const stored=['local-changed','descendant'].includes(scene),o={dtoVersion:1,operationId:id(101),kind:'upload',phase:stored?'settled':'commit_may_be_written',lastWriteStep:'commit',storageStatus:stored?'stored':scene==='collision'?'invalid':'unknown',localApplyStatus:'not_applicable',relation:{local:scene==='local-changed'?'changed':'not_compared',remote:scene==='descendant'?'descendant':'unknown'},generation:generation(1),payloadDigest:hash(5),recoveryActions:[action('verify_original_upload',{operationId:id(101)})]};
    if(stored){o.commit=head(101);o.storedAt=time;}
    if(scene==='collision'){o.failure=failure('sync_operation_collision','verify',[action('inspect_error_details'),action('repreview_upload')]);o.recoveryActions=o.failure.recoveryActions;}
    if(scene==='endpoint'){o.failure=failure('sync_endpoint_changed','verify',[action('return_to_original_connection')]);o.recoveryActions=o.failure.recoveryActions;}
    if(scene==='journal'){o.failure=failure('sync_journal_invalid','persist',[action('inspect_local_protection')]);o.recoveryActions=o.failure.recoveryActions;}
    return o;
  }
  function create(){
    let s={scene:'ready',observation:observation('ready'),direction:'download',backup:true,plan:null,fact:null,recovery:null,receipt:null,view:'preview',mcpSource:'',guardRevision:1,planGuard:0,writeCount:0,localWrites:0,backupCount:0,networkReads:0,localReads:0,stopped:false,selection:null,error:'',calls:[]};
    const resultScenes=['unknown','local-changed','descendant','collision','endpoint','journal'];
    function prepare(intent='update'){
      s.networkReads++;s.calls.push('prepare_'+(s.direction==='upload'?'upload':'download'));
      const o=s.observation;
      if(['initialization_ambiguous','invalid','incomplete'].includes(o.state)||o.state==='conflict'&&intent!=='resolve'||['auth','network','capacity'].includes(s.scene)||s.scene==='mcp-source'&&!s.mcpSource||s.scene==='no-content'){s.plan=null;return null;}
      if(s.direction==='download'&&['empty','generation_empty'].includes(o.state)){s.plan=null;return null;}
      const common={dtoVersion:1,operationId:id(201),planDigest:hash(4),bindingId:o.bindingId,payloadDigest:hash(5),expectedViewDigest:o.viewDigest,preparedAt:time,expiresAt:'2026-11-06T09:15:00Z'};
      if(s.direction==='upload'){
        s.plan={...common,initialization:o.state==='empty'?'on_confirm':'none',parents:copy(o.heads),intent,payloadBytes:28,payloadSchema:copy(payloadSchema),protectionMode:'plain',sourceGuard:{savedSourceDigest:hash(6),scopeDigest:hash(7),mcpSourcesDigest:hash(8)}};
        if(s.plan.initialization==='none')s.plan.generation=copy(o.generation);
        if(intent==='resolve')s.plan.resolution={selectionMode:'head',selectedHead:copy(s.selection),scopeDigest:hash(7),previewDigest:hash(9)};
      }else s.plan={...common,generation:copy(o.generation),head:copy(o.heads[0]),scopeDigest:hash(7),targetDigest:hash(8),backupBeforeUpdate:s.backup};
      s.planGuard=s.guardRevision;return copy(s.plan);
    }
    return {
      state:()=>copy(s),
      scene(value){if(!scenes.some(r=>r[0]===value))return;s={...s,scene:value,observation:observation(value),direction:['upload','empty','generation_empty','mcp-source'].includes(value)?'upload':'download',backup:true,plan:null,fact:resultScenes.includes(value)?originalFact(value):null,recovery:null,receipt:null,view:resultScenes.includes(value)?'result':'preview',mcpSource:'',selection:null,error:'',stopped:false,guardRevision:s.guardRevision+1};},
      prepare,
      direction(value){s.direction=value;s.plan=null;s.guardRevision++;return prepare();},
      backup(value){s.backup=value;s.guardRevision++;return prepare();},
      mcp(value){s.mcpSource=value;s.guardRevision++;return prepare();},
      changeSource(){s.guardRevision++;s.plan=null;s.error='预览后本机来源或 MCP 选择已变化，请重新比较。';},
      confirm(){
        if(!s.plan||s.planGuard!==s.guardRevision||s.scene==='expired'){s.error='计划、来源或范围已变化/过期，未开始写入。';s.plan=null;return false;}
        s.view='result';s.stopped=false;s.calls.push(s.direction==='upload'?'publish_upload':'apply_download');
        if(s.direction==='download'){
          s.fact={dtoVersion:1,operationId:s.plan.operationId,kind:'download',phase:'settled',lastWriteStep:'none',storageStatus:'stored',localApplyStatus:'not_attempted',relation:{local:'not_compared',remote:'observed_equal'},generation:copy(s.plan.generation),payloadDigest:s.plan.payloadDigest,commit:copy(s.plan.head),storedAt:time,recoveryActions:[]};
          if(['backup-failed','download-stale'].includes(s.scene)){s.fact.failure=failure(s.scene==='backup-failed'?'sync_backup_failed':'sync_local_changed','download_guard',[action(s.scene==='backup-failed'?'inspect_local_protection':'repreview_download')]);s.fact.recoveryActions=s.fact.failure.recoveryActions;return true;}
          if(s.backup)s.backupCount++;s.localWrites++;s.fact.localApplyStatus='applied';s.fact.relation.local='observed_equal';return true;
        }
        s.fact={dtoVersion:1,operationId:s.plan.operationId,kind:'upload',phase:'prepared',lastWriteStep:'none',storageStatus:'not_attempted',localApplyStatus:'not_applicable',relation:{local:'not_compared',remote:'unknown'},payloadDigest:s.plan.payloadDigest,recoveryActions:[action('verify_original_upload',{operationId:s.plan.operationId})]};
        if(s.plan.generation)s.fact.generation=copy(s.plan.generation);return true;
      },
      advance(){const f=s.fact;if(!f||s.stopped||f.kind!=='upload')return;
        if(f.phase==='prepared'&&s.plan.initialization==='on_confirm'){f.phase='initialization_may_be_written';f.lastWriteStep='generation';f.generation=generation(1);s.writeCount++;}
        else if(['prepared','initialization_may_be_written'].includes(f.phase))f.phase='generation_verified';
        else if(f.phase==='generation_verified'){f.phase='payload_may_be_written';f.lastWriteStep='payload';f.storageStatus='unknown';s.writeCount++;}
        else if(f.phase==='payload_may_be_written')f.phase='payload_verified';
        else if(f.phase==='payload_verified'){f.phase='commit_may_be_written';f.lastWriteStep='commit';s.writeCount++;}
        else if(f.phase==='commit_may_be_written'){f.phase='settled';f.storageStatus='stored';f.commit={operationId:f.operationId,digest:hash(1)};f.storedAt=time;f.relation={local:'observed_equal',remote:'observed_equal'};}
      },
      stop(){if(!s.fact)return;s.stopped=true;if(s.fact.lastWriteStep==='none'){s.fact.phase='cancelled_before_write';s.fact.storageStatus='not_attempted';}s.calls.push('cancel');},
      readResult(){if(s.fact)s.view='result';s.localReads++;s.calls.push('read_transfer_result');return copy(s.fact);},
      verify(){const f=s.fact;if(!f||!f.recoveryActions.some(a=>a.kind==='verify_original_upload'))return false;s.networkReads++;s.calls.push('verify_original_upload');const before=f.operationId;f.phase='settled';f.storageStatus='stored';f.generation=f.generation||generation(1);f.commit={operationId:before,digest:hash(1)};f.storedAt=time;f.relation={local:'not_compared',remote:s.observation.state==='initialization_ambiguous'?'initialization_ambiguous':'observed_equal'};return true;},
      assess(){if(s.fact){s.calls.push('assess_current_relation');s.fact.relation.local='changed';}return copy(s.fact);},
      recover(index,mode){const candidate=s.observation.recoveryCandidates[index];if(!candidate)return null;s.networkReads++;s.calls.push('prepare_recovery');s.recovery={dtoVersion:1,recoveryPlanId:id(901),planDigest:hash(4),kind:mode==='download'?'recovery_download':'recovery_export',sourceBindingId:s.observation.bindingId,generation:copy(candidate.generation),head:copy(candidate.head),payloadDigest:hash(5),payloadSchema:copy(payloadSchema),scopeDigest:hash(7),expectedViewDigest:s.observation.viewDigest,backupBeforeUpdate:true,preparedAt:time,expiresAt:'2026-11-06T09:15:00Z'};if(mode==='download')s.recovery.targetDigest=hash(8);s.view='recovery';return copy(s.recovery);},
      applyRecovery(){if(!s.recovery)return false;s.calls.push('apply_recovery');if(s.recovery.kind==='recovery_download'){s.backupCount++;s.localWrites++;}else s.localWrites++;s.receipt={dtoVersion:1,recoveryPlanId:s.recovery.recoveryPlanId,kind:s.recovery.kind,outcome:s.recovery.kind==='recovery_export'?'exported':'applied',sourceBindingId:s.recovery.sourceBindingId,generation:copy(s.recovery.generation),head:copy(s.recovery.head),payloadDigest:s.recovery.payloadDigest,artifactId:id(902),finishedAt:time,recoveryActions:[action('select_empty_directory')]};s.view='recovered';return true;},
      selectEmpty(){s.calls.push('select_empty_directory');s.observation=observation('empty');s.observation.bindingId=hash(2);s.scene='empty';s.direction='upload';s.view='preview';s.plan=null;s.guardRevision++;},
      resolve(index){if(s.observation.state!=='conflict'||!s.observation.heads[index])return null;s.selection=copy(s.observation.heads[index]);s.direction='upload';s.view='resolution';return prepare('resolve');},
      lateHead(){s.observation=observation('conflict');s.guardRevision++;s.error='已观察头集合变化，原解决预览已失效。';s.view='preview';},
      resetPreview(){s.view='preview';s.error='';return prepare();}
    };
  }
  const controller=create();let selectedScope=null;
  const stateLabels={empty:'目录确空',generation_empty:'唯一初始化记录，暂无配置',ready:'配置可读取',conflict:'云端存在并发配置',initialization_ambiguous:'此目录存在多个初始化记录',incomplete:'云端列举不完整',invalid:'云端配置损坏'};
  const phaseLabels={prepared:'确认完成，尚未发送',initialization_may_be_written:'初始化可能已发送',generation_verified:'代次已核对',payload_may_be_written:'载荷可能已发送',payload_verified:'载荷已核验',commit_may_be_written:'提交可能已发送',settled:'本次处理已停止',cancelled_before_write:'未发送前已取消'};
  const actionLabels={verify_original_upload:'核对原上传',refresh_remote:'重新读取云端',repreview_upload:'重新预览上传',repreview_download:'重新预览下载',choose_conflict_source:'选择冲突来源',prepare_recovery_download:'查看恢复到本机方案',prepare_recovery_export:'查看恢复导出方案',select_empty_directory:'选择新的同步目录',return_to_original_connection:'返回原连接核对',inspect_local_protection:'查看本机保护记录',inspect_error_details:'查看错误详情',retry_later:'稍后再核对',free_remote_space:'查看容量处理说明'};
  const errorMessages={auth:'认证未通过，不能将云端当空目录。',network:'网络中断，未确认云端内容。',capacity:'云端存储空间不足，应用不会自动删除文件。',incomplete:'未获得完整稳定列举，不能预览写入。',invalid:'远端元数据/摘要无效，未降级为空。','no-content':'当前没有所选内容，不能更新。'};
  function render(){
    const u=root.OCXD023,s=controller.state();
    if(s.view==='result'){result();return;}
    if(s.view==='recovery'){const p=s.recovery;u.dialog(p.kind==='recovery_export'?'确认导出恢复文件':'确认专用恢复到本机',u.facts([['所选代次',p.generation.generationId],['所选提交',p.head.operationId],['恢复范围','所选配置及精确依赖，缺项时不执行'],['远端动作','只读；原歧义目录保持'],['本机动作',p.kind==='recovery_export'?'导出到明确目标，不自动导入或上传':'先保护备份，再应用到本机；不记普通同步完成']])+(p.kind==='recovery_export'?'<label>本次导出目标（仅演示）<input class="input" aria-label="恢复文件目标" value="恢复文件.ocxdconf"></label>':'')+'<p>此为专用恢复计划，不复用普通下载绕过多代次门禁。</p>',p.kind==='recovery_export'?'确认导出恢复文件':'确认保护并恢复',()=>{if(p.kind==='recovery_export'&&!root.document.querySelector('[aria-label="恢复文件目标"]').value.trim())return;controller.applyRecovery();render();});return;}
    if(s.view==='recovered'){u.dialog('恢复内容已处理',u.facts([['处理结果',s.receipt.outcome==='exported'?'恢复文件已导出（内存演示）':'恢复内容已应用（内存演示）'],['普通同步状态','未记录为同步完成'],['原目录','保留全部内容，没有远端写入']])+'<p>请核对本机恢复内容并正常保存为明确来源，再选择另一个已核实空目录重新开始。不会自动上传恢复材料。</p>','选择新的同步目录',newDirectory);return;}
    if(s.view==='resolution'){u.dialog('预览并发配置解决方案',u.facts([['明确选择',s.plan.resolution.selectedHead.operationId],['父集合',s.plan.parents.map(x=>x.operationId).join('\n')],['所选范围','渠道、模型及精确依赖'],['范围外影响','保留其它配置；原分支文件均保留']])+'<p>确认生成新的解决操作；不能保证迟到分支或同时决议永远不存在。</p>'+u.button('模拟迟到分支','sync-late'),'确认解决并发布',()=>{controller.confirm();render();});return;}
    let body='<p role="status">'+u.esc(stateLabels[s.observation.state])+'</p>';
    if(s.observation.state==='initialization_ambiguous'){
      body+='<p>已暂停普通上传/下载应用。先只读查看可恢复内容，原目录完整保留。</p>'+s.observation.recoveryCandidates.map((r,i)=>'<details open><summary>可核验分支 '+(i+1)+'</summary>'+u.facts([['代次',r.generation.generationId],['提交',r.head.operationId],['内容','渠道 / 模型 / 精确依赖 · 合成可读分支'],['业务保存时间','未知']])+'<div class="mp-actions">'+u.button('查看恢复导出方案 '+(i+1),'sync-export-'+i)+u.button('查看恢复到本机方案 '+(i+1),'sync-recover-'+i)+'</div></details>').join('')+u.button('选择新的同步目录','sync-new-directory');
      u.dialog('此目录存在多个初始化记录',body,null);return;
    }
    if(s.observation.state==='conflict'){
      body+='<p>请选择一个已核验来源，再查看全部范围影响；时间不作为自动赢家。</p>'+s.observation.heads.map((r,i)=>'<section>'+u.facts([['并发配置 '+(i+1),r.operationId],['所选内容',i?'保留另一设备的模型调整':'采用本次明确选择的模型配置'],['业务保存时间','未知']])+u.button('预览此来源 '+(i+1),'sync-resolve-'+i)+'</section>').join('');u.dialog('云端存在并发配置',body,null);return;
    }
    const blocked=!!errorMessages[s.scene]||selectedScope?.length===0,init=s.plan?.initialization==='on_confirm',same=s.scene==='same';
    body+=u.select('本次来源','sync-direction',[['upload','本机配置 → 云端（上传）'],['download','云端配置 → 本机（下载）']],s.direction)+u.facts([['本机保存时间',s.scene==='unknown-time'?'未知':s.scene==='upload'?'2026-11-06 09:00 UTC':'2026-11-05 09:00 UTC'],['云端保存时间',['empty','generation_empty','unknown-time'].includes(s.scene)?'未知':'2026-11-06 08:00 UTC'],['本次核对时间',s.observation.observedAt],['本次范围',selectedScope===null?'所选渠道、模型与精确依赖；未选对象保留':selectedScope.length+' 项已选对象及精确依赖；未选内容保留']]);
    if(blocked)body+='<p class="mp-alert" role="alert">'+u.esc(errorMessages[s.scene]||'尚未选择同步内容。')+'</p>';
    else if(init)body+='<p>此目录尚未初始化。确认后将初始化并上传所选配置；打开或切换来源不会创建初始化记录。</p>';
    else if(s.scene==='unknown-time')body+='<p>无法根据业务时间判新；云端有有效配置，默认下载。可明确切换来源。</p>';
    if(s.observation.orphanCount)body+='<p>发现孤立载荷，但它不是可应用配置；不自动删除。</p>';
    if(s.direction==='download')body+='<label><input type="checkbox" data-sync23-backup'+(s.backup?' checked':'')+'>更新前备份</label>';
    if(s.scene==='mcp-source')body+='<h3>同名 MCP 定义不同</h3><p>documentation 在 Codex / Claude Code 的定义字段不同；请选择来源，或不勾选该项。未选择不上传。</p>'+u.select('MCP 采用来源','sync-mcp',[['','请选择来源'],['codex','Codex'],['claude','Claude Code'],['skip','本次不同步此 MCP']],s.mcpSource);
    body+='<h3>本次变化</h3><p>'+(same?'所选内容一致，无需更新。':s.plan?'将更新所选模型配置及引用，保持原始 ID；未选内容保持。':'尚不能生成可确认变化，请核对来源与范围。')+'</p><p class="mp-alert" role="alert">'+u.esc(s.error)+'</p><details><summary>本次冻结计划</summary><pre class="mp-code">'+u.esc(JSON.stringify(s.plan,null,2))+'</pre></details><div class="mp-actions">'+u.button('重新比较','sync-recompare')+u.button('模拟本机随后修改','sync-source-change')+u.button('查看最近原操作','sync-read-result',!s.fact)+'</div>';
    u.dialog('同步预览',body,same?null:init?'初始化并上传':s.direction==='upload'?'确认上传':'确认下载',()=>{controller.confirm();render();},{disabled:blocked||!s.plan||same,onCancel:()=>u.close()});
  }
  function result(){
    const u=root.OCXD023,s=controller.state(),f=s.fact;if(!f)return;
    const stored=f.storageStatus==='stored',local={not_compared:'未比较当前本机',observed_equal:'本次观察一致',changed:'本机另有修改',unknown:'本机关系未知'}[f.relation.local],remote={unknown:'远端当前关系未知',observed_equal:'本次观察一致',descendant:'远端已有后续版本',conflict:'远端存在并发配置',initialization_ambiguous:'多个初始化记录，普通写入停止',generation_changed:'原代次已失效'}[f.relation.remote];
    let body=u.facts([['原操作',f.operationId],['处理阶段',phaseLabels[f.phase]],['原上传存储事实',stored?'原提交与载荷已核验存储':f.storageStatus==='invalid'?'身份或内容不符':'尚未确认'],['当前本机',local],['当前云端',remote],['本机应用',f.kind==='upload'?'不适用':f.localApplyStatus==='applied'?'已完成本机应用':'尚未应用，不能称下载完成']])+ '<p>'+ (f.lastWriteStep==='none'?'未发送前可取消；读取本地操作记录不联网。':'可能已有远端写入。停止后续操作不会撤回已发送内容；可核对同一个原操作。')+'</p>'+
      (f.failure?'<p class="mp-alert" role="alert">'+u.esc(f.failure.code)+'</p>':'')+'<div class="mp-actions">'+u.button('读取本地操作记录','sync-read-result')+u.button('停止后续操作','sync-stop',s.stopped||f.phase==='settled'||f.phase==='cancelled_before_write')+u.button('比较当前本机关系','sync-assess',!stored)+f.recoveryActions.map(a=>u.button(actionLabels[a.kind]||a.kind,'sync-action-'+a.kind)).join('')+'</div><details><summary>存储、应用与关系 DTO</summary><pre class="mp-code">'+u.esc(JSON.stringify(f,null,2))+'</pre></details>';
    const active=f.kind==='upload'&&!['settled','cancelled_before_write'].includes(f.phase)&&!s.stopped;
    u.dialog('同步操作结果',body,active?'模拟下一发送/核验阶段':null,()=>{controller.advance();render();});
  }
  function newDirectory(){const u=root.OCXD023;u.dialog('选择另一个明确的同步目录','<label>新同步目录<input class="input" aria-label="新同步目录" value="/new-config-scope"></label><p>仅核实该目录可访问、完整列举且为空，不创建集合，不清空原目录。请先核对恢复后的本机来源。</p><p id="s23NewError" role="alert"></p>','只读核实空目录',()=>{const target=root.document.querySelector('[aria-label="新同步目录"]').value;if(!target.startsWith('/')||target==='/'){root.document.getElementById('s23NewError').textContent='请选择另一个明确目录，不能使用根目录。';return;}u.dialog('核对恢复来源','<p>新目录已核实为空（声明响应），远端尚未写入。请确认恢复内容已在本机核对并正常保存；不会把其它本机配置自动当成恢复来源。</p><label><input type="checkbox" id="s23SourceReviewed">已核对本机恢复来源</label>','前往初始化预览',()=>{if(!root.document.getElementById('s23SourceReviewed').checked)return;controller.selectEmpty();controller.prepare();render();});});}
  function open(scene){if(scene)controller.scene(scene);if(controller.state().view==='preview')controller.prepare();render();}
  function mount(){
    const rail=root.document.getElementById('protoStdCards');if(rail){const panel=root.document.createElement('section');panel.className='proto-card';panel.dataset.protoContexts='sync overview';panel.innerHTML='<div class="proto-note-title">0.2.23 · 文件同步恢复</div>'+root.OCXD023.select('同步声明状态','sync-scene',scenes,'ready')+root.OCXD023.button('打开当前同步状态','sync-demo-open');rail.append(panel);root.prototypePanel?.refresh();}
    root.document.addEventListener('change',event=>{const key=event.target.dataset.o23Select;if(key==='sync-scene'){open(event.target.value);return;}if(key==='sync-direction'){controller.direction(event.target.value);render();}if(key==='sync-mcp'){controller.mcp(event.target.value);render();}if(event.target.matches('[data-sync23-backup]')){controller.backup(event.target.checked);render();}});
    root.document.addEventListener('click',event=>{const el=event.target.closest('[data-o23]');if(!el||el.disabled)return;const a=el.dataset.o23;
      if(a==='sync-demo-open')open();
      if(a==='sync-recompare'){controller.resetPreview();render();}
      if(a==='sync-source-change'){controller.changeSource();render();}
      if(a==='sync-read-result'){controller.readResult();if(controller.state().fact){controller.state();result();}}
      if(a==='sync-stop'){controller.stop();render();}
      if(a==='sync-assess'){controller.assess();render();}
      if(a==='sync-action-verify_original_upload'){controller.verify();render();}
      if(a.startsWith('sync-action-')&&a!=='sync-action-verify_original_upload')root.OCXD023.dialog(actionLabels[a.slice(12)]||'操作说明','<p>按后端本次 recoveryActions 和原操作/连接身份继续；不会自动重传、清空或更换端点。</p>','返回原记录',result);
      if(a.startsWith('sync-export-')){controller.recover(Number(a.split('-').at(-1)),'export');render();}
      if(a.startsWith('sync-recover-')){controller.recover(Number(a.split('-').at(-1)),'download');render();}
      if(a.startsWith('sync-resolve-')){controller.resolve(Number(a.split('-').at(-1)));render();}
      if(a==='sync-late'){controller.lateHead();render();}
      if(a==='sync-new-directory')newDirectory();
    });
  }
  return {create,controller,scenes,observation,originalFact,payloadSchema,open,mount,setScope:ids=>{selectedScope=[...ids];}};
});
