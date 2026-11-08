"""Copy complete controlled round evidence, keeping its original relative paths.
Private browser/sandbox resource trees are excluded explicitly and listed. Raw
installers stay in their immutable input artifact and are bound by the receipt.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

root=Path(sys.argv[1]).resolve()
output=root/'artifact'
output.mkdir(exist_ok=False)
files=[]
excluded=[]
for directory in ['sweep','evidence','current-runtime-stage/maintained-runtime']:
    source=root/directory
    if not source.exists():continue
    for current, dirs, names in os.walk(source, followlinks=False):
        base=Path(current)
        for name in list(dirs):
            p=base/name
            if p.is_symlink() or name.startswith(('sandbox-','OpenCodeX Native ')) or name in ['.git','WebView2','EBWebView','node_modules']:
                dirs.remove(name);excluded.append(dict(path=str(p.relative_to(root)),reason='owned private sandbox/browser/resource tree; controlled receipts/logs retained separately'))
        for name in names:
            p=base/name
            if p.is_symlink():raise ValueError('Evidence symlink refused: '+str(p.relative_to(root)))
            if p.suffix not in {'.json','.log','.png','.txt','.bin','.sha256'} and not (directory=='current-runtime-stage/maintained-runtime' and p.suffix=='.tgz'):continue
            relative=p.relative_to(root)
            target=output/relative;target.parent.mkdir(parents=True,exist_ok=True)
            shutil.copy2(p,target)
            files.append(dict(path=str(relative),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
p=root/'input/build-identities.json'
if p.exists():
    target=output/'input/build-identities.json';target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,target)
with (output/'evidence-manifest.json').open('x',encoding='utf-8') as f:
    json.dump(dict(accepted=False,originalsPreserved=True,files=files,excluded=excluded),f,ensure_ascii=False,indent=2)
