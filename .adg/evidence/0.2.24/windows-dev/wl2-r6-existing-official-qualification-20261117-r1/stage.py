from pathlib import Path
import hashlib,json,shutil
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');PUBLIC=Path('C:/Users/Public/ocx-r6-official-existing-20261117-r1')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest()
assert load(ROOT/'product01/result.json')['exitCode']==0
assert '--features' in load(ROOT/'product01/result.json')['arguments']
source=load(ROOT/'source-commit.json');assert load(ROOT/'product01/source.json')['head']==source['head']
PUBLIC.mkdir(exist_ok=False);(PUBLIC/'input').mkdir();(PUBLIC/'out01').mkdir()
files=[]
for name in ['opencodex-desktop.exe','ocxd.exe','ocx.exe']:
    p=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug'/name
    dst=PUBLIC/'input'/name;shutil.copyfile(p,dst);b=dst.read_bytes();assert sha(b)==sha(p.read_bytes())
    files.append(dict(name=name,bytes=len(b),sha256=sha(b),source=str(p)))
bundle=Path('C:/Users/Public/ocxw1d-98b5378c87/input/runtime-bundle-0.2.24-win32-x64.tar.gz')
dst=PUBLIC/'input'/bundle.name;shutil.copyfile(bundle,dst);b=dst.read_bytes();assert sha(b)=='14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7'
files.append(dict(name=dst.name,bytes=len(b),sha256=sha(b),source=str(bundle)))
for name in ['launch-r6-official-existing.cmd','launch-r6-official-existing.ps1','export-r6-official-existing.ps1']:
    p=CODE/'test/windows024-native'/name;dst=PUBLIC/name;shutil.copyfile(p,dst);assert dst.read_bytes()==p.read_bytes()
manifest=dict(scope='isolated-r6-existing-official-qualification',expectedSid='S-1-5-21-587408064-856152790-2583328096-1003',expectedSession=7,sourceHead=source['head'],sourceTree=source['tree'],files=files,actualProductionExecuted=False,publicStartOpen=False)
with (PUBLIC/'input-manifest.json').open('x',encoding='utf-8') as f:json.dump(manifest,f,indent=2)
with (ROOT/'staged-input.json').open('x',encoding='utf-8') as f:json.dump(dict(public=str(PUBLIC),**manifest,scripts=[dict(path=str(PUBLIC/n),bytes=(PUBLIC/n).stat().st_size,sha256=sha((PUBLIC/n).read_bytes())) for n in ['launch-r6-official-existing.cmd','launch-r6-official-existing.ps1','export-r6-official-existing.ps1']]),f,indent=2)
print(json.dumps(dict(head=source['head'],public=str(PUBLIC),files=len(files),actualProductionExecuted=False)))
