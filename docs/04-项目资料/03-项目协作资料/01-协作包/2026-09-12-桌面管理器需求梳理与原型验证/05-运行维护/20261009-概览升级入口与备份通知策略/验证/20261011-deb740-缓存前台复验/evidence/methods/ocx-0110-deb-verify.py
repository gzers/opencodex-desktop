from pathlib import Path
from datetime import datetime, timezone
import fcntl, hashlib, json, subprocess

base = Path('/private/tmp/ocx-0110-deb740-foreground')
sha = lambda b: hashlib.sha256(b).hexdigest()
source_sha = 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3'
def save(name, value):
    with (base/name).open('x') as f:
        json.dump(value,f,ensure_ascii=False,indent=2); f.write(chr(10))

launch = json.loads((base/'ordinary-launch.json').read_text())
pid = launch['pid']
exe = base/'test-install/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop'
rows = subprocess.check_output(['ps','-axo','pid=,lstart=,command='],text=True).splitlines()
test_rows = [r.strip() for r in rows if r.rstrip().endswith(str(exe))]
assert len(test_rows)==1 and int(test_rows[0].split()[0])==pid
daily = [r.strip() for r in rows if r.strip().startswith('75764 ')][0]
assert daily == launch['daily_process']
env = subprocess.check_output(['ps','eww','-p',str(pid),'-o','command='],text=True)
assert 'DYLD_INSERT_LIBRARIES=' not in env and 'OPENCODEX_UPDATE_PROBE_LOG=' not in env
assert f'OPENCODEX_SANDBOX_ROOT={base / "test-data"}' in env
lock = base/'test-data/manager-state/app.lock'
metadata = json.loads(lock.read_text())
assert metadata['pid'] == pid
held=False
with lock.open('r+b') as f:
    try:
        fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
        fcntl.flock(f,fcntl.LOCK_UN)
    except BlockingIOError:
        held=True
assert held
before=json.loads((base/'cache-before.json').read_text())
files={}
for name,expected in before['files'].items():
    data=(base/'test-data'/name).read_bytes()
    actual={'sha256':sha(data),'bytes':len(data)}
    assert actual==expected
    files[name]=actual
assert sha(exe.read_bytes())==launch['test_exe_sha256']
ax=(base/'ordinary-final-ax.txt').read_text()
assert 'tauri://localhost#overview' in ax and '尚未接入 OpenCodex' in ax
save('ordinary-final-verification.json',{'source_sha':source_sha,'verified_at_utc':datetime.now(timezone.utc).isoformat(),'pid':pid,'only_one_test_instance':True,'process':test_rows[0],'daily_process_unchanged':True,'daily_process':daily,'injection_environment_absent':True,'lock_held':held,'lock_metadata':metadata,'cache_and_schedule_unchanged':True,'files':files,'mounted_ui_evidence':'ordinary-final-ax.txt','test_exe_sha256':launch['test_exe_sha256'],'release_accepted':False})

download=json.loads((base/'windows-download-verification.json').read_text())
candidate=base/'windows-download/11706329022'
evidence=base/'windows-download/11706169265'
receipt_paths=[candidate/'test/out/windows-candidate/installer-verification.json',evidence/'installer-verification.json']
assert receipt_paths[0].read_bytes()==receipt_paths[1].read_bytes()
receipt=json.loads(receipt_paths[0].read_text(encoding='utf-8-sig'))
save('windows-receipt-readable.json',receipt)
print(json.dumps({'ordinary_verified':True,'pid':pid,'receipt':receipt},ensure_ascii=False,indent=2))
