from pathlib import Path
import subprocess,json,sqlite3,ctypes,stat,hashlib,base64,gzip,os
base=Path(__file__).resolve().parent;out=base/'native-partition-r1'
report=json.loads((out/'runner-summary.json').read_text());assert report['exitCode'] is not None
work=Path(report['temporaryRoot']);root=work/'fixtures';profile=root/'profile';sha=lambda v:hashlib.sha256(v).hexdigest()
assert work.parent==Path('/private/tmp') and work.name.startswith('ocxd024-versioned-lifecycle-')
rows=subprocess.check_output(['/bin/ps','-ww','-axo','pid=,ppid=,pgid=,comm='],text=True)
assert not [line for line in rows.splitlines() if str(root) in line], 'owned executable still running; preserve scene and do not delete'
# Snapshot owned bytes before ANY coordinator or fixture deletion. No runtime
# record authentication secret is exported, even though the Profile is synthetic.
files=[];omitted=[]
for target in [profile/'.codex',profile/'.grok',profile/'.codex-desktop/desktop/manager-state',profile/'.codex-desktop/opencodex/home']:
    if not target.exists():continue
    assert not target.is_symlink()
    for p in sorted(target.rglob('*')):
        assert not p.is_symlink()
        if not p.is_file():continue
        s=p.stat();raw=p.read_bytes();entry={'path':str(p.relative_to(profile)),'bytes':len(raw),'sha256':sha(raw),'device':s.st_dev,'inode':s.st_ino,'uid':s.st_uid,'gid':s.st_gid,'mode':stat.S_IMODE(s.st_mode),'flags':s.st_flags}
        if p.name in ['runtime-port.json','.ocxd-owner.json','admin-token','admin-token.json','admin-api-token']:
            omitted.append(entry);continue
        entry['rawBase64']=base64.b64encode(raw).decode();files.append(entry)
payload=json.dumps({'accepted':False,'rootExists':(profile/'.codex').exists(),'files':files,'sensitiveRuntimeObjectsOmitted':omitted},separators=(',',':')).encode()
compressed=gzip.compress(payload,mtime=0);(out/'owned-original-bytes.json.gz').write_bytes(compressed)
assert gzip.decompress((out/'owned-original-bytes.json.gz').read_bytes())==payload
(out/'owned-byte-manifest.json').write_text(json.dumps({'accepted':False,'fileCount':len(files),'rawBytes':len(payload),'rawSha256':sha(payload),'gzipBytes':len(compressed),'gzipSha256':sha(compressed),'losslessRoundTripVerified':True,'sensitiveRuntimeObjectsOmitted':omitted},indent=2))
# Independent bounded native kernel TCP framing. Do not treat ps/tasklist or
# an empty CLI record as listener absence.
lib=ctypes.CDLL('/usr/lib/libSystem.B.dylib',use_errno=True)
ports=set()
summary=json.loads((out/'native/summary.json').read_text())
for r in [summary.get('record'),summary.get('rootCycles',{}).get('secondRecord')]:
    if isinstance(r,dict) and isinstance(r.get('port'),int) and r['port']>0:ports.add(r['port'])
for attempt in range(3):
    n=ctypes.c_size_t();assert lib.sysctlbyname(b'net.inet.tcp.pcblist64',None,ctypes.byref(n),None,0)==0 and 48<=n.value<=16*1024*1024
    buffer=ctypes.create_string_buffer(n.value)
    if lib.sysctlbyname(b'net.inet.tcp.pcblist64',buffer,ctypes.byref(n),None,0)!=0:assert ctypes.get_errno()==12;continue
    raw=buffer.raw[:n.value];assert int.from_bytes(raw[:4],'little')==24 and int.from_bytes(raw[-24:-20],'little')==24
    if raw[:24]!=raw[-24:]:continue
    body=raw[24:-24];assert len(body)%472==0
    listeners=[]
    for pos in range(0,len(body),472):
        row=body[pos:pos+472];assert int.from_bytes(row[:4],'little')==472
        if int.from_bytes(row[292:296],'little')==1:listeners.append(int.from_bytes(row[22:24],'big'))
    assert all(port not in listeners for port in ports)
    tcp={'framingVerified':True,'exportedPrimaryPortsAbsent':sorted(ports),'snapshotSha256':sha(raw),'secondaryPorts':'Each completed production Stop must prove its entire current listener collection; independent cleanup additionally checks both exported primary ports.'};break
