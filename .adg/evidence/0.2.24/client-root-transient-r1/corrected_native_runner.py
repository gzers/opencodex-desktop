from pathlib import Path
import subprocess,json,hashlib,os,re,time,shutil,tempfile,sqlite3,stat
base=Path(__file__).resolve().parent;out=base/"official-corrected";out.mkdir()
work=Path(tempfile.mkdtemp(prefix="ocxd024-native-coordination-",dir="/private/tmp"));root=work/"fixtures";root.mkdir(mode=0o700)
profile=root/"profile";profile.mkdir(mode=0o700);(root/"tmp").mkdir(mode=0o700);res=root/"resources";res.mkdir()
code=Path("/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes")
source=code/"apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle"
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for name in ["runtime-bundle.manifest.json"]:shutil.copy2(source/name,res/name)
assert sha(res/"runtime-bundle.manifest.json")=="e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
log=(base/"root-transient-final-tests.log").read_text();m=re.search(r"Running unittests src/lib.rs \(([^)]+)\)",log);assert m
compiled=code/"apps/desktop/tauri"/m.group(1);exe=root/"library-tests";shutil.copy2(compiled,exe);assert sha(exe)==sha(compiled)
client=root/"transient/profile/.codex";digest=hashlib.sha256(str(client).encode()).hexdigest();namespace=Path("/private/tmp")/("opencodex-runtime-v1-"+str(os.getuid()))
locks=[Path(str(namespace/kind/digest)+suffix) for kind in ["native-write-locks","catalog-write-locks"] for suffix in [".sqlite",".sqlite-journal",".sqlite-wal",".sqlite-shm"]]
assert not any(p.exists() for p in locks)
lpath=namespace/"client-desktop-lifecycle.sqlite"
def snapshot(p):
 s=p.lstat();assert stat.S_ISREG(s.st_mode) and s.st_nlink==1 and s.st_uid==os.getuid() and stat.S_IMODE(s.st_mode)==0o600
 return {"path":str(p),"device":s.st_dev,"inode":s.st_ino,"mode":stat.S_IMODE(s.st_mode),"uid":s.st_uid,"gid":s.st_gid,"bytes":s.st_size,"sha256":sha(p)}
before=snapshot(lpath)
assert not any(Path(str(lpath)+s).exists() for s in ["-journal","-wal","-shm"])
def exclusive(p):
 c=sqlite3.connect(p.as_uri()+"?mode=rw",uri=True,timeout=0);c.execute("begin exclusive");c.execute("rollback");c.close()
exclusive(lpath);assert snapshot(lpath)==before
(out/"global-l-before.json").write_text(json.dumps(before,indent=2))
policy="(version 1)\n(allow default)\n(deny network*)\n(deny file-write*)\n(allow file-write* (subpath "+json.dumps(str(root))+"))\n(allow file-write* (literal \"/dev/null\"))\n(deny file-read* (subpath \"/Users/ezio\"))\n(deny mach-lookup (global-name \"com.apple.securityd\") (global-name \"com.apple.secd\"))\n"
for path in locks+[Path(str(lpath)+s) for s in ["","-journal","-wal","-shm"]]:policy+="(allow file-write* (literal "+json.dumps(str(path))+"))\n"
pattern="^"+re.escape(str(namespace/"native-write-locks"/("."+digest+".sqlite.adopt-")))+"[0-9]+-[0-9a-f-]+"+re.escape(".tmp")+"(-journal|-wal|-shm)?$"
policy+="(allow file-write* (regex "+json.dumps(pattern)+"))\n(allow file-write-mode (literal "+json.dumps(str(namespace))+"))\n"
sb=work/"fixture.sb";sb.write_text(policy);(out/"fixture.sb").write_text(policy)
env={"PATH":"/usr/bin:/bin:/usr/sbin:/sbin","HOME":str(profile),"CODEX_HOME":str(client),"TMPDIR":str(root/"tmp")+"/","LANG":"en_US.UTF-8","OCXD_NATIVE_COORDINATION_ROOT":str(root),"OCXD_NATIVE_ROOT_TRANSIENT":"1"}
env["OCXD_NATIVE_COORDINATION_REUSE"]='/private/tmp/ocxd024-native-coordination-vz9xlkc2/fixtures/program'
for label,argv in [("network",["/usr/bin/nc","-v","-G","1","-z","192.0.2.1","443"]),("daily-home",["/bin/cat","/Users/ezio/.zshrc"])]:
 p=subprocess.run(["/usr/bin/sandbox-exec","-f",str(sb),*argv],env=env,cwd=root,capture_output=True);(out/(label+".stdout.log")).write_bytes(p.stdout);(out/(label+".stderr.log")).write_bytes(p.stderr);assert p.returncode and b"Operation not permitted" in p.stderr
