use super::super::{handles::HandleOperation, history::tests::linked, tests::plan};
use super::*;
fn rows(p: &StoragePlan) -> Vec<HandleRow> {
    p.roots
        .iter()
        .enumerate()
        .flat_map(|(root, tree)| {
            tree.nodes.iter().enumerate().flat_map(move |(i, n)| {
                ["original", "history-source"].map(move |role| HandleRow {
                    key: format!("tree/{root}/{role}/{i:04}"),
                    volume: n.volume,
                    file_id: n.file_id.clone(),
                    directory: n.kind == Kind::DirectoryAnchor,
                    historical_work: false,
                })
            })
        })
        .collect()
}
#[test]
fn wl2_r6_handle_rows_self_hashed_unknown_roles_wrong_ordinals_and_flags_refuse_atomically() {
    let (mut fixture, p, _, _) = plan();
    let g = linked(&p);
    let bad = [
        ("tree/0/unknown/0000", true, false),
        ("tree/2/original/0000", true, false),
        ("tree/00/original/0000", true, false),
        ("tree/0/original/0", true, false),
        ("tree/0/original/4096", true, false),
        ("tree/0/original/9999", true, false),
        ("tree/0/original/0000/extra", true, false),
        ("tree/0/pending-work/0000", false, true),
        ("tree/0/old-work/0/0000", false, true),
        ("tree/0/old-work/0000/0", false, true),
        ("tree/0/old-work/0000/0000", true, false),
        ("tree/0/initial-work/0000", false, true),
        ("tree/0/stopped-anchor/0000", false, true),
        ("lease/future/coord/0/file", false, false),
        ("lease/old/0/coord/0/file", false, false),
        ("lease/final/coord/4/file", false, false),
        ("lease/final/coord/00/file", false, false),
        ("lease/final/coord/0/file", true, false),
        ("lease/final/coord/0/file", false, true),
        ("lease/final/coord/0/parent/0", true, false),
        ("lease/final/coord/1/ancestor/0", true, false),
        ("lease/final/coord/1/claim-parent", true, false),
        ("lease/final/coord/0/claim-parent", false, false),
        ("lease/final/coord/1/parent/00", true, false),
        ("lease/final/bound/unknown/0", true, false),
        ("lease/final/bound/profile/0", false, false),
        ("lease/final/bound/profile/0/extra", true, false),
    ];
    let before = encoded(&g).unwrap();
    for (key, directory, historical_work) in bad {
        let mut rowset = rows(&p);
        rowset.push(HandleRow {
            key: key.into(),
            volume: p.roots[0].custody_volume,
            file_id: "999".into(),
            directory,
            historical_work,
        });
        let event = g
            .handle_event(HandleOperation::OriginalHandlesReleaseIntent, Some(rowset))
            .unwrap();
        let mut trial: Progress = serde_json::from_slice(&before).unwrap();
        assert!(
            trial.apply_handles(&p, &event).is_err(),
            "self-hashed bad canonical row {key}"
        );
        assert_eq!(
            encoded(&trial).unwrap(),
            before,
            "row grammar refusal must preserve complete progress"
        );
    }
    println!("r6 handle-row grammar27 self-hashed extra rows refuse exact role/root/ordinal/kind/history; original complete inventory present and hash/count recomputed, all progress unchanged, no parsed native authority");
    fixture.retain();
}
#[test]
fn wl2_r6_handle_rows_all_actual_producer_grammars_and_tree_object_bindings_validate_as_data() {
    let (mut fixture, p, _, _) = plan();
    let g = linked(&p);
    for r in rows(&p) {
        validate(&r, &p, &g).unwrap();
    }
    let volume = p.roots[0].custody_volume;
    for key in [
        "tree/0/old-work/0000/0000",
        "tree/1/old-work/0001/0002",
        "lease/final/coord/0/file",
        "lease/final/coord/3/file",
        "lease/old/0000/coord/1/file",
        "lease/final/coord/0/claim-parent",
        "lease/final/coord/0/ancestor/0",
        "lease/final/coord/2/parent/12",
        "lease/old/0002/bound/profile/0",
        "lease/old/0002/bound/home/6",
        "lease/final/bound/client/7",
    ] {
        let work = key.starts_with("tree/");
        let directory = !work && !key.ends_with("/file");
        validate(
            &HandleRow {
                key: key.into(),
                volume,
                file_id: "999".into(),
                directory,
                historical_work: work,
            },
            &p,
            &g,
        )
        .unwrap();
    }
    for (root, tree) in p.roots.iter().enumerate() {
        let (i, node) = tree
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.kind == Kind::CustodiedFile)
            .unwrap();
        let row = HandleRow {
            key: format!("tree/{root}/initial-work/{i:04}"),
            volume: node.volume,
            file_id: g.nodes[&node.node_id].1.clone().unwrap(),
            directory: false,
            historical_work: true,
        };
        validate(&row, &p, &g).unwrap();
        let mut bad = row;
        bad.file_id = "999".into();
        assert!(validate(&bad, &p, &g).is_err());
        let root_node = &tree.nodes[0];
        let row = HandleRow {
            key: format!("tree/{root}/stopped-anchor/0000"),
            volume: root_node.volume,
            file_id: root_node.file_id.clone(),
            directory: true,
            historical_work: false,
        };
        validate(&row, &p, &g).unwrap();
        let mut bad = row;
        bad.file_id = "999".into();
        assert!(validate(&bad, &p, &g).is_err());
    }
    println!("r6 handle-row DATA exact producer grammars retain original/history/initialwork/stoppedanchor and oldwork/leasecoord/ancestor/bound grammar; tree IDs/generation bindings independently checked, native full expected inventory remains separate");
    fixture.retain();
}
