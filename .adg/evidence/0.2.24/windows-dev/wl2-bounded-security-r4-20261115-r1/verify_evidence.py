from pathlib import Path
import json
root=Path(__file__).resolve().parent
for name in ["final-status.json","final-report.md","product01/result.json","product01/source-manifest.json","library08/result.json","library08/source-manifest.json","library07-scene-archive.json","library07-scene-cleanup-rejected.json"]:
    if not (root/name).is_file(): raise SystemExit(f"missing {name}")
state=json.loads((root/"final-status.json").read_text())
assert state["gates"]["allAccepted"] is False
assert state["gates"]["startOpen"] is False
assert state["cleanup"]["library07SceneRemoved"] is False
assert json.loads((root/"product01/result.json").read_text())["exitCode"]==0
assert json.loads((root/"library08/result.json").read_text())["exitCode"]==0
print("verify_evidence: 0")
