import hashlib, json, os, shutil, subprocess, sys, time
from pathlib import Path

def save(path, data):
    path.write_text(json.dumps(data, indent=2) + '\n', encoding='utf-8')

def fingerprint(path):
    data=path.read_bytes()
    return {'bytes':len(data), 'sha256':hashlib.sha256(data).hexdigest()}

role, attempt, toolchain = sys.argv[1:4]
tmp=Path(os.environ['TEMP'])/'ocx024-r7-external-record-20261118-r1'
out=tmp/f'{role}-{attempt}'
out.mkdir(parents=True)
product=role=='product'
test=Path('test/windows024-native')
app=Path('apps/desktop/tauri')
source_dir=test/'r7-query-harness/src/infrastructure/windows_client_originals024'
sources=[source_dir/'external_repair_record.rs',source_dir/'external_repair_whole_tree.rs',source_dir/'external_repair_steps.rs']
if product:
    sources.append(app/'src/infrastructure/windows_client_originals024/coordination/preparation/tests.rs')
inputs=[]
for p in sources:
    dest=out/'executed-source'/p
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(p, dest)
    inputs.append({'path':p.as_posix(),**fingerprint(dest)})
package='opencodex-desktop' if product else 'windows024-r7-query-proof'
target=(test/'target'/('w1c-product' if product else 'r7-query')).resolve()
cargo=Path(toolchain)/'rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/cargo.exe'
args=[str(cargo),'test','--locked','--offline']
for key,value in [('debug','0'),('incremental','false'),('codegen-units','16' if product else '256')]:
    args+=['--config',f'profile.test.package.{package}.{key}={value}']
args+=['--manifest-path',str((app if product else test/'r7-query-harness')/'Cargo.toml'),'--target','x86_64-pc-windows-msvc']
if product:
    args+=['--features','custom-protocol']
args+=['--lib','-j','1','external_steps','--','--test-threads=1','--nocapture']
os.environ['CARGO_TARGET_DIR']=str(target)
save(out/'input.json',{'arguments':args,'targetDir':str(target),'sources':inputs})
begin=time.monotonic()
with (out/'stdout.log').open('wb') as stdout, (out/'stderr.log').open('wb') as stderr:
    result=subprocess.run(args,stdout=stdout,stderr=stderr)
save(out/'result.json',{'raw':result.returncode,'elapsedMs':round((time.monotonic()-begin)*1000)})
if result.returncode==0:
    log=(out/'stderr.log').read_text(encoding='utf-8')
    line=next(line for line in log.splitlines() if 'Running unittests' in line)
    exe=(Path.cwd()/line.rsplit('(',1)[1].rstrip(')')).resolve()
    save(out/'artifact.json',{'path':str(exe),**fingerprint(exe)})
print(out)
print((out/'result.json').read_text())
sys.exit(result.returncode)