else:raise AssertionError('unstable kernel TCP view')
namespace=Path('/private/tmp/opencodex-runtime-v1-'+str(os.getuid()));lpath=namespace/'client-desktop-lifecycle.sqlite'
lib.acl_get_file.argtypes=[ctypes.c_char_p,ctypes.c_int];lib.acl_get_file.restype=ctypes.c_void_p
lib.acl_to_text.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_ssize_t)];lib.acl_to_text.restype=ctypes.c_void_p
lib.acl_free.argtypes=[ctypes.c_void_p];lib.acl_free.restype=ctypes.c_int

lib.listxattr.argtypes=[ctypes.c_char_p,ctypes.c_void_p,ctypes.c_size_t,ctypes.c_int];lib.listxattr.restype=ctypes.c_ssize_t
lib.getxattr.argtypes=[ctypes.c_char_p,ctypes.c_char_p,ctypes.c_void_p,ctypes.c_size_t,ctypes.c_uint32,ctypes.c_int];lib.getxattr.restype=ctypes.c_ssize_t
def xattrs(path):
    encoded=os.fsencode(path);n=lib.listxattr(encoded,None,0,1);assert 0<=n<=256*1024
    if n==0:return {}
    names=ctypes.create_string_buffer(n);assert lib.listxattr(encoded,names,n,1)==n
    raw=names.raw[:n];assert raw.endswith(b'\0');result={};total=n
    for name in sorted(raw[:-1].split(b'\0')):
        assert name and name not in result
        size=lib.getxattr(encoded,name,None,0,0,1);assert 0<=size<=256*1024
        value=ctypes.create_string_buffer(size or 1);assert lib.getxattr(encoded,name,value,size,0,1)==size
        total+=size;assert total<=256*1024
        result[base64.b64encode(name).decode()]=base64.b64encode(value.raw[:size]).decode()
    return result

def snapshot(path):
    s=path.lstat();assert stat.S_ISREG(s.st_mode) and s.st_nlink==1 and s.st_uid==os.getuid()
    ctypes.set_errno(0);acl=lib.acl_get_file(os.fsencode(path),0x100)
    if not acl:
        assert ctypes.get_errno()==2
        assert (path.lstat().st_dev,path.lstat().st_ino)==(s.st_dev,s.st_ino)
        aclbytes=None
    else:
        length=ctypes.c_ssize_t();text=lib.acl_to_text(acl,ctypes.byref(length));assert text
        aclbytes=ctypes.string_at(text,length.value);assert lib.acl_free(text)==0 and lib.acl_free(acl)==0
    return {'device':s.st_dev,'inode':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'flags':s.st_flags,'bytes':s.st_size,'sha256':sha(path.read_bytes()),'aclBase64':None if aclbytes is None else base64.b64encode(aclbytes).decode(),'xattrs':xattrs(path)}

def exclusive(path):
    c=sqlite3.connect(path.as_uri()+'?mode=rw',uri=True,timeout=0)
    try:c.execute('begin exclusive');c.execute('rollback')
    finally:c.close()

