"""One new PATCH only after independent readback proved the failed write absent."""
import hashlib
import importlib.util
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("ghread", ROOT / "github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, current = api.request("pulls/5", token)
payload_path = ROOT / "pr-payload.json"
payload = json.loads(payload_path.read_text(encoding="utf-8"))
assert current["draft"] and not current["merged"] and current["head"]["sha"] == "8be87e2d54a895d4fe02e31300100a1ffed90121"
if current["body"] != payload["body"]:
    previous = json.loads((ROOT / "pr-after-transport-error.original.json").read_text())
    assert current["body"] == previous["body"] and current["title"] == previous["title"]
    with (ROOT / "pr-reconciled-write-attempt.json").open("x", encoding="utf-8") as stream:
        json.dump({"atUtc": datetime.now(timezone.utc).isoformat(),
                   "previousBodyVerifiedUnchangedSha256": hashlib.sha256(current["body"].encode()).hexdigest(),
                   "automaticRetryAllowed": False, "draftDescriptionOnly": True}, stream, indent=2)
    config = "\n".join([
        "url = " + json.dumps(api.BASE + "pulls/5"),
        "header = " + json.dumps("Authorization: Bearer " + token),
        'header = "Accept: application/vnd.github+json"',
        'header = "Content-Type: application/json"',
    ]) + "\n"
    result = subprocess.run([
        r"D:\Software\Develop\Git\mingw64\bin\curl.exe", "-q", "--config", "-",
        "--request", "PATCH", "--data-binary", "@" + str(payload_path),
        "-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",
        "--silent", "--show-error", "--connect-timeout", "10", "--max-time", "30",
        "--max-filesize", str(6 * 1024 * 1024), "--write-out", "%{http_code}",
        "--output", str(ROOT / "pr-git-curl-response.original.json"),
    ], input=config.encode(), capture_output=True, timeout=35)
    # Uncertain outcomes still require a read, never a second automatic PATCH.
    (ROOT / "pr-reconciled-transport.json").write_text(json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "exitCode": result.returncode,
        "httpStatus": result.stdout.decode("ascii", errors="replace"),
        "certificateChecksEnabled": True, "redirectsFollowed": False,
    }, indent=2) + "\n")
_, after = api.request("pulls/5", token)
(ROOT / "pr-after.original.json").write_text(json.dumps(after, indent=2) + "\n")
assert after["body"] == payload["body"] and after["title"] == payload["title"]
assert after["draft"] and not after["merged"] and after["auto_merge"] is None
print(json.dumps({"pr": 5, "draft": True, "head": after["head"]["sha"],
                  "bodyReadbackMatches": True, "merged": False}))
