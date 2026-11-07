"""Run the v2 production transport against an owned ephemeral TLS WebDAV.

Use the Python environment from webdav-requirements.txt. All certificates,
remote files and server processes are removed; only logs/cleanup facts remain.
"""
import argparse
import json
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument("--evidence", type=Path, required=True)
args = parser.parse_args()
evidence = args.evidence.resolve()
evidence.mkdir(parents=True, exist_ok=True)
support = Path(__file__).resolve().parent
crate = support.parents[1]
result = 1
server = None
with tempfile.TemporaryDirectory(prefix="ocxd023-dav-owned-") as temp:
    root = Path(temp)
    remote = root / "remote"
    (remote / "v2-owned").mkdir(parents=True)
    certificate, key = root / "certificate.pem", root / "private.pem"
    ca, ca_key, csr = root / "ca.pem", root / "ca-private.pem", root / "server.csr"
    extension = root / "server.ext"
    extension.write_text("basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n")
    with (evidence / "certificate-generation.log").open("wb") as log:
        subprocess.run([
            "openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
            "-days", "1", "-subj", "/CN=OwnedFixtureCA",
            "-addext", "basicConstraints=critical,CA:TRUE",
            "-keyout", str(ca_key), "-out", str(ca),
        ], stdout=log, stderr=subprocess.STDOUT, check=True)
        subprocess.run([
            "openssl", "req", "-newkey", "rsa:2048", "-nodes", "-subj", "/CN=localhost",
            "-keyout", str(key), "-out", str(csr),
        ], stdout=log, stderr=subprocess.STDOUT, check=True)
        subprocess.run([
            "openssl", "x509", "-req", "-in", str(csr), "-CA", str(ca), "-CAkey", str(ca_key),
            "-CAcreateserial", "-days", "1", "-extfile", str(extension), "-out", str(certificate),
        ], stdout=log, stderr=subprocess.STDOUT, check=True)
    try:
        with (evidence / "server.log").open("wb") as server_log:
            server = subprocess.Popen([
                sys.executable, str(support / "local_webdav.py"),
                str(remote), str(certificate), str(key),
            ], stdout=subprocess.PIPE, stderr=server_log)
            selector = selectors.DefaultSelector()
            selector.register(server.stdout, selectors.EVENT_READ)
            if not selector.select(timeout=15):
                raise RuntimeError("owned fixture did not become ready")
            line = server.stdout.readline()
            selector.close()
            ready = json.loads(line)
            env = os.environ.copy()
            env.update({"OCXD_V2_FIXTURE_URL": ready["url"], "OCXD_V2_FIXTURE_CA": str(ca)})
            with (evidence / "transport-tests.log").open("wb") as log:
                result = subprocess.run([
                    "cargo", "test", "--offline", "--features", "integration-test",
                    "--test", "config_v2_transport_real", "--", "--ignored", "--nocapture",
                ], cwd=crate, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=300).returncode
    finally:
        if server is not None:
            server.terminate()
            try:
                server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait(timeout=5)
            if server.stdout:
                server.stdout.close()
(evidence / "cleanup.json").write_text(json.dumps({
    "exitCode": result, "serverStopped": server is None or server.poll() is not None,
    "ownedTemporaryRootRemoved": not Path(temp).exists(),
    "dailyConfigurationTouched": False, "businessEndpointUsed": False,
}, indent=2) + "\n")
sys.exit(result)
