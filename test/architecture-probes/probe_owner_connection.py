#!/usr/bin/env python3
"""Read-only desktop endpoint inventory + isolated two-client owning-server proof.
No account, active-thread mutation, production restart, or real model call.
"""
import argparse, importlib.util, json, os, queue, shlex, signal, subprocess
import tempfile, threading, time, hashlib
from pathlib import Path
from datetime import datetime, timezone


def inventory():
    rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,comm='], text=True).splitlines()
    candidates = []
    for row in rows:
        parts = row.strip().split(None, 2)
        if len(parts) == 3 and parts[2].endswith('/codex'):
            candidates.append((int(parts[0]), int(parts[1])))
    result = []
    for pid, ppid in candidates:
        raw = subprocess.run(['ps', '-p', str(pid), '-o', 'args='], capture_output=True, text=True)
        try: args = shlex.split(raw.stdout)
        except ValueError: continue
        if 'app-server' not in args: continue
        listeners = [args[i+1] for i, a in enumerate(args[:-1]) if a == '--listen']
        listeners += [a.split('=', 1)[1] for a in args if a.startswith('--listen=')]
        unix = subprocess.run(['lsof', '-nP', '-a', '-p', str(pid), '-U', '-Fn'], capture_output=True, text=True)
        named_sockets = [s[1:] for s in unix.stdout.splitlines() if s.startswith('n/')]
        tcp = subprocess.run(['lsof', '-nP', '-a', '-p', str(pid), '-iTCP', '-sTCP:LISTEN', '-Fn'], capture_output=True, text=True)
        tcp_listeners = [s[1:] for s in tcp.stdout.splitlines() if s.startswith('n')]
        result.append({'pid': pid, 'ppid': ppid, 'listenArgs': listeners or ['default stdio'],
                       'namedUnixSockets': named_sockets, 'tcpListeners': tcp_listeners,
                       'endpointInspectionComplete': unix.returncode in (0,1) and tcp.returncode in (0,1)})
    # No argv, environment, log content, or private application channels are persisted.
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--codex', required=True)
    parser.add_argument('--native-helper', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('native_probe_helper', args.native_helper)
    helper = importlib.util.module_from_spec(spec); spec.loader.exec_module(helper)
    report = {'atUtc': datetime.now(timezone.utc).isoformat(),
              'codexVersion': subprocess.check_output([args.codex, '--version'], text=True).strip(),
              'scriptSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'liveInventory': inventory(), 'checks': [], 'activeAppModified': False,
              'realModelCalls': 0}
    live = report['liveInventory']
    report['currentAppAttachment'] = {
        'status': 'unsupported_current_launch' if live and all(x['endpointInspectionComplete'] and not x['tcpListeners'] and not x['namedUnixSockets'] and all('stdio' in s for s in x['listenArgs']) for x in live) else 'inconclusive',
        'scope': 'Observed app-server processes only; no named public endpoint found. No stdio interception or restart attempted.'}

    from ws_json_client import WsJson
    class Client(helper.AppServer):
        def __init__(self,path):
            self.notifications=[];self.next_id=0;self.ws=WsJson(path=path)
        def send(self,value):self.ws.send(value)
        def receive(self,deadline):
            message=self.ws.recv(deadline)
            if 'method' in message and 'id' in message:self.send({'id':message['id'],'error':{'code':-32601,'message':'Probe does not execute tools'}})
            return message
        def close(self):self.ws.close()

    def initialize(c, name):
        r=c.call('initialize', {'clientInfo':{'name':name,'version':'0.1'},'capabilities':{'experimentalApi':True}})
        c.send({'method':'initialized','params':{}})
        return r

    def check(name, detail): report['checks'].append({'id':name,'status':'passed','observed':detail})
    with tempfile.TemporaryDirectory(prefix='ocx-owner-', dir='/tmp') as tmp:
        root=Path(tmp);home=root/'home';codex_home=home/'.codex';codex_home.mkdir(parents=True)
        workspace=root/'workspace';workspace.mkdir();sock=root/'owner.sock'
        http=helper.ThreadingHTTPServer(('127.0.0.1',0),helper.LocalResponses);http.requests=[]
        worker=threading.Thread(target=http.serve_forever,daemon=True);worker.start()
        (codex_home/'config.toml').write_text(f'''
model="gpt-5.4"
model_provider="audit_local"
model_reasoning_effort="low"
approval_policy="never"
sandbox_mode="read-only"
web_search="disabled"
check_for_update_on_startup=false
[model_providers.audit_local]
name="owner probe"
base_url="http://127.0.0.1:{http.server_port}/v1"
wire_api="responses"
requires_openai_auth=false
request_max_retries=0
stream_max_retries=0
supports_websockets=false
[analytics]
enabled=false
[feedback]
enabled=false
''')
        env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(home),'CODEX_HOME':str(codex_home),
             'TMPDIR':str(root),'RUST_LOG':'error','HTTP_PROXY':f'http://127.0.0.1:{http.server_port}',
             'HTTPS_PROXY':f'http://127.0.0.1:{http.server_port}','NO_PROXY':'localhost,127.0.0.1'}
        owner=None; clients=[];owner_err=(root/'owner.stderr').open('w+')
        try:
            owner=subprocess.Popen([args.codex,'app-server','--listen','unix://'+str(sock)],
                                   env=env,cwd=workspace,stdout=subprocess.DEVNULL,stderr=owner_err,start_new_session=True)
            deadline=time.monotonic()+8
            while not sock.exists() and owner.poll() is None and time.monotonic()<deadline:time.sleep(.05)
            if not sock.exists():raise RuntimeError('isolated owner did not publish Unix socket')
            for label in ['a','b']:
                c=Client(sock)
                clients.append(c);report['phase']='initialize_'+label;initialize(c,'ownership_probe_'+label)
            a,b=clients
            start=a.call('thread/start',{'cwd':str(workspace)})
            tid=start['thread']['id'];a.turn(tid)
            assert http.requests[-1]['model']=='gpt-5.4'
            assert b.call('thread/read',{'threadId':tid})['thread']['id']==tid
            check('two_clients_same_owner_read','Both public proxies read the synthetic thread in one isolated owning server.')
            b.call('thread/settings/update',{'threadId':tid,'model':'gpt-5.4-mini','effort':'high'})
            a.turn(tid)
            assert http.requests[-1]['model']=='gpt-5.4-mini'
            assert http.requests[-1]['reasoning']['effort']=='high'
            check('second_client_controls_same_owner','Client B changes settings; client A next turn actually sends mini/high.')
            b.close();clients.remove(b);a.turn(tid)
            assert http.requests[-1]['model']=='gpt-5.4-mini'
            check('management_client_disconnect','Disconnecting B does not end owner or A; next turn still succeeds.')
            report['isolatedOwner']={'status':'passed','transport':'unix WebSocket through standard HTTP Upgrade','localRequests':http.requests}
        except Exception as error:
            report['isolatedOwner']={'status':'failed','error':str(error),'phase':report.get('phase'),'localRequests':http.requests,'diagnostics':{x.name:x.read_text()[-3000:] for x in root.glob('*.stderr')}}
        finally:
            for c in reversed(clients):c.close()
            if owner is not None:
                if owner.poll() is None:owner.terminate()
                try:owner.wait(timeout=5)
                except subprocess.TimeoutExpired:os.killpg(owner.pid,signal.SIGKILL);owner.wait(timeout=5)
                # Only this probe's separately-created process group is cleaned up.
                try:os.killpg(owner.pid,signal.SIGTERM)
                except ProcessLookupError:pass
            owner_err.close();http.shutdown();http.server_close();worker.join(timeout=2)
    report['status']='passed' if report.get('isolatedOwner',{}).get('status')=='passed' else 'failed'
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(report,ensure_ascii=False,indent=2))
    return 0 if report['status']=='passed' else 1
if __name__=='__main__':raise SystemExit(main())
