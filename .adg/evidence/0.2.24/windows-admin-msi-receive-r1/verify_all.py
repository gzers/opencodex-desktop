"""Offline exact receive check; administrator pass is not ordinary-user acceptance."""
from pathlib import Path,PurePosixPath
import subprocess,json,hashlib
here=Path(__file__).resolve().parent;docs=next(p for p in here.parents if (p/'.adg/project.json').is_file());code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
def load(p):return json.loads(p.read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
ingest=load(here/'ingest.json');assert len(ingest['files'])==ingest['fileCount']==181 and sum(f['bytes'] for f in ingest['files'])==ingest['totalBytes']
for f in ingest['files']:
 p=PurePosixPath(f['path']);assert not p.is_absolute() and '..' not in p.parts
 path=docs/Path(*p.parts);raw=path.read_bytes();assert not path.is_symlink() and len(raw)==f['bytes'] and sha(raw)==f['sha256'];assert raw==subprocess.check_output(['git','show',ingest['source']+':'+f['path']],cwd=docs)
paths=subprocess.check_output(['git','diff','--name-only','-z','fd2538e4d...0f1dc195ed2e39bbbeffd23eff05f97a78905c57'],cwd=code).split(b'\0');paths=[p.decode() for p in paths if p];assert len(paths)==10
for path in paths:assert subprocess.check_output(['git','show','6b33016ba:'+path],cwd=code)==subprocess.check_output(['git','show','0f1dc195ed2e39bbbeffd23eff05f97a78905c57:'+path],cwd=code)
command=['python3',str(code/'test/windows024-native/verify-msi-evidence.py'),'--docs-repo',str(docs),'--code-repo',str(code),'--self-test']
p=subprocess.run(command,check=True,capture_output=True);proof=json.loads(p.stdout)
assert proof['proofVerified'] and proof['administratorTransactionsPassed'] and not proof['ordinaryUserAcceptance'] and not proof['cleanupComplete'] and not proof['allAccepted'] and len(proof['negativeEvidenceRejected'])==9
r=load(docs/'.adg/evidence/0.2.24/windows-dev/msi-local-admin-20261114-r1/result.json')
assert r['msi']['actualInstallExit']==0 and r['msi']['failedUninstallExit']==1603 and r['msi']['helperExit']==1 and r['msi']['normalUninstallExit']==0 and r['msi']['finalProductState']==-1
assert r['cleanup']['accountRemoved'] and not r['cleanup']['profileRemoved'] and r['cleanup']['nativeError']==32 and r['cleanup']['profileBytes']==118556840 and r['cleanup']['generatedRootBytes']==184522112
assert r['cleanup']['deletionAutoReviewRejected'] and r['cleanup']['noBypass']
if (here/'manifest.json').exists():
 m=load(here/'manifest.json');assert not m['accepted'] and len(m['files'])==m['fileCount']
 for f in m['files']:
  raw=(here/f['path']).read_bytes();assert len(raw)==f['bytes'] and sha(raw)==f['sha256']
print(json.dumps({'ingestedFiles':181,'bytes':ingest['totalBytes'],'codeNetFiles':10,'originalProof':proof,'administratorMsiInstall':0,'administratorFailedUninstall':1603,'administratorNormalUninstall':0,'ordinaryUserAcceptance':False,'cleanupComplete':False,'profileRemainingBytes':118556840,'publicRemainingBytes':184522112,'packageBodyRehashedOnMac':False,'accepted':False},indent=2))
