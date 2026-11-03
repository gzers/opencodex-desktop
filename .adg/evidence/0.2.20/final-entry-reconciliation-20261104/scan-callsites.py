from pathlib import Path
import re,json,subprocess
repo=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes');src=repo/'apps/desktop/ui/src';static=[];dynamic=[]
for p in sorted(src.rglob('*')):
 if not p.is_file()or p.suffix not in['.vue','.ts']or 'dev'in p.relative_to(src).parts:continue
 rel=str(p.relative_to(repo));s=p.read_text();frozen=subprocess.check_output(['git','show','bda7c92d:'+rel],cwd=repo).decode();assert s==frozen
 if p.suffix=='.vue':
  for m in re.finditer(r'<([A-Z][A-Za-z0-9]*(?:Dialog|Modal))(?=[\s/>])',s):static.append({'file':rel,'component':m.group(1),'line':s[:m.start()].count(chr(10))+1})
 if rel.endswith('stores/app.ts')or rel.endswith('app/modal/store.ts'):continue
 for m in re.finditer(r'(?:\bapp\.(?:openModal|openInputModal)|\bmodal\.open|\buseModalStore\(\)\.open)\s*\(',s):
  end=s.find('title',m.end());title='dynamic'if end<0 else s[end:s.find(chr(10),end)].strip()[:150];dynamic.append({'file':rel,'line':s[:m.start()].count(chr(10))+1,'call':m.group(0),'titleSource':title})
r={'productSource':'bda7c92da4e1f2e02e6bc2c641e0747748b98dd5','scanWorktreeHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo).decode().strip(),'allScannedSourceBytesEqualProductTag':True,'scope':'static Dialog/Modal tag references and actual AppModal/openInput callers; shared wrappers excluded,not unique window count','static':static,'dynamic':dynamic,'correction':'previous25-call inventory omitted inline useModalStore().open inChannelEditor; source always contained it','limits':'custom runtime/tray/panel/native windows use separate route/native ledger'}
Path('/tmp/ocx020-final-callsite-inventory-bda.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+chr(10));print('static',len(static),'dynamic',len(dynamic));print([x for x in dynamic if 'useModalStore'in x['call']])
