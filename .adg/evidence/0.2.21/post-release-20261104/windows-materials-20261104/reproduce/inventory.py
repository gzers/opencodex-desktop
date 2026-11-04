import json,subprocess,re,hashlib
from pathlib import Path
root=Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/post021-windows-materials')
out=root.parent/'.toolchain/post021-materials'
def git(*args):return subprocess.check_output(['git','-C',str(root),*args]).decode('utf-8')
def sha(data):return hashlib.sha256(data).hexdigest()
groups={
 'panel':['card','setting-row','install-row','agent-path-row','skill-row','repo-item','backup-item','about-hero','about-panel','mcp-form-card','choice','modal','select-menu','notice','menu-hint','ui-fault'],
 'control':['btn','tag','select-trigger','notif-ctl'],
 'overlay':['ui-dialog','notification-panel','toast','process-progress','card-overlay','env-gate','model-agent-panel'],
 'sticky':['sticky-page-tabs','detail-workspace-head','detail-collapse']}
files=list((root/'apps/desktop/ui/src').rglob('*.vue'))
inventory=[]
for group,names in groups.items():
 for name in names:
  consumers=[]
  for file in files:
   for number,line in enumerate(file.read_text(encoding='utf-8').splitlines(),1):
    if ('class=' in line or ':class=' in line) and re.search(r'(?<![-\w])'+re.escape(name)+r'(?![-\w])',line):consumers.append({'path':file.relative_to(root).as_posix(),'line':number})
  inventory.append({'material':group,'class':name,'sourceConsumers':consumers,'count':len(consumers),'applicability':'production' if consumers else 'legacy or dynamically composed selector; no static production template hit'})
hosts=[]
for file in files:
 for number,line in enumerate(file.read_text(encoding='utf-8').splitlines(),1):
  if '<UiDialog' in line:hosts.append({'path':file.relative_to(root).as_posix(),'line':number,'titles':re.findall(r'(?::title|title)="([^"]*)"',line)})
scope={}
for name in ['apps/desktop/ui/src/components/ui/UiDialog.vue','apps/desktop/ui/src/components/layout/DetailWorkspace.vue']:
 before=git('show','7a8441f7e:'+name);after=(root/name).read_text(encoding='utf-8')
 blocks=lambda text:{key:re.findall(pattern,text,re.S) for key,pattern in {'scripts':r'<script[^>]*>(.*?)</script>','template':r'<template>(.*?)</template>'}.items()}
 a,b=blocks(before),blocks(after)
 scope[name]={key:{'unchanged':a[key]==b[key],'beforeSha256':sha('\n'.join(a[key]).encode()),'afterSha256':sha('\n'.join(b[key]).encode())} for key in a}
changed=git('diff','--name-only','7a8441f7e','HEAD').splitlines()
manifest={'base':'7a8441f7e647e4b3540d5907e44ac4f186f90998','head':git('rev-parse','HEAD').strip(),'docsBase':'c6f0a3f29ae59f6384f61f25dea773061f41bf11','commits':git('log','--format=%H %s','7a8441f7e..HEAD').splitlines(),'files':[{'path':name,'sha256':sha((root/name).read_bytes())} for name in changed],
 'dist':[{'path':file.relative_to(root).as_posix(),'sha256':sha(file.read_bytes()),'bytes':file.stat().st_size} for file in sorted((root/'apps/desktop/ui/dist').rglob('*')) if file.is_file()],
 'unchangedInteractionContracts':scope,'tokensUnchanged':git('diff','7a8441f7e','HEAD','--','apps/desktop/ui/src/styles/tokens.css')=='','syncBusinessUnchanged':git('diff','7a8441f7e','HEAD','--','apps/desktop/ui/src/features/sync','apps/desktop/ui/src/routes/SyncRoute.vue','apps/desktop/tauri/src/modules/sync','apps/desktop/tauri/src/modules/model_config')=='',
 'nativeConfigurationUnchanged':git('diff','7a8441f7e','HEAD','--','apps/desktop/tauri/tauri.conf.json','apps/desktop/tauri/src/lib.rs','apps/desktop/tauri/Cargo.toml','apps/desktop/ui/vite.config.ts','apps/desktop/ui/package.json','apps/desktop/ui/package-lock.json')==''}
(out/'surface-inventory.json').write_text(json.dumps({'groups':groups,'surfaces':inventory,'publicDialogHostCount':len(hosts),'publicDialogHosts':hosts},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'source-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'classes':len(inventory),'publicDialogHosts':len(hosts),'changedFiles':len(changed),'interactionUnchanged':all(v['scripts']['unchanged'] and v['template']['unchanged'] for v in scope.values()),'source':manifest['head']},ensure_ascii=False))
