import json,hashlib,copy
from pathlib import Path
w=Path.cwd();b=w/'.toolchain/post021-msvc-followup';target=w/'.toolchain/post021-windows-final-msvc-41913003716/target'
assert not target.exists(),'fresh target required'
source=w/'.toolchain/post021-windows-final/model-regression/sandbox/manager-state'
library=json.loads((source/'model-family-templates.json').read_bytes());library['templates']=[t for t in library['templates']if t['source']=='builtin'];library['submissionReceipts']=[]
assert len(library['templates'])==17
local_id='7e89780b-058f-4d51-91ce-925376425431';node_id='7e89780b-058f-4d51-91ce-925376425432'
row=copy.deepcopy(json.loads((source/'model-channel-layout.json').read_bytes())['channels'][1]);row['id']=local_id;row['label']='Local scope fixture';row['connection'].update(providerId='local-fixture',nodes=[{'id':node_id,'name':'Disabled local fixture','url':'http://127.0.0.1:60443/v1'}],activeNodeId=node_id,credentialRef=None,enabled=False);row['modelOrder']=[];row['discovered']=[]
layout={'schemaVersion':1,'templateRefs':[],'channels':[row],'defaultChannelId':local_id}
config={'defaultProvider':'local-fixture','providers':{'local-fixture':{'name':'Local scope fixture','adapter':'openai-chat','authMode':'key','baseUrl':'http://127.0.0.1:60443/v1','models':[],'selectedModels':[],'disabled':True}},'codexAutoStart':False,'oauthOpenBrowser':False,'syncOnStart':False,'autoUpdate':False,'claudeCodeEnabled':False,'clientIntegrations':{'codex':False,'grok':False,'claude':False}}
fixtures=json.loads((b/'cloud-fixtures-reviewed/fixture-manifest.json').read_bytes());initial=fixtures['stages']['initial']
expected={'channelIds':[local_id,*initial['channelIds']],'templateIds':[t['id']for t in library['templates']]+initial['templateIds']}
sandbox=target/'sandbox';state=sandbox/'manager-state';state.mkdir(parents=True);(sandbox/'opencodex-home').mkdir()
for p,v in [(state/'model-family-templates.json',library),(state/'model-channel-layout.json',layout),(sandbox/'opencodex-home/config.json',config),(sandbox/'expected-first-scope.json',expected)]:p.write_text(json.dumps(v,ensure_ascii=False,separators=(',',':'))+chr(10),encoding='utf-8',newline=chr(10))
for name in ['config-sync-scope.json','config-sync-initialized.json']:assert not(state/name).exists()
manifest={'status':'prepared-not-executed','source':'Known synthetic native library export; builtin entries only, preseeded before first UI', 'localChannelIds':[local_id],'localTemplateIds':[t['id']for t in library['templates']], 'initialCloudChannelIds':initial['channelIds'],'initialCloudTemplateIds':initial['templateIds'],'expected':expected,'inputDigests':{str(p.relative_to(target)).replace(chr(92),'/'):hashlib.sha256(p.read_bytes()).hexdigest()for p in [state/'model-family-templates.json',state/'model-channel-layout.json',sandbox/'opencodex-home/config.json',sandbox/'expected-first-scope.json']}}
(b/'first-scope-input-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8',newline=chr(10));print('Prepared local1+cloud1 channels, local17+cloud1 templates. No app launch, scope write or sync.')
