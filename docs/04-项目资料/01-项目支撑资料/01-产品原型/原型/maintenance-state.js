/* 0.1.10 原型状态：纯内存，无 API、命令、文件或偏好写入。 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.MaintenanceModel = api;
})(typeof globalThis === 'object' ? globalThis : this, function () {
  'use strict';
  const day = 86400000;
  const event = (id, module, trigger, category, severity, nature, delivery, extra = {}) => Object.freeze({
    id, module, trigger, category, severity, nature, delivery,
    source:module==='诊断'?'diagnostic':category==='update'?'update':category==='sync'?'sync':category==='run'?'runtime':'useraction',
    checkPeriod: '按操作', reminderPeriod: '不重复提醒', cooldown: 30000,
    dedupe: '事件 ID + 对象', recovery: '成功后解除同对象失败', retention: '30 天',
    redaction: '只记录对象、固定摘要；不接收路径、令牌、凭据或原始错误', ...extra
  });
  const registry = Object.freeze([
    event('update.available', '更新', '检查发现新版本', 'update', 'info', '状态变化', ['center'], {checkPeriod:'24h（示例）', cooldown:day}),
    event('update.latest', '更新', '用户检查且无新版本', 'update', 'info', '一次性', ['toast', 'log']),
    event('update.failed', '更新', '检查、下载或校验失败', 'update', 'warning', '状态变化', ['toast', 'center', 'log'], {checkPeriod:'24h（示例）', reminderPeriod:'持续失败 24h 再提醒', cooldown:day}),
    event('update.complete', '更新', '受控更新完成', 'update', 'info', '一次性', ['toast', 'center', 'log']),
    event('backup.created', '备份', '管理器偏好备份成功', 'system', 'info', '一次性', ['toast', 'log']),
    event('backup.cleaned', '备份', '确认清理或轮换完成', 'system', 'info', '一次性', ['log']),
    event('backup.failed', '备份', '创建失败、校验不通过', 'system', 'danger', '状态变化', ['toast', 'center', 'log']),
    event('backup.restored', '恢复', '校验、保护当前偏好后恢复', 'system', 'info', '一次性', ['toast', 'center', 'log']),
    event('run.failed', '启动与代理', '代理启动失败', 'run', 'danger', '状态变化', ['toast', 'center', 'system', 'log']),
    event('run.recovered', '启动与代理', '代理恢复就绪', 'run', 'info', '状态变化', ['center', 'log']),
    event('install.failed', '安装', '受控安装失败', 'system', 'danger', '一次性', ['toast', 'center', 'log']),
    event('sync.failed', '同步', '同步失败或覆盖冲突', 'sync', 'warning', '状态变化', ['toast', 'center', 'log'], {checkPeriod:'15min（示例）', reminderPeriod:'持续失败 1h 再提醒', cooldown:3600000}),
    event('diagnostic.risk', '诊断', '周期检查发现环境风险', 'system', 'warning', '周期检查', ['center', 'log'], {checkPeriod:'6h（示例）', reminderPeriod:'风险持续 24h 再提醒', cooldown:day})
  ]);
  const definitions = Object.freeze({
    manager: {label:'桌面管理器', current:'0.1.9', candidate:'0.1.10', source:'stable · 签名更新源（示例）'},
    runtime: {label:'OpenCodex 运行时', current:'2.50.0', candidate:'2.51.0', source:'官方 npm 包（示例）'}
  });
  function cleanupCandidates(items, policy, now) {
    const normal = items.filter(item => !item.pinned).sort((a, b) => b.created - a.created);
    const recent = new Set(normal.slice(0, policy.count).map(item => item.id));
    return normal.filter(item => !recent.has(item.id) && now - item.created > policy.days * day);
  }
  function create(now = Date.now()) {
    let serial = 0, noticeSerial = 0;
    const state = {
      updates: Object.fromEntries(Object.entries(definitions).map(([key, item]) => [key, {...item, phase:'idle', generation:0, lastChecked:null}])),
      policy: {count:10, days:30, cleanup:'manual'},
      updatePolicy: {channel:'stable', checkHours:24},
      backups: Array.from({length:14}, (_, index) => ({id:'sample-'+index, name:'偏好备份 '+(index+1), created:now-index*4*day, pinned:index===13, integrity:index===11?'invalid':'valid', reason:index%2?'升级前':'手动', scope:'管理器偏好'})),
      notifications: [], deliveries: []
    };
    function emit(id, object, time = Date.now()) {
      const spec = registry.find(item => item.id === id);
      if (!spec) throw new Error('未注册事件：'+id);
      if (id==='update.latest' || id==='update.available' || id==='update.complete') resolve('update.failed', object);
      if (id==='backup.created' || id==='backup.restored') resolve('backup.failed', object);
      if (id==='run.recovered') resolve('run.failed', object);
      const key = id+':'+object;
      const last = state.notifications.find(item => item.key === key && !item.resolved);
      if (last && time-last.lastAt < spec.cooldown) {last.occurrences++; return null;}
      const item = last || {uid:++noticeSerial, key, id, object, severity:spec.severity, category:spec.category, read:false, resolved:false, occurrences:0};
      item.lastAt=time; item.occurrences++;
      if (!last) state.notifications.unshift(item);
      state.deliveries.unshift({id, object, time, delivery:spec.delivery.slice()});
      state.deliveries.length=Math.min(state.deliveries.length, 30);
      return spec;
    }
    function resolve(id, object) {state.notifications.filter(item => item.id===id && item.object===object).forEach(item => {item.resolved=true;});}
    function begin(target, phase = 'checking') {
      const u=state.updates[target];
      if (!['checking','applying'].includes(phase)) return null;
      if (!u || ['checking','applying'].includes(u.phase)) return null;
      if (phase==='applying' && u.phase!=='available') return null;
      u.before=u.phase; u.phase=phase; u.generation++;
      return u.generation;
    }
    function finish(target, generation, outcome, time = Date.now()) {
      const u=state.updates[target];
      if (!u || u.generation!==generation || !['checking','applying'].includes(u.phase)) return false;
      const applying=u.phase==='applying';
      if (!(applying?['complete','failed']:['available','latest','failed']).includes(outcome)) return false;
      u.phase=outcome; u.lastChecked=time;
      if (outcome==='complete') u.current=u.candidate;
      emit('update.'+(outcome==='failed'?'failed':outcome==='complete'?'complete':outcome==='latest'?'latest':'available'), target, time);
      return true;
    }
    function cancel(target) {
      const u=state.updates[target];
      if (u && ['checking','applying'].includes(u.phase)) {u.phase=u.before||'idle'; u.generation++;}
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
      state.backups.unshift(item); emit('backup.created', 'preferences', time);
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
    return {state, emit, resolve, begin, finish, cancel, setScenario, setPolicy, clean, backup, pin, restore};
  }
  return {create, registry, definitions, cleanupCandidates};
});
