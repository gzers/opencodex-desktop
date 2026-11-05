import json,subprocess,shutil
from pathlib import Path
b=Path('.toolchain/post021-msvc-followup');artifacts=[]
for line in (b/'connected-native/panel-fix-build.jsonl').read_text(encoding='utf-8').splitlines():
 try:v=json.loads(line)
 except ValueError:continue
 if v.get('reason')=='compiler-artifact'and v.get('executable')and v['target']['name']=='opencodex_desktop_lib'and v['profile']['test']:artifacts.append(v)
assert len(artifacts)==1
source=Path(artifacts[0]['executable']);target=b/'gnu-tests';target.mkdir(exist_ok=True);exe=target/'panel-tests.exe';shutil.copyfile(source,exe);shutil.copyfile(source.parent.parent/'WebView2Loader.dll',target/'WebView2Loader.dll')
with(target/'manifest.log').open('w')as stream:subprocess.run(['python','.toolchain/post021-materials/manifest-candidate.py',str(exe)],check=True,stdout=stream,stderr=subprocess.STDOUT)
with(b/'connected-native/panel-fix-rust-tests.log').open('w',encoding='utf-8')as stream:r=subprocess.run([str(exe),'commands::panel::tests::','--nocapture'],stdout=stream,stderr=subprocess.STDOUT)
print((b/'connected-native/panel-fix-rust-tests.log').read_text(encoding='utf-8'));raise SystemExit(r.returncode)
