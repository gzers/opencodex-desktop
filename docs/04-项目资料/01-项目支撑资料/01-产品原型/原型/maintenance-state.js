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
    let serial = 0, noticeSerial = 0;
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
        state.notifications.filter(item=>item.id===eventId && item.object===object && (!stage||!item.stage||item.stage===stage)).forEach(item=>{item.resolved=true;});
      }
      const delivery=context.background && spec.backgroundDelivery ? spec.backgroundDelivery : spec.delivery;
      const key=JSON.stringify([id,object,context.revision||'',context.channel||'',context.stage||'',spec.nature==='一次性'?context.operationId||'':'']);
      const last = state.notifications.find(item => item.key === key && !item.resolved);
      if (last && time-last.lastAt < spec.cooldown) {last.occurrences++; return null;}
      const item = last || {uid:++noticeSerial, key, id, object, severity:spec.severity, category:spec.category, delivery:delivery.slice(), stage:context.stage, read:false, resolved:false, occurrences:0};
      item.lastAt=time; item.occurrences++;
      if (!last) state.notifications.unshift(item);
      state.deliveries.unshift({id, object, time, origin, delivery:delivery.slice()});
      state.deliveries.length=Math.min(state.deliveries.length, 30);
      return spec;
    }
    function resolve(id, object) {state.notifications.filter(item => item.id===id && item.object===object).forEach(item => {item.resolved=true;});}
    function begin(target, phase = 'checking', origin = 'user.action') {
      const u=state.updates[target];
      if (!['checking','applying'].includes(phase)) return null;
      if (!config.jobs.find(job=>job.target===target)?.triggers.includes(origin)) return null;
      if (!u || ['checking','applying'].includes(u.phase)) return null;
      if (phase==='applying' && u.phase!=='available') return null;
      u.origin=origin; u.before=u.phase; u.phase=phase; u.generation++;
      return u.generation;
    }
    function finish(target, generation, outcome, time = Date.now()) {
      const u=state.updates[target];
      if (!u || u.generation!==generation || !['checking','applying'].includes(u.phase)) return false;
      const applying=u.phase==='applying';
      if (!(applying?['complete','failed']:['available','latest','failed']).includes(outcome)) return false;
      u.phase=outcome;
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
      if (u && ['checking','applying'].includes(u.phase)) {u.phase=u.before||'idle'; u.generation++;u.nextDue=time+300000;emit('update.cancelled',target,time,{operationId:target+':'+u.generation});}
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
      state.updatePolicy={channel,checkHours:channel==='stable'?24:channel==='beta'?6:0};
      for(const [target,u] of Object.entries(state.updates)) {
        cancel(target,time);u.generation++;u.phase='idle';u.lastChecked=null;u.failures=0;u.nextDue=time;
        if(target==='manager')u.source=(channel==='manual'?'stable':channel)+' · 签名更新源（示例）';
      }
      return true;
    }
    function confirmApply(target, generation, version, withBackup=true, time=Date.now(), backupValid=true) {
      const u=state.updates[target];
      if(!u||u.phase!=='available'||u.generation!==generation||u.candidate!==version)return null;
      if(withBackup) {
        if(!backupValid){emit('backup.failed','preferences',time);return null;}
        backup('升级前',time);
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
    function backup(reason='手动', time=Date.now()) {
      const item={id:'new-'+(++serial), name:'偏好备份 '+serial, created:time, pinned:false, integrity:'valid', reason, scope:'管理器偏好'};
      state.backups.unshift(item); emit('backup.created', 'preferences', time, {operationId:item.id});
      if (state.policy.cleanup==='after-create') clean(cleanupCandidates(state.backups, state.policy, time).map(item => item.id), time);
      return item;
    }
    function pin(id) {const item=state.backups.find(item => item.id===id); if(item) item.pinned=!item.pinned;}
    function restore(id, time=Date.now()) {
      const item=state.backups.find(item => item.id===id);
      if (!item || item.integrity!=='valid') {emit('backup.failed', 'preferences', time); return false;}
      // 防止本次创建触发轮换后移除待恢复源；恢复事务期间保护源。
      const wasPinned=item.pinned; item.pinned=true;
      const safety=backup('恢复前', time); safety.pinned=true;
      item.pinned=wasPinned;
      emit('backup.restored', 'preferences', time); return true;
    }
    return {state, emit, resolve, begin, finish, cancel, requestChecks, setUpdatePolicy, confirmApply, setScenario, setPolicy, clean, backup, pin, restore};
  }
  return {create, registry, config, definitions, cleanupCandidates};
});
