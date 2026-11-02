"""Owned loopback TLS WebDAV acceptance. No daily server or configuration.

Creates an ephemeral certificate and Basic-auth fixture in a temporary root,
runs the explicit production reqwest integration tests and stops the server.
"""
import base64
import hashlib
import http.server
import json
import os
from pathlib import Path
import ssl
import subprocess
import tempfile
import threading


def main():
    repo = Path(__file__).resolve().parents[2]
    with tempfile.TemporaryDirectory(prefix="ocx-owned-https-") as directory:
        root = Path(directory)
        cert, key = root / "certificate.pem", root / "key.pem"
        ca, ca_key, csr = root / "ca.pem", root / "ca-key.pem", root / "server.csr"
        config = root / "openssl.cnf"
        config.write_text("[req]\ndistinguished_name=dn\nx509_extensions=v3\nprompt=no\n[dn]\nCN=localhost\n[v3]\nsubjectAltName=DNS:localhost,IP:127.0.0.1\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,digitalSignature,keyEncipherment,keyCertSign\n")
        subprocess.run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", str(ca_key), "-out", str(ca), "-days", "1", "-config", str(config)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        config.write_text("[req]\ndistinguished_name=dn\nprompt=no\n[dn]\nCN=localhost\n[v3]\nsubjectAltName=DNS:localhost,IP:127.0.0.1\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
        subprocess.run(["openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes", "-keyout", str(key), "-out", str(csr), "-config", str(config)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(["openssl", "x509", "-req", "-in", str(csr), "-CA", str(ca), "-CAkey", str(ca_key), "-CAcreateserial", "-out", str(cert), "-days", "1", "-extfile", str(config), "-extensions", "v3"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        objects = {}
        lock = threading.Lock()
        auth = "Basic " + base64.b64encode(b"fixture-user:fixture-password").decode()

        class Handler(http.server.BaseHTTPRequestHandler):
            protocol_version = "HTTP/1.1"

            def log_message(self, *_):
                pass

            def reply(self, status, body=b"", etag=None):
                self.send_response(status)
                self.send_header("Content-Length", str(len(body)))
                if etag:
                    self.send_header("ETag", etag)
                self.end_headers()
                self.wfile.write(body)

            def admitted(self):
                if self.headers.get("Authorization") != auth or not self.path.startswith("/owned/"):
                    self.reply(401)
                    return False
                return True

            def do_GET(self):
                if not self.admitted():
                    return
                with lock:
                    item = objects.get(self.path)
                self.reply(200, item[0], item[1]) if item else self.reply(404)

            def do_PUT(self):
                if not self.admitted():
                    return
                size = int(self.headers.get("Content-Length", "0"))
                if size > 64 * 1024 * 1024:
                    self.reply(413)
                    return
                body = self.rfile.read(size)
                with lock:
                    old = objects.get(self.path)
                    match, absent = self.headers.get("If-Match"), self.headers.get("If-None-Match")
                    if (match is not None and (not old or old[1] != match)) or (absent == "*" and old):
                        self.reply(412)
                        return
                    etag = '"' + hashlib.sha256(body).hexdigest() + '"'
                    objects[self.path] = (body, etag)
                self.reply(204 if old else 201, etag=etag)

            def do_PROPFIND(self):
                if self.admitted():
                    self.reply(207, b'<?xml version="1.0"?><d:multistatus xmlns:d="DAV:"/>')

        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        server.daemon_threads = True
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(cert, key)
        server.socket = context.wrap_socket(server.socket, server_side=True)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        env = dict(os.environ, OCXD_CONFIG_WEBDAV_URL=f"https://localhost:{server.server_port}", OCXD_CONFIG_WEBDAV_PATH="owned/acceptance", OCXD_CONFIG_WEBDAV_USER="fixture-user", OCXD_CONFIG_WEBDAV_PASSWORD="fixture-password", OCXD_CONFIG_WEBDAV_CA=str(ca))
        try:
            result = subprocess.run(["cargo", "test", "--locked", "--features", "integration-test", "--test", "config_sync_webdav_real", "--", "--ignored", "--test-threads=1", "--nocapture"], cwd=repo / "apps/desktop/tauri", env=env)
        finally:
            server.shutdown()
            server.server_close()
        print(json.dumps({"scope": "owned loopback HTTPS only; synthetic credentials; temporary local configurations", "remote_objects": len(objects), "exit_code": result.returncode}))
        raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
