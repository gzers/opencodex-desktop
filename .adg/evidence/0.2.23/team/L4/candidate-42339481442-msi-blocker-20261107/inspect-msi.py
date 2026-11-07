import hashlib,json,pathlib,zipfile
root=pathlib.Path(__file__).resolve().parent
p=root/"msi-compiler-74a3f861.zip"
assert p.stat().st_size==8688 and hashlib.sha256(p.read_bytes()).hexdigest()=="95c1ef58095a98481d45c059c81bd69377b9b7441d72943a47a08a9cd017ae79"
out=root/"msi-diagnostic-original";out.mkdir(exist_ok=True)
members=[]
with zipfile.ZipFile(p) as z:
    assert len(z.infolist())<100
    for i in z.infolist():
        n=pathlib.PurePosixPath(i.filename)
        assert not n.is_absolute() and ".." not in n.parts and ":" not in i.filename and chr(92) not in i.filename
        if i.is_dir():continue
        assert i.file_size<1024**2
        if n.suffix.lower() not in [".json",".log",".txt",".wxs"]:continue
        b=z.read(i);dest=out.joinpath(*n.parts);dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(b)
        members.append({"path":i.filename,"sha256":hashlib.sha256(b).hexdigest(),"size_bytes":len(b)})
        if n.suffix in [".json",".log",".txt"]:
            text=b.decode("utf-8-sig",errors="replace")
            print(json.dumps({"path":i.filename,"tail":text.splitlines()[-28:]},ensure_ascii=True))
(root/"msi-zip-verification.json").write_text(json.dumps({"runId":42344849595,"jobId":128320263582,"sourceCommit":"74a3f8612dded1fb20d41c3a76ca3b578eed422c","artifactId":13562828954,"size_bytes":8688,"sha256":"95c1ef58095a98481d45c059c81bd69377b9b7441d72943a47a08a9cd017ae79","wholeZipSha256Verified":True,"scope":"synthetic-payload MSI compiler only; no installed application acceptance","files":members},indent=2)+"\n",encoding="utf-8",newline="\n")
