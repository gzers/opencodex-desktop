"""Owned TLS WebDAV fixture; dependency installation is deliberately external.

Run with Python that has wsgidav and cheroot. Arguments are a temporary owned
root and a temporary cert/key pair; the fixture binds only 127.0.0.1.
"""
import json
from pathlib import Path
import sys

from cheroot import wsgi
from cheroot.ssl.builtin import BuiltinSSLAdapter
from wsgidav.wsgidav_app import WsgiDAVApp

root, certificate, key = map(Path, sys.argv[1:4])
if not root.is_absolute() or not root.is_dir():
    raise SystemExit("provide an existing absolute owned temporary root")
app = WsgiDAVApp({
    "provider_mapping": {"/": str(root)},
    "simple_dc": {"user_mapping": {"*": {"fixture": {"password": "fixture-only-password", "roles": []}}}},
    "http_authenticator": {"accept_basic": True, "accept_digest": False, "default_to_digest": False},
    "dir_browser": {"enable": False},
    "verbose": 0,
})
server = wsgi.Server(("127.0.0.1", 0), app)
server.ssl_adapter = BuiltinSSLAdapter(str(certificate), str(key))
server.prepare()
print(json.dumps({"url": "https://localhost:" + str(server.bind_addr[1]), "root": str(root)}), flush=True)
server.serve()
