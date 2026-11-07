import hashlib,json,pathlib,zipfile
root=pathlib.Path(__file__).resolve().parent;p=root/"ancestor-probe-two-user.zip"
assert p.stat().st_size==337598 and hashlib.sha256(p.read_bytes()).hexdigest()=="1f30fcb3352b4afb827a2990ac5791a2e548ad205bf95c8f53071bc045d742f2"
out=root/"probe-original";out.mkdir(exist_ok=True)
files=[]
with zipfile.ZipFile(p) as z:
    assert len(z.infolist())<1000
    for i in z.infolist():
        n=pathlib.PurePosixPath(i.filename);assert not n.is_absolute() and ".." not in n.parts and ":" not in i.filename and chr(92) not in i.filename
        if i.is_dir() or n.suffix not in [".json",".log"]:continue
        assert i.file_size<4*1024**2
        b=z.read(i);dest=out.joinpath(*n.parts);dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(b)
        files.append({"path":i.filename,"sha256":hashlib.sha256(b).hexdigest(),"size_bytes":len(b)})
        if n.name in ["ancestor-open-probe.json","two-user-native.json","failure.json"]:
            print(json.dumps({"path":i.filename,"data":json.loads(b.decode("utf-8-sig"))},ensure_ascii=True))
        if n.name=="app.log":print(json.dumps({"path":i.filename,"tail":b.decode("utf-8-sig",errors="replace").splitlines()[-14:]},ensure_ascii=True))
(root/"probe-member-manifest.json").write_text(json.dumps(files,indent=2)+"\n",encoding="utf-8",newline="\n")
