from pathlib import Path
import hashlib,io,json,subprocess,sys
root=Path(sys.argv[1]).resolve()
revision=sys.argv[2]
round_dir=Path(__file__).resolve().parent
reports=[]
for name in ['library03','product01','product02']:
    rows=json.loads((round_dir/name/'source-manifest.json').read_text(encoding='utf-8-sig'))
    query=''.join(revision+':'+r['path']+chr(10) for r in rows).encode()
    raw=subprocess.check_output(['git','-C',str(root),'cat-file','--batch'],input=query)
    stream=io.BytesIO(raw)
    raw_differences=[]
    for row in rows:
        header=stream.readline().split();assert header[1]==b'blob',header
        blob=stream.read(int(header[2]));assert stream.read(1)==bytes([10])
        snapshot=(round_dir/name/'executed-source'/row['path']).read_bytes()
        assert len(snapshot)==row['bytes'] and hashlib.sha256(snapshot).hexdigest()==row['sha256'],row['path']
        assert snapshot==(root/row['path']).read_bytes(),row['path']
        assert snapshot.replace(bytes([13,10]),bytes([10]))==blob.replace(bytes([13,10]),bytes([10])),(name,row['path'])
        if snapshot!=blob:raw_differences.append(row['path'])
    reports.append(dict(run=name,inputs=len(rows),snapshotBytesVerified=True,
        currentSourceMatchesSnapshot=True,committedSourceMatchesAfterOnlyCRLFNormalization=True,
        rawLineEndingDifferences=raw_differences))
print(json.dumps(dict(sourceCommit=revision,runs=reports,nativeAuthority=False),indent=2))
