from pathlib import Path
import json,gzip,hashlib,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());seen=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in seen
 seen.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
source=json.loads((base/'compiled-source.json').read_text());assert source['baseCommit']==manifest['sourceCommit'] and source['cleanSource'] and len(source['files'])==22 and source['binarySha256']=='1313d1ec98cf69a64c9f50e2e03c8eb0863c41fff0f2a64f7e59db7fba54f323'
assert json.loads((base/'release-build-final.json').read_text())['exitCode']==0
assert json.loads(subprocess.check_output([sys.executable,str(base/'verify_failed.py')],text=True))==json.loads((base/'verified-failure.json').read_text())
out=base/'native-xattr-r1';prefix='ROOT_REGISTRATION_COST ';lines=gzip.decompress((out/'native.stderr.txt.gz').read_bytes()).decode().splitlines();costs=[json.loads(v.split(prefix,1)[1]) for v in lines if v.startswith(prefix)];record=json.loads((base/'verified-native-partition.json').read_text());assert len(costs)==len(record['samples'])==2
keys=['transaction_storage_begin','transaction_source','transaction_admission','transaction_sql','transaction_candidate','transaction_commit','transaction_postcommit','registration_facts']
for i,cost in enumerate(costs):
 assert cost['passed'];m=cost['metrics'];s=record['samples'][i]
 assert s['totalNanos']==m['registration_total']['inclusive_nanos'] and s['exclusiveStageNanos']=={k:m[k]['inclusive_nanos'] for k in keys}
 assert s['partitionNanos']==sum(s['exclusiveStageNanos'].values()) and 0.999<s['partitionNanos']/s['totalNanos']<=1
 assert s['frameChecks']==m['frame']['calls'] and s['archiveChecks']==m['frame_archives']['calls'] and s['nativeXattrCalls']==m['xattr_native_call']['calls']
assert record['samples'][1]['frameChecks']==48 and record['samples'][1]['historicalBegin']==3 and record['samples'][1]['historicalFinish']==3 and record['samples'][1]['historicalFullPasses']==6
cleanup=json.loads((base/'cleanup-final.json').read_text());assert cleanup['actualNativeRounds']==1 and cleanup['archiveComparedBeforeRemoval'] and cleanup['temporaryNativeRootRemoved'] and cleanup['rawProfileFiles']==79 and cleanup['exactNewCoordinatorsRemoved']==3 and cleanup['globalLUnchangedAndReleased'] and cleanup['normalStopBudgetUnchanged']
summary=json.loads((out/'native/summary.json').read_text());cycles=summary['rootCycles'];assert cycles['secondClosed']['state']=='root_absent' and cycles['secondClosed']['revision']==18 and cycles['closedGenerationCount']==2 and cycles['secondStopGate'] is False
print(json.dumps({'accepted':False,'sourceCommit':manifest['sourceCommit'],'actualNativeRounds':1,'firstCycleStartStatusStopMs':[8038,229,6366],'secondCycleStartStatusStopMs':[10241,819,11345],'secondRegistrationMs':round(record['samples'][1]['totalNanos']/1e6,3),'secondRootMovedAndClosed':True,'retainedState':'RootAbsent18','completeClosedGenerations':2,'stop10Passed':False,'failureBoundary':'coordination-final-deadline-after-release-and-program-source-recheck','rawProfileFilesVerified':79,'allOwnedTemporaryRootsRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in manifest['files'])},indent=2))
