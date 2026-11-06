/* Installation identity recovery. Explicit in-memory stages; never runs an installer. */
(function(root,factory){const api=factory(root);if(typeof module==='object'&&module.exports)module.exports=api;else{root.runtimeRecovery023=api;api.mount();}})(typeof window==='undefined'?globalThis:window,function(root){
  'use strict';
  const copy=value=>JSON.parse(JSON.stringify(value));
  const scenes=[['normal','完整恢复'],['diagnosis-failed','诊断读取失败'],['source-invalid','来源摘要不符'],['backup-failed','保护备份失败'],['occupied','其它用户正在使用'],['permission','安装根无写权限'],['replace-failed','替换后待核对'],['reconcile','完成但记录待核验']];
  function create(){
    let s={scene:'normal',view:'diagnosis',source:null,operation:null,serial:0,prepares:0,backups:0,replacements:0,revision:1};
    return {
      state:()=>copy(s),
      scene(value){if(!scenes.some(r=>r[0]===value))return;s.scene=value;s.view='diagnosis';s.source=null;s.operation=null;s.revision++;},
      source(value){s.source={kind:value,version:'合成修订1',digest:'sha256:'+'b'.repeat(64),valid:s.scene!=='source-invalid'};s.view='source';},
      prepare(){s.prepares++;if(!s.source?.valid)return null;s.view='confirm';return {operationId:'mock-install-'+(s.serial+1),revision:s.revision,source:copy(s.source),writes:0};},
      apply(plan){if(!plan||plan.revision!==s.revision||!s.source?.valid)return false;s.serial++;s.operation={operationId:plan.operationId,phase:'backup',mayHaveWritten:false,backup:'pending',failure:null,needsRestart:false};s.view='operation';return true;},
      next(){const o=s.operation;if(!o)return;
        if(o.phase==='backup'){if(s.scene==='backup-failed'){o.phase='failed';o.failure='保护备份未完成，原程序与配置保留。';o.backup='failed';return;}s.backups++;o.backup='verified';o.phase='stop_own_service';}
        else if(o.phase==='stop_own_service'){if(['occupied','permission'].includes(s.scene)){o.phase='blocked';o.failure=s.scene==='occupied'?'其它用户仍在使用安装根，未停止其服务。':'安装根无写权限，未放宽目录权限。';return;}o.phase='replacing';o.mayHaveWritten=true;s.replacements++;}
        else if(o.phase==='replacing'){o.phase=s.scene==='replace-failed'?'unknown':'verifying';if(o.phase==='unknown')o.failure='替换结果尚未确认，旧程序、保护备份和原记录均保留。';}
        else if(o.phase==='verifying'){o.phase=s.scene==='reconcile'?'verified_pending':'done';o.needsRestart=true;}
      },
      cancel(){const o=s.operation;if(!o){s.view='diagnosis';return 'cancelled_before_write';}if(o.mayHaveWritten)return 'cannot_retract';o.phase='cancelled_before_write';return o.phase;},
      status(){s.view=s.operation?'operation':'diagnosis';return copy(s.operation);},
      verify(){const o=s.operation;if(!o)return false;if(o.phase==='unknown'){o.phase='verifying';o.failure='首次失败：替换结果曾未知；现只核对原操作。';}else if(o.phase==='verified_pending')o.phase='done';return true;}
    };
  }
  const controller=create();let plan=null;
  const phaseLabels={backup:'创建保护备份',stop_own_service:'核对服务拥有权',replacing:'替换受管运行包',verifying:'读回包 / 入口 / 清单 / 来源',done:'恢复完成',unknown:'原恢复结果待核对',verified_pending:'包体已核验，完成记录待核对',failed:'保护失败，未继续替换',blocked:'前置条件未满足',cancelled_before_write:'已在替换前停止'};
  function paths(){return root.directoryRuntime023.controller.state().snapshot;}
  function render(){
    const u=root.OCXD023,s=controller.state(),p=paths(),head='';
    if(s.view==='diagnosis'){
      u.dialog('安装身份诊断',head+(s.scene==='diagnosis-failed'?'<p role="alert">安装记录无法读取，未生成恢复计划。</p>':u.facts([['安装目标',p.runtimePrefix],['稳定入口',p.runtimeEntry],['实际包','合成修订0'],['清单版本','合成修订1 · 不一致'],['来源记录','受管来源 · 待恢复']])+'<p>仅恢复新安装根的受管包与记录；配置、凭据和旧目录保留。</p>'),s.scene==='diagnosis-failed'?'重新诊断':'选择恢复来源',()=>{if(s.scene==='diagnosis-failed'){controller.scene('normal');render();}else{controller.source('registry');render();}});return;
    }
    if(s.view==='source'){
      u.dialog('明确本次恢复来源',head+'<div class="mp-template-form">'+u.select('安装来源','install-source',[['registry','明确受管源版本'],['offline','合成离线包']],s.source.kind)+'</div>'+u.facts([['版本',s.source.version],['包摘要',s.source.digest],['来源核验',s.source.valid?'声明 mock：校验通过':'摘要与清单不符，不能恢复'],['目标',p.runtimePrefix]])+'<p>离线场景只选择声明样例，不读取磁盘文件；不会绑定独立旧程序。</p>','查看恢复预览',()=>{plan=controller.prepare();if(plan)render();},{disabled:!s.source.valid,onExtra:()=>{controller.status();render();},extraLabel:'返回诊断'});return;
    }
    if(s.view==='confirm'){
      u.dialog('确认安装恢复',u.facts([['安装目标',p.runtimePrefix],['程序入口',p.runtimeEntry],['受管源版本',s.source.version],['受管源摘要',s.source.digest],['执行前保护','确认后创建备份，成功后才允许替换'],['服务前置','仅停止已证明属于当前用户的实例'],['用户数据','配置、凭据及备份保留']])+'<p>本体 .app 不被运行包替换；恢复后分别核验包、入口、清单和来源。</p>','确认恢复',()=>{if(controller.apply(plan))render();},{onExtra:()=>{controller.source(s.source.kind);render();},extraLabel:'上一步'});return;
    }
    const o=s.operation;if(!o)return;
    const active=['backup','stop_own_service','replacing','verifying'].includes(o.phase);
    const body=head+u.facts([['原恢复操作',o.operationId],['处理阶段',phaseLabels[o.phase]],['保护备份',o.backup==='verified'?'完整性已核验':o.backup==='failed'?'失败，未替换程序':'尚未完成'],['包 / 入口 / 清单 / 来源',o.phase==='done'?'四项均读回一致':o.phase==='verified_pending'?'程序一致，完成记录待核对':'尚未确认全部一致'],['重启要求',o.needsRestart?'恢复后需重新启动并检查':'未确认']])+(o.failure?'<p class="mp-alert" role="alert">'+u.esc(o.failure)+'</p>':'')+
      '<p>'+(['replacing','verifying'].includes(o.phase)?'已经进入替换/核验阶段，不能宣称撤回。关闭窗口仅隐藏进度，可重开原操作。':'所有动作只更新本页内存。关闭后可查看原操作结果。')+'</p><div class="mp-actions">'+u.button('停止后续操作','install-stop',o.mayHaveWritten||!active)+u.button('核对原恢复','install-verify',!['unknown','verified_pending'].includes(o.phase))+u.button('查看原恢复记录','install-status')+'</div>';
    u.dialog(phaseLabels[o.phase],body,active?'模拟下一阶段':o.phase==='done'?'完成':null,active?()=>{controller.next();render();}:()=>u.close());
  }
  function open(){controller.status();render();}
  function mount(){
    const rail=root.document.getElementById('protoStdCards');if(rail){const tools=root.document.createElement('section');tools.className='proto-card';tools.dataset.protoContexts='settings:installation';tools.innerHTML='<div class="proto-note-title">0.2.23 · 安装恢复演示</div>'+root.OCXD023.select('恢复演示状态','install-scene',scenes,'normal');rail.append(tools);}
    root.document.addEventListener('change',event=>{const key=event.target.dataset.o23Select;if(key==='install-scene'){controller.scene(event.target.value);plan=null;render();}if(key==='install-source'){controller.source(event.target.value);render();}});
    root.document.addEventListener('click',event=>{const el=event.target.closest('[data-o23]');if(!el||el.disabled)return;if(el.dataset.o23==='install-stop'){controller.cancel();render();}if(el.dataset.o23==='install-verify'){controller.verify();render();}if(el.dataset.o23==='install-status'){controller.status();render();}});
  }
  return {create,controller,scenes,open,mount};
});
