from pathlib import Path
import hashlib,json,subprocess,shutil,time,datetime
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
prior=ROOT/'library01';target=ROOT/'library02';target.mkdir(exist_ok=False)
artifact=load(ROOT/'library01-artifact.json');exe=Path(artifact['path']);assert sha(exe)==artifact['sha256']
for row in load(prior/'source-manifest.json'):
    p=prior/'executed-source'/row['path'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'];assert p.read_bytes()==(CODE/row['path']).read_bytes(),row['path']
shutil.copytree(prior/'executed-source',target/'executed-source');shutil.copyfile(prior/'source-manifest.json',target/'source-manifest.json')
filters=['wl2_r6_capture_','windows_client_originals024::v2::adapter::tests::','wl2_r6_original_actions','wl2_r6_stop_action_data_','wl2_r6_stop_phase_data_','--test-threads=1','--nocapture']
source=load(prior/'source.json');source.update(actualExecutable=str(exe),actualExecutableSha256=artifact['sha256'],reusedExecutableFrom='library01',invocationOnly=True,time=datetime.datetime.now().astimezone().isoformat())
(target/'source.json').write_text(json.dumps(source,ensure_ascii=False,indent=2),encoding='utf-8')
(target/'run-command.json').write_text(json.dumps(dict(program=str(exe),arguments=filters,profile='debug',producer='library01',sha256=artifact['sha256'],rustAndDriverInputsExactlyEqualWorktree=True),indent=2),encoding='utf-8')
started=time.monotonic_ns()
with (target/'stdout.log').open('wb') as out,(target/'stderr.log').open('wb') as err:rc=subprocess.run([str(exe),*filters],cwd=CODE,stdin=subprocess.DEVNULL,stdout=out,stderr=err,check=False).returncode
assert sha(exe)==artifact['sha256']
(target/'result.json').write_text(json.dumps(dict(exitCode=rc,elapsedMs=(time.monotonic_ns()-started)//1000000,arguments=filters,profile='debug',accepted=False,reusedExactExecutableFrom='library01',executableSha256=artifact['sha256']),indent=2),encoding='utf-8')
print(json.dumps(dict(rawExit=rc,sha256=artifact['sha256'])));raise SystemExit(rc)
