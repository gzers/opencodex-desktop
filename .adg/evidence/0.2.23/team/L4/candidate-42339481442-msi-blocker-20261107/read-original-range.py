"""Read bounded original JSON/text members; no whole-ZIP digest claim."""
import datetime,hashlib,io,json,os,pathlib,re,subprocess,urllib.request,urllib.error,urllib.parse,zipfile
ROOT=pathlib.Path(__file__).resolve().parent
REPO=r"C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code"
ARTIFACT=13562902460
SIZE=715103222
EXPECTED_SHA="5ec3a80ef2f1ce0aa8140b7fb13a2f89df7e442833f08ed0c2523da90e9f552a"
SOURCE="9437625f4d5432a2fcbc5918fdc4904b6baf7fb9"
class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,*args):return None
class Remote(io.RawIOBase):
    def __init__(self,url):self.url=url;self.pos=0;self.transferred=0;self.calls=0
    def seekable(self):return True
    def readable(self):return True
    def tell(self):return self.pos
    def seek(self,offset,whence=0):
        self.pos=offset if whence==0 else self.pos+offset if whence==1 else SIZE+offset
        assert 0<=self.pos<=SIZE
        return self.pos
    def read(self,n=-1):
        if n<0:n=SIZE-self.pos
        n=min(n,SIZE-self.pos)
        if not n:return b""
        assert n<=12*1024**2,"Refuse unbounded range"
        end=self.pos+n-1
        req=urllib.request.Request(self.url,headers={"Range":f"bytes={self.pos}-{end}","User-Agent":"OpenCodex-L4-range-audit","Accept-Encoding":"identity"})
        with urllib.request.urlopen(req,timeout=40) as response:
            assert response.status==206,"Range not honored"
            assert response.headers.get("Content-Range")==f"bytes {self.pos}-{end}/{SIZE}","Range identity mismatch"
            data=response.read(n+1)
        assert len(data)==n
        self.pos+=n;self.transferred+=n;self.calls+=1
        assert self.transferred<40*1024**2
        return data
def main():
    env=os.environ.copy();env["GIT_TERMINAL_PROMPT"]="0";env["GCM_INTERACTIVE"]="never"
    raw=subprocess.run(["git","-C",REPO,"credential","fill"],input=b"protocol=https\nhost=github.com\npath=gzers/opencodex-desktop.git\n\n",capture_output=True,env=env)
    assert raw.returncode==0
    fields=dict(x.split("=",1) for x in raw.stdout.decode().splitlines() if "=" in x)
    secret=fields.get("password");assert secret
    req=urllib.request.Request(f"https://api.github.com/repos/gzers/opencodex-desktop/actions/artifacts/{ARTIFACT}/zip",headers={"Authorization":"Bearer "+secret,"User-Agent":"OpenCodex-L4-range-audit","Accept":"application/vnd.github+json"})
    try:
        urllib.request.build_opener(NoRedirect).open(req,timeout=35)
        raise RuntimeError("Expected artifact redirect")
    except urllib.error.HTTPError as e:
        assert e.code in [301,302,303,307,308]
        location=e.headers.get("Location","")
    assert urllib.parse.urlparse(location).scheme=="https"
    remote=Remote(location);members=[]
    out=ROOT/"original";out.mkdir(exist_ok=True)
    with zipfile.ZipFile(remote) as z:
        entries=z.infolist();assert len(entries)<100000
        for entry in entries:
            p=pathlib.PurePosixPath(entry.filename)
            assert not p.is_absolute() and ".." not in p.parts and ":" not in entry.filename and chr(92) not in entry.filename
            assert (entry.external_attr>>16)&0o170000!=0o120000
            if entry.is_dir() or entry.file_size>8*1024**2:continue
            if not (entry.filename.startswith("ordinary-rust/") and (p.name in ["rust-ordinary.json","frozen-artifacts.json","ordinary-suite.json"] or re.fullmatch(r"test-\d+\.json",p.name))):continue
            b=z.read(entry);assert len(b)==entry.file_size
            dest=out.joinpath(*p.parts);dest.parent.mkdir(exist_ok=True,parents=True);dest.write_bytes(b)
            members.append({"path":entry.filename,"size_bytes":len(b),"sha256":hashlib.sha256(b).hexdigest(),"zipCrc32":f"{entry.CRC:08x}","memberCrcAndSizeVerified":True})
    def read(rel):return json.loads((out/rel).read_text(encoding="utf-8-sig"))
    parent=read("ordinary-rust/rust-ordinary.json");child=read("ordinary-rust/ordinary/ordinary-suite.json");frozen=read("ordinary-rust/frozen-artifacts.json")
    assert parent["expectedCommit"]==child["expectedCommit"]==SOURCE
    tests=child["tests"];assert len(tests)==len(frozen["artifacts"])==49
    assert all(type(t["exitCode"]) is int and t["exitCode"]==0 and t["exitAuthority"]=="retained-CreateProcess-handle" for t in tests)
    assert child["allBinariesAttempted"] is True and child["exitCode"]==0
    assert child["tokenUser"]==child["tokenOwner"]==child["accountSid"]
    for key in ["sourceBytesUnchanged","sourceOrArtifactAclRestored","desktopAclRestored","accountAndProfileRemoved"]:assert parent[key] is True
    assert parent["remainingNewSidProcesses"]==[] and parent["aclRestoreErrors"]==[]
    result={"observedUtc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runId":42339481442,"jobId":128303762460,"sourceCommit":SOURCE,"artifactId":ARTIFACT,"expectedZipSize":SIZE,"expectedWholeZipSha256":EXPECTED_SHA,"wholeZipSha256Verified":False,"method":"bounded original HTTP ranges; zipfile member CRC and declared size plus local SHA256","rangeBytesTransferred":remote.transferred,"httpRanges":remote.calls,"originalMembers":members,"ordinaryBinaries":{"expected":49,"completedExitReceipts":49,"allAuthoritativeIntegerZero":True,"allBinariesAttempted":True,"docTestsExecuted":child.get("docTestsExecuted"),"fullWorkspacePassed":child.get("fullWorkspacePassed"),"result":child["result"]},"cleanupVerifiedFromParent":True,"ordinaryIdentityMatches":True,"nativeProgramsExtractedOrExecuted":False,"candidateJobResult":"failure","failedStep":"Build NSIS and MSI candidates","installerAcceptanceExecuted":False}
    (ROOT/"verification.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8",newline="\n")
    print(json.dumps({"verifiedOriginalMembers":len(members),"binaryExitReceipts":49,"allIntegerZero":True,"cleanupVerified":True,"wholeZipSha256Verified":False,"rangeBytesTransferred":remote.transferred,"docTestsExecuted":child.get("docTestsExecuted")}))
try:main()
except Exception as e:
    (ROOT/"range-read-failure.json").write_text(json.dumps({"type":type(e).__name__,"message":"Range audit failed; signed URLs and credentials withheld"})+"\n",encoding="utf-8")
    print(json.dumps({"auditFailed":True,"type":type(e).__name__}));raise SystemExit(1)
