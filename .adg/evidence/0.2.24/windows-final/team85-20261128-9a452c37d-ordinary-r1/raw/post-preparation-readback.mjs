import assert from 'node:assert/strict';
const targets = await (await fetch('http://127.0.0.1:51722/json/list', {signal: AbortSignal.timeout(2000)})).json();
const target = targets.find(x => x.id === '3AC00C5C55DDC676F2A4C8892153A0F4' && x.type === 'page' && new URL(x.url).origin === 'http://tauri.localhost');
assert(target, 'Original isolated Full target is unavailable');
assert.equal(new URL(target.webSocketDebuggerUrl).host, '127.0.0.1:51722');
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {const timer = setTimeout(() => reject(Error('CDP open deadline')), 2000); socket.onopen = () => {clearTimeout(timer); resolve()}; socket.onerror = () => {clearTimeout(timer); reject(Error('CDP connection failed'))}; });
let id = 0;
const evaluate = (expression, limit) => new Promise((resolve, reject) => {
  const currentId = ++id;
  const timer = setTimeout(() => {socket.removeEventListener('message', read); reject(Error('CDP response deadline'));}, limit);
  function read(event) {const message = JSON.parse(event.data); if (message.id !== currentId) return; clearTimeout(timer); socket.removeEventListener('message', read); if (message.error || message.result.exceptionDetails) reject(Error('CDP evaluation failed')); else resolve(message.result.result.value); }
  socket.addEventListener('message', read);
  socket.send(JSON.stringify({id:currentId,method:'Runtime.evaluate',params:{expression,awaitPromise:true,returnByValue:true}}));
});
const capture = async name => {
 const begin = new Date().toISOString();
 const expression = '('+ (async (name) => {
  const begin = performance.now();
  let timer;
  try {
   const raw = await Promise.race([window.__TAURI_INTERNALS__.invoke(name), new Promise((_, reject) => {timer = setTimeout(() => reject({code:'readback_deadline'}), 5000);})]);
   const data = name === 'get_status_snapshot' ? {runtime:raw.matrix?.runtime, source:raw.source, statusFailure:raw.status_failure, canStart:raw.can_start, canStop:raw.can_stop, canRestart:raw.can_restart, pidPresent:raw.pid!=null, portPresent:raw.port!=null, home:raw.facts?.opencodex_home} : {keys:Object.keys(raw), status:raw.status, summary:raw.summary, checks: Array.isArray(raw.checks) ? raw.checks.map(check=>({id:check.id,status:check.status,code:check.code})) : null};
   return {elapsedMs:Math.round(performance.now()-begin), ok:true, data};
  } catch(error) {
   return {elapsedMs:Math.round(performance.now()-begin), ok:false, error: typeof error === 'string' ? error.slice(0,600) : {code:error?.code,kind:error?.kind,detail:typeof error?.detail === 'string' ? error.detail.slice(0,600) : undefined,message:typeof error?.message === 'string' ? error.message.slice(0,600) : undefined}};
  } finally {clearTimeout(timer);}
 }).toString()+')('+JSON.stringify(name)+')';
 return {command:name,startedUtc:begin,...await evaluate(expression, 6500)};
};
try {
 const view = await evaluate('({ route:location.hash, unknown:document.body.innerText.includes("运行状态未知"), retry:document.body.innerText.includes("重试状态"), diagnostics:document.body.innerText.includes("查看诊断"), startButtons:[...document.querySelectorAll("button")].filter(x=>/^启动(?: OpenCodex)?$/.test(x.textContent.trim())).map(x=>x.disabled) })',2500);
 const status = await capture('get_status_snapshot');
 const doctor = await capture('run_doctor');
 console.log(JSON.stringify({kind:'post-preparation-readonly-retry',targetId:target.id,view,status,doctor},null,2));
} finally {socket.close();}
