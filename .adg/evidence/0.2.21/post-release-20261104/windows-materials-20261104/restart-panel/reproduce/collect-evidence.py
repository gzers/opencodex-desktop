import json,hashlib,shutil,subprocess
from pathlib import Path
base=Path(__file__).resolve().parent;workspace=base.parent.parent
repo=workspace/'post021-windows-materials';docs=workspace/'docs-post021-windows-materials'
ev=docs/'.adg/evidence/0.2.21/post-release-20261104/windows-materials-20261104/restart-panel'
def dump(path,data):
    path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8',newline=chr(10))
def copy(source,target):
    target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
def git(*args):return subprocess.check_output(['git','-C',str(repo),*args])
head=git('rev-parse','HEAD').decode().strip()
for label in ['baseline','candidate']:
    root=base/label
    names=['delivery-stopped-terminal.json','delivery-after-panel-save.json','delivery-after-panel-restart.json','delivery-after-panel-owned.json','delivery-after-panel-readback.json','stopped-service-panel.json','panel-cold-foreground.json','panel-warm-final.json','cold-service-completed.json','cold-service-first.json','missing-record-repeat.json','restart-before.json','restart-toast.json','delivery-panel-cold.json','delivery-ordinary-save.json','delivery-ordinary-restart.json','delivery-ordinary-owned.json','delivery-ordinary-readback.json','delivery-timeout.json','delivery-timeout-recovery.json','delivery-reopened.json','delivery-overlay-open.json','delivery-overlay-close.json','missing-record-final.json','missing-record-ui-final.json','corrupt-record-final.json','stale-listener-final.json','missing-token-final.json','missing-saved-config-final.json','service-stopped-final.json','no-panel-presence.json','save-no-panel-valid.json','no-panel-valid-owned-restart.json','no-panel-valid-readback.json','rename-local-fixture-final.json','rename-local-restart-final.json','rename-local-retry-final.json','rename-local-stable-final.json','rename-local-owned-restart.json','rename-local-readback.json','rename-local-client-readback.txt','native-client-binding.json','models-fixture.jsonl','warm-service-server-identity.json','warm-service-server.stdout','server-identity.json','server.stdout','server.stderr','manifest-adaptation.json']
    for name in names:
        if(root/name).exists():copy(root/name,ev/'raw'/label/name)
    copy(root/'sandbox/logs/app.log',ev/'raw'/label/'manager.log')
    launch=json.loads((root/'launch.json').read_text(encoding='utf-8'));dump(ev/'raw'/label/'launch-identity.json',{k:v for k,v in launch.items() if k!='environment'})
for name in ['frontend-final.log','frontend-build-final.log','delivery-build.log','fmt-final.log','clippy-final.log','restart-ui-tests.log','w2-ui-targeted.log']:
    copy(base/name,ev/'checks'/name)
for path in (base/'backend-tests').glob('*.log'):copy(path,ev/'checks/backend'/path.name)
for name in ['frontend-full.log','panel-cargo-check.log','panel-ui-tests.log','ui-recheck.log','clippy.log']:
    copy(base/name,ev/'failures'/name)
for label,names in [('baseline',['panel-first.json','panel-first-real.json','panel-process-cold-final.json']),('candidate',['invalid-no-panel-final-no-panel-owned-restart.json','panel-process-cold-final.json','panel-cold-valid.json','panel-timeout-native.json','rename-restart.json','rename-retry-managed.json'])]:
    for name in names:
        if(base/label/name).exists():copy(base/label/name,ev/'failures'/label/name)
for name in ['launch.py','run-service.py','official-server.ts','models-fixture.ts','status-driver.ts','native-errors.py','panel-trace-final.js','prepare-panel.js','timeout-native.js','timeout-recover.js','restart-call.js','save-rename.js','save-delivery.js','ui-restart-final.js','run-final-tests.py']:
    copy(base/name,ev/'reproduce'/name)
copy(base.parent/'cdp-eval.mjs',ev/'reproduce/cdp-eval.mjs')
copy(base/'sources.json',ev/'governance-sources.json');copy(base/'native-environment.json',ev/'native-environment.json')
copy(docs/'.adg/evidence/0.2.21/post-release-20261104/windows-materials-20261104/environment-os.json',ev/'environment-os.json')
for path in (ev/'after').glob('final-no-panel-*.jpg'):
    target=ev/'failures/mislabeled-screenshots'/path.name;target.parent.mkdir(parents=True,exist_ok=True);path.rename(target)
paths=git('diff','--name-only','684e85c29..HEAD').decode().splitlines()
production=[p for p in paths if '/src/' in p]
differences=[p for p in production if git('show','cc62887fc:'+p)!=git('show',head+':'+p)];assert differences==['apps/desktop/ui/src/routes/PanelRoute.vue']
manifest={'codeCommit':head,'buildCommit':head,'docsSource':json.loads((base/'sources.json').read_text(encoding='utf-8'))['implementation']['commit'],'baselineCode':'684e85c29f76584de4ab53026dcc315e34b57c62','productSourceEquivalentSupplementalBuild':'cc62887fc5f5d3024d3665dc9f7e1892706a62ea','changedPaths':[{'path':p,'blob':git('rev-parse',head+':'+p).decode().strip(),'sha256':hashlib.sha256(git('show',head+':'+p)).hexdigest()} for p in paths],'frontendAssets':[{'path':str(p.relative_to(repo)).replace(chr(92),'/'),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in (repo/'apps/desktop/ui/dist').rglob('*') if p.is_file()],'nativeExeSha256':hashlib.sha256((base/'candidate/opencodex-desktop.exe').read_bytes()).hexdigest(),'exePublished':False,'productionChangesAfterSupplementalBuild':True,'sourceDifferencesAfterSupplementalBuild':differences,'backendProductionSourceEquivalent':all(git('show','cc62887fc:'+p)==git('show',head+':'+p) for p in production if p.startswith('apps/desktop/tauri/'))}
dump(ev/'source-manifest.json',manifest)
print('collected evidence for '+head)
