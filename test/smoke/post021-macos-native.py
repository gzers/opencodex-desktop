#!/usr/bin/env python3
"""Owned native app + synthetic HTTPS snapshots; no daily data or public network."""
import argparse
import base64
import hashlib
import http.server
import importlib.util
import json
from pathlib import Path
import shutil
import ssl
import sys
import threading
import time
import traceback

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('owned_native', HERE / 'ocx021-macos-native.py')
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)

class Run(native.NativeRun):
    def prepare(self):
        super().prepare()
        self.requests = []
        self.mode = 'unknown'
        fixtures = Path(self.args.fixture_dir).resolve(strict=True)
        owner = self
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *args): pass
            def do_GET(self):
                expected = 'Basic ' + base64.b64encode(b'synthetic-post021:synthetic-post021-password').decode()
                authenticated = self.headers.get('Authorization') == expected
                owner.requests.append({'method': 'GET', 'path': self.path, 'mode': owner.mode, 'authenticated': authenticated})
                if not authenticated: self.send_error(401); return
                name = self.path.removeprefix('/native021-only/')
                if owner.mode == 'failed': self.send_error(503); return
                if owner.mode == 'absent': self.send_error(404); return
                if not self.path.startswith('/native021-only/') or '/' in name: self.send_error(404); return
                folder = fixtures / owner.mode
                file = folder / name
                if not file.is_file(): self.send_error(404); return
                body = file.read_bytes()
                etags = json.loads((folder / 'etags.json').read_text())
                self.send_response(200)
                self.send_header('ETag', etags[name]); self.send_header('Content-Length', str(len(body)))
                self.end_headers(); self.wfile.write(body)
            def do_PUT(self):
                owner.requests.append({'method': 'PUT', 'path': self.path}); self.send_error(405)
            def do_PROPFIND(self):
                owner.requests.append({'method': 'PROPFIND', 'path': self.path}); self.send_error(405)
        tls = self.root / 'fixture-tls'; tls.mkdir()
        ca = tls / 'ca.pem'; ca_key = tls / 'ca.key'; leaf = tls / 'server.pem'; key = tls / 'server.key'; csr = tls / 'server.csr'
        conf = tls / 'openssl.cnf'
        conf.write_text('[req]\ndistinguished_name=dn\nprompt=no\n[dn]\nCN=Owned native fixture\n[ca]\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,keyCertSign,cRLSign\n[server]\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=IP:127.0.0.1\n')
        commands = [
            ['req','-new','-x509','-newkey','rsa:2048','-nodes','-days','1','-keyout',ca_key,'-out',ca,'-config',conf,'-extensions','ca'],
            ['req','-new','-newkey','rsa:2048','-nodes','-keyout',key,'-out',csr,'-config',conf],
            ['x509','-req','-in',csr,'-CA',ca,'-CAkey',ca_key,'-CAcreateserial','-out',leaf,'-days','1','-extfile',conf,'-extensions','server'],
        ]
        for args in commands:
            result = native.command(['/usr/bin/openssl', *args], env=self.env)
            if result.returncode: raise RuntimeError('Fixture certificate creation failed: ' + result.stderr)
        self.server = http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); context.load_cert_chain(leaf,key)
        self.server.socket = context.wrap_socket(self.server.socket,server_side=True)
        port = self.server.server_port; self.endpoint = 'https://127.0.0.1:' + str(port)
        self.server_thread = threading.Thread(target=self.server.serve_forever,daemon=True); self.server_thread.start()
        self.env['SSL_CERT_FILE'] = str(ca)
        empty_ca_dir = tls / 'empty'; empty_ca_dir.mkdir(); self.env['SSL_CERT_DIR'] = str(empty_ca_dir)
        self.profile.write_text(self.profile.read_text() + '\n(allow network-outbound (remote ip "localhost:' + str(port) + '"))\n')
        shutil.copyfile(self.profile,self.out / 'candidate.sb')
        denied = native.command(['/usr/bin/sandbox-exec','-f',self.profile,'/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443'],env=self.env)
        assert 'Operation not permitted' in denied.stderr and denied.returncode != 0
        self.check('only-owned-loopback-allowed','pass', endpoint=self.endpoint, caTrust='process SSL_CERT_FILE only', externalDenied=True)

    def business_hashes(self):
        names=['opencodex-home/config.json','manager-state/sync-endpoints.json','manager-state/config-sync-scope.json','manager-state/saved-content-versions.json']
        return {n: hashlib.sha256((self.root/n).read_bytes()).hexdigest() if (self.root/n).exists() else None for n in names}

    def verify(self):
        self.ready_environment()
        self.press('同步'); self.press('WebDAV 同步'); self.press('连接设置')
        self.wait(lambda s: self.dialogs(s),name='post021-connection')
        for label,value in [('服务地址',self.endpoint),('独立配置同步目录','native021-only'),('用户名','synthetic-post021'),('连接密码','synthetic-post021-password')]:
            self.set_text(label,value,secure=label=='连接密码')
        self.press('保存连接'); self.wait(lambda s:not self.dialogs(s),name='post021-connection-saved')
        snap=self.wait(lambda s:self.endpoint+'/native021-only' in self.text(s),name='post021-display-address')
        self.capture('post021-display-address',snap)
        self.check('post021-address-boundary','pass', expected=self.endpoint+'/native021-only')
        until=time.monotonic()+20
        while not (self.root/'manager-state/config-sync-initialized.json').exists() and time.monotonic()<until:time.sleep(.1)
        assert (self.root/'manager-state/config-sync-initialized.json').exists(), 'Real UI did not initialize first complete catalogue'
        selection=json.loads((self.root/'manager-state/config-sync-scope.json').read_text())
        assert selection['channelIds'] and selection['templateIds']
        snap=self.wait(lambda s:self.matches(s,'选择 渠道与模型',('AXCheckBox',)),name='post021-first-scope-ui')
        checked=self.matches(snap,'选择 渠道与模型',('AXCheckBox',))[0].get('value')
        assert checked in [1,True,'1'], 'Backend first selection was not adopted by the real UI'
        self.capture('post021-first-scope-ui',snap)
        self.check('post021-native-first-scope','pass', selectedChannels=len(selection['channelIds']),selectedTemplates=len(selection['templateIds']),nativeGroupChecked=checked)
        self.press('概览')
        before=self.business_hashes(); old_pid=self.proc.pid; previous_reads=len(self.requests)
        self.stop_observer.set(); self.observer.join(timeout=3)
        self.proc.terminate(); self.proc.wait(timeout=10); self.log.close()
        shutil.copyfile(self.out/'candidate-stdout.log',self.out/'first-launch.log')
        self.stop_observer.clear(); self.launch()
        self.ready_environment()
        snap=self.wait(lambda s:'已读取 · 保存时间未知' in self.text(s),name='post021-startup-version-read')
        self.capture('post021-startup-unknown',snap)
        assert self.proc.pid != old_pid and len(self.requests) >= previous_reads + 3
        self.check('post021-startup-readonly-version','pass', previousOwnedPid=old_pid,currentOwnedPid=self.proc.pid,observedState='present, reliable save time unknown')
        for mode,expected in [('known','上次观察'),('absent','未发现快照'),('failed','读取失败'),('unknown','已读取 · 保存时间未知')]:
            self.mode=mode; self.press('检测版本')
            snap=self.wait(lambda s:expected in self.text(s),name='post021-version-'+mode)
            self.capture('post021-version-'+mode,snap)
            self.check('post021-native-version-'+mode,'pass',expected=expected)
        after=self.business_hashes()
        assert before==after, 'Version observation changed business files'
        assert all(r['method']=='GET' for r in self.requests), 'Unexpected remote write/probe'
        native.write_json(self.out/'fixture-requests.json',self.requests)
        self.check('post021-readonly-side-effects','pass', hashes=after, getCount=len(self.requests), remoteWrites=0, systemTrustChanged=False)
        self.report['scopeCompleted']=True

    def close(self):
        try:super().close()
        finally:
            if hasattr(self,'server'):
                self.server.shutdown(); self.server.server_close(); self.server_thread.join(timeout=3)
                native.write_json(self.out/'fixture-requests.json',self.requests)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['app','out','fixture-dir','expected-build-commit','node','npm-cli','official-entry']:p.add_argument('--'+name,required=True)
    p.add_argument('--bun');args=p.parse_args();args.stage='debug-preverify';args.scope='overview';args.hold_seconds=0
    run=Run(args)
    try:run.prepare();run.launch();run.verify()
    except (Exception,KeyboardInterrupt) as error:
        run.report['error']=str(error);run.report['errorKind']='blocked' if isinstance(error,native.Blocked) else 'fail'
        run.check('post021-native-prerequisite',run.report['errorKind'],reason=str(error))
        (run.out/'exception.log').write_text(traceback.format_exc())
        try:snap=run.snapshot('post021-failure');run.capture('post021-failure',snap)
        except Exception:pass
    finally:run.close()
    return 0 if run.report['result']=='pass' else 1

if __name__=='__main__':sys.exit(main())
