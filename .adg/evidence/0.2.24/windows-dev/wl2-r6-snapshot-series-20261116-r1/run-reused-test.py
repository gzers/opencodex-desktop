from pathlib import Path
import json,hashlib,subprocess,shutil,time,datetime
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
proof=load(ROOT/'release-executable-reuse-proof.json');exe=Path(proof['actualExecutable'])
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(exe)==proof['sha256']
producer=ROOT/'library05';target=ROOT/'library06';target.mkdir(exist_ok=False)
manifest=load(producer/'source-manifest.json')
for row in manifest:
    raw=(producer/'executed-source'/row['path']).read_bytes()
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    if row['path'].endswith('.rs'):assert raw==(CODE/row['path']).read_bytes()
shutil.copytree(producer/'executed-source',target/'executed-source')
shutil.copyfile(producer/'source-manifest.json',target/'source-manifest.json')
source=load(producer/'source.json');args=source['arguments'];filters=args[args.index('--')+1:]
assert filters[-2:]==['--test-threads=1','--nocapture']
source.update(reusedExecutableFrom='library04',actualExecutable=str(exe),actualExecutableSha256=proof['sha256'],invocationOnly=True,producerRustSourcesEqual=True,time=datetime.datetime.now().astimezone().isoformat())
(target/'source.json').write_text(json.dumps(source,ensure_ascii=False,indent=2),encoding='utf-8')
command=dict(program=str(exe),arguments=filters,profile='release',reuseProof='release-executable-reuse-proof.json',driverSha256=sha(Path(__file__)),nativeAssertionsOrDeadlinesChanged=False)
(target/'run-command.json').write_text(json.dumps(command,indent=2),encoding='utf-8')
started=time.monotonic_ns()
with (target/'stdout.log').open('wb') as out,(target/'stderr.log').open('wb') as err:
    rc=subprocess.run([str(exe),*filters],cwd=CODE,stdin=subprocess.DEVNULL,stdout=out,stderr=err,check=False).returncode
assert sha(exe)==proof['sha256']
(target/'result.json').write_text(json.dumps(dict(exitCode=rc,elapsedMs=(time.monotonic_ns()-started)//1000000,arguments=filters,profile='release',accepted=False,actualExecutable=str(exe),executableSha256=proof['sha256'],reusedExactExecutableFrom='library04'),indent=2),encoding='utf-8')
print(json.dumps(dict(rawExit=rc,executable=str(exe),sha256=proof['sha256'],profile='release')))
raise SystemExit(rc)
