from pathlib import Path
import os,subprocess,shutil,json
root=Path(__file__).parent;code=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes');test=code/'apps/desktop/tauri/tests/post022_four_real_install_tmp.rs';expected=(root/'real-install-test.rs.txt').read_bytes();env=os.environ.copy();env.update({'POST022_TEST_ROOT':str(root),'POST022_NODE':shutil.which('node'),'POST022_NPM':shutil.which('npm')})
(root/'probe-home').mkdir(exist_ok=True);env.update({'HOME':str(root/'probe-home'),'CARGO_HOME':'/Users/ezio/.cargo','RUSTUP_HOME':'/Users/ezio/.rustup','XDG_CACHE_HOME':str(root/'probe-home/cache'),'XDG_CONFIG_HOME':str(root/'probe-home/config')})
result=None
try:
 with (root/'real-install.log').open('w') as out:result=subprocess.run(['cargo','test','--test','post022_four_real_install_tmp','actual_stable_to_maintained_upgrade_in_isolated_prefix','--','--exact','--nocapture','--test-threads=1'],cwd=code/'apps/desktop/tauri',env=env,stdout=out,stderr=subprocess.STDOUT,timeout=180)
finally:
 assert test.read_bytes()==expected;test.unlink();(root/'real-install-source-cleanup.json').write_text(json.dumps({'temporaryTestRemoved':not test.exists(),'exitCode':result.returncode if result else None})+'\n')
print((root/'real-install.log').read_text()[-6000:]);raise SystemExit(result.returncode if result else 1)
