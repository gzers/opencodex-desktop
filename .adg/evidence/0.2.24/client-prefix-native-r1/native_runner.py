from pathlib import Path
import subprocess,json,hashlib,tempfile,shutil,os,re,time,stat,sqlite3,ctypes,sys,base64
base=Path(__file__).resolve().parent
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
out=base/'native-prefix-r1';out.mkdir()
work=Path(tempfile.mkdtemp(prefix='ocxd024-versioned-lifecycle-',dir='/private/tmp'))
root=work/'fixtures';root.mkdir(mode=0o700);profile=root/'profile';profile.mkdir(mode=0o700);(root/'tmp').mkdir(mode=0o700)
resources=root/'resources';resources.mkdir(mode=0o700)
source=code/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle'
sha=lambda data:hashlib.sha256(data).hexdigest()
resource_rows=[]
for name,expected in [('runtime-bundle.manifest.json','e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3'),('runtime-bundle-0.2.24-darwin-arm64.tar.gz','5b10c1a888000d54d14783622d9d8330c33f4e1b14a5f163d007337d0424ec7f')]:
    assert sha((source/name).read_bytes())==expected
    shutil.copyfile(source/name,resources/name);assert sha((resources/name).read_bytes())==expected
    resource_rows.append({'source':str(source/name),'copiedTo':str(resources/name),'bytes':(resources/name).stat().st_size,'sha256':expected})
manifest=json.loads((resources/'runtime-bundle.manifest.json').read_bytes())
assert manifest['runtimeVersion']=='2.99.0' and manifest['runtimeArchiveSha256']=='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'
log=(base/'release-build-final.log').read_text();match=re.search(r'Executable unittests src/lib.rs \(([^)]+)\)',log);assert match,log[-2000:]
compiled=code/'apps/desktop/tauri'/match.group(1)
compiled_receipt=json.loads((base/'compiled-source.json').read_text())
assert sha(compiled.read_bytes())==compiled_receipt['binarySha256']
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip()==compiled_receipt['baseCommit']
for item in compiled_receipt['files']:assert sha((code/item['path']).read_bytes())==item['sha256']
exe=root/'tests';shutil.copy2(compiled,exe)
assert sha(exe.read_bytes())==sha(compiled.read_bytes())
client=profile/'.codex';namespace=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()))
digest=sha(str(client).encode());locks=[]
for kind in ['native-write-locks','catalog-write-locks']:
    for suffix in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']:locks.append(Path(str(namespace/kind/digest)+suffix))
h=str(client);state_db=str(client/'state_5.sqlite');history=sha((str(len(h))+':'+h+str(len(state_db))+':'+state_db).encode())
for suffix in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']:locks.append(Path(str(namespace/'history-write-locks'/history)+suffix))
assert all(not os.path.lexists(p) for p in locks)
assert not [p for p in (namespace/'native-write-locks').iterdir() if p.name.startswith('.'+digest+'.sqlite.adopt-')]
lib=ctypes.CDLL('/usr/lib/libSystem.B.dylib',use_errno=True)
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

def protection(path):
    s=path.lstat();assert stat.S_ISREG(s.st_mode) and s.st_nlink==1 and s.st_uid==os.getuid()
    ctypes.set_errno(0);acl=lib.acl_get_file(os.fsencode(path),0x100)
    if not acl:
        assert ctypes.get_errno()==2
        assert (path.lstat().st_dev,path.lstat().st_ino)==(s.st_dev,s.st_ino)
        value=None
    else:
        length=ctypes.c_ssize_t();text=lib.acl_to_text(acl,ctypes.byref(length));assert text
        value=ctypes.string_at(text,length.value);assert lib.acl_free(text)==0 and lib.acl_free(acl)==0
    return {'device':s.st_dev,'inode':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'flags':s.st_flags,'bytes':s.st_size,'sha256':sha(path.read_bytes()),'aclBase64':None if value is None else base64.b64encode(value).decode(),'xattrs':xattrs(path)}

def exclusive(path):
    c=sqlite3.connect(path.as_uri()+'?mode=rw',uri=True,timeout=0)
    try:c.execute('begin exclusive');c.execute('rollback')
    finally:c.close()

