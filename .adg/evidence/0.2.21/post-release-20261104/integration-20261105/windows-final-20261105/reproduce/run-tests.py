import sys,json,subprocess,shutil
from pathlib import Path
base=Path(__file__).parent
artifacts=[]
for line in (base/'backend-build.jsonl').read_text(encoding='utf-8').splitlines():
 try:item=json.loads(line)
 except ValueError:continue
 if item.get('reason')=='compiler-artifact' and item.get('executable') and item['target']['name']=='opencodex_desktop_lib' and item['profile']['test']:artifacts.append(item)
assert len(artifacts)==1
root=base/'backend-tests';root.mkdir(exist_ok=True);exe=root/'restart-tests.exe'
shutil.copy2(artifacts[0]['executable'],exe);shutil.copy2(Path(artifacts[0]['executable']).parent.parent/'WebView2Loader.dll',root/'WebView2Loader.dll')
subprocess.run([sys.executable,str(base.parent/'post021-materials/manifest-candidate.py'),str(exe)],check=True,stdout=(root/'manifest.log').open('w'))
code=0
for name in sys.argv[1:] or ['modules::model_config::cache_refresh::tests::','modules::model_config::management::tests::']:
 with (root/(name.replace(':','-')+'.log')).open('w',encoding='utf-8') as log:result=subprocess.run([str(exe),name,'--test-threads=2'],stdout=log,stderr=subprocess.STDOUT)
 print((root/(name.replace(':','-')+'.log')).read_text(encoding='utf-8'));code|=result.returncode
sys.exit(code)
