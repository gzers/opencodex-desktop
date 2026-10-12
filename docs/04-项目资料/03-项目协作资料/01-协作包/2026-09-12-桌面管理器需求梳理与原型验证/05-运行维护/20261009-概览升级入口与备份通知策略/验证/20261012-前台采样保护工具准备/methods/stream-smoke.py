"""Owned synthetic subprocesses only. Not a native-app or resource-collector test."""
import datetime, json, os, selectors, subprocess, sys, time
from pathlib import Path
from run_guard import classify, read_available, stop_owned, stream_timeout, validate_finished

EMITTER = '''import json,sys,time
mode=sys.argv[1]
def emit(v):
 print(json.dumps(v),flush=True)
def sample(t,front=True):
 return dict(kind="sample",elapsed_s=t,target_pid=42,target_frontmost=front,target_window_on_screen=True)
emit(sample(0));emit(dict(kind="ready",elapsed_s=0))
if mode=="stall": time.sleep(30)
for i in range(1,5):
 time.sleep(.1);emit(sample(i*.1,front=not(mode=="loss" and i==3)))
if mode=="loss": time.sleep(30)
emit(dict(kind="stopped",armed=True,elapsed_s=.5))
'''
results=[]
for mode,expected in [('healthy',None),('loss','foreground_or_visible_window_lost'),('stall','monitor_stream_stalled_over_one_second')]:
 monitor=subprocess.Popen([sys.executable,'-u','-c',EMITTER,mode],stdout=subprocess.PIPE,bufsize=0)
 collector=subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)'])
 os.set_blocking(monitor.stdout.fileno(),False)
 selector=selectors.DefaultSelector();selector.register(monitor.stdout,selectors.EVENT_READ)
 started=last_received=time.monotonic();ready=False;previous=None;pending=b'';events=[];reason=None;ended=False
 try:
  while not reason and not ended:
   for key,_ in selector.select(timeout=.05):
    batch,pending,eof=read_available(key.fileobj,pending)
    if batch: last_received=time.monotonic()
    for event in batch:
     events.append(event)
     if event['kind']=='ready': ready=True
     if ready:
      reason=classify(event,previous)
      if event['kind']=='sample':previous=event['elapsed_s']
     if reason:break
     if event['kind']=='stopped':ended=True
    if eof:ended=True
   reason=reason or stream_timeout(ready,last_received,started,time.monotonic())
   if time.monotonic()-started>3:raise AssertionError('synthetic test exceeded bound')
  if not reason:reason=validate_finished(events,dict(process_identity_stable=True))
  before_stop=dict(monitor_running=monitor.poll() is None,collector_running=collector.poll() is None)
 finally:
  stop_owned(collector);stop_owned(monitor);selector.close();monitor.stdout.close()
 elapsed=time.monotonic()-started
 assert reason==expected,(mode,reason,expected)
 assert collector.poll() is not None and monitor.poll() is not None
 assert elapsed<2,elapsed
 results.append(dict(mode=mode,passed=True,reason=reason,elapsed_seconds=elapsed,before_owned_cleanup=before_stop,monitor_stopped=True,collector_stopped=True))
# One read can contain multiple lines and a partial tail; do not rely on TextIO buffering.
r,w=os.pipe()
try:
 os.write(w,b'{"kind":"ready"}\n{"kind":"sample"}\n{"kind":')
 with os.fdopen(r,'rb',buffering=0) as pipe:
  batch,pending,eof=read_available(pipe,b'')
  assert len(batch)==2 and pending==b'{"kind":' and not eof
  os.write(w,b'"stopped"}\n')
  batch,pending,eof=read_available(pipe,pending)
  assert batch==[dict(kind='stopped')] and pending==b'' and not eof
  os.close(w);w=None
  batch,pending,eof=read_available(pipe,pending)
  assert batch==[] and eof
finally:
 if w is not None:os.close(w)
results.append(dict(mode='burst_and_split_raw_pipe',passed=True))
report=dict(captured_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),scope='synthetic I/O/watchdog/owned-child cleanup only; real app and full run_guard integration untested',count=len(results),failed=0,cases=results)
p=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).with_name('stream-smoke.json')
with p.open('x') as s:json.dump(report,s,indent=2);s.write('\n')
print(json.dumps(dict(count=len(results),failed=0,report=str(p))))
