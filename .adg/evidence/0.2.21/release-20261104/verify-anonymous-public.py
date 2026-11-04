"""Read exact public assets and both updater endpoints without credentials."""
from pathlib import Path
import argparse, hashlib, json, subprocess, urllib.parse, datetime

p=argparse.ArgumentParser()
p.add_argument('--asset-report',required=True,type=Path)
p.add_argument('--out',required=True,type=Path)
a=p.parse_args()
expected=json.loads(a.asset_report.read_text())
assert expected['sourceCommit']=='6e2197fd0b8a36dcb8d893b62693c5a2a77151cc'
assert expected['releaseId']==433749817 and len(expected['assets'])==8
a.out.mkdir(parents=True,exist_ok=False)
base='https://github.com/gzers/opencodex-desktop/releases'
rows=[]
def download(url,name):
    dest=a.out/name
    r=subprocess.run(['curl','--fail','--location','--silent','--show-error','--proto','=https',
        '--connect-timeout','10','--max-time','50','--retry','2','--retry-delay','1',
        '--output',str(dest),'--write-out','%{url_effective}',url],capture_output=True,text=True)
    (a.out/(name+'.download.log')).write_text(r.stdout+chr(10)+r.stderr)
    assert r.returncode==0,(url,r.returncode,r.stderr)
    assert r.stdout.startswith('https://')
    return dest
for row in expected['assets']:
    url=base+'/download/v0.2.21/'+urllib.parse.quote(row['name'])
    f=download(url,row['name'])
    actual=hashlib.sha256(f.read_bytes()).hexdigest()
    assert actual==row['sha256'] and f.stat().st_size==row['bytes'],row['name']
    rows.append(dict(row,anonymousUrl=url,anonymousDownloadPassed=True))
tag=json.loads((a.out/'latest.json').read_text())
latest=download(base+'/latest/download/latest.json','latest-stable.json')
assert json.loads(latest.read_text())==tag and tag['version']=='0.2.21'
result={'result':'pass','sourceCommit':expected['sourceCommit'],'releaseId':expected['releaseId'],
    'tag':'v0.2.21','latest':'v0.2.21','anonymousTagReadback':True,'anonymousLatestReadback':True,
    'sameAssetIdentity':True,'credentialHeadersUsed':False,'assets':rows,
    'manifestSha256':hashlib.sha256((a.out/'latest.json').read_bytes()).hexdigest(),
    'verifiedAt':datetime.datetime.now(datetime.timezone.utc).isoformat()}
(a.out/'anonymous-public-verification.json').write_text(json.dumps(result,indent=2)+chr(10))
print(json.dumps(result,indent=2))
