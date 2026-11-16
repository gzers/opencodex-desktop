from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
BASE="229bcb934e986b2a5a5e302b3a796884e8cb985c"
token=re.compile(r'"(?:\\.|[^"\\])*"|//[^\n]*|[A-Za-z_][A-Za-z_0-9]*|\d+|[^\s]')
def body(text,impl,name):
    start=text.index("impl "+impl+" {");match=re.search(r"fn\s+"+name+r"(?:<[^>]*>)?\s*\(",text[start:]);assert match
    words=[w for w in token.findall(text[start+match.start():]) if not w.startswith('//')]
    begin=words.index('{');depth=0;out=[]
    for w in words[begin:]:
        if w=='{':depth+=1
        if w=='}':depth-=1
        out.append(w)
        if depth==0:break
    assert depth==0
    return out
rows=[]
p="apps/desktop/tauri/src/infrastructure/windows_client_originals024/v2/routes.rs"
old=subprocess.check_output(["git","-C",str(CODE),"show",f"{BASE}:{p}"]).decode();new=(CODE/p).read_text(encoding="utf-8")
for name in ["capture","check","declared"]:
    a=body(old,"RoutePin",name);b=body(new,"RoutePin",name);assert a==b
    rows.append(dict(method=f"RoutePin::{name}",tokensEqual=True,tokens=len(a),sha256=hashlib.sha256(json.dumps(a).encode()).hexdigest()))
coord="apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination.rs"
old=subprocess.check_output(["git","-C",str(CODE),"show",f"{BASE}:{coord}"]).decode();new=(CODE/coord).read_text(encoding="utf-8")
assert re.search(r"_not_send:\s*PhantomData<\*mut\s*\(\)>",old) and re.search(r"_not_send:\s*PhantomData<\*mut\s*\(\)>",new)
for name in ["check_inner","release"]:
    a=body(old,"NativeLease",name);b=body(new,"NativeLease",name);assert a==b
    rows.append(dict(method=f"NativeLease::{name}",tokensEqual=True,tokens=len(a),sha256=hashlib.sha256(json.dumps(a).encode()).hexdigest()))
changed=subprocess.check_output(["git","-C",str(CODE),"diff","--",coord,"apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination/segments.rs"]).decode()
assert not re.search(r"^\+.*unsafe\s+impl.*Send",changed,re.M)
with (ROOT/"owned-boundary-equivalence.json").open("x",encoding="utf-8") as out:json.dump(dict(baseline=BASE,methods=rows,activeNonSendMarkerRetained=True,unsafeImplSendAdded=False,meaning="RoutePin rules unchanged except exact visibility for same-window source store; active NativeLease checking/release unchanged, historical object representation separated and native-tested"),out,indent=2)
print(json.dumps(dict(verifiedMethods=len(rows),activeMarkerRetained=True,unsafeImplSendAdded=False)))
