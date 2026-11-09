"""Read and archive this one user1 batch; never launches or retries a product."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path

p=argparse.ArgumentParser();p.add_argument('batch',type=Path);p.add_argument('destination',type=Path);a=p.parse_args()
batch=a.batch.resolve();dest=a.destination.resolve()
if dest.exists():raise ValueError('Never overwrite an earlier user1 evidence batch')
dest.mkdir(parents=True)
def read(name,default=None):
    f=batch/name
    return json.loads(f.read_text(encoding='utf-8-sig')) if f.exists() else default
for folder in ['code','user1']:
    base=batch/folder
    if not base.exists():continue
    for f in base.rglob('*'):
        if not f.is_file():continue
        if f.is_symlink():raise ValueError('Evidence symlink refused')
        if f.suffix not in ['.json','.log','.txt','.ps1','.mjs']:continue
        target=dest/f.relative_to(batch);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(f.read_bytes())
for name in ['preflight.json','controller-launch.json','controller.json','provisioned.json','controller-finished.json','assembly-boundaries.json','cleanup-supplement.json','host-environment.json']:
    f=batch/name
    if f.exists():(dest/name).write_bytes(f.read_bytes())
manifest=batch/'program/core-build.json'
if not manifest.exists():
    assembly=read('assembly-boundaries.json',{})
    candidate=assembly.get('programDirectory')
    if candidate:manifest=Path(candidate)/'core-build.json'
if manifest.exists():(dest/'program-manifest.json').write_bytes(manifest.read_bytes())
result=read('user1/user1-result.json',{})
if result.get('trace'):
    # Array position is the immutable event order; a receipt's own sequence may
    # also be present in the raw row. Do not confuse it with the event ordinal.
    ordered=[{'eventOrdinal':i+1,'rawTraceRow':row} for i,row in enumerate(result['trace'])]
    (dest/'event-order.json').write_text(json.dumps({'accepted':False,'orderSource':'Original trace array position, no replay or inferred missing events','events':ordered},ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
control=read('controller-finished.json',{})
native=read('user1/runtime-record-native.json',{})
receipts=[json.loads(f.read_text(encoding='utf-8-sig')) for f in sorted((batch/'user1').glob('action-*.json')) if not f.name.endswith('.raw.json')]
starts=[r for r in receipts if r.get('action')=='start']
success=[r for r in starts if r.get('response',{}).get('value',{}).get('result')=='started']
logfile=batch/'user1/desktop-logs/app.log'
lines=logfile.read_text(encoding='utf-8-sig').splitlines() if logfile.exists() else []
start_lines=[{'file':'user1/desktop-logs/app.log','line':i+1,'text':s} for i,s in enumerate(lines) if '[manager] start:' in s]
admission=[v for v in start_lines if 'lifetime_admission=' in v['text'] or 'ocxd-lifetime-admission-failed' in v['text']]
first=start_lines[0] if start_lines else None
if admission:classification='native-ownership-admission-refused'
elif first and 'exit_code=' in first['text']:classification='actual-start-process-exited-before-confirmation'
elif first and 'lifecycle_timeout' in first['text']:classification='start-confirmation-timeout'
elif not starts:classification='receipt-missing-or-precondition-incomplete'
elif result.get('startConfirmed'):classification='start-confirmed'
else:classification='insufficient-evidence-for-cause'
started=read('user1/start-runtime-step.json',{})
public=started.get('record') or result.get('failure',{}).get('publicFacts',{}).get('record')
match=(bool(public and native) and public['pid']==native['pid'] and public['port']==native['port'] and native['native']['pid']==native['pid'])
report={'accepted':False,'source':'8445d65dca0af76c3275f8617a76c1a726fb8873','batch':batch.name,
 'start':{'rawReceiptCount':len(starts),'successfulReceiptCount':len(success),'uniqueSuccessfulReceipt':len(starts)==len(success)==1,'pidPortConsistentWithNativeRecord':match if public else None,'publicRecord':public,'nativeRecord':native or None,'recordObserved':bool(native),'pidPortUnavailableReason':'No protected runtime record observed; do not treat launcher PIDs as service PIDs' if not native else None,'classification':classification,'firstStartTrace':first,'admissionTrace':admission,'admissionStageKindWin32':'not-observed; no inference of successful admission' if not admission else 'preserved verbatim in admissionTrace'},
 'stop':{'executed':result.get('normalStopExecuted',False),'confirmed':result.get('normalStopConfirmed',False),'reasonIfNotExecuted':'Start not confirmed; no normal Stop permitted' if not result.get('normalStopExecuted') else None,'traceFile':'user1/stop-runtime-step.json' if (dest/'user1/stop-runtime-step.json').exists() else None,'realOriginalExitEvidence':'user1/original-exits-before-cleanup.json' if (dest/'user1/original-exits-before-cleanup.json').exists() else None,'separateStopCliExitCode':None,'separateStopCliReason':'Normal product Stop is authenticated HTTP; do not invent a CLI exit code'},
 'cleanup':{k:control.get(k) for k in ['accountSid','accountName','profile','accountRemoved','profileRemoved','userDirectoryRemoved','desktopAclRestored','identityUnproven','cleanupFailure','residualProcesses','programUnchanged']}}
(dest/'four-part-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
env=read('user1/environment.json',{})
env.update({'productSource':report['source'],'documentHandoff':'e6366f07287467c508592af18ca065935e22a451','artifactId':13667461176,'artifactRun':42602603906,'artifactBytes':304962844,'artifactSha256':'048b4d344c1bd8bbf81adb0a73571f14eaa697e122aafdad7ea6aeda7e42f274','runtimeVersion':'2.97.0-desktop023.lifecycle.2','runtimeBytes':15413257,'runtimeSha256':'c02f4710496775d8f4dfe271bd90d6afdf7dbf960f6e696637271ab9f37bc70b','scriptCheckout':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'accepted':False,'preconditionController':control})
(dest/'environment.json').write_text(json.dumps(env,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
(dest/'.gitattributes').write_text('* -text -whitespace'+chr(10),encoding='utf-8')
inventory=[{'path':f.relative_to(dest).as_posix(),'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted(dest.rglob('*')) if f.is_file()]
(dest/'evidence-manifest.json').write_text(json.dumps({'accepted':False,'files':inventory},ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
print(json.dumps(report,ensure_ascii=False,indent=2))
