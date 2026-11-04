#!/usr/bin/env python3
"""Native name-only channel creation/discovery/save with an owned loopback API."""
import argparse
import http.server
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import threading
import time
import traceback

HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('owned_native',HERE/'ocx021-macos-native.py')
native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)

class Run(native.NativeRun):
    def prepare(self):
        super().prepare();self.requests=[];owner=self
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                authenticated=self.headers.get('Authorization')=='Bearer synthetic-l2-key'
                owner.requests.append({'method':'GET','path':self.path,'authenticated':authenticated})
                if not authenticated:self.send_error(401);return
                if self.path!='/v1/models':self.send_error(404);return
                body=json.dumps({'data':[{'id':'vendor/native-model'}]}).encode()
                self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
            def do_POST(self):owner.requests.append({'method':'POST','path':self.path});self.send_error(405)
        self.server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler)
        self.thread=threading.Thread(target=self.server.serve_forever,daemon=True);self.thread.start()
        self.endpoint='http://127.0.0.1:'+str(self.server.server_port)+'/v1'
        self.profile.write_text(self.profile.read_text()+'\n(allow network-outbound (remote ip "localhost:'+str(self.server.server_port)+'"))\n')
        shutil.copyfile(self.profile,self.out/'candidate.sb')
        denied=native.command(['/usr/bin/sandbox-exec','-f',self.profile,'/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443'],env=self.env)
        assert denied.returncode!=0 and 'Operation not permitted' in denied.stderr
        self.check('L2-owned-loopback-only','pass',endpoint=self.endpoint,externalDenied=True)

    def verify(self):
        self.channels();before=(self.root/'opencodex-home/config.json').read_bytes()
        self.press('新增渠道');snap=self.wait(lambda s:self.dialogs(s),name='L2-new-channel')
        assert not self.matches(snap,'运行渠道标识',('AXTextField',))
        assert '允许访问可信私有网络' not in self.text(snap)
        self.set_text('渠道名称','中文 渠道 / one');self.set_text('服务基地址',self.endpoint);self.set_text('API Key','synthetic-l2-key',secure=True)
        self.capture('L2-new-name-only',self.snapshot());self.press('下一步');self.press('获取模型列表')
        snap=self.wait(lambda s:self.matches(s,'选择新渠道型号 vendor/native-model',('AXCheckBox',)),name='L2-model-fetched')
        checkbox=self.matches(snap,'选择新渠道型号 vendor/native-model',('AXCheckBox',))[0]
        self.ax('press',checkbox['path'],'选择新渠道型号 vendor/native-model')
        assert (self.root/'opencodex-home/config.json').read_bytes()==before
        self.capture('L2-discovery-before-save',self.snapshot());self.press('加入草稿')
        snap=self.wait(lambda s:self.matches(s,'保存渠道草稿'),name='L2-channel-draft')
        assert not self.matches(snap,'运行渠道标识',('AXTextField',))
        self.press('保存渠道草稿');self.wait(lambda s:self.matches(s,'立即重启 Codex'),name='L2-written')
        raw=json.loads((self.root/'opencodex-home/config.json').read_text())
        keys=[k for k in raw['providers'] if k!='native021-old'];assert len(keys)==1
        key=keys[0];assert key.startswith('channel-') and raw['providers'][key]['name']=='中文 渠道 / one'
        assert raw['providers'][key]['selectedModels']==['vendor/native-model']
        assert raw['providers']['native021-old']==self.before['providers']['native021-old']
        self.check('L2-new-save-readback','pass',internalKey=key,selectedModels=raw['providers'][key]['selectedModels'],displayName='中文 渠道 / one')
        if self.matches(self.snapshot(),'稍后'):self.press('稍后')
        self.press('编辑',1);self.wait(lambda s:self.matches(s,'渠道名称',('AXTextField',)),name='L2-reopened')
        self.set_text('渠道名称','openai');self.press('保存渠道草稿');self.wait(lambda s:self.matches(s,'立即重启 Codex'),name='L2-renamed-written')
        renamed=json.loads((self.root/'opencodex-home/config.json').read_text());assert renamed['providers'][key]['name']=='openai'
        if self.matches(self.snapshot(),'稍后'):self.press('稍后')
        self.press('编辑',1);snap=self.wait(lambda s:self.matches(s,'渠道名称',('AXTextField',)),name='L2-rename-reopened')
        assert self.matches(snap,'渠道名称',('AXTextField',))[0]['value']=='openai'
        self.capture('L2-reserved-name-reopened',snap)
        self.press('检测全部节点');until=time.monotonic()+20
        while len(self.requests)<2 and time.monotonic()<until:time.sleep(.1)
        assert len(self.requests)>=2 and all(r['method']=='GET' and r['authenticated'] for r in self.requests)
        self.wait(lambda s:'发现 1' in self.text(s) or '1 个模型' in self.text(s),name='L2-node-result')
        self.capture('L2-node-result',self.snapshot())
        assert json.loads((self.root/'opencodex-home/config.json').read_text())==renamed
        self.check('L2-native-rename-and-measure','pass',displayName='openai',internalKeyUnchanged=True,realGetRequests=len(self.requests),generatedRequests=0,measurementDoesNotSave=True)
        native.write_json(self.out/'model-requests.json',self.requests);self.report['scopeCompleted']=True

    def close(self):
        try:super().close()
        finally:
            if hasattr(self,'server'):
                self.server.shutdown();self.server.server_close();self.thread.join(timeout=3)
                native.write_json(self.out/'model-requests.json',self.requests)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['app','out','expected-build-commit']:p.add_argument('--'+name,required=True)
    args=p.parse_args();args.stage='debug-preverify';args.scope='all';args.hold_seconds=0
    args.node=args.npm_cli=args.official_entry=args.bun=None
    run=Run(args)
    try:run.prepare();run.launch();run.verify()
    except (Exception,KeyboardInterrupt) as error:
        run.report['error']=str(error);run.report['errorKind']='blocked' if isinstance(error,native.Blocked) else 'fail';run.check('L2-native-run',run.report['errorKind'],reason=str(error))
        (run.out/'exception.log').write_text(traceback.format_exc())
        try:snap=run.snapshot('L2-failure');run.capture('L2-failure',snap)
        except Exception:pass
    finally:run.close()
    return 0 if run.report['result']=='pass' else 1
if __name__=='__main__':sys.exit(main())
