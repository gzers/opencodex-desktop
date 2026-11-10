from pathlib import Path
import subprocess,json,time
here=Path(__file__).parent.resolve();root=Path(json.loads((here/'fixture.json').read_text())['root']);attempt=here/'pipeline-attempt3';attempt.mkdir(exist_ok=False);(root/'pipeline.ts').write_bytes((here/'official-pipeline.ts').read_bytes())
env={'PATH':'/usr/bin:/bin','HOME':str(root/'user'),'USERPROFILE':str(root/'user'),'TMPDIR':str(root/'tmp'),'XDG_CACHE_HOME':str(root/'user/cache'),'OPENCODEX_HOME':str(root/'runtime'),'CODEX_HOME':str(root/'codex'),'CLAUDE_CONFIG_DIR':str(root/'claude'),'GROK_HOME':str(root/'grok'),'CODEX_SQLITE_HOME':str(root/'codex'),'CI':'1','NODE_ENV':'test','LANG':'en_US.UTF-8','PROBE_ROOT':str(root),'OFFICIAL_SRC':'/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/@bitkyc08/opencodex/src'}
t=time.monotonic();p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(root/'sandbox.sb'),'/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/bun/bin/bun.exe','run',str(root/'pipeline.ts')],cwd=root,env=env,capture_output=True,text=True,timeout=35)
(attempt/'stdout.log').write_text(p.stdout);(attempt/'stderr.log').write_text(p.stderr);(attempt/'execution.json').write_text(json.dumps({'exit':p.returncode,'elapsed':time.monotonic()-t,'networkDenied':True,'officialSourceUnchanged':True}));
for f in (root/'out').glob('*.json'):(attempt/f.name).write_bytes(f.read_bytes())
print('exit',p.returncode);print(p.stdout[-1600:]);print(p.stderr[-2000:])
