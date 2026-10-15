import importlib.util
import json
from datetime import datetime, timezone
from pathlib import Path
ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("ghread", ROOT / "github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, before = api.request("pulls/5", token)
assert before["draft"] and before["state"] == "open" and not before["merged"]
assert before["head"]["sha"] == "8be87e2d54a895d4fe02e31300100a1ffed90121"
payload = json.loads((ROOT / "pr-payload.json").read_text(encoding="utf-8"))
if not (ROOT / "pr-update-attempt.json").exists():
    (ROOT / "pr-update-attempt.json").write_text(json.dumps({
        "atUtc": datetime.now(timezone.utc).isoformat(), "pr": 5,
        "scope": "draft title and body only", "automaticMutationRetryAllowed": False,
    }, indent=2) + "\n")
    _, response = api.request("pulls/5", token, payload, "PATCH")
    (ROOT / "pr-update-response.original.json").write_text(json.dumps(response, indent=2) + "\n")
_, after = api.request("pulls/5", token)
assert after["title"] == payload["title"] and after["body"] == payload["body"]
assert after["draft"] and not after["merged"] and after["auto_merge"] is None
(ROOT / "pr-after.original.json").write_text(json.dumps(after, indent=2) + "\n")
print(json.dumps({"pr": 5, "draft": after["draft"], "merged": after["merged"],
                  "head": after["head"]["sha"], "titleAndBodyReadbackMatched": True}))
