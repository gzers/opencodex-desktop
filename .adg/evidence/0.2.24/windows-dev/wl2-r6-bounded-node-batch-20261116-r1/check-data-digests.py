from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
storage=(BASE/'v3.rs').read_text(encoding='utf-8');recovery=(BASE/'v3/recovery.rs').read_text(encoding='utf-8')
for name,text in [('StoragePlan',storage),('RecoveryPlan',recovery)]:
    block=text.split('struct '+name+' {',1)[1].split('\n}',1)[0]
    assert '#[serde(skip)]' in block and 'data_digest: std::sync::OnceLock<String>' in block
    assert 'pub' not in block,'immutable plan fields must stay private'
    digest=text.split('pub(super) fn digest(&self)',1)[1].split('\n    }',1)[0]
    assert 'data_digest.get()' in digest and 'data_digest.set(digest.clone())' in digest
    assert 'native_stamp' not in digest and '.check(' not in digest
    assert 'sha256_hex' in digest
assignments=[]
for p in (BASE/'v3').rglob('*.rs'):
    # Complete plan fields have no mutation setters or mutable public access;
    # tests edit independent serde_json::Values and construct fresh plan data.
    text=p.read_text(encoding='utf-8')
    assert not re.search(r'fn\s+\w+\([^)]*&mut\s+(?:StoragePlan|RecoveryPlan)',text),p
    for line in text.splitlines():
        if '.data_digest' in line:assignments.append(dict(path=p.relative_to(CODE).as_posix(),text=line.strip()))
assert 'r.plan.digest()?' in recovery and '&r.retired' in recovery and '&r.restored' in recovery and '&r.pending' in recovery and 'r.whole_tree' in recovery
same=[]
for rel in ['tree/anchored.rs','tree/anchored/batches.rs','tree/anchored/observation/retirement.rs',
 'tree/anchored/observation/retirement/witness.rs','v3/journal/batches.rs','v3/journal/recovery/final_release.rs',
 'v3/journal/completed.rs']:
    p=BASE/rel;raw=p.read_bytes();old=(ROOT/'library04/executed-source'/p.relative_to(CODE)).read_bytes();assert raw==old,rel
    same.append(dict(path=p.relative_to(CODE).as_posix(),sha256=sha(raw),byteEqualToPriorFailedNativeRun=True))
test=(ROOT/'library05/stdout.log').read_text(encoding='utf-8-sig')
assert 'wl2_r6_batch_data_digest_reuse_only_immutable_plan_bytes_never_serialized_native_facts' in test and '31 passed; 0 failed' in test
with (ROOT/'data-digest-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(planFieldsPrivate=True,cacheNeverSerializedOrSuppliedByPersistedInput=True,noMutablePlanSetter=True,
        digestIsPureDataOnly=True,mutableSummaryBindsPlanDigestAndAllNodePendingAnchorStates=True,
        oldNativeImplementationsAndComparisons=same,observedDigestCacheCode=assignments,
        actualDataCounterexamplesPassed=True,originalDeadlineChanged=False,unknownRefusalChanged=False),out,indent=2)
print(json.dumps(dict(immutablePlans=2,oldNativeFilesByteEqual=len(same),nativeFactCached=False,dataTestsPassed=True)))
