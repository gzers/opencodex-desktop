"""Offline composition of the adopted names, record and lifetime revisions.
The final archive is a candidate until the Desktop/native/release gates pass.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import tempfile

ROOT=Path(__file__).resolve().parents[2]
VERSION='2.97.0-desktop023.lifecycle.1'


def load(name,path):
    spec=importlib.util.spec_from_file_location(name,ROOT/path)
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value


def build(output,base):
    output=Path(output)
    assert not output.exists(),'Fresh canonical output required'
    names=load('names_runtime_builder','.github/scripts/build_runtime_023.py')
    record=load('record_runtime_builder','test/smoke/build-runtime-record-prototype.py')
    producer=load('lifetime_runtime_builder','test/smoke/build-runtime-lifetime-prototype.py')
    with tempfile.TemporaryDirectory(prefix='ocxd023-lifecycle-composition-',dir=names.TEMP_ROOT) as directory:
        work=Path(directory)
        parent=names.build(work/'names',base)
        record.build(work/'names'/parent['archive'],work/'record')
        record_report=json.loads((work/'record/verification.json').read_bytes())
        producer.build(work/'record'/record_report['archive'],work/'producer')
        producer_report=json.loads((work/'producer/verification.json').read_bytes())
        original,modes=names.read_archive((work/'producer'/producer_report['archive']).read_bytes())
        results=[]
        for _ in range(2):
            files=dict(original)
            package=json.loads(files['package.json'])
            package['version']=VERSION
            del package['desktop023RecordPrototype']
            del package['desktop023LifetimeProducerPrototype']
            package['desktop023Lifetime']={
                'schema':'ocxd-runtime-lifetime/1',
                'creationOwner':'TokenUser',
                'authority':'protected own-H binding, signed challenge, retained process/Job/lease',
                'desktopConsumerVersion':'0.2.23',
                'baseNamesArchiveSha256':parent['sha256'],
                'recordPatchSha256':record_report['patchSha256'],
                'sourceFiles':producer_report['reviewedProducerPostimages'],
                'upstreamPublished':False,
            }
            files['package.json']=names.json_bytes(package)
            compatibility=json.loads(files[names.COMPATIBILITY])
            entries={row['path']:row for row in compatibility['files']}
            entries['package.json']={**entries['package.json'],'sha256':names.sha(files['package.json'])}
            compatibility['files']=[entries[key] for key in sorted(entries)]
            files[names.COMPATIBILITY]=names.json_bytes(compatibility)
            assert {p for p in files if files[p]!=original[p]}=={'package.json',names.COMPATIBILITY}
            results.append(names.canonical_archive(files,modes))
        assert results[0]==results[1]
        decoded,_=names.read_archive(results[0]);assert decoded==files
        absent=[]
        for row in json.loads(files[names.COMPATIBILITY])['files']:
            if row['path'] in files:assert names.sha(files[row['path']])==row['sha256']
            else:absent.append(row['path'])
        assert sorted(absent)==['bun.lock','scripts/model-metadata.source.json']
        assert 'src/lib/desktop-lifecycle-trace.ts' not in files
        archive='opencodex-'+VERSION+'.tgz'
        report=dict(schemaVersion=1,kind='project-maintained-offline-runtime',maintainer='gzers/opencodex-desktop',
            installation='desktop-integrated-runtime',upstreamPublished=False,version=VERSION,archive=archive,
            sha256=names.sha(results[0]),bytes=len(results[0]),base=parent['base'],original=parent['original'],
            composition=dict(names=parent['sha256'],record=record_report['sha256'],producer=producer_report['sha256']),
            runtimeFiles={p:names.sha(files[p]) for p in ['src/config/atomic-write.ts','src/lib/windows-runtime-private-record.ts',*producer.POSTIMAGES]},
            compatibilityInheritedAbsentMembersUnchanged=absent,
            files=[dict(path=p,sha256=names.sha(files[p]),bytes=len(files[p])) for p in sorted(files)],
            reproducibility=dict(builds=2,equalBytes=True,sha256=[names.sha(value) for value in results],gzipMtime=0,tarMtime=0,tarOwner='0:0',order='sorted-path',format='pax'),
            accepted=False,diagnosticOnly=True,releaseAccepted=False)
        output.mkdir(parents=True)
        (output/archive).write_bytes(results[0]);(output/'maintained-runtime.json').write_bytes(names.json_bytes(report))
        (output/'maintained-runtime.sha256').write_text(report['sha256']+'  '+archive+'\n')
        (output/'runtime023-rebuild-summary.json').write_bytes(names.json_bytes({key:report[key] for key in ['version','archive','sha256','bytes','composition','reproducibility','accepted','releaseAccepted']}))
        return report


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('output',type=Path);p.add_argument('--archive',type=Path,required=True)
    a=p.parse_args();result=build(a.output,a.archive)
    print(json.dumps({key:result[key] for key in ['version','archive','sha256','bytes','composition','reproducibility','accepted','releaseAccepted']}))
