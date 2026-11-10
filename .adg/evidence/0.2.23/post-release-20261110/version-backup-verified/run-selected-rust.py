from pathlib import Path
import subprocess,json,os,time,hashlib
r=Path('/private/tmp/ocxd-version-backup-verification-20261110');r.mkdir(exist_ok=True)
binary=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes/apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-58a8535d3845a745')
need=['creates_upgrade_backup_from_real_preferences','protection_helper_keeps_raw_bytes_and_holds_shared_lock','backup_failure_has_zero_business_writes_and_cannot_claim_protection','backup_images_require_actual_protection_and_keep_original_before_after_without_receipt_targets','key_replace_preserves_unselected_pool_slots','portable_authorization_is_materialized_into_file_without_system_credentials','explicit_key_only_replace_preserves_other_pool_slots_and_clear_keeps_auth_mode','file_import_preserves_pool_headers_and_active_key_without_runtime_binding','portable_pool_is_complete_and_private_blocks_never_enter_public_fields']
listed=subprocess.check_output([binary,'--list'],text=True).splitlines();names=[]
for n in need:
 matches=[x.removesuffix(': test') for x in listed if x.endswith('::'+n+': test')];assert len(matches)==1,(n,matches);names+=matches
rows=[]
for index,name in enumerate(names,1):
 start=time.monotonic();p=subprocess.run([binary,name,'--exact','--test-threads=1'],capture_output=True,text=True,timeout=45,env={**os.environ,'TMPDIR':str(r)+'/'});log=p.stdout+p.stderr
 (r/(str(index).zfill(2)+'.log')).write_text(log);assert '1 passed' in log and p.returncode==0,(name,log[-1500:])
 rows.append({'test':name,'result':'pass','exitCode':p.returncode,'seconds':round(time.monotonic()-start,3),'log':str(index).zfill(2)+'.log'});print(name,'PASS',flush=True)
(r/'rust-results.json').write_text(json.dumps({'sourceCommit':'0828a3955','binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cargoOfflineBuild':'success','tests':rows,'count':len(rows),'allPassed':True,'actualDailyConfigTouched':False,'scope':'Actual production Rust modules/temporary file fixtures; not raw-native full import UX or299 integration acceptance'},indent=2)+'\n')
print('Completed',len(rows))
