/* 0.1.10 原型状态：纯内存，无 API、命令、文件或偏好写入。 */
(function (root, factory) {
  const api = factory(typeof module === 'object' && module.exports ? require('./maintenance-events.js') : root.MaintenanceEvents);
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.MaintenanceModel = api;
})(typeof globalThis === 'object' ? globalThis : this, function (config) {
  'use strict';
  const day = 86400000;
  const registry=config.events;
  const definitions = Object.freeze({
    manager: {label:'桌面管理器', current:'0.1.9', candidate:'0.1.10', source:'stable · 签名更新源（示例）'},
    runtime: {label:'OpenCodex 面板', current:'2.50.0', candidate:'2.51.0', source:'官方 npm 包（示例）'}
  });
  function cleanupCandidates(items, policy, now) {
    const normal = items.filter(item => !item.pinned).sort((a, b) => b.created - a.created);
    const recent = new Set(normal.slice(0, policy.count).map(item => item.id));
    return normal.filter(item => !recent.has(item.id) && now - item.created > policy.days * day);
  }
  function create(now = Date.now()) {
    let serial = 0, noticeSerial = 0, operationSerial = 0;
    const state = {
      updates: Object.fromEntries(Object.entries(definitions).map(([key, item]) => [key, {...item, phase:'idle', generation:0, lastChecked:null, nextDue:now+config.jobs.find(job=>job.target===key).startupDelay, failures:0, origin:'user.action'}])),
      policy: {count:10, days:30, cleanup:'manual'},
      updatePolicy: {channel:'stable', checkHours:24},
      backups: Array.from({length:14}, (_, index) => ({id:'sample-'+index, name:'偏好备份 '+(index+1), created:now-index*4*day, pinned:index===13, integrity:index===11?'invalid':'valid', reason:index%2?'升级前':'手动', scope:'管理器偏好'})),
      notifications: [], deliveries: []
    };
    function emit(id, object, time = Date.now(), context = {}) {
      const spec = registry.find(item => item.id === id);
      if (!spec) throw new Error('未注册事件：'+id);
      const origin=context.origin||spec.triggers[0];
      if (!spec.triggers.includes(origin)) throw new Error('事件触发源未注册：'+origin);
      for (const recovery of spec.recovery) {
        const [eventId,stage]=recovery.split(':');
        const fields=spec.recoveryMatch?.[eventId]||[];
        state.notifications.filter(item=>item.id===eventId && item.object===object && (!stage||(item.stage===stage&&context.stage===stage)) && fields.every(field=>typeof context[field]==='string' && context[field].length>0 && item[field]===context[field])).forEach(item=>{item.resolved=true;});
      }
      const delivery=context.background && spec.backgroundDelivery ? spec.backgroundDelivery : spec.delivery;
      const key=JSON.stringify([id,object,context.revision||'',context.channel||'',context.stage||'',context.action||'',context.errorCode||'',spec.dedupeFields?.map(field=>context[field]||'')||[],spec.nature==='一次性'?context.operationId||'':'']);
      const last = state.notifications.find(item => item.key === key && !item.resolved);
      if (last && time-last.lastAt < spec.cooldown) {last.occurrences++; return null;}
      const item = last || {uid:++noticeSerial, key, id, object, severity:spec.severity, category:spec.category, delivery:delivery.slice(), stage:context.stage, action:context.action, operationId:context.operationId, revision:context.revision, channel:context.channel, errorCode:context.errorCode, read:false, resolved:false, occurrences:0};
      item.lastAt=time; item.occurrences++;
      if (!last) state.notifications.unshift(item);
      state.deliveries.unshift({id, object, time, origin, delivery:delivery.slice()});
      state.deliveries.length=Math.min(state.deliveries.length, 30);
      return spec;
    }
    function begin(target, phase = 'checking', origin = 'user.action') {
      const u=state.updates[target];
      if (!['checking','applying'].includes(phase)) return null;
      if (!config.jobs.find(job=>job.target===target)?.triggers.includes(origin)) return null;
      if (!u || ['checking','applying'].includes(u.phase)) return null;
      if (phase==='applying' && u.phase!=='available') return null;
      if (u.phase==='restart-required') return null;
      u.origin=origin; u.before=u.phase; u.phase=phase; u.generation++;
      u.progress={stage:phase==='checking'?'checking':'preparing',percent:null};
      return u.generation;
    }
    function finish(target, generation, outcome, time = Date.now()) {
      const u=state.updates[target];
      if (!u || u.generation!==generation || !['checking','applying'].includes(u.phase)) return false;
      const applying=u.phase==='applying';
      if (!(applying?['complete','failed',...(target==='manager'?['restart-required']:[])]:['available','latest','failed']).includes(outcome)) return false;
      u.phase=outcome;
      if(outcome==='restart-required') {u.progress={stage:'restart-required',percent:null};emit('update.restart.required',target,time,{origin:'operation.result',revision:u.candidate,channel:state.updatePolicy.channel});return true;}
      u.progress={stage:outcome,percent:null};
      if (!applying) {
        const job=config.jobs.find(item=>item.target===target), interval=job.intervals[state.updatePolicy.channel]||day;
        if (outcome==='failed') {u.failures++;u.nextDue=time+(job.retryDelays[u.failures-1]||interval);}
        else {u.lastChecked=time;u.failures=0;u.nextDue=time+interval;}
      }
      if (outcome==='complete') u.current=u.candidate;
      emit('update.'+(outcome==='failed'?'failed':outcome==='complete'?'complete':outcome==='latest'?'latest':'available'), target, time, {origin:applying?'operation.result':u.origin, background:!applying&&u.origin!=='user.action', revision:u.candidate, channel:target==='manager'?state.updatePolicy.channel:'official', stage:applying?'applying':'checking', operationId:target+':'+generation});
      return true;
    }
    function cancel(target, time=Date.now()) {
      const u=state.updates[target];
      if(u?.phase==='applying'&&u.progress?.stage==='installing')return false;
      if (u && ['checking','applying'].includes(u.phase)) {u.phase=u.before||'idle';u.progress={stage:'cancelled',percent:null}; u.generation++;u.nextDue=time+300000;emit('update.cancelled',target,time,{operationId:target+':'+u.generation});}
    }
    function progress(target,generation,stage,percent=null) {
      const u=state.updates[target], stages=['preparing','downloading','verifying','installing'];
      if(!u||u.phase!=='applying'||u.generation!==generation||!stages.includes(stage))return false;
      if(stages.indexOf(stage)<stages.indexOf(u.progress?.stage))return false;
      if(percent!==null&&(!Number.isFinite(percent)||percent<0||percent>100))return false;
      if(stage==='downloading'&&u.progress.stage===stage&&percent!==null&&u.progress.percent!==null&&percent<u.progress.percent)return false;
      u.progress={stage,percent:stage==='downloading'?percent:null};return true;
    }
    function restart(target,time=Date.now()) {
      const u=state.updates[target];
      if(target!=='manager'||u.phase!=='restart-required')return false;
      // 仅模拟重启后的版本回读；等待重启时不能提前改当前版本。
      u.current=u.candidate;u.phase='complete';u.progress={stage:'complete',percent:null};
      emit('update.complete',target,time,{origin:'operation.result',revision:u.candidate,channel:state.updatePolicy.channel,stage:'applying',operationId:target+':'+u.generation});return true;
    }
    function requestChecks(origin, time=Date.now()) {
      if (!Object.hasOwn(config.triggers,origin)) throw new Error('未注册触发源：'+origin);
      const requests=[];
      for (const job of config.jobs.filter(item=>item.implemented==='prototype')) {
        const u=state.updates[job.target];
        if (!job.triggers.includes(origin) || (origin!=='user.action' && (state.updatePolicy.channel==='manual' || time<u.nextDue))) continue;
        const generation=begin(job.target,'checking',origin);
        if (generation!==null) requests.push({target:job.target,generation});
      }
      return requests;
    }
    function setUpdatePolicy(channel,time=Date.now()) {
      if (!['stable','beta','manual'].includes(channel)) return false;
      const u=state.updates.manager;
      if(channel===state.updatePolicy.channel || u.phase==='restart-required' || (u.phase==='applying'&&u.progress?.stage==='installing'))return false;
      state.updatePolicy={channel,checkHours:channel==='stable'?24:channel==='beta'?6:0};
      // 管理器通道不属于面板官方包；保留面板缓存、退避、确认与在途事务。
      cancel('manager',time);u.generation++;u.phase='idle';u.lastChecked=null;u.failures=0;u.nextDue=time;
      u.source=(channel==='manual'?'stable':channel)+' · 签名更新源（示例）';
      return true;
    }
    function confirmApply(target, generation, version, withBackup=true, time=Date.now(), backupValid=true) {
      const u=state.updates[target];
      if(!u||u.phase!=='available'||u.generation!==generation||u.candidate!==version)return null;
      if(withBackup) {
        const context={action:'create',stage:'creating',operationId:'upgrade:'+target+':'+generation};
        if(!backupValid){emit('backup.failed','preferences',time,{...context,errorCode:'creation.failed'});return null;}
        backup('升级前',time,context.operationId);
      }
      return begin(target,'applying');
    }
    function setScenario(target, phase) {
      const u=state.updates[target];
      if (!u || !['idle','available','latest','failed'].includes(phase)) return;
      u.generation++; u.phase=phase;
    }
    function setPolicy(count, days, cleanup) {
      if (!Number.isInteger(count) || count<1 || count>100 || !Number.isInteger(days) || days<1 || days>365 || !['manual','after-create'].includes(cleanup)) return false;
      state.policy={count, days, cleanup}; return true;
    }
    function clean(ids, time = Date.now()) {
      // 每次确认重算；过期预览、固定项、策略内项目不会被删除。
      const allowed = new Set(cleanupCandidates(state.backups, state.policy, time).map(item => item.id));
      const selected = new Set(ids.filter(id => allowed.has(id)));
      state.backups=state.backups.filter(item => !selected.has(item.id));
      if (selected.size) emit('backup.cleaned', 'preferences', time);
      return selected.size;
    }
    function backup(reason='手动', time=Date.now(), operationId='create:'+ ++operationSerial) {
      const item={id:'new-'+(++serial), name:'偏好备份 '+serial, created:time, pinned:false, integrity:'valid', reason, scope:'管理器偏好'};
      state.backups.unshift(item); emit('backup.created', 'preferences', time, {operationId,action:'create',stage:'creating'});
      if (state.policy.cleanup==='after-create') clean(cleanupCandidates(state.backups, state.policy, time).map(item => item.id), time);
      return item;
    }
    function pin(id) {const item=state.backups.find(item => item.id===id); if(item) item.pinned=!item.pinned;}
    function restore(id, time=Date.now()) {
      const item=state.backups.find(item => item.id===id);
      const context={action:'restore',stage:'restoring',operationId:'restore:'+ ++operationSerial};
      if (!item || item.integrity!=='valid') {emit('backup.restore.failed', id, time, {...context,errorCode:item?'integrity.invalid':'source.missing'}); return false;}
      // 防止本次创建触发轮换后移除待恢复源；恢复事务期间保护源。
      const wasPinned=item.pinned; item.pinned=true;
      const safety=backup('恢复前', time); safety.pinned=true;
      item.pinned=wasPinned;
      emit('backup.restored', id, time, context); return true;
    }
    return {state, emit, begin, finish, cancel, progress, restart, requestChecks, setUpdatePolicy, confirmApply, setScenario, setPolicy, clean, backup, pin, restore};
  }
  return {create, registry, config, definitions, cleanupCandidates};
});
