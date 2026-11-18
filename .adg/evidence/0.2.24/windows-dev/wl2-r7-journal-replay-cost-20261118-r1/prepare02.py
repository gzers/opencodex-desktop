from pathlib import Path
import subprocess

file = Path('apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal_tests.rs')
current = file.read_text(encoding='utf-8')
baseline = subprocess.check_output(['git', 'show', 'HEAD:' + file.as_posix()]).decode('utf-8')
marker = '#[test]\nfn wl2_r6_capture_canonical_data_namespace'
new = current[current.index('#[test]\nfn wl2_r7_journal_replay_cost_'):current.index(marker)]
new = new.replace('node.kind))', 'node.kind == Kind::CustodiedFile))')
new = new.replace('for (root_id, node_id, kind) in nodes {', 'for (root_id, node_id, custodied) in nodes {')
new = new.replace('if kind == Kind::CustodiedFile {', 'if custodied {')
start = new.index('    println!("r7-journal-pure-data=')
end = new.index('    drop(journal);', start)
new = new[:start] + '''    let data = serde_json::json!({
        "iterations": iters,
        "policyDigestUs": policy_us,
        "eventEncodeAndDigestUs": event_us,
        "eventBytes": bytes.len(),
    });
    println!("r7-journal-pure-data={data}");
''' + new[end:]
file.write_bytes(baseline.replace(marker, new + marker, 1).encode('utf-8'))
