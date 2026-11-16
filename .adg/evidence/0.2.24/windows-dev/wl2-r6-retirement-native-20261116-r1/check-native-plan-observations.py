from pathlib import Path,PureWindowsPath
import json,re,hashlib
ROOT=Path(__file__).resolve().parent
known={'.codex':{'.opencodex-native-main.claim.sqlite','.opencodex-native-main.owner.sqlite'},'.grok':set()}
artifacts={'opencodex-catalog.json','models_cache.json','opencodex-catalog-audit.jsonl'}
rows=[]
for name in ['library02','library03','library04','library05','library06']:
    text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
    for line in text.splitlines():
        marker=next((m for m in ['r6-native-retirement-plans=','r6-native-retirement-existing-plan='] if m in line),None)
        if not marker:continue
        value=json.loads(line.split(marker,1)[1]);plans=value if isinstance(value,list) else [value]
        for plan in plans:
            assert plan['schema']=='windows-anchored-native-retirement-data-v3' and plan['runtimeAuthority'] is False
            original=plan['originalTree'];current=plan['currentTree'];root=PureWindowsPath(current['root']);leaf=root.name
            assert leaf in known and PureWindowsPath(original['root'])==root
            old={n['relative']:n for n in original['nodes']};now={n['relative']:n for n in current['nodes']}
            assert len(old)==len(original['nodes']) and len(now)==len(current['nodes'])
            expected={p for p,n in now.items() if not n['directory'] and p not in known[leaf]}
            retired={r['relative']:r for r in plan['retirements']};assert set(retired)==expected and len(retired)==len(plan['retirements'])
            targets=set();ids=set()
            for relative,r in retired.items():
                source=now[relative];target=PureWindowsPath(r['target']);custody=PureWindowsPath(original['custody'])
                assert target.parent==custody and str(target).lower() not in targets;targets.add(str(target).lower())
                assert r['currentVolume']==source['volume']==original['custodyVolume'] and r['currentFileId']==source['fileId']
                assert r['sha256']==source['sha256'] and r['bytes']==source['bytes'] and r['security']==source['security']
                assert r['targetParentId']==original['custodyId']
                parent=PureWindowsPath(relative).parent.as_posix();parent='' if parent=='.' else parent
                assert parent in old and old[parent]['kind']=='DirectoryAnchor' and r['sourceParentId']==old[parent]['fileId']
                ident=(r['currentVolume'],r['currentFileId']);assert ident not in ids;ids.add(ident)
                assert r['originalAbsent']==(relative not in old)
                if r['originalAbsent']:assert leaf=='.codex' and relative in artifacts and r['originalFileId'] is None
                else:assert old[relative]['kind']=='CustodiedFile' and r['originalFileId']==old[relative]['fileId'] and r['currentFileId']!=r['originalFileId']
            for relative,n in old.items():
                if n['kind']!='CustodiedFile':
                    assert relative in now and now[relative]['fileId']==n['fileId'] and now[relative]['volume']==n['volume'] and now[relative]['security']==n['security']
            rows.append(dict(run=name,root=leaf,currentFileRows=len(retired),originalFileRows=sum(n['kind']=='CustodiedFile' for n in old.values()),originalAbsentRows=sum(r['originalAbsent'] for r in retired.values()),sha256=hashlib.sha256(json.dumps(plan,sort_keys=True).encode()).hexdigest(),dataInternallyConsistent=True))
assert rows
with (ROOT/'native-plan-data-check.json').open('x',encoding='utf-8') as out:json.dump(dict(rows=rows,meaning='offline consistency of captured native observation bytes only; not an independent Win32 execution or restore permission',mutationAuthority=False),out,indent=2)
print(json.dumps(dict(observedRootPlansChecked=len(rows),mutationAuthority=False)))
