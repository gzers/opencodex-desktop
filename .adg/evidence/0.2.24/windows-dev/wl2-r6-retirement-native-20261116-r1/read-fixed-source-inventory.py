from pathlib import Path
import hashlib,json,tarfile,io
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BUNDLE=Path('C:/Users/Public/ocxw1d-98b5378c87/input/runtime-bundle-0.2.24-win32-x64.tar.gz')
FIXED='14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7'
OFFICIAL='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'
EXTRACTED=CODE/'test/windows024-native/target/installer-w1/attempt01/stage/platform-bundle/runtime-bundle/runtime/opencodex/node_modules/@bitkyc08/opencodex'
def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda:source.read(1024*1024),b''):h.update(block)
    return h.hexdigest()
assert digest(BUNDLE)==FIXED
with tarfile.open(BUNDLE,'r:gz') as outer:
    matches=[m for m in outer.getmembers() if m.isfile() and m.name.endswith('runtime/opencodex/sources/opencodex-2.99.0.tgz')]
    assert len(matches)==1 and matches[0].size<128*1024*1024
    raw=outer.extractfile(matches[0]).read();assert hashlib.sha256(raw).hexdigest()==OFFICIAL
files=['src/codex/paths.ts','src/codex/journal.ts','src/codex/codex-home-owner.ts','src/codex/inject/config-write-section.ts','src/codex/catalog/write-audit.ts','src/codex/catalog/write-audit-contract.ts','src/grok/inject.ts','src/lib/redact.ts']
facts={
 'src/codex/paths.ts':['CODEX_CONFIG_PATH','CODEX_PROFILE_PATH','DEFAULT_CATALOG_PATH','CODEX_MODELS_CACHE_PATH'],
 'src/codex/codex-home-owner.ts':['CODEX_HOME_JOURNAL_FILE'],
 'src/grok/inject.ts':['const configPath = join(grokHome','const backupPath = join(grokHome','copyBackupOnce(destination'],
 'src/codex/catalog/write-audit.ts':['CODEX_CATALOG_AUDIT_FILE','redactUserPath(event.opencodexHome).slice','MAX_HOME_CHARACTERS'],
}
rows=[]
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as package:
    members=package.getmembers()
    for name in files:
        matched=[m for m in members if m.isfile() and (m.name=='package/'+name or m.name==name)]
        assert len(matched)==1 and matched[0].size<2*1024*1024
        content=package.extractfile(matched[0]).read();assert content==(EXTRACTED/name).read_bytes()
        target=ROOT/'fixed-official-source'/name;target.parent.mkdir(parents=True,exist_ok=True)
        with target.open('xb') as out:out.write(content)
        lines=content.decode('utf-8').splitlines();locations=[]
        for pattern in facts.get(name,[]):
            hits=[dict(line=i+1,sourceLine=line) for i,line in enumerate(lines) if pattern in line];assert hits,pattern
            locations.append(dict(pattern=pattern,matches=hits))
        rows.append(dict(path=name,bundleInnerMember=matched[0].name,bytes=len(content),sha256=hashlib.sha256(content).hexdigest(),extractedBytesExactlyEqual=True,evidence=target.relative_to(ROOT).as_posix(),readOnlyLocations=locations))
value=dict(bundle=str(BUNDLE),bundleSha256=FIXED,officialArchiveSha256=OFFICIAL,files=rows,
 inference='Fixed-source paths/profile/journal/config/Grok backup and redacted/truncated audit projection remain required production scope. This static inventory gives no content ownership or mutation authority, and adds no accepted node-name exception; unknown nonartifact side-effects still reject pending exact phase/transform proof.',
 officialCodeImportedOrExecuted=False,officialArchiveOrExtractedSourcesModified=False,realCredentialsReadOrWritten=False,modelOrNetworkRequest=False)
with (ROOT/'fixed-source-scope-inventory.json').open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
print(json.dumps(dict(verifiedSourceFiles=len(rows),bundleSha256=FIXED,officialArchiveSha256=OFFICIAL,readOnly=True)))
