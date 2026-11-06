from pathlib import Path
from types import SimpleNamespace
import importlib.util,json,hashlib,shutil,subprocess,traceback
repo=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes')
ev=repo.parent/'governance-main/.adg/evidence/0.2.21/post-release-20261104/integration-20261105/release-0.2.22-20261106'
source=ev/'macos-ota/ocx021-mac-ota-5b4e902a-0059-476b-b48d-fcc0b8a6133a';report=json.loads((source/'report.json').read_text());assert report['result']=='pass' and report['completed']
container=Path(report['container']);old=container/'baseline-original/OpenCodeX Desktop.app';hashes=json.loads((source/'after-ota-file-hashes.json').read_text());root=Path(report['root'])
assert root.is_relative_to(container) and str(container).startswith('/private/tmp/ocx021-mac-ota-')
spec=importlib.util.spec_from_file_location('native',repo/'test/smoke/ocx021-macos-native.py');native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
args=SimpleNamespace(app=str(old),out=str(ev/'macos-rollback'),stage='signed-final-reverify',scope='old-original-loads-post-022-state-clone',expected_build_commit='6e2197fd0b8a36dcb8d893b62693c5a2a77151cc',expected_version='0.2.21',hold_seconds=0,node=None,npm_cli=None,official_entry=None,bun=None)
run=native.NativeRun(args)
try:
 subprocess.run(['codesign','--verify','--deep','--strict',str(old)],check=True)
 run.prepare()
 for rel,digest in hashes.items():
  original=root/rel;assert original.is_relative_to(root) and hashlib.sha256(original.read_bytes()).hexdigest()==digest
  target=run.root/rel;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(original,target)
 run.launch();run.press('模型');run.press('渠道模型');run.wait(lambda s:'ota021-preserve' in run.text(s),name='rollback-old-channel')
 run.press('编辑');snap=run.wait(lambda s:'渠道配置' in run.text(s),name='rollback-old-channel-detail')
 assert all(x in run.text(snap) for x in ['ota021-preserve','vendor/a:8b','Mixed_Case/V2'])
 run.capture('rollback-old-original-business-readback',snap)
 after={rel:hashlib.sha256((run.root/rel).read_bytes()).hexdigest() for rel in hashes};assert after==hashes
 native.write_json(run.out/'post022-clone-file-hashes.json',after)
 run.check('original-021-loads-byte-identical-post-022-state','pass',version='0.2.21',source=args.expected_build_commit,filesPreserved=len(after),method='Read-only native startup and model detail on a disposable byte-identical copy of the successful OTA state; original OTA root and bundles untouched')
 run.report['scopeCompleted']=True
except Exception as e:
 run.report['error']=str(e);run.report['errorKind']='fail';(run.out/'exception.log').write_text(traceback.format_exc());raise
finally:run.close()
assert run.report['result']=='pass'
print(json.dumps({'result':run.report['result'],'out':str(run.out),'files':len(hashes),'scope':args.scope}))
