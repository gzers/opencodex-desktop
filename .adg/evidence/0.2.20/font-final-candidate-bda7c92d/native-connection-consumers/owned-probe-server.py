from pathlib import Path
import subprocess,ssl,http.server,json,base64,os
root=Path("/tmp/ocx020-native-connection-probe")
root.mkdir(exist_ok=True)
def run(args): subprocess.run(args,check=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
run(["openssl","req","-x509","-newkey","rsa:2048","-nodes","-keyout",str(root/"ca.key"),"-out",str(root/"ca.pem"),"-subj","/CN=OCX Owned Native Probe CA","-days","1","-addext","basicConstraints=critical,CA:TRUE"])
run(["openssl","req","-newkey","rsa:2048","-nodes","-keyout",str(root/"server.key"),"-out",str(root/"server.csr"),"-subj","/CN=localhost"])
(root/"server.ext").write_text("subjectAltName=DNS:localhost,IP:127.0.0.1\nbasicConstraints=critical,CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
run(["openssl","x509","-req","-in",str(root/"server.csr"),"-CA",str(root/"ca.pem"),"-CAkey",str(root/"ca.key"),"-CAcreateserial","-out",str(root/"server.pem"),"-days","1","-extfile",str(root/"server.ext")])
expected="Basic "+base64.b64encode(b"owned-native-user:owned-native-password-020").decode()
class Handler(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args): pass
 def do_PROPFIND(self):
  ok=self.headers.get("Authorization")==expected
  with (root/"requests.jsonl").open("a") as log: log.write(json.dumps({"method":self.command,"path":self.path,"authenticationMatched":ok})+"\n")
  body=b'<d:multistatus xmlns:d="DAV:"></d:multistatus>' if ok else b"owned fixture credentials refused"
  self.send_response(207 if ok else 401);self.send_header("Content-Type","application/xml");self.send_header("Content-Length",str(len(body)));self.end_headers();self.wfile.write(body)
 def do_GET(self):
  self.send_response(404);self.end_headers()
server=http.server.ThreadingHTTPServer(("127.0.0.1",0),Handler)
ctx=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);ctx.load_cert_chain(root/"server.pem",root/"server.key");server.socket=ctx.wrap_socket(server.socket,server_side=True)
(root/"server.json").write_text(json.dumps({"pid":os.getpid(),"url":"https://127.0.0.1:"+str(server.server_port),"onlyProbe":True,"noPutImplementation":True},indent=2))
print(server.server_port,flush=True)
server.serve_forever()
