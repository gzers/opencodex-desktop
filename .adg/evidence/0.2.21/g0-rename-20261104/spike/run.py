import pathlib,subprocess,os,sys,json,time
p=pathlib.Path(__file__).resolve().parent
env=os.environ.copy()
env.update({'G0_RUNTIME_ROOT':str(p/'runtime'),'G0_TRACE':'1','G0_FIXTURE_HEADER':'fixture-environment-value','OPENCODEX_SANDBOX':'1','OPENCODEX_SANDBOX_ROOT':str(p/'runtime/manager')})
command=['/usr/bin/sandbox-exec','-f',str(p/'runtime.sb'),str(pathlib.Path(env.get('G0_TARGET_DIR',str(p/'target')))/'debug/ocx-g0-rename-spike')]
attempt=len(list((p/'logs').glob('run-*.meta.json')))+1
name='run-'+str(attempt).zfill(2)
started=time.time()
with (p/'logs'/ (name+'.jsonl')).open('w') as out,(p/'logs'/(name+'.stderr')).open('w') as err:
 status=subprocess.run(command,env=env,stdout=out,stderr=err).returncode
meta={'command':command,'exit_code':status,'elapsed_seconds':round(time.time()-started,3),'attempt':attempt}
(p/'logs'/(name+'.meta.json')).write_text(json.dumps(meta,indent=2)+'\n')
print(json.dumps(meta))
print((p/'logs'/(name+'.jsonl')).read_text())
if status: print((p/'logs'/(name+'.stderr')).read_text()[-6000:])
sys.exit(status)
