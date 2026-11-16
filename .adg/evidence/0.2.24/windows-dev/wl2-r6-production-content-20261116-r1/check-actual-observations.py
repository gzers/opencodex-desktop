from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
manifest=load(ROOT/'fixed-audit-source-manifest.json');expected={row['path']:row['sha256'] for row in manifest['files']}
rows=[];pins=[]
for run in sys.argv[1:]:
    for line in (ROOT/run/'stdout.log').read_text(encoding='utf-8-sig').splitlines():
        marker='r6-production-content-fixed-audit='
        if marker in line:
            value=json.loads(line.split(marker,1)[1]);assert value['rawExit']==0 and not value['serviceStarted'] and not value['networkOrModelRequests']
            assert dict(value['sources'])==expected
            assert value['bunSha256']=='cb1d8e6564e655e49d30cead359750a1cbbd12f42651caa4b62244d7bd93c244'
            assert len(value['rows'])==12 and sum(not r['unknownRefused'] for r in value['rows'])==8
            for r in value['rows']:
                if r['unknownRefused']:assert r['localProjection'] is None
                else:assert r['localProjection']==r['fixedOfficialProjection']
            assert any('Uſers' in r['input'] and r['localProjection']==r['input'] for r in value['rows'])
            assert any('测试用户' in r['input'] and '[USER]' in r['fixedOfficialProjection'] for r in value['rows'])
            rows.append(dict(run=run,observation=value,componentOnly=True,actualProductionOwner=False))
        marker='r6-production-content-source-pin='
        if marker in line:
            value=json.loads(line.split(marker,1)[1]);assert value['syntheticOnly'] and value['originalBytesUnchanged'] and value['wrongHashRefused']
            assert value['writeRawError'] and value['replaceRawError'] and not value['officialFilesModified']
            path=Path(value['source']);assert path.is_file() and sha(path.read_bytes())==value['expectedSha256']
            pins.append(dict(run=run,observation=value))
assert rows
if sys.argv[-1]=='library04':assert len(pins)==1
with (ROOT/('actual-observations-'+'-'.join(sys.argv[1:])+'.json')).open('x',encoding='utf-8') as out:
    json.dump(dict(fixedBunProjectionRuns=rows,sourcePinNegatives=pins,officialModified=False,
        projectionParitySupportedInputsOnly=True,unknownProjectionRefuses=True,actualTypedStopOrProductionRestoreExecuted=False),out,ensure_ascii=False,indent=2)
print(json.dumps(dict(fixedBunRuns=len(rows),supportedComparisons=sum(len(r['observation']['rows']) for r in rows),sourcePinNegatives=len(pins),productionExecuted=False)))
