from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
rows=[];all_scenes=[]
for run in ["library01","library02","library03","library04","library05","library06","library07"]:
    scenes=json.loads((ROOT/f"{run}-private-scenes.json").read_text(encoding="utf-8-sig"))["scenes"];all_scenes.extend(scenes)
    text=(ROOT/run/"stdout.log").read_text(encoding="utf-8-sig")
    for name,body in re.findall(r"test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)",text,re.S):
        leaf=name.rsplit("::",1)[-1]
        if leaf!="wl2_r6_work_observation_data_rejects_incomplete_context_skip_changed_reaccept_and_text_permission":continue
        assert body.rstrip().endswith("ok")
        paths=re.findall(r"r4-retained-synthetic-fixture=([^\r\n]+)",body);assert len(paths)==1
        row=next(s for s in scenes if s["source"]==paths[0]);assert hashlib.sha256(Path(row["archive"]).read_bytes()).hexdigest()==row["archiveSha256"]
        dirs=[p.relative_to(Path(row["source"])).as_posix() for p in sorted(Path(row["source"]).rglob("*")) if p.is_dir()]
        expected={"Profile","Profile/.codex","Profile/.codex/empty","Profile/.codex/nested","Profile/custody","coordination","Profile/.grok","Profile/grok-custody"}
        assert set(dirs)==expected,(leaf,dirs)
        rows.append(dict(run=run,test=leaf,source=row["source"],archive=row["archive"],archiveSha256=row["archiveSha256"],files=row["files"],directories=dirs))
assert len(rows)==4
with (ROOT/"success-cleanup-plan.json").open("x",encoding="utf-8") as out:json.dump(rows,out,indent=2)
counts=dict(ownCompletedScenes=len(rows),failedAndPendingRetained=len(all_scenes)-len(rows),crossVolumePendingRootsRetained=sum(s["source"].startswith("D:") for s in all_scenes))
with (ROOT/"cleanup-counts.json").open("x",encoding="utf-8") as out:json.dump(counts,out,indent=2)
print(json.dumps(counts))
