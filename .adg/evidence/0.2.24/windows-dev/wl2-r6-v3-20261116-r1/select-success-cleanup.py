from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
names=["wl2_r6_anchored_existing_tree_continuous_claim_node_roundtrip","wl2_r6_anchored_grok_names_are_ordinary_and_large_work_file_keeps_original_ids","wl2_r6_v3_every_node_requires_full_order_and_anchors_require_release","wl2_r6_v3_full_native_plan_rejects_missing_duplicate_extra_anchor_wrong_parent_and_unknown_fields","wl2_r6_v3_node_events_reject_skip_reverse_context_drift_cross_node_and_text_closed"]
rows=[]
for run in ["library01","library03"]:
    scenes=json.loads((ROOT/f"{run}-private-scenes.json").read_text(encoding="utf-8-sig"))["scenes"]
    text=(ROOT/run/"stdout.log").read_text(encoding="utf-8-sig")
    blocks=re.findall(r"test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\ntest result:|\Z)",text,re.S)
    for name,body in blocks:
        leaf=name.rsplit("::",1)[-1]
        if leaf not in names:continue
        assert body.rstrip().endswith("ok")
        paths=re.findall(r"r4-retained-synthetic-fixture=([^\r\n]+)",body);assert len(paths)==1
        row=next(s for s in scenes if s["source"]==paths[0]);assert hashlib.sha256(Path(row["archive"]).read_bytes()).hexdigest()==row["archiveSha256"]
        dirs=[p.relative_to(Path(row["source"])).as_posix() for p in sorted(Path(row["source"]).rglob("*")) if p.is_dir()]
        expected={"Profile","Profile/.codex","Profile/.codex/empty","Profile/.codex/nested","Profile/custody","coordination"}
        if "grok_names" in leaf or "v3_" in leaf:expected|={"Profile/.grok","Profile/grok-custody"}
        assert set(dirs)==expected,(leaf,dirs)
        rows.append(dict(run=run,test=leaf,source=row["source"],archive=row["archive"],archiveSha256=row["archiveSha256"],files=row["files"],directories=dirs))
assert len(rows)==10
with (ROOT/"success-cleanup-plan.json").open("x",encoding="utf-8") as out:json.dump(rows,out,indent=2)
print(json.dumps(dict(ownCompletedScenes=len(rows),failedAndPendingRetained=60)))
