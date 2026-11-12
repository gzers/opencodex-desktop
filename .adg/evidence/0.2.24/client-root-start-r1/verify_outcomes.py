from pathlib import Path
import hashlib,json,uuid
root=Path(__file__).resolve().parent
log=(root/'product-start-delivery.log').read_text()
rows=[json.loads(line.split('ROOT_START_PREPARATION_EVIDENCE ',1)[1]) for line in log.splitlines() if 'ROOT_START_PREPARATION_EVIDENCE ' in line]
assert len(rows)==2 and {v['closedGenerations'] for v in rows}=={1,3}
def ordered_head(head):
    refs=head['references']
    refs=None if refs is None else {k:refs[k] for k in ['attempt','originalSha256','scopeSha256','phases','liveSha256']}
    out={'revision':head['revision'],'ticket':head['ticket'],'state':head['state'],'references':refs,
         'restorePlan':head['restorePlan'],'completionSha256':head['completionSha256']}
    if 'rootRetirement' in head:
        movement=head['rootRetirement'];out['rootRetirement']={'sourceRevision':movement['sourceRevision'],'plan':{k:movement['plan'][k] for k in ['id','sha256']}}
    return json.dumps(out,ensure_ascii=False,separators=(',',':')).encode()
verified=[]
for row in rows:
    assert row['accepted'] is False and row['readonlyBytesUnchanged'] and row['pendingRestartRefused'] and row['noOfficialService']
    closed=row['closed'];preparing=row['preparing'];captured=row['captured'];generation=preparing['rootGeneration']
    assert closed['revision']==row['closedGenerations']*5 and closed['state']=='root_absent'
    assert preparing['state']=='preparing' and preparing['revision']==closed['revision']+1 and preparing['references'] is None
    assert generation['predecessorRevision']==closed['revision'] and generation['preparingRevision']==preparing['revision']
    assert generation['predecessorSha256']==hashlib.sha256(ordered_head(closed)).hexdigest()
    active=json.loads(bytes(generation['binding']));origin=active['originalRoot'];scope=json.loads(bytes(active['scope']))
    assert origin['id']==origin['generationSlot']==preparing['ticket']==captured['ticket']
    assert str(uuid.UUID(origin['id']))==origin['id'] and origin['id']!=closed['ticket']
    assert len(scope)==9 and scope[7].endswith('/.codex') and 'namespace' not in origin
    assert captured['revision']==preparing['revision']+2 and captured['state']=='captured' and 'rootGeneration' not in captured
    assert captured['references']['attempt']==origin['id'] and captured['references']['phases']==[] and captured['references']['liveSha256'] is None
    verified.append({'closedGenerations':row['closedGenerations'],'newPreparingRevision':preparing['revision'],
                     'capturedRevision':captured['revision'],'schemaCanonicalPredecessorShaVerified':True,
                     'loggedNewOriginAndScopeVerified':True})
external=[json.loads(line.split('ROOT_START_EXTERNAL_INDEX_EVIDENCE ',1)[1]) for line in log.splitlines() if 'ROOT_START_EXTERNAL_INDEX_EVIDENCE ' in line]
assert len(external)==1 and external[0]['accepted'] is False
assert all(external[0][k] for k in ['externalWriteCompleted','sameInodeRetained','observationRefused','noAuxiliaryParentSqliteFd'])
print(json.dumps({'accepted':False,'verifiedPreparations':verified,'externalSameInodeWriteRefused':True,
                  'scope':'schema-canonical corroboration of logged preparation tests, not a raw-disk replay or official service acceptance'},indent=2))
