from pathlib import Path
import subprocess,json,os
here=Path(__file__).parent.resolve();root=Path(json.loads((here/'fixture.json').read_text())['root']);upstream='/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/@bitkyc08/opencodex/src';bun='/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/bun/bin/bun.exe'
env={'PATH':'/usr/bin:/bin','HOME':str(root/'user'),'TMPDIR':str(root/'tmp'),'OPENCODEX_HOME':str(root/'user/runtime'),'CODEX_HOME':str(root/'user/codex'),'CLAUDE_CONFIG_DIR':str(root/'claude'),'CI':'1','NODE_ENV':'test','LANG':'en_US.UTF-8','OFFICIAL_SRC':upstream,'PROBE_ROOT':str(root)}
for name in ['agent-write-set','stable-lock']:
 target=root/(name+'.ts');target.write_bytes((here/(name+'-probe.ts')).read_bytes());attempt=here/(name+'-attempt1');attempt.mkdir(exist_ok=False)
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(root/'sandbox.sb'),bun,'run',str(target)],cwd=root,env=env,capture_output=True,text=True,timeout=25);(attempt/'stdout.log').write_text(p.stdout);(attempt/'stderr.log').write_text(p.stderr);(attempt/'execution.json').write_text(json.dumps({'exit':p.returncode,'networkDenied':True,'writesOnlyFixture':True}));out=root/'out'/('agent-result.json' if name=='agent-write-set' else 'lock-result.json')
 if out.exists():(attempt/'result.json').write_bytes(out.read_bytes())
 print(name,'exit',p.returncode,p.stdout[-800:],p.stderr[-1300:])
