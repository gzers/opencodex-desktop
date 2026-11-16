from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
names={"wl2_r6_stop_action_data_exact_ready_context_order_no_replay_or_deadline_laundering","wl2_r6_stop_phase_data_exact_prestop_teardown_stopped_and_ready_generation","wl2_r6_release_history_fresh_read_after_original_expiry_does_not_reauthorize_lease","wl2_r6_snapshot_data_strict_both_roots_context_and_reference_rejects_text_native_authority","wl2_r6_v3_every_node_requires_full_order_and_anchors_require_release","wl2_r6_v3_full_native_plan_rejects_missing_duplicate_extra_anchor_wrong_parent_and_unknown_fields","wl2_r6_v3_node_events_reject_skip_reverse_context_drift_cross_node_and_text_closed","wl2_r6_lease_data_same_chain_rejects_bad_state_epoch_scope_digest_and_node_escape","wl2_r6_work_observation_data_rejects_incomplete_context_skip_changed_reaccept_and_text_permission"}
rows=[];all_scenes=[]
for run in ["library01","library02","library03"]:
    scenes=json.loads((ROOT/f"{run}-private-scenes.json").read_text(encoding="utf-8-sig"))["scenes"];all_scenes.extend(scenes)
    text=(ROOT/run/"stdout.log").read_text(encoding="utf-8-sig")
    for name,body in re.findall(r"test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)",text,re.S):
        leaf=name.rsplit("::",1)[-1]
        if leaf not in names:continue
        assert body.rstrip().endswith("ok")
        paths=re.findall(r"r4-retained-synthetic-fixture=([^\r\n]+)",body);assert len(paths)==1
        row=next(s for s in scenes if s["source"]==paths[0]);assert hashlib.sha256(Path(row["archive"]).read_bytes()).hexdigest()==row["archiveSha256"]
        dirs=[p.relative_to(Path(row["source"])).as_posix() for p in sorted(Path(row["source"]).rglob("*")) if p.is_dir()]
        expected={"synthetic-locks"} if "release_history" in leaf else {"Profile","Profile/.codex","Profile/.codex/empty","Profile/.codex/nested","Profile/custody","coordination","Profile/.grok","Profile/grok-custody"}
        assert set(dirs)==expected,(leaf,dirs)
        rows.append(dict(run=run,test=leaf,source=row["source"],archive=row["archive"],archiveSha256=row["archiveSha256"],files=row["files"],directories=dirs))
assert len(rows)==12
with (ROOT/"success-cleanup-plan.json").open("x",encoding="utf-8") as out:json.dump(rows,out,indent=2)
counts=dict(ownCompletedScenes=len(rows),failedAndPendingRetained=len(all_scenes)-len(rows),crossVolumePendingRootsRetained=sum(s["source"].startswith("D:") for s in all_scenes))
with (ROOT/"cleanup-counts.json").open("x",encoding="utf-8") as out:json.dump(counts,out,indent=2)
print(json.dumps(counts))
