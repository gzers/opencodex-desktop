import hashlib
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parent
def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))
manifest = read("manifest.json")
actual = {path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*")
          if path.is_file() and path.relative_to(ROOT).as_posix() not in {"manifest.json", "verification.json"}}
assert actual == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
launch, exited = read("native-depth-launch.json"), read("native-depth-exit.json")
assert launch["source"] == "8be87e2d54a895d4fe02e31300100a1ffed90121"
assert launch["exeSha256"] == "26d5458d1e25ab6c9da74229e638f1c2b475ca1e1934dd0538b542d1f6bf38c9"
assert launch["applicationUser"]["elevationType"] == 3 and not launch["applicationUser"]["elevated"]
assert exited["pid"] == launch["pid"] and exited["creationTimeUtc"] == launch["creationTimeUtc"]
assert exited["exitObserved"] and exited["exitCode"] == 0 and not exited["forcedTermination"]
response = read("depth-inspection.original.json")
assert not response.get("error") and not response["result"].get("exceptionDetails")
result = response["result"]["result"]["value"]
canvas, = result["canvases"]
assert canvas["attributes"]["depth"] is False and canvas["attributes"]["stencil"] is False
assert canvas["depthBits"] == canvas["stencilBits"] == canvas["glError"] == 0
assert (canvas["width"], canvas["height"]) == (1496, 1342)
assert result["about"]["version"] == "0.1.10" and not result["runtimePatched"]
assert "commit=8be87e2d54a895d4fe02e31300100a1ffed90121" in (ROOT / "app.original.log").read_text(encoding="utf-8-sig")
summary = {"status": "verified-new-exact-candidate-native-context", "files": len(actual),
           "source": launch["source"], "actualDepthBits": 0, "drawingBuffer": [1496, 1342],
           "normalExit": True, "physicalDpiGatePassed": False,
           "memoryBudgetPassed": False, "fullNativeGatePassed": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
