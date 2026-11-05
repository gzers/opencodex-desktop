import json,subprocess,time
from pathlib import Path
w=Path.cwd();node='C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node.exe';b=w/'.toolchain/post021-msvc-followup';folder=b/'rate-only-replay';folder.mkdir(exist_ok=True)
command=[node,str(w/'post021-windows-materials/apps/desktop/ui/qa/windows-materials.mjs'),'http://127.0.0.1:9452',str(folder/'output'),'performance',str(w/'.toolchain/post021-windows-final-msvc-41913003716/baseline/sandbox'),'fixed']
with(folder/'stdout.log').open('wb')as stdout,(folder/'stderr.log').open('wb')as stderr:
 start=time.monotonic();process=subprocess.Popen(command,cwd=w/'post021-windows-materials',stdout=stdout,stderr=stderr);(folder/'started.json').write_text(json.dumps({'pid':process.pid,'command':command,'at':time.time()},indent=2),encoding='utf-8');code=process.wait(timeout=150)
 report={'returnCode':code,'elapsedSeconds':time.monotonic()-start,'command':command,'stdoutBytes':(folder/'stdout.log').stat().st_size,'stderrBytes':(folder/'stderr.log').stat().st_size,'outputFiles':[str(p.name)for p in(folder/'output').glob('*')],'scope':'diagnostic-only original MSVC with disable-frame-rate-limit; no acceptance inference from empty output'};(folder/'process-result.json').write_text(json.dumps(report,indent=2)+chr(10),encoding='utf-8');print(json.dumps(report))
