import hashlib,json,pathlib,subprocess,zipfile,os
root=pathlib.Path(__file__).resolve().parent
archive=root/"msi-compiler-e2929112.zip"
assert archive.stat().st_size==129319 and hashlib.sha256(archive.read_bytes()).hexdigest()=="2336d9281d837f256a91e16fd1422e88d6722cf06a06db647dab56f40399f804"
out=root/"msi-fixed-original";out.mkdir(exist_ok=True)
files=[]
with zipfile.ZipFile(archive) as z:
    assert len(z.infolist())<100
    for i in z.infolist():
        p=pathlib.PurePosixPath(i.filename)
        assert not p.is_absolute() and ".." not in p.parts and ":" not in i.filename and chr(92) not in i.filename
        if i.is_dir() or p.suffix.lower() not in [".json",".log",".wxs",".txt"]:continue
        assert i.file_size<1024**2
        b=z.read(i);dest=out.joinpath(*p.parts);dest.parent.mkdir(exist_ok=True,parents=True);dest.write_bytes(b)
        files.append({"path":p.as_posix(),"size_bytes":len(b),"sha256":hashlib.sha256(b).hexdigest()})
result=json.loads((out/"compiler-result.json").read_text(encoding="utf-8-sig"))
assert result["source"]=="e29291120d0dbfbd3c23f9f5aba71d327ee6aaa3"
assert result["productAcceptance"] is False and result["installerExecuted"] is False
assert result["bundleExit"]==0 and result["candleExit"]==0
print(json.dumps({"compilerResult":result,"textMembers":len(files),"zipVerified":True},ensure_ascii=True))
verification={"runId":42345059395,"jobId":128320909246,"artifactId":13563036574,"sourceCommit":result["source"],"size_bytes":129319,"sha256":"2336d9281d837f256a91e16fd1422e88d6722cf06a06db647dab56f40399f804","wholeZipSha256Verified":True,"programsExtractedOrRun":False,"compilerResult":result,"files":files}
(root/"msi-fixed-verification.json").write_text(json.dumps(verification,indent=2)+"\n",encoding="utf-8",newline="\n")
