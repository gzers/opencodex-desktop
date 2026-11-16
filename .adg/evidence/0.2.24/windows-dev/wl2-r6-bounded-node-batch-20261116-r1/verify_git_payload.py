from pathlib import Path
import hashlib,io,json,subprocess,sys
root=Path(sys.argv[1]);revision=sys.argv[2]
rows=json.loads((root/"payload-manifest.json").read_text())["files"]
p=root/"payload-manifest.json";b=p.read_bytes();rows.append(dict(path=p.name,bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
queries=[f"{revision}:{root.as_posix()}/{x['path']}" for x in rows]
raw=subprocess.check_output(["git","cat-file","--batch"],input=("\n".join(queries)+"\n").encode())
stream=io.BytesIO(raw)
for row in rows:
    header=stream.readline().split();assert header[1]==b"blob",header
    value=stream.read(int(header[2]));assert stream.read(1)==b"\n"
    assert len(value)==row["bytes"] and hashlib.sha256(value).hexdigest()==row["sha256"],row["path"]
print(json.dumps(dict(revision=revision,verified=len(rows),method="git cat-file --batch",originalBytesEqual=True)))
