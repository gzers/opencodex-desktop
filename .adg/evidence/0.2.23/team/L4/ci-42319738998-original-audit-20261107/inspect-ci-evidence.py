import hashlib,json,pathlib,zipfile,re
root=pathlib.Path(__file__).resolve().parent
p=root/"windows-startup-evidence-42319738998.zip"
assert p.stat().st_size==23854943
with p.open("rb") as f:assert hashlib.file_digest(f,"sha256").hexdigest()=="fc4db5380ee6679a1ff0f771437852d84619441c6dfe662bb3971f6fd1ce1d45"
out=root/"receipt-text";out.mkdir(exist_ok=True)
members=[];summaries=[];total=0
with zipfile.ZipFile(p) as z:
    assert len(z.infolist())<10000
    for i in z.infolist():
        n=pathlib.PurePosixPath(i.filename)
        assert not n.is_absolute() and ".." not in n.parts and ":" not in i.filename and chr(92) not in i.filename
        assert (i.external_attr>>16)&0o170000!=0o120000
        if i.is_dir() or n.suffix not in [".json",".log",".txt"] or i.file_size>8*1024**2:continue
        if any(x in n.parts for x in ["node_modules","target","runtime-bundle"]):continue
        b=z.read(i);total+=len(b);assert total<48*1024**2
        target=out.joinpath(*n.parts);target.parent.mkdir(exist_ok=True,parents=True);target.write_bytes(b)
        members.append({"path":i.filename,"sha256":hashlib.sha256(b).hexdigest(),"size_bytes":len(b)})
        if n.name in ["rust-ordinary.json","ordinary-suite.json"]:
            v=json.loads(b.decode("utf-8-sig"))
            keys=["result","expectedCommit","exitCode","failureCode","failureType","failureLine","allBinariesAttempted","docTestsExecuted","sourceBytesUnchanged","sourceOrArtifactAclRestored","desktopAclRestored","accountAndProfileRemoved","remainingNewSidProcesses","aclRestoreErrors"]
            summaries.append({"path":i.filename,"fields":{k:v[k] for k in keys if k in v},"completedTests":len(v.get("tests",[]))})
        if n.name.endswith(".stdout.log"):
            s=b.decode("utf-8-sig",errors="replace")
            matches=re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored",s)
            if matches:summaries.append({"path":i.filename,"summaries":matches,"failures":re.findall(r"^---- (.+) stdout ----",s,re.M)})
result={"runId":42319738998,"artifactId":13553234235,"zipSize":p.stat().st_size,"sha256Verified":True,"nativeExecutablesExtractedOrRun":False,"files":members,"receiptSummaries":summaries}
(root/"audit.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(json.dumps({"verified":True,"textFiles":len(members),"bytes":total,"receipts":[x for x in summaries if "fields" in x],"failedStdout":[x for x in summaries if any(int(g[2]) for g in x.get("summaries",[]))]},ensure_ascii=True))
