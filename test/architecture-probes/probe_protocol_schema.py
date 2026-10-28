#!/usr/bin/env python3
"""Generate this binary's protocol and parse isolated context settings; no model execution."""
import argparse,json,subprocess,tempfile,hashlib
from pathlib import Path
from datetime import datetime,timezone
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--codex',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
report={'atUtc':datetime.now(timezone.utc).isoformat(),'codexVersion':subprocess.check_output([a.codex,'--version'],text=True).strip(),'checks':[],'schemas':{},'featureCases':[],'realModelCalls':0}
with tempfile.TemporaryDirectory(prefix='ocx-schema-') as tmp:
 root=Path(tmp);home=root/'home';ch=home/'.codex';ch.mkdir(parents=True)
 env={'PATH':'/usr/bin:/bin','HOME':str(home),'CODEX_HOME':str(ch),'TMPDIR':str(root),'HTTP_PROXY':'http://127.0.0.1:9','HTTPS_PROXY':'http://127.0.0.1:9','NO_PROXY':'127.0.0.1'}
 r=subprocess.run([a.codex,'app-server','generate-json-schema','--experimental','--out',str(root/'schemas')],env=env,capture_output=True,text=True,timeout=12);assert r.returncode==0,r.stderr
 names=['ThreadStartParams','ThreadResumeParams','ThreadSettingsUpdateParams','TurnStartParams','ProjectListParams','ThreadListParams']
 for f in (root/'schemas').rglob('*.json'):
  if f.stem in names:
   data=json.loads(f.read_text());props=data.get('properties',{})
   report['schemas'][f.stem]={'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'fields':list(props),'config':props.get('config')}
 assert 'config' in report['schemas']['ThreadStartParams']['fields'] and 'config' in report['schemas']['ThreadResumeParams']['fields']
 report['checks'].append({'id':'start_resume_support_config','status':'passed'})
 assert 'config' not in report['schemas']['ThreadSettingsUpdateParams']['fields'] and 'config' not in report['schemas']['TurnStartParams']['fields']
 report['checks'].append({'id':'loaded_settings_have_no_generic_config','status':'passed'})
 assert 'projectId' in report['schemas']['ThreadStartParams']['fields'] and 'projectId' in report['schemas']['ThreadListParams']['fields']
 report['checks'].append({'id':'native_project_identity_in_protocol','status':'passed'})
 base='''model="gpt-6.1-sol"
model_provider="fixture"
check_for_update_on_startup=false
[model_providers.fixture]
name="context fixture"
base_url="http://127.0.0.1:9/v1"
wire_api="responses"
requires_openai_auth=false
supports_websockets=false
'''
 cases=[('default','',True),('background','[features.async_compaction]\nenabled=true\nmodel_async_compact_threshold_percent=65\n',True),('history_notes','[features.context_management]\nexperimental_mode=true\n[features.token_budget]\nenabled=true\nuse_history_notes_extension=true\n',True),('invalid_background_type','[features]\nasync_compaction="wrong"\n',False),('invalid_remote_enum','[model_providers.fixture.capabilities]\nremote_compaction="fixture_invalid"\n',False),('remote_v2','[model_providers.fixture.capabilities]\nremote_compaction="v2"\n',True)]
 for label,extra,valid in cases:
  (ch/'config.toml').write_text(base+extra)
  r=subprocess.run([a.codex,'features','list'],env=env,cwd=root,capture_output=True,text=True,timeout=10)
  selected=[line for line in r.stdout.splitlines() if line.split() and line.split()[0] in ['async_compaction','context_management','token_budget']]
  ok=(r.returncode==0)==valid
  if valid:assert len(selected)==3,{'case':label,'featureOutput':r.stdout[-2000:]}
  if label=='background':assert any(line.split()[0]=='async_compaction' and line.split()[-1]=='true' for line in selected),selected
  if label=='history_notes':assert all(any(line.split()[0]==key and line.split()[-1]=='true' for line in selected) for key in ['context_management','token_budget']),selected
  report['featureCases'].append({'case':label,'expectedAccepted':valid,'exit':r.returncode,'featureStates':selected,'status':'passed' if ok else 'failed','error':r.stderr[-1000:] if r.returncode else None})
  assert ok,report['featureCases'][-1]
  report['checks'].append({'id':'parse_'+label,'status':'passed'})
report['status']='passed';report['scriptSha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'status':report['status'],'checks':report['checks'],'featureCases':report['featureCases']},ensure_ascii=False,indent=2))
