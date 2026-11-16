from pathlib import Path
import hashlib,json,tarfile,io
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
archive=Path('C:/Users/Public/ocxw1d-98b5378c87/input/runtime-bundle-0.2.24-win32-x64.tar.gz')
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(archive.read_bytes())=='14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7'
paths=['src/lib/redact.ts','src/lib/redact-folding.ts','src/codex/catalog/write-audit.ts','src/codex/catalog/write-audit-contract.ts']
with tarfile.open(archive,'r:gz') as bundle:
    candidates=[n for n in bundle.getnames() if n.endswith('runtime/opencodex/sources/opencodex-2.99.0.tgz')];assert len(candidates)==1
    raw=bundle.extractfile(candidates[0]).read()
assert sha(raw)=='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'
package=CODE/'test/windows024-native/target/installer-w1/attempt01/stage/platform-bundle/runtime-bundle/runtime/opencodex/node_modules/@bitkyc08/opencodex'
rows=[]
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as official:
    for path in paths:
        member='package/'+path;value=official.extractfile(member).read();extracted=(package/path).read_bytes();assert value==extracted
        target=ROOT/'fixed-official-source'/path;target.parent.mkdir(parents=True,exist_ok=True)
        with target.open('xb') as out:out.write(value)
        lines=value.decode('utf-8').splitlines();matches=[]
        for number,line in enumerate(lines,1):
            if any(s in line for s in ['USER_HOME_PATH_PATTERNS','redactUserPath','MAX_HOME_CHARACTERS','new Date().toISOString()','opencodexHome:','SENSITIVE_SEGMENT_PATTERN']):
                matches.append(dict(line=number,text=line))
        rows.append(dict(path=path,originalArchiveMember=member,bytes=len(value),sha256=sha(value),exactBundleAndExtractedBytes=True,
            evidence=target.relative_to(ROOT).as_posix(),sourceLocations=matches))
with (ROOT/'fixed-audit-source-manifest.json').open('x',encoding='utf-8') as out:json.dump(dict(bundle=str(archive),
    bundleSha256='14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7',
    officialArchiveSha256='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c',files=rows,
    officialFilesModified=False,officialServiceOrModelRequests=False,sourceReadOnly=True),out,ensure_ascii=False,indent=2)
print(json.dumps(dict(fixedOfficialSources=len(rows),exactBundleAndExtractedBytes=True,officialModified=False)))
