import http.server,json,os
from pathlib import Path
b=Path(__file__).resolve().parent;log=b/'models-reviewed-requests.jsonl'
class Handler(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_GET(self):
  auth=self.headers.get('Authorization')=='Bearer synthetic-l2-key'
  with log.open('a',encoding='utf-8')as f:f.write(json.dumps({'method':'GET','path':self.path,'authenticated':auth})+chr(10))
  if self.path!='/v1/models' or not auth:self.send_error(401);return
  raw=json.dumps({'data':[{'id':'vendor/native-model'}]}).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(raw)));self.end_headers();self.wfile.write(raw)
 def do_POST(self):
  with log.open('a',encoding='utf-8')as f:f.write(json.dumps({'method':'POST','path':self.path})+chr(10))
  self.send_error(405)
server=http.server.ThreadingHTTPServer(('127.0.0.1',60443),Handler)
(b/'models-reviewed-identity.json').write_text(json.dumps({'pid':os.getpid(),'port':60443,'script':str(Path(__file__).resolve())}),encoding='utf-8')
print('models-only loopback ready',flush=True);server.serve_forever()
