import {test} from 'node:test'
import assert from 'node:assert/strict'
import {readFile} from 'node:fs/promises'
import {runInNewContext} from 'node:vm'
import {runtimeInstallHost,runtimeInstallExpression,assertRuntimeInstallTerminal,closeRuntimeInstallTerminal} from './windows-021-runtime-host.mjs'
import {confirmNativeBackupDirectory} from './windows-path-ui.mjs'

function installFixture(version,change={}){
 const publicDialogs=version==='0.2.21',state={status:'安装完成 · 2.91.0 · 联网安装',error:'',outcome:'入口 C:/owned/runtime/ocx.mjs · SHA-256 abcdef',phase:'done',installing:false,taskOutcome:{version:'2.91.0'},taskError:'',progress:'100',footerCount:publicDialogs?0:1,closeCount:1,closeEnabled:true,legacyFinish:!publicDialogs,retryEnabled:false,present:true,native:publicDialogs,...change}
 class NativeDialog {}
 const d=state.native?new NativeDialog():{}
 d.querySelector=selector=>selector==='[data-testid=wiz-status]'?{textContent:state.status}:selector==='[data-testid=wiz-error]'?state.error?{textContent:state.error}:null:selector==='[data-testid=wiz-outcome]'?state.outcome?{textContent:state.outcome}:null:selector==='[data-testid=wiz-finish]:not(:disabled)'?state.legacyFinish?{}:null:selector==='[data-testid=wiz-allow-scripts]:not(:disabled)'?state.retryEnabled?{}:null:selector==='[role=progressbar]'?{getAttribute:()=>state.progress}:selector==='[data-testid=wiz-console]'?{textContent:'日志：owned/console.log'}:null
 d.querySelectorAll=selector=>selector==='footer'?Array(state.footerCount).fill({}):selector==='.ui-dialog-header .modal-close'?Array(state.closeCount).fill({disabled:!state.closeEnabled}):[]
 const runtime={install:{phase:state.phase},installing:state.installing,installOutcome:state.taskOutcome,installError:state.taskError}
 const document={querySelector:selector=>selector==='#app'?{__vue_app__:{config:{globalProperties:{$pinia:{state:{value:{runtime}}}}}}}:selector===runtimeInstallHost(version).selector&&state.present?d:null}
 const clicks=[],evaluate=async expression=>JSON.parse(JSON.stringify(runInNewContext(expression,{document,HTMLDialogElement:NativeDialog})))
 const driver={evaluate,wait:async expression=>{const value=await evaluate(expression);assert(value,'terminal/closure wait condition');return value},click:async(label,selector)=>{clicks.push({label,selector});state.present=false}}
 return {state,runtime,driver,clicks,read:()=>evaluate(runtimeInstallExpression(version))}
}
test('021 completion comes from outcome plus settled task done and 100% progress, without a finish button',async()=>{
 const f=installFixture('0.2.21'),before=structuredClone(f.runtime),scene=await f.read()
 assert.equal(scene.legacyFinish,false);assert.equal(scene.done,true);assert.equal(scene.phase,'done');assert.equal(scene.progress,'100')
 assertRuntimeInstallTerminal(scene,'0.2.21','done')
 const closure=await closeRuntimeInstallTerminal(f.driver,'0.2.21','done')
 assert.equal(closure.exit,'public-header-X')
 assert.deepEqual(f.clicks,[{label:'关闭',selector:'dialog.runtime-install-dialog[open][data-testid=runtime-install-modal] > .ui-dialog-header'}])
 assert.deepEqual(f.runtime,before,'observation must not mutate production task state')
})
test('021 never accepts outcome-only, premature done, busy/error, missing X or an empty completion footer',async()=>{
 for(const change of [{outcome:''},{phase:'registering'},{installing:true},{taskOutcome:null},{taskError:'readback failed'},{error:'failed'},{status:'正在核对安装结果'},{progress:null},{progress:'99'},{native:false},{closeCount:0},{closeCount:2},{closeEnabled:false},{footerCount:1},{present:false}]){
  const f=installFixture('0.2.21',change),scene=await f.read()
  assert.throws(()=>assertRuntimeInstallTerminal(scene,'0.2.21','done'),JSON.stringify(change))
  await assert.rejects(closeRuntimeInstallTerminal(f.driver,'0.2.21','done'));assert.deepEqual(f.clicks,[])
 }
})
test('020 still requires and uses its enabled completion footer, never the 021 X contract',async()=>{
 const f=installFixture('0.2.20');assert((await f.read()).done)
 await closeRuntimeInstallTerminal(f.driver,'0.2.20','done')
 assert.deepEqual(f.clicks,[{label:'完成',selector:'[data-testid=runtime-install-modal]'}])
 const incomplete=installFixture('0.2.20',{legacyFinish:false})
 assert.equal((await incomplete.read()).done,false);await assert.rejects(closeRuntimeInstallTerminal(incomplete.driver,'0.2.20','done'));assert.deepEqual(incomplete.clicks,[])
})
for(const version of ['0.2.20','0.2.21'])test(version+' real settled install failure closes using its versioned host',async()=>{
 const f=installFixture(version,{status:'安装失败',error:'npm pack E404 日志：owned/failure.log',outcome:'',phase:'failed',taskOutcome:null,progress:null,legacyFinish:false,taskError:'npm pack'})
 const scene=await f.read();assert.equal(scene.done,false);assert.equal(scene.scriptsEnabled,false,'an absent retry button is not an enabled retry')
 await closeRuntimeInstallTerminal(f.driver,version,'failed')
 assert.equal(f.clicks[0].label,'关闭');assert.equal(f.clicks[0].selector,runtimeInstallHost(version).selector+(version==='0.2.21'?' > .ui-dialog-header':''))
})
test('021 failure waits for task settlement before enabling X closure',async()=>{
 const f=installFixture('0.2.21',{status:'安装失败',error:'npm pack E404',outcome:'',phase:'failed',installing:true,taskOutcome:null,legacyFinish:false})
 const originalWait=f.driver.wait;let settled=false
 f.driver.wait=async expression=>{
  if(!settled){assert.equal(await f.driver.evaluate(expression),false);assert.deepEqual(f.clicks,[]);f.runtime.installing=false;settled=true}
  return originalWait(expression)
 }
 await closeRuntimeInstallTerminal(f.driver,'0.2.21','failed');assert.equal(f.clicks.length,1)
})
test('unsupported install versions fail before DOM evaluation or actions',async()=>{
 for(const version of ['0.2.22','0.2.21-rc.1','v0.2.21','0.2.021',null]){
  let calls=0;assert.throws(()=>runtimeInstallHost(version))
  await assert.rejects(closeRuntimeInstallTerminal({wait:async()=>{calls++},click:async()=>{calls++}},version,'done'));assert.equal(calls,0)
 }
})
for(const version of ['0.2.20','0.2.21'])test(version+' backup open confirmation uses the correct native or legacy host then waits for close',async()=>{
 let open=true;const native=version==='0.2.21',selector=native?'dialog.app-modal-dialog[open]':'.modal-mask [role=dialog]',clicks=[]
 class Dialog {}
 const d=native?new Dialog():{};d.open=true;d.textContent='打开备份目录 备份目录可能包含敏感信息 继续打开'
 const document={querySelector:actual=>{assert.equal(actual,selector);return open?d:null},querySelectorAll:actual=>{assert.equal(actual,'dialog[open]');return open?[d]:[]}}
 await confirmNativeBackupDirectory({wait:async expression=>assert(runInNewContext(expression,{document,HTMLDialogElement:Dialog})),click:async(text,root)=>{clicks.push({text,root});open=false}},version)
 assert.deepEqual(clicks,[{text:'继续打开',root:selector}])
})
test('021 path confirmation refuses a foreign title or a second native layer without opening a directory',async()=>{
 for(const title of ['别的确认','打开备份目录']){
  let clicks=0;class Dialog{};const d=new Dialog();d.open=true;d.textContent=title
  const document={querySelector:()=>d,querySelectorAll:()=>title==='打开备份目录'?[d,d]:[d]}
  await assert.rejects(confirmNativeBackupDirectory({wait:async expression=>assert(runInNewContext(expression,{document,HTMLDialogElement:Dialog})),click:async()=>{clicks++}},'0.2.21'))
  assert.equal(clicks,0)
 }
})
test('the concrete install and recovery drivers invoke the shared terminal contract, without unconditional legacy exits',async()=>{
 const install=await readFile(new URL('./windows-install-ui.mjs',import.meta.url),'utf8'),recovery=await readFile(new URL('./windows-install-recovery.mjs',import.meta.url),'utf8'),path=await readFile(new URL('./windows-path-ui.mjs',import.meta.url),'utf8')
 assert(install.includes('evaluate(runtimeInstallExpression(about.version))'));assert(install.includes("closeRuntimeInstallTerminal({evaluate,wait,click},about.version,'done')"))
 for(const terminal of ['done','failed'])assert(recovery.includes("report.about.version,'"+terminal+"')"))
 assert(path.includes('confirmNativeBackupDirectory({wait,click},about.version)'))
 for(const source of [install,recovery]){assert(!source.includes("await click('完成'"));assert(!source.includes("await click('关闭')"));assert(!source.includes('wiz-finish'))}
})
