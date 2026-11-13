"""Read-only receive proof. Never upgrades component results to MSI acceptance."""
from pathlib import Path,PurePosixPath
import hashlib,json,subprocess,sys,base64
r=Path(__file__).resolve().parent;docs=next(p for p in r.parents if (p/'.adg/project.json').is_file());code=Path(sys.argv[1]).resolve()
def load(path):return json.loads(path.read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def relative(text):
 p=PurePosixPath(text.replace('\\','/'));assert not p.is_absolute() and '..' not in p.parts;return Path(*p.parts)
ingest=load(r/'ingest.json');assert len(ingest['files'])==ingest['fileCount']==211 and sum(v['bytes'] for v in ingest['files'])==ingest['totalBytes']
for item in ingest['files']:
 path=docs/relative(item['path']);raw=path.read_bytes();assert not path.is_symlink() and len(raw)==item['bytes'] and sha(raw)==item['sha256'];assert raw==subprocess.check_output(['git','show',ingest['sourceDocs']+':'+item['path']],cwd=docs)
evidence=docs/'.adg/evidence/0.2.24/windows-dev/w1d-native-entry-20261113'
for name,count in [('component-raw-manifest.json',94),('msi-attempts-manifest.json',51)]:
 rows=load(evidence/name);assert len(rows)==count;seen=set()
 for item in rows:
  path=relative(item['relative']);assert path.as_posix() not in seen;seen.add(path.as_posix());raw=(evidence/path).read_bytes();assert len(raw)==item['bytes'] and sha(raw)==item['sha256'],str(path)
source=load(evidence/'final-source-verification.json');assert source['outsidePaths']==[] and len(source['sourceFiles'])==9
bindings=[]
for f in source['sourceFiles']:
 raw=subprocess.check_output(['git','show','a02b244e9:'+f['path']],cwd=code);exact=sha(raw)==f['sha256'];crlf=not exact and sha(raw.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n'))==f['sha256'];assert exact or crlf,f['path'];assert raw==subprocess.check_output(['git','show',source['head']+':'+f['path']],cwd=code);bindings.append({'path':f['path'],'gitSha256':sha(raw),'recordedSha256':f['sha256'],'exact':exact,'onlyCRLF':crlf})
report=load(evidence/'final-result.json');assert report['nativeComponentsAccepted'] and not report['allAccepted'] and not report['actualMsiUninstallAccepted']
assert report['msiPolicy']=={'policyChanged':False,'DisableMsi':1,'exitCode':1625,'reason':'Non-assigned apps are disabled for non-admin users'}
assert not report['officialModified'] and not report['dailyConfigurationModified'] and not report['W2Executed'] and not report['released']
events={out:[json.loads(l) for l in (evidence/'attempts'/out/'native/native-events.jsonl').read_text().splitlines()] for out in ['out04','out05']}
def event(out,name):
 rows=[v['facts'] for v in events[out] if v['event']==name];assert len(rows)==1,(out,name);return rows[0]
assert event('out04','component-fixture')['msiInstalled'] is False
for name in ['argv-cmd','argv-powershell']:assert event('out04',name)['exitCode']==23
args=[v['facts'] for v in events['out04'] if v['event']=='argv'];assert len(args)==2 and {v['shell'] for v in args}=={'cmd','powershell'}
for v in args:
 assert v['passed'] and v['expected']==v['observed']['args'];assert '' in v['expected'] and '&|<>^!()' in v['expected'] and 'quoted"inside' in v['expected'];assert base64.b64decode(v['observed']['stdin'])==b'stdin binary \0 Unicode \xe9\x9b\xaa\r\n';assert v['observed']['home']==v['observed']['profile'] and v['observed']['h']!=v['observed']['home'];assert '/9.0.0/' in v['observed']['node'].replace('\\','/')
ps=load(evidence/'attempts/out04/native/powershell-version.json');assert ps['version']=='7.6.9' and ps['passing']=='Windows' and ps['target'].endswith('ocx.exe')
assert event('out04','fresh-bare-discovery')['exitCode']==0 and event('out04','fresh-bare-discovery')['stdoutBytes']>0
assert event('out04','disabled-bare-discovery')['stdoutBytes']==0
for action in ['Enable','Disable']:
 for phase in ['pending','command','path','record','notified']:
  name='resume-'+action+'-'+phase;assert event('out04',name)['exitCode']==0
  pending=load(evidence/('attempts/out04/native/pending-'+action+'-'+phase+'.private.json'));assert pending['schemaVersion']==3 and pending['action']==action.lower() and 'command' not in pending
assert {v['facts']['object'] for v in events['out04'] if v['event']=='external-refused'}=={'entry','path','record','pending'}
for label,expected in [('official-version','opencodex 2.99.0'),('official-home','18241')]:
 rows=[v['facts'] for v in events['out04'] if v['event']==label];assert len(rows)==2 and rows[0]['exitCode']==0 and rows[1]['passed'];assert rows[1]['current']['version']=='2.99.0' and rows[1]['script'].endswith('ocx.mjs');assert (evidence/('attempts/out04/native/'+label+'.stdout')).read_text().strip()==expected
for out in ['out04','out05']:
 p=evidence/'attempts'/out/'native';assert load(p/'path-before.private.json')==load(p/'path-after.private.json');assert load(p/'child-result.json')['passed'];assert load(p/'child-result.json')['exitCode']==0
f=event('out04','final');assert f['componentAccepted'] and f['pathRestored'] and not f['allAccepted'] and not f['msiUninstall'] and f['noServiceStarted']
assert event('out05','schema2-refused')['migrationExecuted'] is False
assert event('out05','preexisting-equivalent-path')['preservedExactly']
assert event('out05','notification-interruption')['injected'] and event('out05','notification-interruption')['continued']
for name,exit_code in [('nsis-never-enabled',0),('nsis-pending-refused',1),('nsis-upgrade-preserves-pending',0),('nsis-upgrade-preserves-active',0),('nsis-normal-revokes',0)]:assert event('out05',name)['exitCode']==exit_code
for out,exit_code in [('out02',1619),('out03',1625)]:
 es=[json.loads(l) for l in (evidence/'attempts'/out/'native/native-events.jsonl').read_text().splitlines()];assert next(v['facts'] for v in es if v['event']=='msi-install')['exitCode']==exit_code
policylog=(evidence/'attempts/out03/native/msi-install.msi.log').read_bytes().decode('utf-16');assert 'DisableMsi' in policylog and 'Non-assigned apps are disabled for non-admin users' in policylog
cleanup=load(evidence/'account-cleanup-readback.json');assert cleanup['testAccountsRemaining']==[] and cleanup['testProfilesRemaining']==[] and not cleanup['dailyPathWriteExecuted'];assert len(cleanup['controllers'])==5
assert all(v['accountRemoved'] and v['profileRemoved'] and v['desktopAclRestored'] for v in cleanup['controllers'])
blocked=load(evidence/'generated-cleanup-blocked.json');assert blocked['automaticApprovalReview']['result']=='rejected' and not blocked['automaticApprovalReview']['commandExecuted'] and not blocked['automaticApprovalReview']['alternateDeletionUsed'];assert sum(v['bytes'] for v in blocked['newRootsRetained'])==blocked['newRootBytes']==1944490289
bundle=load(evidence/'bundle01/result.json');assert bundle['passed'] and bundle['productExecutable'] and not bundle['installed']
print(json.dumps({'ingestedFiles':211,'ingestedBytes':ingest['totalBytes'],'nativeRawFiles':94,'msiFailedRawFiles':51,'sourceBindings':bindings,'cmdAndPowerShell7DefaultPassed':True,'official299ReadPassed':True,'bareCommandPassed':True,'schema3PendingCells':10,'nsisMacroCells':5,'realMsiUninstallPassed':False,'msiInstallExit':1625,'policyChanged':False,'testAccountsProfilesRemoved':True,'newWindowsRootBytesRetained':1944490289,'unitBytesRetained':blocked['unitFileBytes'],'windowsExecutedHere':False,'binaryArtifactsRehashedHere':False,'accepted':False},indent=2))
