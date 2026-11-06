from pathlib import Path
import subprocess,hashlib,json
code=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes');tmp=Path(__file__).parent
files={'apps/desktop/tauri/src/modules/runtime/install.rs':('runtime-cases.rs.txt',True),'apps/desktop/tauri/src/modules/model_config/sync_version_tests.rs':('sync-cases.rs.txt',False),'apps/desktop/tauri/src/modules/model_config/config_file_rename_tests.rs':('rename-cases.rs.txt',False)}
assert not subprocess.check_output(['git','status','--porcelain'],cwd=code).strip()
original={};patched={};result=None
try:
 for path,(case,inside) in files.items():
  p=code/path;b=p.read_bytes();original[path]=b;s=b.decode();extra=(tmp/case).read_text();boundary=s.index(chr(10)+'#[cfg(test)]'+chr(10)+'mod diagnostics_tests') if inside else len(s);pos=s.rfind('}',0,boundary) if inside else len(s);s=s[:pos]+extra+s[pos:] if inside else s+chr(10)+extra;patched[path]=s.encode();p.write_bytes(patched[path])
 with (tmp/'rust-tests.txt').open('w') as log:result=subprocess.run(['cargo','test','--lib','post022_simulation_','--','--nocapture','--test-threads=1'],cwd=code/'apps/desktop/tauri',stdout=log,stderr=subprocess.STDOUT)
finally:
 for path,b in original.items():
  p=code/path;assert p.read_bytes()==patched[path], 'Concurrent source change: refuse overwrite '+path;p.write_bytes(b)
 (tmp/'rust-cleanup.json').write_text(json.dumps({'sourcesRestored':{p:hashlib.sha256((code/p).read_bytes()).hexdigest()==hashlib.sha256(b).hexdigest() for p,b in original.items()},'productStatus':subprocess.check_output(['git','status','--porcelain'],cwd=code).decode(),'exitCode':result.returncode if result else None},indent=2)+chr(10))
print((tmp/'rust-tests.txt').read_text());raise SystemExit(result.returncode if result else 1)