lpath=namespace/'client-desktop-lifecycle.sqlite'
assert lpath.exists();l_before=protection(lpath);assert l_before['mode']==0o600
assert all(not os.path.lexists(Path(str(lpath)+suffix)) for suffix in ['-journal','-wal','-shm'])
exclusive(lpath);assert protection(lpath)==l_before
(out/'global-l-before.json').write_text(json.dumps(l_before,indent=2))
policy='(version 1)\n(allow default)\n(deny network*)\n(allow network-bind (local ip "localhost:*"))\n(allow network-inbound (local ip "localhost:*"))\n(allow network-outbound (remote ip "localhost:*"))\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(root))+'))\n(allow file-write* (literal "/dev/null"))\n(deny file-read* (subpath "/Users/ezio"))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n'
policy+=''.join('(allow file-write* (literal '+json.dumps(str(p))+'))\n' for p in locks)
for suffix in ['','-journal','-wal','-shm']:policy+='(allow file-write* (literal '+json.dumps(str(lpath)+suffix)+'))\n'
policy+='(allow file-write-mode (literal '+json.dumps(str(namespace))+'))\n'
pattern='^'+re.escape(str(namespace/'native-write-locks'/('.'+digest+'.sqlite.adopt-')))+'[0-9]+-[0-9a-f-]+'+re.escape('.tmp')+'(-journal|-wal|-shm)?$'
policy+='(allow file-write* (regex '+json.dumps(pattern)+'))\n';sandbox=work/'fixture.sb';sandbox.write_text(policy);(out/'fixture.sb').write_text(policy)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_NATIVE_ROOT':str(root),'OCXD024_ALIAS_GRAPH':'1','OCXD024_FRESH_CLIENTS':'codex-absent','OCXD024_ROOT_CYCLES':'2'}
positive=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sandbox),'/usr/bin/python3','-c','import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); s.listen(); print("LOOPBACK_BOUND",s.getsockname()[1]); s.close()'],env=env,cwd=root,capture_output=True)
(out/'loopback-bind.stdout.log').write_bytes(positive.stdout);(out/'loopback-bind.stderr.log').write_bytes(positive.stderr)
assert positive.returncode==0 and positive.stdout.startswith(b'LOOPBACK_BOUND ')
for label,args in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home-open',['/usr/bin/head','-c','1','/Users/ezio/.zshrc'])]:
    p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sandbox),*args],env=env,cwd=root,capture_output=True)
    (out/(label+'.stdout.log')).write_bytes(p.stdout);(out/(label+'.stderr.log')).write_bytes(p.stderr)
    assert p.returncode and b'Operation not permitted' in p.stderr and not p.stdout
