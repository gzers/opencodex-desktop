import pathlib, tempfile, shutil, json, subprocess, os, sys
source=pathlib.Path(__file__).resolve().parent
context=json.loads((source/'context.json').read_text())
work=pathlib.Path(tempfile.mkdtemp(prefix='ocx-g0-rename-replay-')).resolve()
for name in ['Cargo.toml','Cargo.lock','build.py','run.py','build.sb','runtime.sb','context.json']:
 shutil.copy2(source/name,work/name)
shutil.copytree(source/'src',work/'src')
(work/'logs').mkdir();(work/'runtime').mkdir()
profile=(work/'runtime.sb').read_text().replace(context['spike'],str(work))
(work/'runtime.sb').write_text(profile)
env=os.environ.copy()
cache=pathlib.Path(context['spike'])/'target'
if cache.exists(): env['G0_TARGET_DIR']=str(cache)
print('isolated replay root: '+str(work),flush=True)
subprocess.run([sys.executable,str(work/'build.py')],cwd=work,env=env,check=True)
subprocess.run([sys.executable,str(work/'run.py')],cwd=work,env=env,check=True)
