"""Read-only audit of environment handoff, not an MSI product acceptance."""
from pathlib import Path,PurePosixPath
import json,hashlib,subprocess
here=Path(__file__).resolve().parent;docs=next(p for p in here.parents if (p/'.adg/project.json').is_file())
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
def load(p):return json.loads(p.read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def rel(text):
 p=PurePosixPath(text.replace('\\','/'));assert not p.is_absolute() and '..' not in p.parts;return Path(*p.parts)
ingest=load(here/'ingest.json');assert len(ingest['files'])==ingest['fileCount']==21 and sum(v['bytes'] for v in ingest['files'])==ingest['totalBytes']
for item in ingest['files']:
 p=docs/rel(item['path']);raw=p.read_bytes();assert not p.is_symlink() and len(raw)==item['bytes'] and sha(raw)==item['sha256'];assert raw==subprocess.check_output(['git','show',ingest['sourceDocs']+':'+item['path']],cwd=docs)
e=docs/'.adg/evidence/0.2.24/windows-dev/msi-followup-environment-20261114-r1'
rows=load(e/'raw-manifest.json');assert len(rows)==10
for item in rows:
 p=e/rel(item['relative']);raw=p.read_bytes();assert not p.is_symlink() and len(raw)==item['bytes'] and sha(raw)==item['sha256'],str(p)
r=load(e/'result.json');local=load(e/'local-environment.json');cleanup=load(e/'cleanup-result.json');source=load(e/'source-readback.json')
assert not r['allAccepted'] and not r['environmentReady'] and r['msiExecutions']==0
assert r['normalMsiUninstall']['status']==r['failedMsiUninstallRetention']['status']=='not-executed'
assert r['terminalTestsRerun']==r['officialReadsRerun']==r['newAccountsCreated']==0
for name in ['policyChanged','installScopeChanged','testTokenElevated','dailyConfigurationModified','officialModified','W2Executed','dualSidExecuted','published']:assert r[name] is False,name
assert r['package']['matchesContract'] and r['package']['bytes']==316784640 and r['package']['sha256']=='9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282'
assert r['environment']['hyperVInventorySucceeded'] and r['environment']['hyperVRegisteredVMs']==[] and r['environment']['dockerDefaultOsType']=='linux'
assert len(local['commands'])==4 and all(not c['result']['keyExists'] for c in local['commands'])
assert local['msiExecutions']==0 and local['accountsCreated']==0 and not local['policyChanged']
assert source['sourceChangedForThisTask'] is False and source['mainTree']==source['codeTree']
for ref in [source['main'],source['codeHead']]:assert subprocess.check_output(['git','rev-parse',ref+'^{tree}'],cwd=code).decode().strip()==source['mainTree']
for item in source['files']:
 raw=subprocess.check_output(['git','show',source['docsConsumed']+':'+item['path']],cwd=docs);assert sha(raw)==item['sha256'] or sha(raw.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n'))==item['sha256']
assert cleanup['accountsCreated']==cleanup['newAccountsRemaining']==cleanup['newProfilesRemaining']==0
assert cleanup['exists'] and cleanup['files']==10 and cleanup['bytes']==5968
assert cleanup['automaticApprovalReview']['result']=='rejected' and cleanup['automaticApprovalReview']['commandExecuted'] is False and cleanup['automaticApprovalReview']['alternateMechanismUsed'] is False
if (here/'manifest.json').exists():
 m=load(here/'manifest.json');assert m['accepted'] is False and len(m['files'])==m['fileCount']
 for item in m['files']:
  p=here/rel(item['path']);raw=p.read_bytes();assert len(raw)==item['bytes'] and sha(raw)==item['sha256']
print(json.dumps({'receivedFiles':21,'receivedBytes':ingest['totalBytes'],'rawInventoryFiles':10,'codeTreesEqual':True,'newProductChanges':False,'environmentReady':False,'msiExecutions':0,'normalUninstallExecuted':False,'failedUninstallRetentionExecuted':False,'currentPolicyKeysAbsent':True,'historic1625StillFailure':True,'packageHashReportedUnchanged':True,'packageRehashedOnMac':False,'newAccounts':0,'newInventoryBytesRetained':5968,'accepted':False},indent=2))
