from pathlib import Path
import json,os
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
results=[]
for name in ['profile01','profile02']:
    p=load(ROOT/name/'result.json');a=p['samples'][0];b=p['samples'][-1]
    row=dict(run=name,rawExit=p['rawExit'],passed=p['passed'],failed=p['failed'],wallMs=p['elapsedMs'],childCpuMs=b['kernelMs']+b['userMs'],
        kernelMs=b['kernelMs'],userMs=b['userMs'],cpuFractionOfOneCore=(b['kernelMs']+b['userMs'])/p['elapsedMs'],logicalProcessors=os.cpu_count(),
        readOps=b['readOps'],otherOps=b['otherOps'],readBytes=b['readBytes'],otherBytes=b['otherBytes'],diagnosticOnly=True,noSourceOrBudgetChanges=True)
    if 'systemTime100ns' in a and 'systemTime100ns' in b:
        a=a['systemTime100ns'];b=b['systemTime100ns'];idle=b['idle']-a['idle'];total=b['kernelIncludesIdle']-a['kernelIncludesIdle']+b['user']-a['user'];row['systemBusyFraction']=1-idle/total
    results.append(row)
with (ROOT/'profile-outcome.json').open('x',encoding='utf-8') as out:json.dump(dict(results=results,
    conclusion='Full scope fails original10s with child near one-core CPU saturation while global sampled utilization is low; evidence does not justify treating this as whole-machine load or relaxing budgets. Single-test diagnostic passes are isolated observations, not full-scope acceptance.',
    nextImplementation='r6-permitted bounded node event batches in same canonical chain, reducing per-node durable record count/full-chain native validation cost while retaining exact intent-before-native, completion-after-native, claim/NLK and all original safety comparisons. Requires its own native prefix and budget evidence before adoption.',
    publicStartOpen=False,originalBudgetsChanged=False,candidateAccepted=False),out,indent=2)
print(json.dumps(dict(profileSummary=results,candidateAccepted=False)))
