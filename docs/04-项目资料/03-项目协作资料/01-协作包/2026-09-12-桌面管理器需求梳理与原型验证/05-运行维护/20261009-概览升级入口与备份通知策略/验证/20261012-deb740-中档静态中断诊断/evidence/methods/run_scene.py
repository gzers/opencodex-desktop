import datetime,json,pathlib,subprocess,sys,time
base=pathlib.Path(__file__).resolve().parent
phase,scene=sys.argv[1:3]
duration=int(sys.argv[3]) if len(sys.argv)>3 else 180
out=base/'candidate0110'/phase
out.mkdir(exist_ok=True)
assert not (out/'launch.json').exists() and not (out/'samples.jsonl').exists()
original=json.loads((base/'candidate0110/idle-high/launch.json').read_bytes())
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
record=dict(pid=31501,monotonic_start=time.monotonic(),scene_preparation_at=now,existing_launch=original,note='Reused existing isolated instance; this is scene warmup, not application launch.',scene=scene,validation_role='Short attribution diagnostic, not acceptance')
(out/'launch.json').write_text(json.dumps(record,indent=2)+'\n')
stop=out/'foreground.stop'
assert not stop.exists() and not (out/'foreground.json').exists()
with (out/'foreground.stdout.log').open('x') as stdout, (out/'foreground.stderr.log').open('x') as stderr:
 monitor=subprocess.Popen(['/private/tmp/ocx-0110-deb740-foreground/foreground-monitor-routes','31501',phase,str(out/'foreground.json'),str(stop)],stdout=stdout,stderr=stderr)
 print(json.dumps(dict(phase=phase,monitor_pid=monitor.pid,warmup_started_at=now)),flush=True)
 try:
  time.sleep(11)
  assert monitor.poll() is None, 'Foreground monitor failed before warmup'
  time.sleep(80)
  assert monitor.poll() is None, 'Foreground monitor failed before measurement'
  print('measurement starts',flush=True)
  result=subprocess.run([sys.executable,str(base/'measure.py'),'observe','candidate0110',phase,str(duration)])
  assert result.returncode==0, 'Resource collector failed'
 finally:
  stop.touch()
  print('monitor exit',monitor.wait(timeout=10),flush=True)