report={'accepted':False,'hypothesis':json.loads((base/'hypothesis.json').read_text()),'temporaryRoot':str(work),'resources':resource_rows,'officialArchiveSha256':manifest['runtimeArchiveSha256'],'binarySha256':sha(exe.read_bytes()),'compiledBinarySource':str(compiled),'exactNewCoordinatorFiles':[str(p) for p in locks],'noBusinessRequests':True,'firstSeenCoordinators':{},'trackedProcesses':{}}
source_paths=['apps/desktop/tauri/src/commands/lifecycle_versioned_tests.rs','apps/desktop/tauri/src/commands/lifecycle_root_cycles_fixture.rs','apps/desktop/tauri/src/infrastructure/macos_client_session_index.rs']
(out/'source.json').write_text(json.dumps({'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'files':[{'path':p,'sha256':sha((code/p).read_bytes())} for p in source_paths],'binarySha256':report['binarySha256'],'compiledSourceReceipt':compiled_receipt,'resources':resource_rows},indent=2))
proc=ctypes.CDLL('/usr/lib/libproc.dylib',use_errno=True)
class Bsd(ctypes.Structure):
    _fields_=[(name,ctypes.c_uint32) for name in ['flags','status','xstatus','pid','ppid','uid','gid','ruid','rgid','svuid','svgid','rfu']]+[('comm',ctypes.c_char*16),('name',ctypes.c_char*32)]+[(name,ctypes.c_uint32) for name in ['nfiles','pgid','pjobc','etdev','etpgid']]+[('nice',ctypes.c_int32),('startSec',ctypes.c_uint64),('startUsec',ctypes.c_uint64)]
assert ctypes.sizeof(Bsd)==136
proc.proc_pidinfo.argtypes=[ctypes.c_int,ctypes.c_int,ctypes.c_uint64,ctypes.c_void_p,ctypes.c_int];proc.proc_pidinfo.restype=ctypes.c_int
proc.proc_pidpath.argtypes=[ctypes.c_int,ctypes.c_void_p,ctypes.c_uint32];proc.proc_pidpath.restype=ctypes.c_int

def process(pid):
    b=Bsd();ctypes.set_errno(0);n=proc.proc_pidinfo(pid,3,0,ctypes.byref(b),ctypes.sizeof(b))
    if n==0 and ctypes.get_errno()==3:return None
    assert n==136 and b.pid==pid,(pid,n,ctypes.get_errno())
    path=ctypes.create_string_buffer(4096);size=proc.proc_pidpath(pid,path,len(path))
    return {'pid':pid,'ppid':b.ppid,'uid':b.uid,'status':b.status,'startSeconds':b.startSec,'startUseconds':b.startUsec,'pgid':b.pgid,'image':os.fsdecode(path.value) if size>0 else None}

def observe():
    for path in locks:
        if os.path.lexists(path):
            try:s=path.lstat()
            except FileNotFoundError:continue
            observed={'device':s.st_dev,'inode':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'regular':stat.S_ISREG(s.st_mode),'links':s.st_nlink}
            if path.suffix!='.sqlite':continue
            previous=report['firstSeenCoordinators'].setdefault(str(path),observed)
            if previous['device']!=s.st_dev or previous['inode']!=s.st_ino:report.setdefault('coordinatorIdentityChanges',[]).append({'path':str(path),'first':previous,'now':observed})
    rows=subprocess.check_output(['/bin/ps','-ww','-axo','pid=,ppid=,pgid=,comm='],text=True)
    for row in rows.splitlines():
        if str(root) not in row:continue
        pid=int(row.split()[0]);info=process(pid)
        if info is not None:report['trackedProcesses'].setdefault(str(pid)+':'+str(info['startSeconds'])+':'+str(info['startUseconds']),info)

start=time.monotonic()
with (out/'native.stdout.log').open('wb') as stdout,(out/'native.stderr.log').open('wb') as stderr:
    p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sandbox),str(exe),'--exact','commands::lifecycle_versioned_tests::official299_versioned_start_status_stop_preserves_native_client','--ignored','--nocapture','--test-threads=1'],env=env,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
    report['pid']=p.pid;(out/'running.json').write_text(json.dumps({'pid':p.pid,'temporaryRoot':str(work),'accepted':False}))
    while p.poll() is None and time.monotonic()-start<360:
        try:observe()
        except Exception as error:report.setdefault('observationErrors',[]).append(type(error).__name__+':'+str(error))
        time.sleep(0.2)
    report['exitCode']=p.poll();report['elapsedSeconds']=round(time.monotonic()-start,3)
    if report['exitCode'] is None:
        report['diagnosticPassed']=False;report['liveProcessRequiresFollowup']=True
        (out/'runner-summary.json').write_text(json.dumps(report,indent=2));print(json.dumps({'pid':p.pid,'live':True,'temporaryRoot':str(work),'accepted':False}),flush=True);sys.exit(2)
try:observe()
except Exception as error:report.setdefault('observationErrors',[]).append(type(error).__name__+':'+str(error))
if (root/'evidence/native').exists():shutil.copytree(root/'evidence/native',out/'native')
logpath=profile/'.codex-desktop/desktop/logs/app.log'
if logpath.exists():shutil.copyfile(logpath,out/'manager.log')
report['globalLAfter']=protection(lpath);report['globalLUnchanged']=report['globalLAfter']==l_before
if report['exitCode'] is not None:
    exclusive(lpath);report['globalLExclusiveReleased']=True
    report['globalLAfterFinalConnection']=protection(lpath)
    report['globalLUnchanged']=report['globalLAfterFinalConnection']==l_before
    report['globalLSidecarsAbsent']=all(not os.path.lexists(Path(str(lpath)+suffix)) for suffix in ['-journal','-wal','-shm'])
output=(out/'native.stdout.log').read_text(errors='replace');summary_path=out/'native/summary.json'
summary=json.loads(summary_path.read_text()) if summary_path.exists() else {}
report['exactOneTestExecuted']='running 1 test' in output and '1 passed; 0 failed;' in output
report['twoCycleCompletionMarker']='OFFICIAL299_TWO_ROOT_CYCLES_DIAGNOSTIC ' in output
report['trackedGenerationsAbsent']=True
for old in report['trackedProcesses'].values():
    now=process(old['pid'])
    if now is not None and (now['startSeconds'],now['startUseconds'])==(old['startSeconds'],old['startUseconds']):report['trackedGenerationsAbsent']=False
report['ownedExecutablesRemaining']=[line for line in subprocess.check_output(['/bin/ps','-ww','-axo','pid=,comm='],text=True).splitlines() if str(root) in line]
report['diagnosticPassed']=report['exitCode']==0 and report['exactOneTestExecuted'] and report['twoCycleCompletionMarker'] and summary.get('rootCycles',{}).get('passed') is True and report['globalLUnchanged'] and report.get('globalLExclusiveReleased') is True and report.get('globalLSidecarsAbsent') is True and report['trackedGenerationsAbsent'] and not report['ownedExecutablesRemaining'] and not report.get('coordinatorIdentityChanges') and not report.get('observationErrors')
(out/'runner-summary.json').write_text(json.dumps(report,indent=2))
print(json.dumps({k:report[k] for k in ['accepted','temporaryRoot','pid','exitCode','elapsedSeconds','diagnosticPassed','globalLUnchanged','exactOneTestExecuted','twoCycleCompletionMarker']},indent=2),flush=True)
sys.exit(0 if report['diagnosticPassed'] else 1)