lbefore=json.loads((out/'global-l-before.json').read_text());assert snapshot(lpath)==lbefore
exclusive(lpath);assert snapshot(lpath)==lbefore
for suffix in ['-journal','-wal','-shm']:assert not os.path.lexists(Path(str(lpath)+suffix))
receipt=out/'coordinator-byte-receipts.json'
removed=json.loads(receipt.read_text()) if receipt.exists() else []
assert len({item['path'] for item in removed})==len(removed)
for item in removed:
    raw=base64.b64decode(item['rawBase64'],validate=True)
    assert len(raw)==item['bytes'] and sha(raw)==item['sha256']
# Exact files were absent before launch. First-seen identities bind each object
# to this isolated run; mismatches refuse cleanup. Sidecars may not be skipped.
for name in report['exactNewCoordinatorFiles']:
    path=Path(name);assert path.parent.parent==namespace
    if not os.path.lexists(path):continue
    assert path.suffix=='.sqlite','live or interrupted coordinator sidecar; preserve'
    info=report['firstSeenCoordinators'].get(name);assert info,'no observed ownership identity'
    before=path.lstat();assert (before.st_dev,before.st_ino)==(info['device'],info['inode'])
    assert stat.S_ISREG(before.st_mode) and before.st_nlink==1 and before.st_uid==os.getuid() and stat.S_IMODE(before.st_mode)==0o600
    for suffix in ['-journal','-wal','-shm']:assert not os.path.lexists(Path(str(path)+suffix))
    exclusive(path);after=path.lstat();assert (after.st_dev,after.st_ino)==(before.st_dev,before.st_ino)
    raw=path.read_bytes();entry={'path':name,'device':after.st_dev,'inode':after.st_ino,'sha256':sha(raw),'rawBase64':base64.b64encode(raw).decode(),'bytes':len(raw)}
    # Write/verify a lossless coordinator receipt before unlinking its owned file.
    prior=next((item for item in removed if item['path']==name),None)
    if prior is not None:assert prior==entry
    else:removed.append(entry)
    encoded=json.dumps(removed,indent=2).encode();temp_receipt=receipt.with_suffix('.next')
    with temp_receipt.open('xb') as f:f.write(encoded);f.flush();os.fsync(f.fileno())
    os.replace(temp_receipt,receipt)
    parent_fd=os.open(out,os.O_RDONLY)
    try:os.fsync(parent_fd)
    finally:os.close(parent_fd)
    assert json.loads(receipt.read_text())==removed and any(item['path']==name and base64.b64decode(item['rawBase64'])==raw for item in removed)
    path.unlink()
assert all(not os.path.lexists(Path(name)) for name in report['exactNewCoordinatorFiles'])
adopt_prefix='.'+Path(report['exactNewCoordinatorFiles'][0]).stem+'.sqlite.adopt-'
adopt_residual=[str(p) for p in (namespace/'native-write-locks').iterdir() if p.name.startswith(adopt_prefix)]
assert not adopt_residual,adopt_residual
assert snapshot(lpath)==lbefore;exclusive(lpath);assert snapshot(lpath)==lbefore
result={'accepted':False,'diagnosticPassed':report['diagnosticPassed'],'rawOwnedBytesCapturedBeforeCleanup':True,'noOwnedProcessesByExecutableInventory':True,'tcp':tcp,'exactOwnedCoordinatorsExclusivelyReacquiredAndRemoved':[{k:v for k,v in row.items() if k!='rawBase64'} for row in removed],'globalLIdentityProtectionBytesUnchanged':lbefore,'globalLExclusiveReleased':True,'exactDigestAdoptTemporaryResiduals':[],'preexistingGlobalLNeverDeletedRebuiltOrRestored':True,'temporaryRootStillRetainedUntilArchiveVerification':str(work),'dailyConfigurationChanged':False,'officialRuntimeModified':False,'businessRequests':False}
(out/'cleanup-native.json').write_text(json.dumps(result,indent=2));print(json.dumps({'coordinatorsRemoved':len(removed),'profileFilesRetainedAsRawEvidence':len(files),'diagnosticPassed':report['diagnosticPassed']},indent=2))
