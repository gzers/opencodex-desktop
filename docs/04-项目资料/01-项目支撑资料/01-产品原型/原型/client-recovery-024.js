/* 0.2.24 client recovery: fixed in-memory examples only. No APIs/files/storage. */
(function(root){
 'use strict';
 const $=s=>root.document.querySelector(s),esc=root.OCXD023.esc;
 const css=root.document.createElement('link');css.rel='stylesheet';css.href=new URL('client-recovery-024.css',root.document.currentScript.src).href;root.document.head.append(css);
 const S={scene:'eligible',open:false,reading:false,executing:false,outcome:false,ready:false,error:'',refreshError:'',confirmations:0,choice:null,action:'finalize',phase:'restore_prepared',revision:6,finalized:false};
 const continuing=()=>S.scene.startsWith('continue')||S.scene.startsWith('rollback')||S.scene.startsWith('rearm');
 const preparing=()=>S.scene.startsWith('prepare');
 const early=()=>S.scene.startsWith('early');
 const rolled=()=>S.phase==='restore_rolled_back'||S.scene==='rolled-back';
 const labels={rollback:'撤销本次恢复',rearm:'重新准备恢复',continue:'确认继续恢复',finalize:'确认完成收尾',finishUnchanged:'确认结束会话',cancelPreparation:'确认取消启动准备'};
 const progress={rollback:'正在撤销本次恢复…',rearm:'正在重新准备原件恢复…',continue:'正在继续恢复客户端配置…',finalize:'正在完成恢复收尾…',finishUnchanged:'正在核对原件并结束原会话…',cancelPreparation:'正在核对并取消启动准备…'};
 const summaries={eligible:['客户端配置已恢复，尚有一项收尾未完成。','确认后重新核对原会话并完成收尾。客户端配置保持不变。'],pending:['配置恢复尚未完成，原会话已保留。','当前现场不能直接收尾，请查看诊断详情和应用日志。'],unknown:['客户端恢复状态尚未核实。','请重新检查；若仍无法核对，请保留原文件并查看应用日志。'],active:['当前会话正在使用。请先正常停止 OpenCodex，再检查恢复状态。',''],none:['没有待处理的客户端配置恢复。',''],unsupported:['当前平台暂不支持此恢复操作。','']};
 function markup(){
  if(S.reading)return '<p role="status">正在核对原会话和恢复记录…</p>';
  if(S.executing)return '<p role="status">'+progress[S.action]+'</p><p>关闭窗口后仍会继续，可重新打开查看。</p>';
  if(S.choice==='cancelPreparation')return '<p><strong>取消这次启动准备？</strong></p><p>尚未创建原件保管目录。确认后重新核对服务已停止，再取消这次未完成的启动准备。</p><p>保持当前配置，不恢复或删除文件，也不会自动启动。</p>';
  if(S.choice==='finishUnchanged')return '<p><strong>核对并结束原会话？</strong></p><p>确认后重新核对服务已停止、原件完整且未改变，再结束这次未完成会话。</p><p>客户端配置保持不变，不会自动启动服务。</p>';
  if(S.choice==='rollback')return '<p><strong>撤销本次恢复？</strong></p><p>将本次恢复的文件退回恢复前的停止状态。启动前原件仍保留，服务保持停止。</p><p>之后可以重新准备恢复，再确认恢复启动前的原件。</p>';
  if(S.choice==='rearm')return '<p><strong>重新准备原件恢复？</strong></p><p>核对保留的原件和当前文件，准备新的恢复计划。准备完成后，还需点击“确认继续恢复”。</p><p>此次准备不会更改客户端配置，服务保持停止。</p>';
  if(S.finalized)return '<p role="status">'+(S.action==='cancelPreparation'?'启动准备已取消，当前配置保持不变。':S.action==='finishUnchanged'?'原件保持完整，原会话已结束。':S.action==='continue'?'客户端配置恢复已完成。':'配置恢复收尾已完成。')+'</p><p>'+(S.ready?'服务保持停止，现在可以启动。':'当前运行状态尚未核实，核对通过后才能启动。')+'</p>'+(S.refreshError?'<p role="alert" class="model-error">'+esc(S.refreshError)+'</p>':'');
  if(S.refreshError)return '<p role="status">本次恢复已撤销。</p><p>当前恢复状态尚未核实，服务保持停止。</p><p role="alert" class="model-error">'+esc(S.refreshError)+'</p>';
  if(S.error)return '<p role="alert" class="model-error">'+esc(S.error)+'</p>';
  const summary=preparing()?['启动准备已中断，尚未开始捕获原件。','可核对后取消本次准备，保持当前配置。']:early()?['启动前原件保持完整，原会话尚未结束。','可以重新核对服务已停止后结束原会话。客户端配置保持不变。']:rolled()?['本次恢复已撤销，文件已回到恢复前的停止状态。','启动前原件仍保留，尚未恢复；服务保持停止。']:continuing()?['客户端配置恢复中断，可以继续原恢复计划。','确认后恢复到本次启动前的配置，保留当前已归档的文件。服务保持停止。']:summaries[S.scene]||summaries.eligible;
  const details=!['unknown','none','unsupported'].includes(S.scene)?'<details style="margin-top:16px"><summary>诊断详情</summary><dl class="p24-facts"><dt>会话阶段</dt><dd>'+(preparing()?'正在登记原会话':early()?'原件已保存':rolled()?'本次恢复已撤销':S.scene==='active'?'服务会话已就绪':'配置恢复待核对')+'</dd><dt>客户端原件</dt><dd>'+ (preparing()?'尚未登记':'已核对') +'</dd><dt>恢复计划</dt><dd>'+ (early()||preparing()?'尚未登记':'已核对') +'</dd><dt>完成记录</dt><dd>'+(rolled()?'已核对':early()||preparing()||continuing()?'尚未登记':'待确认')+'</dd><dt>会话编号</dt><dd><code>f394bd75-355b-4873-83cc-6818efeb547a</code></dd></dl></details>':'';
  return '<p role="status">'+summary[0]+'</p>'+(summary[1]?'<p>'+summary[1]+'</p>':'')+details;
 }
 function run(action){
  if(S.executing)return;
  S.action=action;S.choice=null;S.executing=true;S.confirmations++;render();
  root.setTimeout(()=>{
   S.executing=false;
   if(S.scene===action+'-error'||action==='finishUnchanged'&&S.scene==='early-error'||action==='cancelPreparation'&&S.scene==='prepare-error'){S.error=action==='cancelPreparation'?'启动准备或停止状态尚未核实；原现场保留。':action==='finishUnchanged'?'原件或停止状态尚未核实；原件与现场保留。':action==='rollback'?'撤销恢复未确认完成；原件与当前文件保留。':action==='rearm'?'恢复准备未确认完成；原件与当前文件保留。':'恢复现场已变化，请重新检查。';if(S.open)render();return;}
   S.revision++;S.outcome=true;
   if(action==='rollback')S.phase='restore_rolled_back';
   else if(action==='rearm')S.phase='restore_prepared';
   else S.finalized=true;
   S.ready=S.finalized&&!['refresh-failed','early-refresh-failed','prepare-refresh-failed'].includes(S.scene);
   S.refreshError=S.scene==='rollback-refresh-failed'?'操作已完成，恢复状态读取失败。请刷新恢复状态。':S.finalized&&!S.ready?'收尾已完成，运行状态读取失败。请刷新运行状态。':'';
   if(S.open)render();
  },S.scene.endsWith('-slow')||S.scene==='slow'?1200:250);
 }
 function render(){
  root.document.querySelectorAll('[data-recovery-rollback]').forEach(el=>el.remove());
  const eligible=preparing()||early()||continuing()||['eligible','slow','refresh-failed'].includes(S.scene),busy=S.reading||S.executing;
  const label=busy?null:S.choice==='rollback'?'确认撤销恢复':S.choice==='rearm'?'确认重新准备':S.choice==='finishUnchanged'?'确认结束会话':S.choice==='cancelPreparation'?'确认取消启动准备':S.refreshError?(S.finalized?'刷新运行状态':'刷新恢复状态'):S.finalized?null:S.error?'重新检查':preparing()?'取消启动准备':early()?'核对并结束会话':rolled()?(S.scene==='rolled-back'?'重新检查':'重新准备恢复'):eligible?(continuing()?'确认继续恢复':'确认完成收尾'):'重新检查';
  root.OCXD023.dialog('客户端配置恢复','<div id="clientRecovery024Body" class="p24-dialog">'+markup()+'</div>',label,()=>{
   if(S.choice){run(S.choice);return;}
   if(S.refreshError){S.refreshError='';S.ready=S.finalized;render();return;}
   if(S.error||!eligible||S.scene==='rolled-back'){inspect();return;}
   if(preparing()){S.choice='cancelPreparation';render();return;}
   if(early()){S.choice='finishUnchanged';render();return;}
   if(rolled()){S.choice='rearm';render();return;}
   run(continuing()?'continue':'finalize');
  },{mode:'confirm',hideConfirm:!label,cancelLabel:S.choice?'返回':busy||S.outcome?'关闭':'取消',onCancel:()=>{if(S.choice){S.choice=null;render();}else root.OCXD023.close();},prototypeAnnotation:{title:'客户端恢复 · 固定内存演示',body:'原计划继续、撤销、重新准备、另行确认继续均为内存示例。早期原件保持时只核对并结束会话。无真实API/文件/服务操作，不自动继续、不自动启动。'}});
  S.open=true;$('#modal').classList.add('client-recovery-024-modal');
  const previous=root.__modalOnClose;
  root.__modalOnClose=()=>{S.open=false;S.choice=null;root.document.querySelectorAll('[data-recovery-rollback]').forEach(el=>el.remove());$('#modal').classList.remove('client-recovery-024-modal');previous?.();};
  if(!busy&&!S.choice&&!S.finalized&&!S.refreshError&&!S.error&&continuing()&&!rolled()){
   const button=root.document.createElement('button');button.className='btn';button.textContent='撤销本次恢复';button.dataset.recoveryRollback='';button.onclick=()=>{S.choice='rollback';render();};$('#modalConfirm').before(button);
  }
 }
 function inspect(){if(S.executing)return;S.reading=true;S.choice=null;S.error='';S.refreshError='';render();root.setTimeout(()=>{S.reading=false;if(S.scene==='error')S.error='模拟读取失败，请重新检查。';if(S.open)render();},120);}
 function open(){if(S.executing||S.outcome)render();else inspect();}
 const host=$('#diag-doctor');if(!host)return;
 const entry=root.document.createElement('section');entry.id='clientRecovery024Entry';entry.style.cssText='margin-top:20px;padding-top:16px;border-top:1px solid var(--line)';
 entry.innerHTML='<div class="card-head"><div><h3>客户端配置恢复</h3><p>检查未完成的配置恢复，核对后处理原会话。</p></div><div class="toolbar"><button type="button" class="btn" data-client-recovery024-open>检查客户端恢复</button></div></div>';
 host.querySelector('.selfcheck')?.before(entry);if(!entry.parentNode)host.append(entry);entry.querySelector('button').addEventListener('click',open);
 root.clientRecovery024={open,snapshot:()=>({...S}),setScene:scene=>{if(S.executing)return;Object.assign(S,{scene,outcome:false,finalized:false,ready:false,error:'',refreshError:'',choice:null,phase:scene.startsWith('rearm')?'restore_rolled_back':'restore_prepared',revision:scene.startsWith('rearm')?7:6});if(S.open)inspect();}};
})(window);
