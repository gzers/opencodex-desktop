import pathlib,subprocess,os,sys,time
p=pathlib.Path(__file__).resolve().parent
env=os.environ.copy()
env['CARGO_TARGET_DIR']=env.get('G0_TARGET_DIR',str(p/'target'))
env['CARGO_NET_OFFLINE']='true'
env['CARGO_INCREMENTAL']='0'
env['CARGO_BUILD_JOBS']='6'
command=['/usr/bin/sandbox-exec','-f',str(p/'build.sb'),'cargo','build','--offline','--locked','--manifest-path',str(p/'Cargo.toml')]
with (p/'logs/build.log').open('a') as log:
 log.write('COMMAND '+repr(command)+'\n');log.flush()
 status=subprocess.run(command,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
print('build_exit='+str(status),flush=True)
sys.exit(status)
