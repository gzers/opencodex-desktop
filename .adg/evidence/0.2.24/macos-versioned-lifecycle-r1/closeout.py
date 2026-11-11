from pathlib import Path
import json,subprocess,shutil,hashlib,stat,os,ctypes,sqlite3
b=Path('/private/tmp/ocxd024-versioned-lifecycle-ihwdax9o');out=b/'evidence'
report={'accepted':False,'frozenCandidate':False,'scope':'Current versioned lifecycle admission and actual official299 Mac lifecycle/client-byte comparison','nativeLatest':json.loads((out/'native-attempt3/native/summary.json').read_text()),'productSemantics':'Replace direct flat resolver with provider full program admission and installation-root equality; original runtime, budgets and assertions unchanged.','failureClassification':[{'round':'before-fix','finding':'Product resolves obsolete flat runtime, refusing valid selected image before spawn'},{'round':'native-attempt1','finding':'Fixture omitted exact native coordination/adoption file permission, original foreground trace shows EPERM'},{'round':'native-attempt2','finding':'Start passed; status fixture lacked entered Tokio runtime; official Stop failed unlisted history-lock path. Neither marked passed'},{'round':'native-attempt3','finding':'Start/status/normal Stop within30/5/10s; strict client catalog/cache/audit byte restoration still failed'}],'fixtureCorrections':['Create parent before create-only evidence directory','Actual status worker runtime context','Exact canonical-home coordination hashes for native/catalog/history locks; no broad namespace writes','Use actual Grok config.toml.bak-opencodex; prior invented-path comparison is not proof for that object'],'cleanup':{}}
rows=subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True)
assert not any(str(b) in line for line in rows.splitlines()),'owned process image/group remains'
report['cleanup']['ownedProgramImagesAndGroupsAbsent']=True
# Keep failed independent lsof evidence; its remote-volume warning cannot be absence proof.
p=subprocess.run(['/usr/sbin/lsof','-nP','-iTCP:58860','-sTCP:LISTEN'],capture_output=True)
(out/'cleanup-lsof.stdout.log').write_bytes(p.stdout);(out/'cleanup-lsof.stderr.log').write_bytes(p.stderr)
report['cleanup']['lsofAcceptedAsAbsenceProof']=False
libc=ctypes.CDLL(None,use_errno=True)
for attempt in range(3):
 size=ctypes.c_size_t(0)
 assert libc.sysctlbyname(b'net.inet.tcp.pcblist64',None,ctypes.byref(size),None,0)==0
 assert 48<=size.value<=16*1024*1024
 buf=ctypes.create_string_buffer(size.value)
 code=libc.sysctlbyname(b'net.inet.tcp.pcblist64',buf,ctypes.byref(size),None,0)
 if code!=0:
  assert ctypes.get_errno()==12
  continue
 data=buf.raw[:size.value]
 assert int.from_bytes(data[:4],'little')==24 and int.from_bytes(data[-24:-20],'little')==24
 if data[:24]!=data[-24:]:continue
 payload=data[24:-24];assert len(payload)%472==0
 listening=[]
 for i in range(0,len(payload),472):
  row=payload[i:i+472];assert int.from_bytes(row[:4],'little')==472
  if int.from_bytes(row[292:296],'little')==1:listening.append(int.from_bytes(row[22:24],'big'))
 break
else:raise AssertionError('native TCP framing never stabilized')
ports=[58860,49228,49571,50298]
assert not (set(ports)&set(listening)),'original primary endpoint reused or still live'
report['cleanup']['nativeTcpTable']={'framingValidated':True,'rows':len(payload)//472,'originalPrimaryPorts':ports,'allOriginalPrimaryPortsAbsent':True}
alllocks=[]
for path in [out/'native-runner-summary.json',out/'foreground-diagnostic/summary.json',out/'native-attempt2/native-runner-summary.json',out/'native-attempt3/native-runner-summary.json']:
 s=json.loads(path.read_text());alllocks.extend(s['locks'])
lockproof=[]
for name in sorted(set(alllocks)):
 p=Path(name)
 if p.exists():
  m=p.lstat();assert stat.S_ISREG(m.st_mode) and m.st_uid==os.getuid() and m.st_nlink==1 and not (m.st_mode&0o077)
  if p.suffix=='.sqlite':
   c=sqlite3.connect('file:'+str(p)+'?mode=rw',uri=True,timeout=0)
   try:c.execute('BEGIN EXCLUSIVE');c.rollback()
   finally:c.close()
   lockproof.append({'path':str(p),'exclusiveSQLiteAcquiredAndReleased':True})
  p.unlink()
report['cleanup']['exactNewCoordinatorLocks']=lockproof
report['cleanup']['allExactNewCoordinatorFilesAbsent']=all(not Path(p).exists() for p in alllocks)
for fixture in [b/'fixtures',b/'foreground-fixture',b/'fixtures-r2',b/'fixtures-r3']:
 if fixture.exists():
  assert not (fixture/'profile/.codex-desktop/opencodex/home/runtime-port.json').exists(),fixture
  shutil.rmtree(fixture)
report['cleanup']['allOwnedFixtureRootsAbsent']=all(not p.exists() for p in [b/'fixtures',b/'foreground-fixture',b/'fixtures-r2',b/'fixtures-r3'])
report['cleanup'].update(forcedServiceCleanupUsed=False,dailyConfigurationModified=False,officialPackageModified=False)
report['remaining']=['Desktop native client recovery ledger/compensation','Windows W0 HOME/Profile, W1/W2 and actual product lifecycle integration','P24-NAME','npm System native proxy discovery','official distribution publish verifier and final fixed candidate']
(out/'summary.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
shutil.copy2(Path(__file__),out/'closeout.py')
print(json.dumps(report['cleanup'],ensure_ascii=False,indent=2))
