from pathlib import Path
import json,base64,hashlib,sqlite3,tempfile,uuid
b=Path(__file__).resolve().parent;sha=lambda raw:hashlib.sha256(raw).hexdigest();log=(b/'atomic-generation-final.log').read_text();rows=[]
for line in log.splitlines():
 if 'SUCCESSOR_GENERATION_EXIT_EVIDENCE ' in line:rows.append(json.loads(line.split('SUCCESSOR_GENERATION_EXIT_EVIDENCE ',1)[1]))
assert len(rows)==5
verified=[]
for row in rows:
 assert row['accepted'] is False and row['actualWaitExitCode']==91 and row['noServiceOrRecapturedCustody']
 with tempfile.TemporaryDirectory(prefix='ocxd024-generation-offline-') as d:
  root=Path(d)
  for entry in row['files']:
   p=Path(entry['path']);assert not p.is_absolute() and '..' not in p.parts;raw=base64.b64decode(entry['rawBase64']);assert len(raw)==entry['bytes'] and sha(raw)==entry['sha256'];dest=root/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw)
  manager=root/'desktop/manager-state';c=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);rawhead=c.execute('select document from current_session').fetchone()[0];head=json.loads(rawhead);raw_events=[r[0] for r in c.execute('select document from events order by revision')];events=[json.loads(raw) for raw in raw_events];binding=json.loads(c.execute('select document from binding').fetchone()[0]);c.close()
  closed=row['predecessor'];assert json.loads((root/'original-closed.json').read_text())==closed==events[4] and head==row['head'] and len(events)==head['revision'] and events[-1]==head
  assert [h['state'] for h in events[:5]]==['preparing','capture_reserved','captured','root_retirement_prepared','root_absent'];assert events[0]['ticket']==binding['originalRoot']['id']==closed['ticket']
  refs=closed['references'];vault=manager/'client-originals'/refs['attempt'];raw=(vault/'original.json').read_bytes();assert sha(raw)==refs['originalSha256'];original=json.loads(raw);assert original['schemaVersion']==5 and original['originalRootAbsence']==binding['originalRoot'];assert sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256']
  movement=closed['rootRetirement'];raw=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(raw)==movement['plan']['sha256'];plan=json.loads(raw);assert plan['source']==events[2] and plan['origin']==binding['originalRoot'];raw=(vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')).read_bytes();assert sha(raw)==closed['completionSha256']
  slots=list((manager/'client-root-generations').iterdir());assert len(slots)==1 and str(uuid.UUID(slots[0].name))==slots[0].name;slot=slots[0];intentraw=(slot/'intent.json').read_bytes();intent=json.loads(intentraw);reference=json.loads((slot/'client-root-intent.json').read_text());assert reference['id']==intent['id']==slot.name and reference['intentSha256']==sha(intentraw) and intent['originalRootAbsent'];assert intent['id']!=closed['ticket']
  if row['point']=='generation_reserved':assert not row['rootExists'] and head==closed and not (slot/'created.json').exists()
  else:
   assert row['rootExists'];createdraw=(slot/'created.json').read_bytes();publishedraw=(slot/'published.json').read_bytes();created=json.loads(createdraw);published=json.loads(publishedraw);assert created['intentSha256']==sha(intentraw) and created['rootEmpty'] and published['intentSha256']==sha(intentraw) and published['createdSha256']==sha(createdraw) and published['rootIdentity']==created['rootIdentity']
   first=json.loads((manager/'client-root-reservation/created.json').read_text());assert first['rootIdentity']!=created['rootIdentity']
   if row['point']=='generation_committed':
    assert head['state']=='preparing' and head['revision']==6 and head['ticket']==slot.name;g=head['rootGeneration'];assert g['preparingRevision']==6 and g['predecessorRevision']==5 and g['predecessorSha256']==sha(raw_events[4]);new=json.loads(bytes(g['binding']));oldscope=json.loads(bytes(binding['scope']));newscope=json.loads(bytes(new['scope']));assert oldscope[:8]==newscope[:8] and oldscope[8]!=newscope[8]==created['rootIdentity'];assert new['originalRoot']['generationSlot']==new['originalRoot']['id']==slot.name;assert new['originalRoot']['createdSha256']==sha(createdraw) and new['originalRoot']['publishedSha256']==sha(publishedraw) and new['originalRoot']['intentSha256']==sha(intentraw)
   else:assert head==closed and head.get('rootGeneration') is None
  if row['point'] in ['generation_event_inserted','generation_head_updated']:assert (manager/'client-session-index.sqlite-journal').exists(),'actual in-transaction process failure evidence must retain its rollback sidecar'
 verified.append({'point':row['point'],'allRawBytesAndIndependentPredecessorVerified':True,'headState':head['state'],'revision':head['revision'],'newGenerationAtomicallyPresent':row['point']=='generation_committed','noStartOrRecapturedCustody':True})
positive=next(json.loads(line.split('SUCCESSOR_ROOT_EVIDENCE ',1)[1]) for line in log.splitlines() if 'SUCCESSOR_ROOT_EVIDENCE ' in line);assert positive['oldVaultBytesUnchanged'] and positive['originalBindingUnchanged'] and positive['noCaptureOrServicePermission']
print(json.dumps({'accepted':False,'verified':verified,'positivePriorEvidenceByteEqualityAsserted':True,'scope':'offline exact byte supplement to real filesystem/SQLite/process-exit tests; no OS service or repeated Start acceptance'},indent=2))
