import importlib.util,argparse,json,shutil,time,traceback,hashlib
from pathlib import Path
repo=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
spec=importlib.util.spec_from_file_location('ota023',repo/'test/smoke/ocx023-macos-ota.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
base=m.base;base.DOCS=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main');base.HISTORY=base.DOCS/'.adg/evidence/0.2.20/final-candidate-updates-20261104';base.FROM_VERSION='0.2.22';base.TO_VERSION='0.2.23';base.BASELINE_SOURCE=m.BASELINE;base.BASELINE_ARCHIVE_SHA=m.BASE_ARCHIVE;base.BASELINE_EXE_SHA=m.BASE_EXE
prior=Path('/private/tmp/ocxd023-release-ota-execute-r2-20261110/ocx021-mac-ota-da04ac31-9130-4f66-877a-11ce4c92b49f')
args=argparse.Namespace(mode='execute',docs_root=str(base.DOCS),out='/private/tmp/ocxd023-rollback-only-20261110',baseline_archive='/tmp/ocx022-release-20261106/assets/OpenCodeX.Desktop_aarch64.app.tar.gz',baseline_signature='/tmp/ocx022-release-20261106/assets/OpenCodeX.Desktop_aarch64.app.tar.gz.sig',parent_beta_published=False,source_sha=None,publication_receipt=None,target_archive=None,target_signature=None)
run=m.Ota023(args)
try:
 run.prepare()
 # Restore only the exact frozen config and backup bytes archived by successful OTA.
 before=json.loads((prior/'before-ota-file-hashes.json').read_text());after=json.loads((prior/'after-ota-old-file-hashes.json').read_text());assert before==after
 paths={}
 for n,rel in [('save-2-readback.json','opencodex-home/config.json')]:
  raw=(prior/n).read_bytes();assert hashlib.sha256(raw).hexdigest()==after[rel];dest=run.root/rel;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw);paths[rel]=after[rel]
 saves=json.loads((prior/'old-two-native-saves.json').read_text())
 for i,row in enumerate(saves,1):
  raw=(prior/('save-'+str(i)+'-preimage.json')).read_bytes();assert hashlib.sha256(raw).hexdigest()==after[row['backup']];dest=run.root/row['backup'];dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw);paths[row['backup']]=after[row['backup']]
 # Separate new-layout sentinel proves the old application doesn't alter the new root.
 fresh=run.root/'data';(fresh/'desktop/manager-state').mkdir(parents=True);(fresh/'opencodex/home').mkdir(parents=True)
 base.dump(fresh/'desktop/manager-state/data-root.json',{'structure_version':'2','opencodex_home_mode':'inside'})
 base.dump(fresh/'opencodex/home/config.json',{'providers':{},'rollbackSentinel':'new023-preserve'})
 new_before=base.inventory(fresh)
 base.dump(run.out/'supplement-inputs.json',{'priorActualOta':str(prior),'oldActualConfigAndBackups':paths,'newLayoutSentinel':new_before,'scope':'Standalone manual rollback from authenticated original022 reading exact post-OTA archived business config+2backup preimages; no repeated upgrade, no claim of all8 files restored here'})
 run.launch();run.press('模型');run.press('渠道模型')
 snap=run.wait(lambda s:'ota021-preserve' in run.text(s),25,'rollback-original022-exact-old-channel');run.capture('rollback-original022-exact-old-channel')
 assert all(base.sha(run.root/rel)==digest for rel,digest in paths.items());assert base.inventory(fresh)==new_before
 run.verify_bundle(run.app,'0.2.22',m.BASELINE,'manual-rollback-original022')
 run.check('manual-rollback-original022-actual-post-ota-config',actualArchivedFiles=3,oldChannelVisible=True,exactFilesUnchanged=True,newLayoutSentinelUnchanged=True,originalSigned022=True,repeatOta=False)
 run.report['completed']=True
except Exception as e:
 run.report['failure']=str(e);run.report['failureType']='fail';(run.out/'failure.log').write_text(traceback.format_exc())
finally:run.close()
raise SystemExit(0 if run.report['result']=='pass' else 1)
