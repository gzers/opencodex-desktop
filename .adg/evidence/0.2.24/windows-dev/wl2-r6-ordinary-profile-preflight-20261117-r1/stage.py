from pathlib import Path
import hashlib,json,re,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
PUBLIC=Path('C:/Users/Public/ocx-r6-profile-preflight-20261117-r1')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
assert load(ROOT/'library01/result.json')['exitCode']==0
log=(ROOT/'library01/stdout.log').read_text(encoding='utf-8-sig')
assert '1 passed; 0 failed; 1 ignored' in log
stderr=(ROOT/'library01/stderr.log').read_text(encoding='utf-8-sig')
matches=re.findall(r'Running unittests[^\r\n]*\(([^\r\n]+\.exe)\)',stderr);assert len(matches)==1,matches
exe=Path(matches[0].replace('\\','/'));exe=exe if exe.is_absolute() else CODE/exe
exe=exe.resolve();assert exe.is_relative_to(CODE/'test/windows024-native/target/w1c-product')
head=subprocess.check_output(['git','-C',str(CODE),'rev-parse','HEAD']).decode().strip()
scope=load(ROOT/'source-commit.json');assert scope['head']==head
PUBLIC.mkdir(exist_ok=False)
inp=PUBLIC/'input';inp.mkdir();(PUBLIC/'out01').mkdir()
files=[]
for source,name in [(exe,'r6-profile-tests.exe')]:
    target=inp/name;shutil.copyfile(source,target);raw=target.read_bytes();assert sha(raw)==sha(source.read_bytes())
    files.append(dict(name=name,bytes=len(raw),sha256=sha(raw),source=str(source)))
for name in ['launch-r6-profile-preflight.cmd','launch-r6-profile-preflight.ps1','export-r6-profile-preflight.ps1']:
    source=CODE/'test/windows024-native'/name;target=PUBLIC/name
    shutil.copyfile(source,target);assert target.read_bytes()==source.read_bytes()
manifest=dict(scope='r6-ordinary-existing-baseline-preparation',expectedSid='S-1-5-21-587408064-856152790-2583328096-1003',expectedSession=7,sourceHead=head,testRustProducer='library01',scriptOnlyAfterBuildChanges=scope['scriptOnlyAfterBuildChanges'],files=files,actualProductionExecuted=False,startOpen=False)
with (PUBLIC/'input-manifest.json').open('x',encoding='utf-8') as f:json.dump(manifest,f,indent=2)
with (ROOT/'staged-input.json').open('x',encoding='utf-8') as f:json.dump(dict(public=str(PUBLIC),**manifest,stagedFiles=[dict(path=str(p),bytes=p.stat().st_size,sha256=sha(p.read_bytes())) for p in sorted(PUBLIC.rglob('*')) if p.is_file()]),f,indent=2)
print(json.dumps(dict(public=str(PUBLIC),sourceHead=head,testExeSHA256=files[0]['sha256'],actualProductionExecuted=False)))
