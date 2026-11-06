from pathlib import Path
import hashlib,json,subprocess
from jsonschema import Draft7Validator

out=Path(__file__).parent
producer=Path('/Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97')
root=Path('/Users/ezio/.codex/worktrees/ocxd-023-ui/ocx-0219-fixes')
hashes={
 'L1-condition-trace-contract.json':'4b3e026a3c37b9e7f82c071181921cf57ef35049e9a4c9b8d9051c1f18b32395',
 'condition-preview.trace.schema.json':'8a12967cc1ed3d52e328796952227c1f399b7aacee8585370607e71480a089a4',
 'condition-preview.error.schema.json':'6e8a71b4c19e5e78009d256d2e4872ba17c32249793a756d823414ff4d7e1fe2',
 'condition-preview.real-cases.json':'974c837759b7c932456b2a9288b51b9af08ac27df3439da8dde7518da7bee01c'}
for name,expected in hashes.items():assert hashlib.sha256((producer/name).read_bytes()).hexdigest()==expected,name
contract=json.loads((producer/'L1-condition-trace-contract.json').read_text())
for name,expected in contract['producer_source_sha256'].items():
 assert hashlib.sha256((root/name).read_bytes()).hexdigest()==expected,name
 assert subprocess.check_output(['git','show',contract['producer_commit']+':'+name],cwd=root)==(root/name).read_bytes(),name
success=Draft7Validator(json.loads((producer/'condition-preview.trace.schema.json').read_text()))
error=Draft7Validator(json.loads((producer/'condition-preview.error.schema.json').read_text()))
cases=json.loads((producer/'condition-preview.real-cases.json').read_text())
for case in cases:
 if case['error']:error.validate(case['error'])
 else:success.validate(case['result'])
receipts=[json.loads(line) for line in (out/'native-receipts.ndjson').read_text().splitlines()]
counts={'success':0,'error':0,'validate':0}
for receipt in receipts:
 result=receipt['result']
 if not result['ok']:error.validate(result['error']);counts['error']+=1
 elif receipt['input']['command']=='model_v2_preview_conditions':
  success.validate(result['value']);counts['success']+=1
  assert [r['rawModelId'] for r in result['value']['rows']]==receipt['input']['args']['request']['rawModelIds']
 else:assert result['value'] is None;counts['validate']+=1
report={'status':'passed','artifactSha256':hashes,'producerCommit':contract['producer_commit'],'producerSourceSha256':contract['producer_source_sha256'],'frozenCases':len(cases),'frozenSuccessRows':sum(len((c['result'] or {}).get('rows',[])) for c in cases),'nativeReceiptsValidated':counts,'nativeRawIdsUnchanged':True,'validation':'Draft7 success/error wire schemas from producer, applied independently to every native receipt, including preserved failed harness attempts. No renderer-computed matching results.'}
(out/'evidence-validation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+chr(10))
print(json.dumps(report,ensure_ascii=False,indent=2))
