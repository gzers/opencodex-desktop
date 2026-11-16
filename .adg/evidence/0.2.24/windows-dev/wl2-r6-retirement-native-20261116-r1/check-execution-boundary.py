from pathlib import Path
import json,re,subprocess,hashlib
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
base=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
path=base/'tree/anchored/observation/retirement.rs';text=path.read_text(encoding='utf-8')
rows=[]
for method in ['retire_current_file','restore_original_file']:
    assert re.search(r'^    fn '+method+r'\(',text,re.M),method
    assert not re.search(r'pub[^\n]*fn '+method+r'\(',text),method
    callers=[]
    for p in base.rglob('*.rs'):
        if p==path:continue
        for number,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1):
            if re.search(r'\.'+method+r'\s*\(',line):callers.append(dict(path=p.relative_to(CODE).as_posix(),line=number))
    assert callers and all(c['path'].endswith('retirement_tests.rs') for c in callers),callers
    rows.append(dict(method=method,private=True,actualCallers=callers,productionCallers=[]))
gate=(base/'mod.rs').read_text(encoding='utf-8');block=gate.split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
assert 'Err(' in block and 'Ok(' not in block
with (ROOT/'execution-boundary.json').open('x',encoding='utf-8') as out:
    json.dump(dict(source='worktree frozen during test run',moduleSha256=hashlib.sha256(path.read_bytes()).hexdigest(),methods=rows,publicDurableGateUnconditionallyRefuses=True,meaning='raw synthetic native tests prove mechanism; no production recovery authority or new canonical node-record execution integrated'),out,indent=2)
print(json.dumps(dict(privateRawMethodsVerified=len(rows),onlyTestCallers=True,publicGateOpen=False)))
