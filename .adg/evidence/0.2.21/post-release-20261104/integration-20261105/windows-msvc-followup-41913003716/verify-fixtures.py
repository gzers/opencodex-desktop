import json,base64,hashlib
from pathlib import Path
b=Path('.toolchain/post021-msvc-followup');root=b/'cloud-fixtures-reviewed'
hash_bytes=lambda raw:'sha256:'+hashlib.sha256(raw).hexdigest()
serial=lambda value:json.dumps(value,ensure_ascii=False,separators=(',',':')).encode()
manifest=json.loads((root/'fixture-manifest.json').read_bytes());documents={};checks=[]
for name,stage in manifest['stages'].items():
 directory=root/name;pointer=json.loads((directory/'config-v1-latest.json').read_bytes());raw=(directory/stage['filename']).read_bytes();envelope=json.loads(raw)
 assert hash_bytes(raw)==pointer['digest']==stage['digest']
 document_bytes=base64.b64decode(envelope['payload']['document_b64']);assert hash_bytes(document_bytes)==envelope['manifest']['documentDigest']
 doc=json.loads(document_bytes);assert hash_bytes(serial(doc['models']['payload']))==doc['models']['digest'];documents[name]=doc
 assert envelope['manifest']['secretObjects']==0 and envelope['manifest']['sections']==['models']
 assert doc['channelKeys']=={} and not doc.get('channelAuthorizations') and doc['webdavPasswords']=={} and doc['contentVersions']==[]
 assert doc['manager']=={'preferences':None,'extensions':None,'webdav':[],'accounts':[]} and doc['assets']=={'skills':[],'mcps':[]}
 rows=doc['models']['payload']['policy']['channels'];templates=doc['models']['payload']['templates']
 assert [r['id']for r in rows]==stage['channelIds'] and [t['id']for t in templates]==stage['templateIds']
 for r in rows:assert r['connection']['provider']['disabled'] and r['connection']['provider']['selectedModels']==[] and r['connection']['provider']['baseUrl']=='https://cloud-fixture.invalid/v1'
 etags=json.loads((directory/'etags.json').read_bytes())
 for filename,etag in etags.items():assert etag=='"'+hashlib.sha256((directory/filename).read_bytes()).hexdigest()+'"'
 checks.append({'stage':name,'channels':len(rows),'templates':len(templates),'digests':'matched independently with Python','credentials':0,'mutations':0})
assert documents['initial']['models']['payload']['policy']['channels'][0]==documents['later']['models']['payload']['policy']['channels'][0]
assert documents['initial']['models']['payload']['templates'][0]==documents['later']['models']['payload']['templates'][0]
expected=json.loads((b/'first-scope-input-manifest.json').read_bytes())['expected'];assert len(expected['channelIds'])==2 and len(expected['templateIds'])==18
for key in ['channelIds','templateIds']:assert len(set(expected[key]))==len(expected[key]) and set(manifest['stages']['initial'][key])<=set(expected[key]) and not set(manifest['stages']['later'][key][1:])&set(expected[key])
report={'result':'fixture-preflight-passed','acceptance':False,'nativeExecuted':False,'checks':checks,'exactInitialExpectation':{'channels':2,'templates':18},'laterObjectsExcludedFromInitialExpectation':True,'initialCloudObjectsUnchangedInLaterStage':True}
(b/'fixture-preflight.json').write_text(json.dumps(report,indent=2)+chr(10),encoding='utf-8',newline=chr(10));print(json.dumps(report))
