import http.server,ssl,json,base64,subprocess,threading
from pathlib import Path
base=Path(__file__).resolve().parent;tls=base/'tls';tls.mkdir(exist_ok=True);(tls/'empty').mkdir(exist_ok=True)
openssl='D:/Software/Develop/Git/usr/bin/openssl.exe'
conf=tls/'openssl.cnf';conf.write_text('[req]'+chr(10)+'distinguished_name=dn'+chr(10)+'prompt=no'+chr(10)+'[dn]'+chr(10)+'CN=Owned Windows post021 fixture'+chr(10)+'[ca]'+chr(10)+'basicConstraints=critical,CA:TRUE'+chr(10)+'keyUsage=critical,keyCertSign,cRLSign'+chr(10)+'[server]'+chr(10)+'basicConstraints=critical,CA:FALSE'+chr(10)+'keyUsage=critical,digitalSignature,keyEncipherment'+chr(10)+'extendedKeyUsage=serverAuth'+chr(10)+'subjectAltName=IP:127.0.0.1'+chr(10))
for args in [['req','-new','-x509','-newkey','rsa:2048','-nodes','-days','1','-keyout',tls/'ca.key','-out',tls/'ca.pem','-config',conf,'-extensions','ca'],['req','-new','-newkey','rsa:2048','-nodes','-keyout',tls/'server.key','-out',tls/'server.csr','-config',conf],['x509','-req','-in',tls/'server.csr','-CA',tls/'ca.pem','-CAkey',tls/'ca.key','-CAcreateserial','-out',tls/'server.pem','-days','1','-extfile',conf,'-extensions','server']]:
 with(tls/'openssl.log').open('a')as log:subprocess.run([openssl,*map(str,args)],check=True,stdout=log,stderr=log)
(base/'https-mode.txt').write_text('unknown')
requests=[];lock=threading.Lock()
class Handler(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_GET(self):
  mode=(base/'https-mode.txt').read_text().strip();auth=self.headers.get('Authorization')=='Basic '+base64.b64encode(b'synthetic-post021:synthetic-post021-password').decode()
  with lock:
   requests.append({'method':'GET','path':self.path,'mode':mode,'authenticated':auth});(base/'https-requests.json').write_text(json.dumps(requests,indent=2)+chr(10))
  if not auth:self.send_error(401);return
  if mode=='failed':self.send_error(503);return
  if mode=='absent':self.send_error(404);return
  name=self.path.removeprefix('/native021-only/')
  if not self.path.startswith('/native021-only/')or '/'in name:self.send_error(404);return
  folder=base/'https-fixtures'/mode;p=folder/name
  if not p.is_file():self.send_error(404);return
  body=p.read_bytes();etag=json.loads((folder/'etags.json').read_text())[name];self.send_response(200);self.send_header('ETag',etag);self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
 def do_PUT(self):
  with lock:requests.append({'method':'PUT','path':self.path});(base/'https-requests.json').write_text(json.dumps(requests,indent=2)+chr(10))
  self.send_error(405)
server=http.server.ThreadingHTTPServer(('127.0.0.1',60441),Handler);context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);context.load_cert_chain(tls/'server.pem',tls/'server.key');server.socket=context.wrap_socket(server.socket,server_side=True);print('Owned TLS read-only fixture 60441 ready',flush=True);server.serve_forever()
