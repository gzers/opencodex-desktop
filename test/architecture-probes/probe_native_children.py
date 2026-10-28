#!/usr/bin/env python3
"""Synthetic model responses drive real Codex spawn_agent; records only fixture metadata."""
import argparse,importlib.util,json,tempfile,threading,time,hashlib
from pathlib import Path
from datetime import datetime,timezone


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--codex',required=True);p.add_argument('--native-helper',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 spec=importlib.util.spec_from_file_location('native_helper',a.native_helper);h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
 report={'atUtc':datetime.now(timezone.utc).isoformat(),'checks':[],'realModelCalls':0,'status':'failed','requests':[],'tools':[],'toolSurface':[]}
 roots={};lock=threading.Lock();limited_release=threading.Event()
 class Stub(h.LocalResponses):
  def do_POST(self):
   body=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
   tool_rows=body.get('tools',[])
   flat=[]
   for t in tool_rows:
    if t.get('type')=='namespace':flat.extend(t.get('tools',[]))
    else:flat.append(t)
   tools={t.get('name'):t for t in flat if t.get('type') in ['function','custom']}
   if not report['toolSurface']:report['toolSurface']=[{'type':t.get('type'),'name':t.get('name'),'children':[c.get('name') for c in t.get('tools',[])]} for t in tool_rows]
   tid=self.headers.get('thread-id') or self.headers.get('session-id') or 'unknown'
   child=self.headers.get('x-openai-subagent')=='collab_spawn'
   report.setdefault('toolOutputs',[]).extend(x.get('output') for x in body.get('input',[]) if isinstance(x,dict) and x.get('type')=='function_call_output')
   serialized=json.dumps(body.get('input',[]));marker=next((m for m in ['E','A','B','C','D'] if 'PROBE_ROOT_'+m in serialized),'unknown')
   with lock:
    report['requests'].append({'threadId':tid,'parent':self.headers.get('x-codex-parent-thread-id'),'sessionId':self.headers.get('session-id'),'subagent':self.headers.get('x-openai-subagent'),'model':body.get('model'),'effort':(body.get('reasoning') or {}).get('effort'),'rootMarker':marker,'instructionMarkerPresent':'NATIVE_CHILD_POLICY_INSTRUCTION_FIXTURE' in json.dumps(body)})
    if not report['tools']:report['tools']=[{'name':n,'parameters':t.get('parameters')} for n,t in tools.items() if n in ['spawn_agent','wait','wait_agent','send_input']]
    stage=roots.get((tid,marker),0);roots[(tid,marker)]=stage+1;n=len(report['requests'])
   if child and 'CHILD_D_' in serialized:limited_release.wait(timeout=5)
   output=[]
   if not child and marker!='unknown' and stage==0 and 'spawn_agent' in tools:
    schema=tools['spawn_agent'].get('parameters',{}).get('properties',{})
    for i in range(2):
     args={'message':'Return a short CHILD_'+marker+'_'+str(i)+' acknowledgement.'}
     if marker=='A' and i==1:
      if 'model' in schema:args['model']='gpt-6-sol'
      if 'reasoning_effort' in schema:args['reasoning_effort']='low'
     output.append({'id':f'fc_{n}_{i}','type':'function_call','status':'completed','call_id':f'call_{n}_{i}','namespace':'multi_agent_v1','name':'spawn_agent','arguments':json.dumps(args)})
   else:
    output=[{'id':f'msg_{n}','type':'message','role':'assistant','status':'completed','content':[{'type':'output_text','text':'Synthetic child acknowledgement.' if child else 'Synthetic parent done.','annotations':[]}]}]
   response={'id':f'resp_{n}','object':'response','status':'completed','output':output,'usage':{'input_tokens':1,'output_tokens':1,'total_tokens':2}}
   events=[{'type':'response.created','response':{**response,'status':'in_progress','output':[]}}]
   for i,item in enumerate(output):
    events.append({'type':'response.output_item.added','output_index':i,'item':{**item,'status':'in_progress',**({'arguments':''} if item['type']=='function_call' else {'content':[]})}})
    if item['type']=='function_call':events.append({'type':'response.function_call_arguments.delta','item_id':item['id'],'output_index':i,'delta':item['arguments']})
    else:events.append({'type':'response.output_text.delta','item_id':item['id'],'output_index':i,'content_index':0,'delta':item['content'][0]['text']})
    events.append({'type':'response.output_item.done','output_index':i,'item':item})
   events.append({'type':'response.completed','response':response})
   raw=''.join('event: '+e['type']+'\ndata: '+json.dumps(e)+'\n\n' for e in events).encode()
   self.send_response(200);self.send_header('Content-Type','text/event-stream');self.send_header('Content-Length',str(len(raw)));self.end_headers();self.wfile.write(raw)
 with tempfile.TemporaryDirectory(prefix='native-child-probe-') as tmp:
  root=Path(tmp);http=h.ThreadingHTTPServer(('127.0.0.1',0),Stub);http.requests=[]
  worker=threading.Thread(target=http.serve_forever,daemon=True);worker.start();server=None
  try:
   server=h.AppServer(a.codex,root,http.server_port)
   # One process; each new thread receives its own supported config override.
   server.call('initialize',{'clientInfo':{'name':'child_scope_probe','version':'0.1'},'capabilities':{'experimentalApi':True}});server.send({'method':'initialized','params':{}})
   projects=[];threads=[]
   for label,model,effort in [('A','gpt-6.1-sol','high'),('B','gpt-6-sol','low'),('C','gpt-6-astra','high')]:
    project=projects[0] if label=='C' else server.call('project/create',{'idempotencyKey':'probe-'+label,'name':'Probe '+label,'roots':[{'path':str(server.workspace)}]})['project']
    if label!='C':projects.append(project)
    start=server.call('thread/start',{'projectId':project['id'],'cwd':str(server.workspace),'developerInstructions':'NATIVE_CHILD_POLICY_INSTRUCTION_FIXTURE: use only synthetic fixture instructions.','config':{'features.multi_agent':True,'features.code_mode':False,'agents.default_subagent_model':model,'agents.default_subagent_reasoning_effort':effort}})
    threads.append((label,start['thread']['id']))
   # Both turns are started before awaiting either terminal event.
   wanted={}
   for label,tid in threads:
    result=server.call('turn/start',{'threadId':tid,'input':[{'type':'text','text':'PROBE_ROOT_'+label+': use the provided synthetic agent task instructions.'}]});wanted[tid]=result['turn']['id']
   deadline=time.monotonic()+15;done=set();events=[]
   while time.monotonic()<deadline:
    for message in server.notifications:
     params=message.get('params',{})
     if message.get('method')=='turn/completed' and wanted.get(params.get('threadId'))==params.get('turn',{}).get('id'):done.add(params['threadId'])
    if len(done)==3 and len([x for x in report['requests'] if x['subagent']=='collab_spawn'])>=6:break
    try:server.notifications.append(server.receive(min(deadline,time.monotonic()+2)))
    except TimeoutError:continue
   children=[x for x in report['requests'] if x['subagent']=='collab_spawn']
   report['threadIds']=dict(threads);report['turnsCompleted']=len(done);report['childRequests']=len(children)
   report['rpcErrors']=[m.get('params',{}) for m in server.notifications if m.get('method')=='error']
   report['agentEventKinds']=sorted({m.get('method') for m in server.notifications if 'agent' in m.get('method','').lower()})
   if not report['tools']:report['outcome']='unsupported_tool_surface'
   elif len(children)<6:report['outcome']='inconclusive_spawn_not_observed'
   else:
    a_children=[x for x in children if x['parent']==threads[0][1] or x['sessionId']==threads[0][1]]
    b_children=[x for x in children if x['parent']==threads[1][1] or x['sessionId']==threads[1][1]]
    c_children=[x for x in children if x['parent']==threads[2][1] or x['sessionId']==threads[2][1]]
    assert len(a_children)==2 and len(b_children)==2 and len(c_children)==2,(a_children,b_children,c_children)
    assert all(x['model']=='gpt-6-astra' and x['effort']=='high' for x in c_children),c_children
    report['checks'].append({'id':'same_project_thread_override','status':'passed','observed':'A and C share projectId but keep distinct per-thread child config.'})
    assert any(x['model']=='gpt-6.1-sol' and x['effort']=='high' for x in a_children),a_children
    assert all(x['model']=='gpt-6-sol' and x['effort']=='low' for x in b_children),b_children
    report['checks'].append({'id':'native_child_defaults_per_loaded_thread','status':'passed','observed':'Three concurrent parent threads receive different child defaults without global config mutation.'})
    assert len({x['threadId'] for x in children})==6,children
    report['checks'].append({'id':'siblings_preserve_distinct_identity','status':'passed'})
    report['instructionProjection']={'status':'passed' if all(x['instructionMarkerPresent'] for x in children) else 'not_inherited','childRequestsWithMarker':sum(x['instructionMarkerPresent'] for x in children),'tested':len(children),'scope':'thread/start developerInstructions inheritance, not a separate subagent-only instruction API'}
    if any('model' in (t.get('parameters',{}).get('properties',{})) for t in report['tools'] if t['name']=='spawn_agent'):
     assert any(x['model']=='gpt-6-sol' and x['effort']=='low' for x in a_children),a_children
     report['checks'].append({'id':'explicit_spawn_override','status':'passed'})
    # Resume the same already-loaded parent with new child defaults, then observe real spawns.
    resume_target=threads[0][1]
    before_resume={x['threadId'] for x in children if x['parent']==resume_target}
    resumed=server.call('thread/resume',{'threadId':resume_target,'config':{'agents.default_subagent_model':'gpt-6-astra','agents.default_subagent_reasoning_effort':'xhigh'}})
    assert resumed['thread']['id']==resume_target
    server.call('turn/start',{'threadId':resume_target,'input':[{'type':'text','text':'PROBE_ROOT_E: spawn two new synthetic children after same-owner loaded resume.'}]})
    deadline=time.monotonic()+8;after_resume=[]
    while time.monotonic()<deadline:
     after_resume=[x for x in report['requests'] if x['subagent']=='collab_spawn' and x['parent']==resume_target and x['threadId'] not in before_resume]
     if len(after_resume)>=2:break
     try:server.notifications.append(server.receive(min(deadline,time.monotonic()+1)))
     except TimeoutError:continue
    assert len(after_resume)==2,after_resume
    loaded_pairs={(x['model'],x['effort']) for x in after_resume}
    report['loadedResumeChildDefaults']={'accepted':True,'expectedNew':{'model':'gpt-6-astra','effort':'xhigh'},'actual':after_resume,'status':'passed' if loaded_pairs=={('gpt-6-astra','xhigh')} else 'unsupported_on_loaded_resume','scope':'same existing loaded parent, thread/resume config, no global mutation'}
    assert loaded_pairs in [{('gpt-6-astra','xhigh')},{('gpt-6.1-sol','high')}],loaded_pairs
    report['checks'].append({'id':'loaded_resume_child_default_semantics','status':'passed','observed':report['loadedResumeChildDefaults']})
    # A projectId on its own does not load manager-specific project policies.
    server.call('config/value/write',{'keyPath':'agents.default_subagent_model','value':'gpt-6-luna','mergeStrategy':'replace'})
    server.call('config/value/write',{'keyPath':'agents.default_subagent_reasoning_effort','value':'low','mergeStrategy':'replace'})
    fresh=server.call('thread/start',{'cwd':str(server.workspace),'projectId':projects[0]['id'],'config':{'features.multi_agent':True,'features.code_mode':False}})
    fresh_id=fresh['thread']['id'];server.call('turn/start',{'threadId':fresh_id,'input':[{'type':'text','text':'PROBE_ROOT_C: test new thread global defaults after clearing local projection.'}]})
    deadline=time.monotonic()+8
    while time.monotonic()<deadline:
     fresh_children=[x for x in report['requests'] if x['subagent']=='collab_spawn' and x['parent']==fresh_id]
     if len(fresh_children)>=2:break
     try:server.notifications.append(server.receive(min(deadline,time.monotonic()+1)))
     except TimeoutError:continue
    assert len(fresh_children)==2 and all(x['model']=='gpt-6-luna' and x['effort']=='low' for x in fresh_children),fresh_children
    report['checks'].append({'id':'new_thread_without_projection_uses_current_global','status':'passed','observed':'New thread in project A, without local config projection, creates children from current global defaults.'})
    limited=server.call('thread/start',{'cwd':str(server.workspace),'config':{'features.multi_agent':True,'features.code_mode':False,'agents.max_threads':1,'agents.default_subagent_model':'gpt-6-luna'}})
    limited_id=limited['thread']['id'];output_before=len(report.get('toolOutputs',[]))
    server.call('turn/start',{'threadId':limited_id,'input':[{'type':'text','text':'PROBE_ROOT_D: two synthetic child calls test the configured limit.'}]})
    deadline=time.monotonic()+8;limit_outputs=[]
    while time.monotonic()<deadline:
     limit_outputs=report.get('toolOutputs',[])[output_before:]
     if any('limit' in str(x).lower() for x in limit_outputs) and any(x['subagent']=='collab_spawn' and x['parent']==limited_id for x in report['requests']):break
     try:server.notifications.append(server.receive(min(deadline,time.monotonic()+1)))
     except TimeoutError:continue
    limited_children=[x for x in report['requests'] if x['subagent']=='collab_spawn' and x['parent']==limited_id]
    limited_release.set()
    assert len(limited_children)==1 and any('limit' in str(x).lower() for x in limit_outputs),{'children':limited_children,'outputs':limit_outputs}
    report['checks'].append({'id':'native_child_concurrency_limit','status':'passed','observed':{'configuredMaxThreads':1,'actualChildRequests':len(limited_children),'limitErrorObserved':True,'scope':'this parent tree, not a cross-project global scheduler'}})
    report['childRequests']=len([x for x in report['requests'] if x['subagent']=='collab_spawn'])
    report['outcome']='passed';report['status']='passed'
  except Exception as error:report['error']=str(error)
  finally:
   limited_release.set()
   if server:server.close()
   http.shutdown();http.server_close();worker.join(timeout=2)
 report['scriptSha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps(report,ensure_ascii=False,indent=2));return 0 if report['status']=='passed' else 1
if __name__=='__main__':raise SystemExit(main())
