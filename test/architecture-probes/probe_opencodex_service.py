#!/usr/bin/env python3
"""Original ocx CLI + temporary roots + real plugin loader + bounded loopback upstream."""
import argparse,hashlib,json,os,signal,socket,subprocess,tempfile,threading,time,urllib.request,urllib.error,importlib.util
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from datetime import datetime,timezone

def free_port():
    s=socket.socket();s.bind(('127.0.0.1',0));port=s.getsockname()[1];s.close();return port

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--package',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--codex');p.add_argument('--native-helper');args=p.parse_args()
    r=args.package.resolve();report={'atUtc':datetime.now(timezone.utc).isoformat(),'opencodexVersion':json.loads((r/'package.json').read_text())['version'],'checks':[],'realModelCalls':0,'dailyConfigModified':False}
    def record(id,detail,status='passed'):report['checks'].append({'id':id,'status':status,'observed':detail})
    class Upstream(BaseHTTPRequestHandler):
        def log_message(self,*a):pass
        def do_CONNECT(self):
            self.server.blocked.append('CONNECT');self.send_error(502)
        def do_GET(self):
            if self.path.endswith('/models'):
                raw=json.dumps({'object':'list','data':[{'id':'gpt-5.4-mini','object':'model'}]}).encode();self.send_response(200);self.send_header('Content-Length',str(len(raw)));self.end_headers();self.wfile.write(raw)
            else:self.send_error(404)
        def do_POST(self):
            if self.path.startswith('http:') or self.path.startswith('https:'):
                self.server.blocked.append('proxy-absolute');self.send_error(502);return
            body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))))
            self.server.requests.append({'path':self.path,'model':body.get('model'),'effort':(body.get('reasoning') or {}).get('effort'),'plugin':self.headers.get('x-probe-plugin'),'threadIdPresent':bool(self.headers.get('thread-id')),'subagentPresent':bool(self.headers.get('x-openai-subagent')),'compactionTriggerPresent':any(x.get('type')=='compaction_trigger' for x in body.get('input',[]) if isinstance(x,dict)),'hasSummaryInstruction':'checkpoint' in json.dumps(body).lower()})
            n=len(self.server.requests);text='Isolated proxy reply.'
            item={'id':f'msg_probe_{n}','type':'message','role':'assistant','status':'completed','content':[{'type':'output_text','text':text,'annotations':[]}]}
            tokens=70000 if 'BACKGROUND_' in json.dumps(body.get('input',[])) and not self.server.requests[-1]['hasSummaryInstruction'] else 1
            background_fixture='BACKGROUND_PROBE' in json.dumps(body.get('input',[])) and self.server.requests[-1]['hasSummaryInstruction']
            if background_fixture:
                self.server.background_started.set();self.server.background_release.wait(timeout=6);self.server.background_finished.set()
            response={'id':f'resp_probe_{n}','object':'response','status':'completed','output':[item],'usage':{'input_tokens':tokens,'output_tokens':1,'total_tokens':tokens+1}}
            if 'CANCEL_PROBE' in json.dumps(body.get('input',[])):
                self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
                try:
                    self.wfile.write(('event: response.created\ndata: '+json.dumps({'type':'response.created','response':{**response,'status':'in_progress','output':[]}})+'\n\n').encode());self.wfile.flush()
                    self.server.cancel_started.set()
                    deadline=time.monotonic()+5
                    while time.monotonic()<deadline:
                        self.wfile.write(b': fixture keepalive\n\n');self.wfile.flush();time.sleep(.04)
                except OSError:self.server.cancel_observed.set()
                return
            if body.get('stream'):
                events=[{'type':'response.created','response':{**response,'status':'in_progress','output':[]}},{'type':'response.output_item.added','output_index':0,'item':{**item,'status':'in_progress','content':[]}},{'type':'response.output_text.delta','item_id':item['id'],'output_index':0,'content_index':0,'delta':text},{'type':'response.output_item.done','output_index':0,'item':item},{'type':'response.completed','response':response}]
                raw=''.join('event: '+e['type']+'\ndata: '+json.dumps(e)+'\n\n' for e in events).encode();mime='text/event-stream'
            else:raw=json.dumps(response).encode();mime='application/json'
            self.send_response(200);self.send_header('Content-Type',mime);self.send_header('Content-Length',str(len(raw)));self.end_headers();self.wfile.write(raw)
    with tempfile.TemporaryDirectory(prefix='ocx-service-',dir='/tmp') as tmp:
        root=Path(tmp);home=root/'home';ocx=home/'.opencodex';codex=home/'.codex';plugins=ocx/'plugins'
        for x in [plugins,codex,home/'.claude']:x.mkdir(parents=True,exist_ok=True);x.chmod(0o700)
        upstream=ThreadingHTTPServer(('127.0.0.1',0),Upstream);upstream.requests=[];upstream.blocked=[];upstream.cancel_started=threading.Event();upstream.cancel_observed=threading.Event();upstream.background_started=threading.Event();upstream.background_release=threading.Event();upstream.background_finished=threading.Event()
        thread=threading.Thread(target=upstream.serve_forever,daemon=True);thread.start();up=upstream.server_port;port=free_port()
        cfg={'port':port,'hostname':'127.0.0.1','runtimeRole':'hub','clientIntegrations':{'codex':False,'grok':False,'claude-desktop':False},'claudeCode':{'enabled':False,'systemEnv':False,'intercept':{'enabled':False}},'providers':{'probe':{'adapter':'openai-responses','baseUrl':f'http://127.0.0.1:{up}/direct/v1','apiKey':'fixture-not-a-secret','allowPrivateNetwork':True}},'defaultProvider':'probe','subagentModels':[],'subagentModelsVersion':3,'multiAgentGuidanceEnabled':False,'websockets':True,'catalogAutoRefresh':{'enabled':False},'tokenGuardian':{'enabled':False},'syncResumeHistory':False,'codexAutoStart':False,'codexShimAutoRestore':False}
        config=ocx/'config.json';config.write_text(json.dumps(cfg));config.chmod(0o600)
        plugin=plugins/'feasibility.mjs';plugin.write_text('export default {name:"feasibility-probe",setup(ctx){ctx.log("fixture-ready");ctx.registerUpstreamRewriter(target=>{const u=new URL(target.url);if(u.hostname!=="127.0.0.1"||u.port!=="'+str(up)+'"||!u.pathname.startsWith("/direct/"))return;u.pathname=u.pathname.replace("/direct/","/hooked/");target.url=u.href;target.headers.set("x-probe-plugin","loaded");});}};');plugin.chmod(0o600)
        env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(home),'CODEX_HOME':str(codex),'OPENCODEX_HOME':str(ocx),'CLAUDE_CONFIG_DIR':str(home/'.claude'),'XDG_CONFIG_HOME':str(root/'xdg'),'XDG_CACHE_HOME':str(root/'cache'),'TMPDIR':str(root),'OCX_TEST_HOME_GUARD':'1','OCX_REAL_HOME':str(Path.home()),'OPENCODEX_ADMIN_AUTH_TOKEN':'ocx_admin_'+'A'*43,'OPENCODEX_API_AUTH_TOKEN':'ocx_'+'B'*43,'OPENCODEX_CODEX_SHIM_AUTO_RESTORE':'0','HTTP_PROXY':f'http://127.0.0.1:{up}','HTTPS_PROXY':f'http://127.0.0.1:{up}','NO_PROXY':'127.0.0.1,localhost','DO_NOT_TRACK':'1'}
        bun=r/'node_modules/bun/bin/bun.exe';log=(root/'service.log').open('w+');proc=None
        def request(path,body=None,method=None,headers=None):
            data=json.dumps(body).encode() if body is not None else None
            h={'Authorization':'Bearer '+env['OPENCODEX_ADMIN_AUTH_TOKEN'] if path.startswith('/api/') else 'Bearer '+env['OPENCODEX_API_AUTH_TOKEN'],'Content-Type':'application/json',**(headers or {})}
            req=urllib.request.Request(f'http://127.0.0.1:{port}'+path,data=data,headers=h,method=method or ('POST' if data is not None else 'GET'))
            try:
                with urllib.request.urlopen(req,timeout=12) as res:return res.status,res.read().decode()
            except urllib.error.HTTPError as error:return error.code,error.read().decode()
        try:
            proc=subprocess.Popen([str(bun),str(r/'src/cli/index.ts'),'start','--port',str(port)],env=env,cwd=root,stdin=subprocess.DEVNULL,stdout=log,stderr=log,start_new_session=True)
            deadline=time.monotonic()+18;ready=None
            while proc.poll() is None and time.monotonic()<deadline:
                try:
                    code,raw=request('/readyz')
                    if code==200:ready=json.loads(raw);break
                except (OSError,ValueError):pass
                time.sleep(.1)
            if ready is None:raise RuntimeError('isolated CLI did not become ready')
            record('original_cli_ready',{'service':'opencodex','ready':True,'pidIsOwn':request('/healthz')[1].find(str(proc.pid))>=0})
            code,raw=request('/v1/responses',{'model':'probe/gpt-5.4-mini','input':[{'role':'user','content':'fixture only'}],'stream':True},headers={'thread-id':'fixture-thread-a','x-openai-subagent':'collab_spawn'})
            if code!=200:raise RuntimeError('Responses status '+str(code)+' '+raw[:1500])
            assert 'Isolated proxy reply.' in raw,raw[:1000]
            assert upstream.requests[-1]['path'].startswith('/hooked/') and upstream.requests[-1]['plugin']=='loaded',upstream.requests
            record('real_plugin_loader_and_stream',upstream.requests[-1])
            record('key_mode_identity_not_forwarded',{'threadIdPresent':upstream.requests[-1]['threadIdPresent'],'subagentPresent':upstream.requests[-1]['subagentPresent']},'passed' if not upstream.requests[-1]['threadIdPresent'] and not upstream.requests[-1]['subagentPresent'] else 'failed')
            code,raw=request('/api/sidecar-settings',{'webSearch':{'reasoning':'high'}},method='PUT')
            disk=json.loads(config.read_text());assert code==200 and disk['webSearchSidecar']['reasoning']=='high',(code,raw[:800])
            record('management_sidecar_persist',{'http':code,'reasoningInOwnConfig':disk['webSearchSidecar']['reasoning']})
            code,raw=request('/api/sidecar-settings');get=json.loads(raw)
            record('sidecar_readback_reasoning_gap',{'http':code,'reasoningPresent':'reasoning' in get['webSearch'],'modelCandidates':len(get.get('webSearchModels',[]))})
            digest=hashlib.sha256(config.read_bytes()).hexdigest()
            code,raw=request('/api/sidecar-settings',{'webSearch':{'enabled':'invalid-fixture-value'}},method='PUT')
            assert code==400 and hashlib.sha256(config.read_bytes()).hexdigest()==digest,(code,raw[:500])
            code,raw=request('/api/sidecar-settings');assert code==200
            record('invalid_save_preserves_persisted_config',{'invalidHttp':400,'configByteIdentical':True,'subsequentReadHttp':code})
            code,raw=request('/api/sidecar-settings',{'webSearch':{'backend':'openai','model':'probe/gpt-5.4-mini'}},method='PUT')
            assert code==400,(code,raw[:800])
            record('search_model_is_backend_scoped',{'http':code,'configuredKeyChannelNotEligible':True,'message':raw[:600]})
            code,raw=request('/api/shadow-call-settings',{'enabled':True,'model':'probe/gpt-5.4-mini'},method='PUT')
            assert code==200,(code,raw[:800])
            helper_models=[]
            for purpose in ['Generate a conversation title for a fixture.','Generate a commit message for a fixture.']:
                code,raw=request('/v1/responses',{'model':'gpt-6-luna','input':[{'role':'user','content':purpose}],'stream':False})
                assert code==200,(code,raw[:800]);helper_models.append(upstream.requests[-1]['model'])
            assert helper_models==['gpt-5.4-mini','gpt-5.4-mini'],helper_models
            record('shadow_call_is_not_title_specific',{'titlePurposeModel':helper_models[0],'commitPurposeModel':helper_models[1]})
            code,raw=request('/api/shadow-call-settings',{'enabled':False},method='PUT');assert code==200

            from ws_json_client import WsJson
            ws=WsJson(port=port,request_path='/v1/responses',headers={'Authorization':'Bearer '+env['OPENCODEX_API_AUTH_TOKEN']})
            try:
                ws.send({'type':'response.create','model':'probe/gpt-5.4-mini','input':[{'role':'user','content':'WS fixture only'}],'stream':True})
                deadline=time.monotonic()+8;types=[]
                while time.monotonic()<deadline:
                    event=ws.recv(deadline);types.append(event.get('type'))
                    if event.get('type') in ('response.completed','error','response.failed'):break
                assert 'response.completed' in types,types
                record('websocket_ingress_full_response',{'events':types,'upstreamHttp':True,'pluginLoadedAtHttpEgress':upstream.requests[-1]['plugin']=='loaded'})
            finally:ws.close()
            ws=WsJson(port=port,request_path='/v1/responses',headers={'Authorization':'Bearer '+env['OPENCODEX_API_AUTH_TOKEN']})
            try:
                ws.send({'type':'response.create','model':'probe/gpt-5.4-mini','input':[{'role':'user','content':'CANCEL_PROBE'}],'stream':True})
                event=ws.recv(time.monotonic()+6);assert event.get('type')=='response.created',event
                assert upstream.cancel_started.is_set()
            finally:ws.close()
            cancelled=upstream.cancel_observed.wait(timeout=3)
            assert cancelled,'WebSocket close did not close bounded fixture upstream in time'
            code,raw=request('/v1/responses',{'model':'probe/gpt-5.4-mini','input':[{'role':'user','content':'after cancel fixture'}],'stream':False})
            assert code==200 and 'Isolated proxy reply.' in raw,(code,raw[:600])
            record('websocket_cancel_reaches_upstream',{'upstreamClosed':cancelled,'newRequestHttp':code})
            assert not (codex/'config.toml').exists()
            (codex/'config.toml').write_text('[features.context_management]\nexperimental_mode=true\n')
            before=len(upstream.requests)
            code,raw=request('/v1/alpha/history/v2/list_windows',{'context':{'session_id':'fixture-thread-a'}},headers={'Authorization':'Bearer fixture-caller-credential','x-opencodex-api-key':env['OPENCODEX_API_AUTH_TOKEN']})
            report['historyRequest']={'http':code,'message':raw[:600],'upstreamCalls':len(upstream.requests)-before}
            assert code==400 and 'native ChatGPT forward provider' in raw and len(upstream.requests)==before,report['historyRequest']
            record('third_party_history_refused_before_egress',report['historyRequest'])
            (codex/'config.toml').unlink()
            code,raw=request('/v1/responses',{'model':'probe/gpt-5.4-mini','input':[{'role':'user','content':'summarize fixture'},{'type':'compaction_trigger'}],'stream':False})
            report['compactionRequest']={'http':code,'upstream':upstream.requests[-1] if len(upstream.requests)>before else None,'responseTypes':[x.get('type') for x in json.loads(raw).get('output',[])] if code==200 else [],'error':raw[:600] if code!=200 else None}
            assert report['compactionRequest']['http']==200 and report['compactionRequest']['upstream']['hasSummaryInstruction'] and not report['compactionRequest']['upstream']['compactionTriggerPresent'],report['compactionRequest']
            record('routed_compaction_is_proxy_summary',report['compactionRequest'])
            record('codex_integration_stays_off',{'configContainsOnlyOwnTestFiles':True,'dailyPathsNotUsed':True,'codexConfigCreated':(codex/'config.toml').exists()})
            if args.codex and args.native_helper:
                spec=importlib.util.spec_from_file_location('native_e2e_helper',args.native_helper)
                helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
                native_root=root/'native-e2e';native_root.mkdir()
                native=helper.AppServer(args.codex,native_root,port)
                try:
                    native.call('initialize',{'clientInfo':{'name':'proxy_compaction_probe','version':'0.1'},'capabilities':{'experimentalApi':True}});native.send({'method':'initialized','params':{}})
                    state=native.call('thread/start',{'cwd':str(native.workspace),'model':'probe/gpt-5.4-mini','config':{'model_providers.audit_local.capabilities.remote_compaction':'v2'}})
                    tid=state['thread']['id'];native.turn(tid)
                    before=len(upstream.requests)
                    native.call('thread/compact/start',{'threadId':tid})
                    deadline=time.monotonic()+10;completed=False
                    while time.monotonic()<deadline:
                        message=native.receive(deadline);native.notifications.append(message)
                        if message.get('method')=='turn/completed' and message.get('params',{}).get('threadId')==tid:
                            if message.get('params',{}).get('turn',{}).get('status')!='completed':raise AssertionError(message)
                            completed=True;break
                    assert completed and len(upstream.requests)>before
                    compact_calls=upstream.requests[before:]
                    native.turn(tid)
                    record('native_engine_compact_then_continue',{'manualCompactCompleted':True,'upstreamCompactionTrigger':any(x['compactionTriggerPresent'] for x in compact_calls),'upstreamSummary':any(x['hasSummaryInstruction'] for x in compact_calls),'subsequentTurnCompleted':True,'path':'real Codex -> real OpenCodex -> local fixture'})
                    bg=native.call('thread/start',{'cwd':str(native.workspace),'model':'probe/gpt-5.4-mini','experimentalRawEvents':True,'config':{'features.async_compaction.enabled':True,'features.async_compaction.model_async_compact_threshold_percent':65,'model_context_window':100000,'model_auto_compact_token_limit':90000,'model_providers.audit_local.capabilities.remote_compaction':'v2'}})
                    bgid=bg['thread']['id'];before=len(upstream.requests)
                    result=native.call('turn/start',{'threadId':bgid,'input':[{'type':'text','text':'BACKGROUND_PROBE synthetic usage threshold test.'}]})
                    bgturn=result['turn']['id'];deadline=time.monotonic()+8;bgdone=False
                    while time.monotonic()<deadline:
                        message=native.receive(deadline);native.notifications.append(message)
                        if message.get('method')=='turn/completed' and message.get('params',{}).get('threadId')==bgid and message.get('params',{}).get('turn',{}).get('id')==bgturn:
                            bgdone=True;break
                    assert bgdone
                    # Hold the background checkpoint and test that foreground work can still complete.
                    started=upstream.background_started.wait(timeout=3)
                    foreground_before_release=False
                    try:
                        native.turn(bgid)
                        foreground_before_release=started and not upstream.background_finished.is_set()
                    finally:upstream.background_release.set()
                    events=[]
                    for message in native.notifications:
                        if message.get('params',{}).get('threadId')==bgid and ('compact' in json.dumps(message).lower() or 'tokenUsage' in message.get('method','')):
                            events.append({'method':message.get('method'),'eventType':message.get('params',{}).get('msg',{}).get('type'),'detail':message.get('params',{}).get('message') or message.get('params',{}).get('msg',{}).get('message'),'tokenUsage':message.get('params',{}).get('tokenUsage')})
                    bgcalls=upstream.requests[before:];summaries=sum(x['hasSummaryInstruction'] for x in bgcalls)
                    report['backgroundCompaction']={'configured':True,'syntheticUsageTokens':70000,'modelContextWindow':100000,'autoCompactThresholdTokens':90000,'backgroundThresholdTokens':65000,'backgroundPercent':65,'summaryRequestsObserved':summaries,'events':events,'backgroundRequestStarted':started,'foregroundCompletedBeforeRelease':foreground_before_release,'status':'passed' if started and foreground_before_release else 'inconclusive','reason':'Background checkpoint and foreground turn overlap observed with explicit context-window threshold.' if started and foreground_before_release else 'No definitive non-blocking overlap observed.'}
                    if started and foreground_before_release:
                        record('background_compaction_nonblocking_overlap',report['backgroundCompaction'])
                    local_bg=native.call('thread/start',{'cwd':str(native.workspace),'model':'probe/gpt-5.4-mini','config':{'features.async_compaction.enabled':True,'features.async_compaction.model_async_compact_threshold_percent':65,'model_context_window':100000,'model_auto_compact_token_limit':90000,'model_providers.audit_local.capabilities.remote_compaction':'unsupported'}})
                    local_bg_id=local_bg['thread']['id'];local_before=len(upstream.requests)
                    result=native.call('turn/start',{'threadId':local_bg_id,'input':[{'type':'text','text':'BACKGROUND_DISABLED_PROBE same threshold, client compaction protocol.'}]})
                    local_turn=result['turn']['id'];deadline=time.monotonic()+8;local_done=False
                    while time.monotonic()<deadline:
                        message=native.receive(deadline);native.notifications.append(message)
                        if message.get('method')=='turn/completed' and message.get('params',{}).get('threadId')==local_bg_id and message.get('params',{}).get('turn',{}).get('id')==local_turn:
                            local_done=True;break
                    assert local_done
                    native.turn(local_bg_id)
                    local_calls=upstream.requests[local_before:]
                    assert len(local_calls)==2 and not any(x['hasSummaryInstruction'] for x in local_calls),local_calls
                    record('background_not_active_without_remote_v2',{'asyncFeatureEnabled':True,'remoteCompaction':'unsupported','syntheticUsageTokens':70000,'modelContextWindow':100000,'thresholdPercent':65,'foregroundRequests':len(local_calls),'summaryRequests':0})


                finally:native.close()

            # Restart only this probe's owned service. No daily process or config is involved.
            proc.terminate();proc.wait(timeout=7)
            env['OCX_PLUGINS']='0'
            proc=subprocess.Popen([str(bun),str(r/'src/cli/index.ts'),'start','--port',str(port)],env=env,cwd=root,stdin=subprocess.DEVNULL,stdout=log,stderr=log,start_new_session=True)
            deadline=time.monotonic()+15;ready=False
            while proc.poll() is None and time.monotonic()<deadline:
                try:
                    code,_=request('/readyz')
                    if code==200:ready=True;break
                except OSError:pass
                time.sleep(.1)
            assert ready,'isolated plugin-disabled service not ready'
            code,raw=request('/v1/responses',{'model':'probe/gpt-5.4-mini','input':[{'role':'user','content':'plugin disabled fixture'}],'stream':True})
            final=upstream.requests[-1]
            assert code==200 and final['path'].startswith('/direct/') and final['plugin'] is None,(code,final)
            record('disable_plugin_keeps_original_route',{'pluginDisabledByPublicEnv':True,'upstream':final,'http':code})
            report['status']='passed'
        except Exception as error:report['status']='failed';report['error']=str(error)
        finally:
            if proc is not None:
                if proc.poll() is None:proc.terminate()
                try:proc.wait(timeout=7)
                except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait(timeout=5)
                try:os.killpg(proc.pid,signal.SIGTERM)
                except ProcessLookupError:pass
            log.flush();log.seek(0);text=log.read()
            report['loaderMarker']='fixture-ready' in text
            report['serviceLogTail']=text[-6000:]
            report['upstreamRequests']=upstream.requests;report['blockedExternalAttempts']=len(upstream.blocked)
            report['isolatedRootManifest']=[str(x.relative_to(root)) for x in home.rglob('*') if x.is_file()]
            upstream.background_release.set();log.close();upstream.shutdown();upstream.server_close();thread.join(timeout=2)
    report['scriptSha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='isolatedRootManifest'},ensure_ascii=False,indent=2));return 0 if report['status']=='passed' else 1
if __name__=='__main__':raise SystemExit(main())
