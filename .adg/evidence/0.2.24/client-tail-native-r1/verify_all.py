from pathlib import Path
import json,hashlib,gzip,subprocess,sys
base=Path(__file__).resolve().parent;m=json.loads((base/'manifest.json').read_text());seen=set()
for row in m['files']:
 p=Path(row['path']);assert not p.is_absolute() and '..' not in p.parts and row['path'] not in seen;seen.add(row['path']);b=(base/p).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==seen
s=json.loads((base/'compiled-source.json').read_text());assert s['cleanSource'] and s['baseCommit']==m['sourceCommit'] and len(s['files'])==24 and s['binarySha256']=='dfac55863e94751d4a7aa278dd299dc7a133df2bc62004d014df97ca22888a35'
assert json.loads((base/'release-build-final.json').read_text())['exitCode']==0
assert json.loads(subprocess.check_output([sys.executable,str(base/'verify_failed.py')],text=True))==json.loads((base/'verified-failure.json').read_text())
c=json.loads((base/'cleanup-final.json').read_text());assert c['actualNativeRounds']==1 and c['rawProfileFiles']==40 and c['archiveComparedBeforeRemoval'] and c['temporaryNativeRootRemoved'] and c['exactNewCoordinatorsRemoved']==3 and c['globalLUnchangedAndReleased'] and c['noStopSuccessInferred']
stderr=gzip.decompress((base/'native-tail-r1/native.stderr.txt.gz').read_bytes()).decode();prefix='COORDINATION_TAIL_COST ';tails=[json.loads(v.split(prefix,1)[1]) for v in stderr.splitlines() if v.startswith(prefix)];assert len(tails)==2 and all(not v['expiredAfter'] for v in tails)
assert 'ROOT_REGISTRATION_COST ' not in stderr
print(json.dumps({'accepted':False,'sourceCommit':m['sourceCommit'],'actualNativeRounds':1,'firstStartMs':7436,'signedStatusMs':229,'failure':'first-stop-procargs-EINVAL-before-normal-exit-proof','indexState':'StopPrepared5','rawProfileFilesVerified':40,'secondCycleExecuted':False,'stopTailCostObtained':False,'startOnlyTailSourceMs':[round(v['completeSourceUs']/1000,3) for v in tails],'normalStopSuccessUnproven':True,'allOwnedTemporaryRootsRemoved':True,'manifestFiles':len(seen),'manifestBytes':sum(v['bytes'] for v in m['files'])},indent=2))
