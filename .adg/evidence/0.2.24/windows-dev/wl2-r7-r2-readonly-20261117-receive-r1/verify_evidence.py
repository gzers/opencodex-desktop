from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
manifest_path=ROOT/"payload-manifest.json"
if "--seal" in sys.argv:
    assert not manifest_path.exists()
    files=[dict(path=p.relative_to(ROOT).as_posix(),bytes=p.stat().st_size,sha256=sha(p.read_bytes())) for p in sorted(ROOT.rglob("*")) if p.is_file() and p!=manifest_path]
    with manifest_path.open("x",encoding="utf-8") as out:json.dump(dict(files=files,nativeAuthority=False),out,indent=2)
manifest=json.loads(manifest_path.read_text())
expected=set()
for row in manifest["files"]:
    path=(ROOT/row["path"]).resolve();assert path.is_relative_to(ROOT) and path!=manifest_path
    assert row["path"] not in expected;expected.add(row["path"])
    raw=path.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
assert expected=={p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p!=manifest_path}
print(json.dumps(dict(verified=len(expected),bytes=sum(f["bytes"] for f in manifest["files"]),nativeAuthority=False)))
