from pathlib import Path
from datetime import datetime
import json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
commit=json.loads((ROOT/'source-commit.json').read_text())
args=['git','-C',str(CODE),'ls-remote','origin','refs/heads/codex/024-windows-native']
p=subprocess.run(args,capture_output=True)
with (ROOT/'source-ls-remote.stdout.log').open('xb') as out:out.write(p.stdout)
with (ROOT/'source-ls-remote.stderr.log').open('xb') as out:out.write(p.stderr)
assert p.returncode==0 and p.stdout.decode().split()[0]==commit['head']
with (ROOT/'source-push-verified.json').open('x',encoding='utf-8') as out:
    json.dump(dict(time=datetime.now().astimezone().isoformat(),sourceHead=commit['head'],sourceTree=commit['tree'],command=args,
        rawExit=p.returncode,remoteBranchMatches=True,codeOnly=True,independentEvidenceCommitFollows=True),out,indent=2)
print(json.dumps(dict(head=commit['head'],remoteMatches=True)))
