#!/usr/bin/env python3
"""Own unmodified official server + integrated native child WKWebView only."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import time
import traceback
import urllib.request

HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('owned_native',HERE/'ocx021-macos-native.py')
native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)

class Run(native.NativeRun):
    def prepare(self):
        super().prepare()
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1',0));self.port=reserve.getsockname()[1]
        self.profile.write_text(self.profile.read_text()+'\n(allow network-outbound (remote ip "localhost:'+str(self.port)+'"))\n(allow network-inbound (local ip "localhost:'+str(self.port)+'"))\n(allow network-bind (local ip "localhost:'+str(self.port)+'"))\n')
        # CFNetwork consults this system Unix socket even for numeric loopback.
        # IP traffic remains limited to the one owned server port.
        self.profile.write_text(self.profile.read_text()+'\n(allow network-outbound (literal "/private/var/run/mDNSResponder"))\n')
        shutil.copyfile(self.profile,self.out/'candidate.sb')
        denied=native.command(['/usr/bin/sandbox-exec','-f',self.profile,'/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443'],env=self.env)
        assert denied.returncode!=0 and 'Operation not permitted' in denied.stderr
        # Node checks inherited stdio paths during startup. Keep that file in
        # the admitted root; copy it out only after the owned server stops.
        self.server_log_path=self.root/'official-server.log'
        self.server_log=open(self.server_log_path,'wb')
        self.server=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(self.profile),str(self.bin/'ocx'),'start','--port',str(self.port)],cwd=self.root,env=self.env,stdout=self.server_log,stderr=subprocess.STDOUT,start_new_session=True)
        self.direct_groups.append({'pid':self.server.pid,'pgid':self.server.pid,'label':'owned-unmodified-official-server'})
        deadline=time.monotonic()+40
        while time.monotonic()<deadline:
            if self.server.poll() is not None:raise RuntimeError('Owned official server exited; see original log')
            try:
                with urllib.request.urlopen('http://127.0.0.1:'+str(self.port)+'/healthz',timeout=1) as reply:
                    if reply.status==200:break
            except Exception:time.sleep(.2)
        else:raise native.Blocked('Owned official server health not ready')
        self.check('integrated-official-server-owned','pass',pid=self.server.pid,port=self.port,network='one owned loopback port; external denied',packageUnmodified=True)

    def verify(self):
        self.press('面板')
        initial=self.wait(lambda s:'官方面板已嵌入主窗口' in self.text(s),name='panel-initial-startup')
        assert len([n for n in initial['nodes'] if n.get('role')=='AXWebArea'])>=2
        self.capture('panel-initial-startup',initial)
        self.ready_environment()
        before=(self.root/'opencodex-home/config.json').read_bytes()
        timings=[]
        for scene in ['return-1','return-2']:
            start=time.monotonic();self.press('面板')
            snap=self.wait(lambda s:'官方面板已嵌入主窗口' in self.text(s),name='integrated-panel-'+scene)
            elapsed=(time.monotonic()-start)*1000
            webviews=[n for n in snap['nodes'] if n.get('role')=='AXWebArea']
            assert len(webviews)>=2,'No official child WKWebView appeared in native AX tree'
            self.capture('integrated-panel-'+scene,snap);timings.append({'scene':scene,'routeToObservedReadyMs':elapsed,'nativeWebAreas':len(webviews)})
            self.press('概览')
        # The panel has no top bar; overview notifications close on route click.
        # Do not inject store state to manufacture native overlay coverage.
        self.report['managerOverlayCoverage']='not exercised: panel has no notification trigger; overview notification closes on route click. Windows contribution covers it; final Windows rerun remains required.'
        assert before==(self.root/'opencodex-home/config.json').read_bytes()
        native.write_json(self.out/'panel-route-timings.json',timings)
        self.check('integrated-mac-official-child-panel','pass',timings=timings,configUnchanged=True,performanceScope='actual AX ready observation; not optical first pixel or Windows performance gate')
        self.report['scopeCompleted']=True

    def close(self):
        if hasattr(self,'server'):
            if self.server.poll() is None:
                os.killpg(self.server.pid,signal.SIGTERM)
                try:self.server.wait(timeout=5)
                except subprocess.TimeoutExpired:os.killpg(self.server.pid,signal.SIGKILL);self.server.wait(timeout=5)
            self.server_log.close()
            shutil.copyfile(self.server_log_path,self.out/'official-server.log')
        super().close()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['app','out','expected-build-commit','node','npm-cli','official-entry']:p.add_argument('--'+name,required=True)
    p.add_argument('--bun');args=p.parse_args();args.stage='debug-preverify';args.scope='overview';args.hold_seconds=0
    run=Run(args)
    try:run.prepare();run.launch();run.verify()
    except (Exception,KeyboardInterrupt) as error:
        run.report['error']=str(error);run.report['errorKind']='blocked' if isinstance(error,native.Blocked) else 'fail';run.check('integrated-panel-native',run.report['errorKind'],reason=str(error));(run.out/'exception.log').write_text(traceback.format_exc())
        try:snap=run.snapshot('panel-failure');run.capture('panel-failure',snap)
        except Exception:pass
    finally:run.close()
    return 0 if run.report['result']=='pass' else 1
if __name__=='__main__':sys.exit(main())
