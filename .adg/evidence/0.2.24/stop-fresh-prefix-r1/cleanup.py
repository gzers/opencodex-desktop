from pathlib import Path
import json,os,subprocess,sqlite3,shutil,ctypes,stat
base=Path('/private/tmp/ocxd024-stop-prefix-ic86mng4')
labels=['native','native-combined']
reports=[json.loads((base/l/'runner-summary.json').read_text()) for l in labels]
roots=[Path(r['temporaryRoot']) for r in reports]
processes=subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines()
remaining=[line for line in processes if any(str(p) in line for p in roots)]
assert not remaining,remaining
ports=[value['record']['port'] for l in labels for value in [json.loads((base/l/'native/summary.json').read_text())] if value.get('record') and value['record'].get('port')]
lib=ctypes.CDLL('/usr/lib/libSystem.B.dylib',use_errno=True)
for attempt in range(3):
 size=ctypes.c_size_t();assert lib.sysctlbyname(b'net.inet.tcp.pcblist64',None,ctypes.byref(size),None,0)==0
 assert 48<=size.value<=16*1024*1024
 buf=ctypes.create_string_buffer(size.value)
 if lib.sysctlbyname(b'net.inet.tcp.pcblist64',buf,ctypes.byref(size),None,0)!=0:
  assert ctypes.get_errno()==12;continue
 raw=buf.raw[:size.value];assert int.from_bytes(raw[:4],'little')==24 and int.from_bytes(raw[-24:-20],'little')==24
 if raw[:24]!=raw[-24:]:continue
 body=raw[24:-24];assert len(body)%472==0
 listens=[]
 for offset in range(0,len(body),472):
  row=body[offset:offset+472];assert int.from_bytes(row[:4],'little')==472
  if int.from_bytes(row[292:296],'little')==1:listens.append(int.from_bytes(row[22:24],'big'))
 assert all(p not in listens for p in ports)
 break
else:raise AssertionError('TCP snapshot changed, no cleanup authority')
databases=[];deleted=[]
for report in reports:
 exact=[Path(p) for p in report['exactNewCoordinatorFiles']]
 for p in exact:
  assert p.is_absolute() and p.parent.parent==Path('/private/tmp/opencodex-runtime-v1-'+str(os.getuid()))
  if p.suffix=='.sqlite' and p.exists():
   before=p.lstat();assert stat.S_ISREG(before.st_mode) and before.st_uid==os.getuid() and before.st_nlink==1
   conn=sqlite3.connect(p.as_uri()+'?mode=rw',uri=True,timeout=0);conn.execute('BEGIN EXCLUSIVE');conn.rollback();conn.close()
   after=p.lstat();assert (before.st_dev,before.st_ino)==(after.st_dev,after.st_ino)
   databases.append(str(p))
 for p in exact:
  if p.exists():p.unlink();deleted.append(str(p))
for p in roots:
 assert p.parent==Path('/private/tmp') and p.name.startswith('ocxd024-versioned-lifecycle-') and p.resolve()==p
 shutil.rmtree(p);assert not p.exists()
final=[line for line in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if any(str(p) in line for p in roots)]
assert not final
summary={'ownedProcessesRemaining':final,'originalPrimaryPortAbsent':True,'tcpSnapshotFramingValid':True,'capturedAdditionalPorts':'Product captured/checked4 per round; independent cleanup rechecks only exported primary ports, not unexported secondary list.','exactDatabasesExclusivelyReacquiredReleased':databases,'exactCoordinatorFilesRemoved':deleted,'ownedRootsRemoved':[str(p) for p in roots],'dailyConfigurationChanged':False,'officialRuntimeModified':False,'businessRequests':False,'noNewAppOrDMG':True}
(base/'cleanup.json').write_text(json.dumps(summary,indent=2));print('cleaned',len(roots),'owned roots and',len(databases),'exclusively rechecked databases')