paths=sorted(set(subprocess.check_output(["git","diff","--name-only"],cwd=code,text=True).splitlines()+subprocess.check_output(["git","ls-files","--others","--exclude-standard"],cwd=code,text=True).splitlines()))
(out/"source.json").write_text(json.dumps({"baseCommit":subprocess.check_output(["git","rev-parse","HEAD"],cwd=code,text=True).strip(),"files":[{"path":n,"sha256":sha(code/n)} for n in paths],"binarySha256":sha(exe)},indent=2))
report={"accepted":False,"temporaryRoot":str(work),"exactNewCoordinatorFiles":[str(p) for p in locks],"preexistingGlobalL":str(lpath),"noServiceOrBusinessRequests":True,"binarySha256":sha(exe),"hypothesis":"Compose actual official299 NativeLease and config Guard with independently registered synthetic storage Original and explicit test-only movement permission. Claim handoff must normally remove the real journal before plan snapshot without any journal exception. Reuse only verified immutable runtime input from prior terminal diagnostic; new independent Profile. Plan binds exact live marker; retire same directory, normally release official helper, read exact released tree and independently close RootAbsent. No service or normal-Stop proof claimed."}
(out/"hypothesis.json").write_text(json.dumps(report,indent=2));start=time.monotonic()
with (out/"native.stdout.log").open("wb") as stdout,(out/"native.stderr.log").open("wb") as stderr:
 p=subprocess.Popen(["/usr/bin/sandbox-exec","-f",str(sb),str(exe),"--exact","infrastructure::official_client_coordination::native::tests::official299_native_handover_holds_through_work_and_preserves_original_ownership","--ignored","--nocapture","--test-threads=1"],env=env,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
 report["pid"]=p.pid;(out/"running.json").write_text(json.dumps(report))
 try:report["exitCode"]=p.wait(timeout=240)
 except subprocess.TimeoutExpired:report["exitCode"]=None
report["elapsedSeconds"]=round(time.monotonic()-start,3)
for name in ["native-summary.json","official-initialize.stdout.log","official-initialize.stderr.log","official-writers-blocked.stdout.log","official-writers-blocked.stderr.log","official-writers-released.stdout.log","official-writers-released.stderr.log","root-transient-references.json","root-transient-original-bytes.json","root-transient-failure.json"]:
 if (root/name).exists():shutil.copy2(root/name,out/name)
report["ownedProcessesRemaining"]=[line for line in subprocess.check_output(["/bin/ps","-axo","pid=,ppid=,pgid=,comm="],text=True).splitlines() if str(root) in line]
(out/"runner-summary.json").write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2),flush=True)
if report["exitCode"] is None or report["ownedProcessesRemaining"]:raise SystemExit("Live handle retained; no cleanup or restart")
exclusive(lpath);after=snapshot(lpath);assert after==before;(out/"global-l-after.json").write_text(json.dumps({"sameIdentityModeBytes":True,"exclusiveReacquiredAndReleased":True,"snapshot":after},indent=2))
removed=[]
for path in locks:
 if not path.exists():continue
 assert path.suffix==".sqlite",str(path)
 exclusive(path);item=snapshot(path);removed.append(item);path.unlink()
(out/"cleanup.json").write_text(json.dumps({"exactOwnedCoordinatorFilesRemoved":removed,"globalLUnchanged":True,"allOwnedProcessesExited":True,"ownedProfileAndProgramRemoved":str(work),"serviceOrBusinessCalls":False},indent=2))
shutil.rmtree(work);assert not work.exists()
