"""Read-only receive audit; component success never overrides failed W1-C."""
from pathlib import Path,PurePosixPath
import json,hashlib,subprocess,sys
here=Path(__file__).resolve().parent
docs=next(p for p in here.parents if (p/'.adg/project.json').is_file())
code=Path(sys.argv[1]).resolve()
def load(p):return json.loads(p.read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=docs)
def checked_relative(text):
 p=PurePosixPath(text.replace('\\','/'));assert not p.is_absolute() and '..' not in p.parts;return Path(*p.parts)
received=load(here/'ingest.json');assert received['fileCount']==len(received['files'])==265
assert sum(f['bytes'] for f in received['files'])==received['totalBytes']
for f in received['files']:
 p=docs/checked_relative(f['path']);raw=p.read_bytes();assert not p.is_symlink() and len(raw)==f['bytes'] and sha(raw)==f['sha256'],f['path'];assert raw==git('show',received['sourceDocs']+':'+f['path'])
base=docs/'.adg/evidence/0.2.24/windows-dev'
native=base/'w1c-native-20261113';build=base/'w1c-msvc-20261113'
raw_manifest=load(native/'raw-manifest.json');assert len(raw_manifest)==139
names=set()
for f in raw_manifest:
 p=checked_relative(f['relative']);assert p.as_posix() not in names;names.add(p.as_posix());raw=(native/p).read_bytes();assert len(raw)==f['bytes'] and sha(raw)==f['sha256'],str(p)
report=load(native/'result.json');assert report['sourceHead']=='e004259feba4a077375db01a79f851470ed0ff74';assert not report['allW1cAccepted'] and report['native']['overallExit']==101
assert report['productBuild']['check02Exit']==0 and report['productBuild']['build02Exit']==0 and report['productBuild']['testNoRunExit']==0
assert not report['productBuild']['releaseBuild'] and not report['published'] and not report['realServiceStarted'] and not report['dailyPathWriteExecuted'] and not report['W2Executed']
assert load(build/'check02.json')['exitCode']==0;assert load(build/'build02-result.json')['buildExit']==0
assert load(build/'private-reader-regression.json')['exitCode']==0
assert '1 passed; 0 failed' in (build/'private-reader-regression.stdout.log').read_text()
events=[json.loads(l) for l in (native/'raw/out04/native/native-events.jsonl').read_text().splitlines()]
def event(name):
 values=[v['facts'] for v in events if v['event']==name];assert len(values)==1,name;return values[0]
assert event('argv-cmd')['exitCode']==23
assert event('argv-powershell')['exitCode']==1 and event('argv-powershell')['stdoutBytes']==0
assert b'| was unexpected at this time.' in (native/'raw/out04/native/argv-powershell.stderr').read_bytes()
assert event('official-product-blocked')['passed'] is False
assert 'runtime_selection_invalid' in event('official-product-blocked')['error']
assert event('component-results')['argvFailedCells']==['argv-powershell'] and not event('component-results')['officialIntegrated']
boundaries=[v['facts'] for v in events if v['event']=='boundary-resumed'];assert len(boundaries)==10
assert {(v['action'],v['boundary']) for v in boundaries}=={(a,b) for a in ['Enable','Disable'] for b in ['pending','command','path','record','notified']}
for v in boundaries:
 label='resume-'+v['action']+'-'+v['boundary'];assert event(label)['exitCode']==0
 p=native/('raw/out04/native/pending-'+v['action']+'-'+v['boundary']+'.private.json')
 assert sha(p.read_bytes())==v['samePendingSha256']
assert len([v for v in events if v['event']=='external-conflict-retained'])==4
for name,exit_code in [('helper-pristine',0),('helper-pending-enable',1),('helper-extra-argument',2),('helper-normal-revoke',0),('hook-never-enabled',0),('hook-pending-enable-refused',1),('hook-upgrade-keeps-terminal',0),('hook-normal-revokes-terminal',0)]:assert event(name)['exitCode']==exit_code,name
assert load(native/'raw/out04/native/account-path-before.private.json')==load(native/'raw/out04/native/account-path-after.private.json')
cleanup=load(native/'cleanup-readback.json');assert cleanup['testAccountsRemaining']==[] and cleanup['testProfilesRemaining']==[] and not cleanup['privateSourceAclsChanged']
blocked=load(native/'cleanup-blocked.json');assert blocked['retainedBytes']==26869205653 and blocked['automaticApprovalReview']['result']=='rejected' and not blocked['automaticApprovalReview']['commandExecuted'] and not blocked['automaticApprovalReview']['alternateDeletionUsed']
assert sum(v['bytes'] for v in blocked['retainedTargets'])==blocked['retainedBytes']
msi=load(build/'msi-hook01/result.json');assert msi['passed'] and len(msi['cells'])==5 and all(v['expected']==v['actual'] for v in msi['cells']);assert msi['action']['type']==1058 and msi['action']['sequence']<msi['action']['removeFilesSequence']
bundle=load(build/'product-bundle02/result.json');assert bundle['passed'] and bundle['productExecutable'] and not bundle['installed'] and len(bundle['outputs'])==2
source_bindings=load(here/'source-bindings.json');assert len(source_bindings)==10 and sum(v['exact'] for v in source_bindings)==7 and sum(v['onlyCRLF'] for v in source_bindings)==3
for item in source_bindings:
 raw=(code/item['path']).read_bytes();assert sha(raw)==item['localSha256'];assert sha(raw)==item['sha256'] or sha(raw.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n'))==item['sha256']
print(json.dumps({'receivedFiles':265,'receivedBytes':received['totalBytes'],'rawNativeFiles':139,'sourceBindings':10,'msvcDevelopmentBuildReported':True,'nativePendingCellsPassed':10,'cmdMixedArgsPassed':True,'powershellMixedArgsPassed':False,'official299ConsumerPassed':False,'msiFullUninstallExecuted':False,'testAccountsProfilesRemoved':True,'windowsGeneratedBytesRetained':26869205653,'windowsBinariesRehashedHere':False,'accepted':False},indent=2))
