"""Loopback TLS catalogue fixture. GET only; never proxies or mutates objects."""
import argparse
import base64
import hashlib
import hmac
import http.server
import json
from pathlib import Path
import ssl
import threading
from datetime import datetime, timezone


class FixtureServer(http.server.ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, root, stage_file, log_file, certificate, key, port=60441):
        self.root = Path(root).resolve(strict=True)
        self.stage_file = Path(stage_file).resolve(strict=True)
        self.log_file = Path(log_file).resolve()
        # Root admission is independent of any HTTP request or app selection.
        for path in [self.root, self.stage_file, self.log_file]:
            if 'post021-msvc-followup' not in path.parts:
                raise ValueError('explicit isolated post021-msvc-followup path required')
        manifest = json.loads((self.root / 'fixture-manifest.json').read_bytes())
        if set(manifest['stages']) != {'initial', 'later'}:
            raise ValueError('two-stage synthetic catalogue required')
        self.objects = {}
        for stage, metadata in manifest['stages'].items():
            folder = self.root / stage
            pointer_bytes = (folder / 'config-v1-latest.json').read_bytes()
            pointer = json.loads(pointer_bytes)
            body = (folder / metadata['filename']).read_bytes()
            digest = 'sha256:' + hashlib.sha256(body).hexdigest()
            if digest != metadata['digest'] or digest != pointer['digest']:
                raise ValueError('fixture pointer/object digest mismatch')
            envelope = json.loads(body)
            document = base64.b64decode(envelope['payload']['document_b64'], validate=True)
            if envelope['manifest']['secretObjects'] != 0 or envelope['manifest']['documentDigest'] != 'sha256:' + hashlib.sha256(document).hexdigest():
                raise ValueError('credential-free valid document required')
            values = {'config-v1-latest.json': pointer_bytes, metadata['filename']: body}
            etags = json.loads((folder / 'etags.json').read_bytes())
            for name, value in values.items():
                if etags[name] != '"' + hashlib.sha256(value).hexdigest() + '"':
                    raise ValueError('strong fixture ETag mismatch')
            self.objects[stage] = {name: (value, etags[name]) for name, value in values.items()}
        self.log_lock = threading.Lock()
        self.log_file.parent.mkdir(parents=True, exist_ok=True)
        # New log required: a replay never silently overwrites prior evidence.
        self.log_file.open('x', encoding='utf-8').close()
        super().__init__(('127.0.0.1', port), FixtureHandler)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(certificate, key)
        self.socket = context.wrap_socket(self.socket, server_side=True)

    def record(self, request, status, stage, authenticated):
        row = {'at': datetime.now(timezone.utc).isoformat(), 'method': request.command,
               'path': request.path, 'status': status, 'stage': stage,
               'authenticated': authenticated}
        with self.log_lock, self.log_file.open('a', encoding='utf-8', newline=chr(10)) as stream:
            stream.write(json.dumps(row, ensure_ascii=False) + chr(10))


class FixtureHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def respond(self, status, body=b'', etag=None, stage=None, authenticated=False):
        self.server.record(self, status, stage, authenticated)
        self.send_response(status)
        self.send_header('Content-Length', str(len(body)))
        self.send_header('Connection', 'close')
        if etag:
            self.send_header('ETag', etag)
        if status == 401:
            self.send_header('WWW-Authenticate', 'Basic realm="synthetic-post021"')
        if status == 405:
            self.send_header('Allow', 'GET')
        self.end_headers()
        if self.command != 'HEAD':
            self.wfile.write(body)
        self.close_connection = True

    def do_GET(self):
        expected = 'Basic ' + base64.b64encode(b'synthetic-post021:synthetic-post021-password').decode()
        authenticated = hmac.compare_digest(self.headers.get('Authorization', ''), expected)
        if not authenticated:
            self.respond(401)
            return
        stage = self.server.stage_file.read_text(encoding='utf-8').strip()
        if stage not in self.server.objects:
            self.respond(503, stage=stage, authenticated=True)
            return
        name = self.path.removeprefix('/native021-only/')
        item = self.server.objects[stage].get(name) if self.path.startswith('/native021-only/') else None
        if item is None:
            self.respond(404, stage=stage, authenticated=True)
            return
        body, etag = item
        self.respond(200, body, etag, stage, True)

    def reject(self):
        self.respond(405)

    do_HEAD = do_PUT = do_POST = do_DELETE = do_PATCH = do_MKCOL = do_PROPFIND = reject


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['root', 'stage-file', 'log-file', 'certificate', 'key']:
        parser.add_argument('--' + name, required=True)
    args = parser.parse_args()
    with FixtureServer(args.root, args.stage_file, args.log_file, args.certificate, args.key) as server:
        print('Synthetic GET-only TLS fixture ready at 127.0.0.1:60441', flush=True)
        server.serve_forever()
