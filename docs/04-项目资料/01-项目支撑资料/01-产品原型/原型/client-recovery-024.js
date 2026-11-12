/* 0.2.24 client recovery: fixed in-memory examples only. No APIs/files/storage. */
(function(root){
 'use strict';
 const $=s=>root.document.querySelector(s), esc=root.OCXD023.esc;
 const css=root.document.createElement('link');css.rel='stylesheet';css.href=new URL('client-recovery-024.css',root.document.currentScript.src).href;root.document.head.append(css);
 const S={scene:'eligible',open:false,reading:false,executing:false,outcome:false,ready:false,error:'',refreshError:'',confirmations:0};
 const continuing=()=>S.scene.startsWith('continue');
 const summaries={eligible:['客户端配置已恢复，尚有一项收尾未完成。','确认后重新核对原会话并完成收尾。客户端配置保持不变。'],pending:['配置恢复尚未完成，原会话已保留。','当前现场不能直接收尾，请查看诊断详情和应用日志。'],unknown:['客户端恢复状态尚未核实。','请重新检查；若仍无法核对，请保留原文件并查看应用日志。'],active:['当前会话正在使用。请先正常停止 OpenCodex，再检查恢复状态。',''],none:['没有待处理的客户端配置恢复。',''],unsupported:['当前平台暂不支持此恢复操作。','']};
 function markup(){
  if(S.reading)return '<p role="status">正在核对原会话和恢复记录…</p>';
  if(S.executing)return '<p role="status">'+(continuing()?'正在继续恢复客户端配置…':'正在完成恢复收尾…')+'</p><p>关闭窗口后仍会继续，可重新打开查看。</p>';
  if(S.outcome)return '<p role="status">'+(continuing()?'客户端配置恢复已完成。':'配置恢复收尾已完成。')+'</p><p>'+(S.ready?'服务保持停止，现在可以启动。':'当前运行状态尚未核实，核对通过后才能启动。')+'</p>'+(S.refreshError?'<p role="alert" class="model-error">'+esc(S.refreshError)+'</p>':'');
  if(S.error)return '<p role="alert" class="model-error">'+esc(S.error)+'</p>';
  const summary=S.scene==='rolled-back'?['本次恢复已撤销，文件已回到恢复前的停止状态。','启动前原件仍保留，尚未恢复；服务保持停止。']:continuing()?['客户端配置恢复中断，可以继续原恢复计划。','确认后恢复到本次启动前的配置，保留当前已归档的文件。服务保持停止。']:summaries[S.scene]||summaries.eligible;
  const details=!['unknown','none','unsupported'].includes(S.scene)?'<details style="margin-top:16px"><summary>诊断详情</summary><dl class="p24-facts"><dt>会话阶段</dt><dd>'+(S.scene==='rolled-back'?'本次恢复已撤销':S.scene==='active'?'服务会话已就绪':'配置恢复待核对')+'</dd><dt>客户端原件</dt><dd>已核对</dd><dt>恢复计划</dt><dd>已核对</dd><dt>完成记录</dt><dd>'+(S.scene==='rolled-back'?'已核对':continuing()?'尚未登记':'待确认')+'</dd><dt>会话编号</dt><dd><code>f394bd75-355b-4873-83cc-6818efeb547a</code></dd></dl></details>':'';
  return '<p role="status">'+summary[0]+'</p>'+(summary[1]?'<p>'+summary[1]+'</p>':'')+details;
 }
 function render(){
  const eligible=continuing()||['eligible','slow','refresh-failed'].includes(S.scene),busy=S.reading||S.executing;
  const label=busy?null:S.outcome&&!S.ready?'刷新运行状态':S.outcome?null:eligible&&!S.error?(continuing()?'确认继续恢复':'确认完成收尾'):'重新检查';
  root.OCXD023.dialog('客户端配置恢复','<div id="clientRecovery024Body" class="p24-dialog">'+markup()+'</div>',label,()=>{
   if(S.outcome){S.ready=true;S.refreshError='';render();return;}
   if(S.error||!eligible){inspect();return;}
   if(S.executing)return;
   S.executing=true;S.confirmations++;render();
   root.setTimeout(()=>{S.executing=false;if(S.scene==='continue-error'){S.error='恢复现场已变化，请重新检查。';if(S.open)render();return;}S.outcome=true;S.ready=S.scene!=='refresh-failed';S.refreshError=S.ready?'':'收尾已完成，运行状态读取失败。请刷新运行状态。';if(S.open)render();},['slow','continue-slow'].includes(S.scene)?1200:250);
  },{hideConfirm:!label,cancelLabel:busy||S.outcome?'关闭':'取消',prototypeAnnotation:{title:'客户端恢复 · 固定内存演示',body:'诊断中心入口、loading/待处理/可收尾/失败/完成/状态刷新均为内存示例。原型不会读取配置、调用服务、提交真实恢复或修改系统。原计划部分中断可显式继续，回退执行尚未接入。'}});
  S.open=true;
  $('#modal').classList.add('client-recovery-024-modal');
  const previous=root.__modalOnClose;
  root.__modalOnClose=()=>{S.open=false;$('#modal').classList.remove('client-recovery-024-modal');previous?.();};
 }
 function inspect(){if(S.executing)return;S.reading=true;S.outcome=false;S.error='';S.refreshError='';render();root.setTimeout(()=>{S.reading=false;if(S.scene==='error')S.error='模拟读取失败，请重新检查。';if(S.open)render();},120);}
 function open(){if(S.executing||S.outcome&&!S.ready)render();else inspect();}
 const host=$('#diag-doctor');if(!host)return;
 const entry=root.document.createElement('section');entry.id='clientRecovery024Entry';entry.style.cssText='margin-top:20px;padding-top:16px;border-top:1px solid var(--line)';
 entry.innerHTML='<div class="card-head"><div><h3>客户端配置恢复</h3><p>检查未完成的配置恢复，核对后处理原会话。</p></div><div class="toolbar"><button type="button" class="btn" data-client-recovery024-open>检查客户端恢复</button></div></div>';
 host.querySelector('.selfcheck')?.before(entry);if(!entry.parentNode)host.append(entry);
 entry.querySelector('button').addEventListener('click',open);
 root.clientRecovery024={open,snapshot:()=>({...S}),setScene:scene=>{if(S.executing)return;Object.assign(S,{scene,outcome:false,ready:false,error:'',refreshError:''});if(S.open)inspect();}};
})(window);
