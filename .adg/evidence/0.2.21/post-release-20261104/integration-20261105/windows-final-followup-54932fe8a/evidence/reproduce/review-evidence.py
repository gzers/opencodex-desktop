import json,hashlib,shutil,subprocess
from pathlib import Path
b=Path(__file__).resolve().parent;w=b.parent.parent;r=w/'post021-windows-materials';d=w/'docs-post021-windows-materials'
evrel='.adg/evidence/0.2.21/post-release-20261104/integration-20261105/windows-final-20261105';ev=d/evrel
def cp(src,dst):dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
def dump(p,value):p.write_text(json.dumps(value,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8',newline=chr(10))
for label in ['model-regression','native-readonly-reviewed','native-first-scope-reviewed','native-channels-reviewed','native-panel-reviewed','native-panel-address-reviewed','native-panel-recovery-reviewed','failures-reviewed']:
 for p in (b/label).iterdir():
  if p.is_file() and p.suffix in ['.json','.png','.jpg'] and p.name!='launch.json':cp(p,ev/'reviewed'/label/p.name)
for name in ['model-native.log','model-complete.log','model-regression.log','native-probe-reviewed-final.log','readonly-reviewed-summary.json','models-reviewed-requests.jsonl','models-reviewed-identity.json','panel-reviewed-service-identity.json','panel-reviewed-first-service-stop.json','panel-reviewed-service.log','panel-reviewed-new-service.log','tooling-reviewed.json','cleanup-reviewed.json','cleanup-reviewed-final.json','https-requests.json']:
 cp(b/name,ev/'reviewed'/name)
for label in ['model-regression','integrated','first-scope-reviewed']:
 cp(b/label/'sandbox/logs/app.log',ev/'reviewed'/label/'manager.log')
for p in (b/'model-regression/sandbox/backups/model-config').rglob('*'):
 if p.is_file():cp(p,ev/'reviewed/model-regression/backups'/p.relative_to(b/'model-regression/sandbox/backups/model-config'))
cp(b/'model-regression/sandbox/opencodex-home/config.json',ev/'reviewed/model-regression/config-readback.json')
for p in (b/'backend-tests').glob('*.log'):cp(p,ev/'reviewed/backend'/p.name)
for name in ['resume-pinned.py','fresh-scope.py','models-reviewed.py','panel-server-reviewed.ts','status-reviewed.ts','review-evidence.py']:
 cp(b/name,ev/'reproduce'/name)
cp(r/'test/smoke/post021-windows-final/native-regression.mjs',ev/'reproduce/native-regression.mjs')
head=subprocess.check_output(['git','-C',str(r),'rev-parse','HEAD'],text=True).strip()
assert subprocess.run(['git','-C',str(r),'diff','--quiet','bd7c0fe8c','--','apps']).returncode==0
manifest=json.loads((ev/'source-manifest.json').read_text(encoding='utf-8-sig'));manifest.update(codeCommit=head,productBuildCommit='bd7c0fe8ccecfa39e0706c885dc233f3854dab74',appsTreeUnchanged=True,rawNativeLogs='reviewed/*/manager.log',changedCodeFiles=subprocess.check_output(['git','-C',str(r),'diff','--name-only','bd7c0fe8c..HEAD'],text=True).splitlines(),dailyServiceCleanupIncident=True,officialDefaultAdapterVerified=False)
dump(ev/'source-manifest.json',manifest)
coverage={'passed':['frontend full 197 files/1289 tests','GNU debug custom-protocol build','native probe final 45 tests','model-regression full windows-webview runner passed on frozen product','exact rename ordinary prewrite/recovery pre-post image assertions','explicit Node retry 18 Windows environment tests; 981 other full-library tests originally passed','four readonly version states with business byte invariance','fresh scope exact local channel/template ID selection; remote fixture had no model objects','Chinese-name channel real loopback discovery/save; generated key; openai display-name full reopen; node measurement config invariant','ordinary panel reopen/notification overlay recovery on official GUI 2.84','manual retry recovers failed address-change rapid reentry'],
 'failedOrUnverified':['physical desktop performance: user only has RDP','MSVC formal build: compiler/SDK unavailable','formal packaged Codex default restart/config readback: not called; default enumeration null','address change plus rapid reentry initially reaches load-failed; retry passed; root cause unconfirmed','native old-view close failure injection: not performed; unit evidence only','full single Rust run with explicit Node: not repeated; only failure group rerun','isolation invariant: fixed-PID cleanup stopped daily installed service PID30196'],
 'scopeLimits':['native sync setup fixture only included preference content remotely; all-object incoming catalogue case not proven by it','sync direction and bounded ETag checks appear in Rust tests; no new native end-to-end ETag publication scenario','Bun1.4.4 copied to isolated tools for official2.84 panel server; daily runtime service not used by it','early reentry driver sets real route store rapidly, not physical mouse input','native screenshots captured by real WebView2 CDP and one Windows sky window capture; not headless'],
 'userDecision':'没有，只有rdp','result':'incomplete','dailyServiceProcessTouched':True,'dailyConfigEditRequested':False,'formalCodexRestartCalled':False,'published':False,'mergedMain':False}
dump(ev/'COVERAGE.json',coverage)
env=json.loads((ev/'environment.json').read_text(encoding='utf-8-sig'));env.update(sourceCommit=head,dailyProcessTouched=True,dailyServiceCleanupIncident='PID30196 from daily installed runtime stopped by prior fixed-PID cleanup; replacement66908 observed, left untouched by reviewed cleanup',listenersAfterCleanup=[]);dump(ev/'environment.json',env)
incident={'action':'prior fixed-PID Stop-Process included PID30196','observedPath':'D:/Software/Develop/OpenCodeX Desktop/data/runtime/opencodex/node_modules/bun/bin/bun.exe','violation':'daily runtime service outside isolated task root stopped; contrary to user isolation instruction','observedAfter':{'pid':66908,'created':'2026-11-05T01:45:17.426602+08:00'},'manager':{'pid':24140,'created':'2026-11-04T19:58:15.083403+08:00'},'limits':'no proof of config byte invariance across this incident; cannot certify zero daily-side effect; no compensating restart/config change attempted','mitigation':'reviewed cleanup matches isolated executable/script plus PID and creation date; excludes daily service and manager','userInformed':True}
dump(ev/'isolation-incident.json',incident)
print(json.dumps({'head':head,'reviewedFiles':sum(p.is_file()for p in (ev/'reviewed').rglob('*')),'isolationPassed':False}))
