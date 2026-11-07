import hashlib,json,pathlib,zipfile
root=pathlib.Path(__file__).resolve().parent
archive_path=root/'windows-candidate-evidence-42319672971.zip'
assert archive_path.stat().st_size==715014352
with archive_path.open('rb') as source:
    assert hashlib.file_digest(source,'sha256').hexdigest()=='0e9fc18ef21e9204edb535fc5e2ab594b48bb4367a88f99a393d3d18545306f5'
out=root/'receipt-text';out.mkdir(exist_ok=False)
files=[];receipts=[];test_outputs=[];total=0
with zipfile.ZipFile(archive_path) as archive:
    entries=archive.infolist()
    assert len(entries)<100000 and sum(x.file_size for x in entries)<5*1024**3
    for entry in entries:
        path=pathlib.PurePosixPath(entry.filename)
        assert not path.is_absolute() and '..' not in path.parts and ':' not in entry.filename and chr(92) not in entry.filename
        assert (entry.external_attr>>16)&0o170000!=0o120000
        if entry.is_dir() or path.suffix.lower() not in ['.json','.log','.txt'] or entry.file_size>8*1024**2:continue
        if any(x in path.parts for x in ['node_modules','runtime-bundle','target','npm-cache']):continue
        total+=entry.file_size;assert total<48*1024**2
        data=archive.read(entry);destination=out.joinpath(*path.parts)
        assert destination.resolve().is_relative_to(out.resolve())
        destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(data)
        files.append({'path':entry.filename,'sha256':hashlib.sha256(data).hexdigest(),'size_bytes':len(data)})
        if path.name in ['rust-ordinary.json','ordinary-suite.json','cleanup.json']:
            value=json.loads(data.decode('utf-8-sig'))
            if isinstance(value,dict):
                keys=['result','stage','scope','fullWorkspacePassed','expectedCommit','accountSid','tokenUser','tokenOwner','nativeSuiteExecuted','nativeExecuted','allBinariesAttempted','docTestsExecuted','exitCode','failureCode','failureType','failureLine','failureSourceLeaf','sourceBytesUnchanged','sourceOrArtifactAclRestored','desktopAclRestored','accountAndProfileRemoved','remainingNewSidProcesses','aclRestoreErrors','cleanup']
                summary={'path':entry.filename,'fields':{k:value[k] for k in keys if k in value},'keys':list(value)}
                if isinstance(value.get('tests'),list):summary['tests']=[{k:t.get(k) for k in ['target','exitCode','exitAuthority','stdout','stderr']} for t in value['tests']]
                receipts.append(summary)
        if path.name.startswith('test-') and path.name.endswith(('.stdout.log','.stderr.log')):
            text=data.decode('utf-8-sig',errors='replace');lines=text.splitlines()
            test_outputs.append({'path':entry.filename,'bytes':len(data),'results':[x for x in lines if any(term in x for term in ['test result:','FAILED','panicked at','running '])][-20:],'tail':lines[-28:]})
result={'runId':42319672971,'source':'7730073e96999382697fe2ad8dd8f877352c75a3','originalZipSha256Verified':True,'archiveMemberProgramsExecuted':False,'selectedFiles':files,'receipts':receipts,'testOutputs':test_outputs,'executablesNotExtracted':True}
(root/'original-receipt-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+chr(10),encoding='utf-8')
print(json.dumps({'textFiles':len(files),'textBytes':total,'receipts':[{k:v for k,v in r.items() if k!='tests'} for r in receipts]},ensure_ascii=True))
