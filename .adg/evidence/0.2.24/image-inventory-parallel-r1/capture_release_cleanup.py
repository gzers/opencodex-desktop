from pathlib import Path
import json,os,subprocess,sqlite3,ctypes,stat,hashlib,base64,gzip
b=Path(__file__).resolve().parent;labels=['native-stop-release'];sha=lambda v:hashlib.sha256(v).hexdigest()
reports=[json.loads((b/l/'runner-summary.json').read_text()) for l in labels];roots=[Path(r['temporaryRoot']) for r in reports]
assert all(r['exitCode']==0 and not r['ownedProcessesRemaining'] for r in reports)
processes=subprocess.check_output(['ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines();assert not [l for l in processes if any(str(p) in l for p in roots)]
ports=[json.loads((b/l/'native/summary.json').read_text())['record']['port'] for l in labels]
lib=ctypes.CDLL('/usr/lib/libSystem.B.dylib',use_errno=True)
for _ in range(3):
 n=ctypes.c_size_t();assert lib.sysctlbyname(b'net.inet.tcp.pcblist64',None,ctypes.byref(n),None,0)==0 and 48<=n.value<=16*1024*1024
 buf=ctypes.create_string_buffer(n.value)
 if lib.sysctlbyname(b'net.inet.tcp.pcblist64',buf,ctypes.byref(n),None,0)!=0:assert ctypes.get_errno()==12;continue
 raw=buf.raw[:n.value];assert int.from_bytes(raw[:4],'little')==24 and int.from_bytes(raw[-24:-20],'little')==24
 if raw[:24]!=raw[-24:]:continue
 body=raw[24:-24];assert len(body)%472==0
 listens=[]
 for pos in range(0,len(body),472):
  row=body[pos:pos+472];assert int.from_bytes(row[:4],'little')==472
  if int.from_bytes(row[292:296],'little')==1:listens.append(int.from_bytes(row[22:24],'big'))
 assert all(p not in listens for p in ports)
 tcp={'framingVerified':True,'primaryPortsAbsent':ports,'snapshotSha256':sha(raw),'secondaryPorts':'Product captured and confirmed4 absent per round; cleanup independently checks only exported primary ports'};break
else:raise AssertionError('unstable TCP table')
for label,report in zip(labels,reports):
 root=Path(report['temporaryRoot'])/'fixtures';profile=root/'profile';files=[]
 # Whole owned Codex/Grok and manager-state evidence, plus exact synthetic HOME
 # files; no binary/runtime/Bun cache duplicate is archived.
 targets=[profile/'.codex',profile/'.grok',profile/'.codex-desktop/desktop/manager-state',profile/'.codex-desktop/opencodex/home']
 for target in targets:
  if not target.exists():continue
  assert not target.is_symlink()
  candidates=[target] if target.is_file() else sorted(target.rglob('*'))
  for p in candidates:
   assert not p.is_symlink()
   if p.is_file():
    data=p.read_bytes();s=p.stat();files.append({'path':str(p.relative_to(profile)),'bytes':len(data),'sha256':sha(data),'rawBase64':base64.b64encode(data).decode(),'device':s.st_dev,'inode':s.st_ino,'uid':s.st_uid,'mode':stat.S_IMODE(s.st_mode)})
 payload=json.dumps({'accepted':False,'scope':'post-failed-normal-Stop exact synthetic Profile and immutable vault bytes; not live OS proof','originalRootStillExists':(profile/'.codex').exists(),'files':files},separators=(',',':')).encode();compressed=gzip.compress(payload,mtime=0);z=b/label/'failed-original-bytes.json.gz';z.write_bytes(compressed);assert gzip.decompress(z.read_bytes())==payload
 (b/label/'failed-byte-manifest.json').write_text(json.dumps({'accepted':False,'fileCount':len(files),'rawBytes':len(payload),'rawSha256':sha(payload),'gzipBytes':len(compressed),'gzipSha256':sha(compressed),'losslessRoundTripVerified':True},indent=2))
 c=sqlite3.connect((profile/'.codex-desktop/desktop/manager-state/client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
 head=json.loads(c.execute('select document from current_session').fetchone()[0]);events=[json.loads(row[0]) for row in c.execute('select document from events order by revision')];c.close();assert len(events)==head['revision'] and events[-1]==head
 (b/label/'failed-index-projection.json').write_text(json.dumps({'accepted':False,'head':head,'events':events,'bytePayload':'failed-original-bytes.json.gz','rootExists':(profile/'.codex').exists()},indent=2))
# Check exact new coordinators only, preserving the preexisting global L.
namespace=Path('/private/tmp/opencodex-runtime-v1-'+str(os.getuid()));lpath=namespace/'client-desktop-lifecycle.sqlite'
def exclusive(p):
 c=sqlite3.connect(p.as_uri()+'?mode=rw',uri=True,timeout=0);c.execute('begin exclusive');c.execute('rollback');c.close()
s=lpath.lstat();lafter={'device':s.st_dev,'inode':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'bytes':s.st_size,'sha256':sha(lpath.read_bytes())};exclusive(lpath)
for label in labels:assert lafter==json.loads((b/label/'global-l-before.json').read_text())
deleted=[]
for report in reports:
 for name in report['exactNewCoordinatorFiles']:
  p=Path(name);assert p.parent.parent==namespace
  if p.exists():
   assert p.suffix=='.sqlite','unexpected live sidecar'
   before=p.lstat();assert stat.S_ISREG(before.st_mode) and before.st_uid==os.getuid() and before.st_nlink==1
   exclusive(p);after=p.lstat();assert(before.st_dev,before.st_ino)==(after.st_dev,after.st_ino)
   data=p.read_bytes();deleted.append({'path':str(p),'device':before.st_dev,'inode':before.st_ino,'sha256':sha(data)});p.unlink()
summary={'accepted':False,'allOriginalToolHandlesTerminal':True,'ownedProcessesRemaining':[],'tcp':tcp,'allNewCoordinatorsExclusivelyReacquiredAndRemoved':deleted,'globalLUnchangedAndExclusiveReleased':lafter,'rawFailedBytesCapturedBeforeCleanup':True,'ownedRootsStillRetainedUntilArchiveVerification':[str(p) for p in roots],'dailyConfigurationChanged':False,'officialRuntimeModified':False,'realBusinessRequests':False,'normalStopAccepted':False}
(b/'native-stop-release-cleanup.json').write_text(json.dumps(summary,indent=2));print(json.dumps({'rounds':len(reports),'exactNewDBsRemoved':len(deleted),'failedEvidenceCaptured':True,'StopAccepted':False}))
