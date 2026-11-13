from pathlib import Path
import tarfile,json,hashlib,subprocess,os,shutil,tempfile
b=Path(__file__).resolve().parent;work=Path(tempfile.mkdtemp(prefix='ocxd024-writer-direct-',dir='/private/tmp'));p=work/'profile';p.mkdir(mode=0o700);c=p/'codex';c.mkdir(mode=0o700);h=p/'home';h.mkdir(mode=0o700)
repo=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');res=repo/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle';sha=lambda v:hashlib.sha256(v).hexdigest()
manifest=(res/'runtime-bundle.manifest.json').read_bytes();assert sha(manifest)=='e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3';d=json.loads(manifest);entry=next(x for x in d['entries'] if x['path']=='runtime/bun/bun')
with tarfile.open(res/d['archiveName']) as t:data=t.extractfile('runtime/bun/bun').read();assert sha(data)==entry['sha256'];bun=work/'bun';bun.write_bytes(data);bun.chmod(0o755)
archive=(res/'official-runtime.tgz').read_bytes();assert sha(archive)=='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'
with tarfile.open(res/'official-runtime.tgz') as t:
 for member in t:
  path=Path(member.name);assert not path.is_absolute() and '..' not in path.parts
  if member.isdir():(work/path).mkdir(parents=True,exist_ok=True)
  elif member.isfile():target=work/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(t.extractfile(member).read());target.chmod(member.mode&0o777)
  else:raise AssertionError(member.name)
script=repo/'apps/desktop/tauri/src/infrastructure/client_config_guard299.mjs';helper=work/'helper.mjs';helper.write_bytes(script.read_bytes())
policy='(version 1)\n(allow default)\n(deny network*)\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(work))+'))\n(deny file-read* (subpath "/Users/ezio"))\n';sb=work/'fixture.sb';sb.write_text(policy)
inputs={'schema':'ocxd-config-write-guard/2','operation':'b4e572c2-c293-4cef-9883-7d36c1b66314','packageRoot':str(work/'package'),'profile':str(p),'home':str(h),'codex':str(c),'target':str(c)}
proc=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sb),str(bun),str(helper),'--',json.dumps(inputs)],env={'HOME':str(p),'OPENCODEX_HOME':str(h),'CODEX_HOME':str(c),'PATH':'/usr/bin:/bin'},input=b'',capture_output=True,timeout=10)
result={'accepted':False,'helperSha256':sha(helper.read_bytes()),'exitCode':proc.returncode,'stdout':proc.stdout.decode(),'stderr':proc.stderr.decode(),'rootNlinkAfter':c.stat().st_nlink,'scope':'direct exact helper phase diagnosis using pinned Bun and unmodified official sources, no NativeLease or service'}
(b/'direct-probe-result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2));shutil.rmtree(work)
