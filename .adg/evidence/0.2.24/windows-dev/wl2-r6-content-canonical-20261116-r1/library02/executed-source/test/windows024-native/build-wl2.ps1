param([Parameter(Mandatory)][string]$Evidence,[string]$Mode='build')
$ErrorActionPreference='Stop'
$toolchain='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. "$PSScriptRoot/msvc-env.ps1" -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$env:RUSTC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe"
$env:RUSTDOC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustdoc.exe"
$env:CARGO_TARGET_DIR="$PSScriptRoot/target/w1c-product"
$cargo="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/cargo.exe"
$arguments=@('build','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--bins','-j','1')
if($Mode -eq 'test'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r3'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_r3_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'h-desktop'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_h_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'record-phase'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_client_originals024::phase::tests','--','--test-threads=1','--nocapture')}
if($Mode -eq 'native-record'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','private_record_pins_bytes_and_generation','--','--exact','tests::private_record_pins_bytes_and_generation','--test-threads=1','--nocapture')
}
if($Mode -eq 'native'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','wl2_','--','--test-threads=1','--nocapture')
}
if($Mode -eq 'native-r3'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','wl2_r3_','--','--test-threads=1','--nocapture')
}
if($Mode -eq 'native-h'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','wl2_h_','--','--test-threads=1','--nocapture')
}
$head=git rev-parse HEAD
if($Mode -eq 'sd-next'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_next_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'sd-next-native'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','wl2_next_','--','--test-threads=1','--nocapture')
}
if($Mode -eq 'sd-query-history'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--test','process','-j','1','wl2_retained_command_parent_and_raw_query_exits','--','--exact','--test-threads=1','--nocapture')
}
if($Mode -eq 'sd-supervisor'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_supervisor024::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'family'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_family_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'family-roles'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_diagnostic_roles024::tests::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'family-native'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--test','process','-j','1','wl2_family_','--','--test-threads=1','--nocapture')
}
if($Mode -eq 'r4'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_r4_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r4-populated'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_r4_native_populated_components','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r4-progress'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_client_originals024::v2::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r4-current'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_client_originals024::v2::current::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r4-locks'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_r4_stable_locks_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'originals-adapter'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_client_originals024::v2::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'originals-record-pin'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_client_originals024::v2::adapter::tests::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'originals-dispatch'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','--','windows_client_originals024::v2::','windows_official_lifecycle024::','windows_supervisor024::','windows_diagnostic_roles024::','--test-threads=1','--nocapture')}
if($Mode -eq 'originals-creator'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--test','process','-j','1','wl2_originals_capture_guard_requires_original_creator_before_resume','--','--exact','--test-threads=1','--nocapture')
}
if($Mode -eq 'sd-stop'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','--','windows_official_lifecycle024::','windows_supervisor024::','windows_diagnostic_roles024::','--test-threads=1','--nocapture')}
if($Mode -eq 'actual-gui-lib'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','windows_owner_host024::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'actual-gui-build'){$arguments=@('build','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--bins','-j','1')}
if($Mode -eq 'actual-gui-observer-build'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','--no-run','-j','1')}
if($Mode -eq 'r4-root-progress'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_r4_unperformed','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r4-retired'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_r4_retired','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r4-tree-v2'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','--','windows_client_originals024::tree::','windows_client_originals024::v2::','--test-threads=1','--nocapture')}
if($Mode -eq 'r4-history-dispatch'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','--','windows_client_originals024::tree::','windows_client_originals024::v2::','windows_official_lifecycle024::','windows_supervisor024::','windows_diagnostic_roles024::','--test-threads=1','--nocapture')}
$diff=git diff --binary
if($Mode -eq 'r6-anchored'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-lease-busy'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_lease_segments_actual_busy_claim','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-source-actions'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_original_actions','windows_client_originals024::v2::adapter::tests::','wl2_restart_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-work-observation'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_work_observation','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-work-regression'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_work_observation','wl2_r6_native_node_driver','wl2_r6_lease_','windows_client_originals024::v3::','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-work-atomic'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_work_observation_atomic_generation','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-work-archive'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_work_observation_atomic_generation','wl2_r6_work_observation_partial_archive','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-snapshot-links'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_snapshot_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-snapshot-regression'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_snapshot_','wl2_r6_work_observation','windows_client_originals024::v3::','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-stop-actions'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_stop_','wl2_r6_release_history_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-stop-regression'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_stop_','wl2_r6_release_history_','wl2_r6_snapshot_','wl2_r6_lease_','windows_client_originals024::v3::','wl2_r6_original_actions','wl2_restart_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-store-retention'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_capture_','wl2_r6_release_objects_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-store-regression'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_capture_','wl2_r6_release_','wl2_r6_lease_','windows_client_originals024::v3::','wl2_r5_coordination_original_ranges_','wl2_r5_coordination_busy_each_role_','wl2_r5_coordination_expiry_','wl2_r5_coordination_raw_data_drift_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-store-namespace'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_capture_canonical_data_namespace','wl2_r6_capture_namespace_old_v2_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-snapshot-series'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_series_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-snapshot-series-release'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_series_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-series-regression-release'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_series_','wl2_r6_snapshot_','wl2_r6_work_observation','windows_client_originals024::v3::','wl2_r6_lease_','wl2_r6_capture_','wl2_r6_release_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-capture-ready'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_capture_ready_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-capture-stop'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_capture_stop_','wl2_r6_stop_action_data_','wl2_r6_stop_phase_data_','wl2_r6_original_actions','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-retirement-native'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_retirement_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-recovery-data'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','windows_client_originals024::v3::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-recovery-chain'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','windows_client_originals024::v3::','wl2_r6_recovery_native_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-recovery-native-chain'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','windows_client_originals024::v3::','wl2_r6_recovery_chain_native_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-recovery-native-chain-release'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','windows_client_originals024::v3::','wl2_r6_recovery_chain_native_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-recovery-native-budget-release'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_recovery_chain_native_all_current_and_original_nodes_use_same_canonical_chain','wl2_r6_recovery_chain_native_interrupted_original_restore_preserves_exact_native_prefix','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-final-release-data'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','windows_client_originals024::v3::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-history-data'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','windows_client_originals024::v3::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-handles-data'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','windows_client_originals024::v3::','wl2_r6_handles_native_exact_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-handles-native'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_handles_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-handles-regression'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_handles_','wl2_r6_recovery_chain_native_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-completed-data'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','windows_client_originals024::v3::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-batch-data'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','windows_client_originals024::v3::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-production-content'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_production_content_','wl2_r6_series_','wl2_r6_capture_','windows_client_originals024::v3::','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-content-canonical'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','wl2_r6_series_','wl2_r6_capture_','windows_client_originals024::v3::','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-batch-native'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','windows_client_originals024::v3::','wl2_r6_batch_native_','wl2_r6_completed_history_','wl2_r6_capture_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-completed-native'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_completed_history_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-completed-regression'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','windows_client_originals024::v3::','wl2_r6_recovery_chain_native_','wl2_r6_handles_handoff_native_','wl2_r6_completed_history_','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-history-native'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_history_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-final-release-native'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_final_release_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-retirement-existing'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_retirement_existing_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r6-retirement-existing-release'){$arguments=@('test','--release','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r6_retirement_existing_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r5-routes-status'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','client_route_declaration::','modules::status::','status_source::','windows_client_originals024::v2::routes::','wl2_r5_','--test-threads=1','--nocapture')}
if($Mode -eq 'r5-coordination'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','windows_client_originals024::coordination::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r5-artifacts'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','--','windows_client_originals024::tree::artifacts::','client_artifact_policy::','--test-threads=1','--nocapture')}
if($Mode -eq 'r5-claim-root'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r5_coordination_root_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r5-claim-matrix'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1','wl2_r5_coordination_root_sharing_matrix','--','--test-threads=1','--nocapture')}
if($Mode -eq 'original-reference'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','windows_client_originals024::','--','--test-threads=1','--nocapture')}
if($Mode -eq 'original-move'){
 $env:OCX024_MOVE_OTHER_PARENT=$toolchain
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_move_','--','--test-threads=1','--nocapture')
}
if($Mode -eq 'original-move-cross-volume'){
 $env:OCX024_MOVE_OTHER_PARENT=$toolchain
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_move_cross_volume_parent_refuses_without_copy_or_deleting_source','--','--exact','tests::wl2_move_cross_volume_parent_refuses_without_copy_or_deleting_source','--test-threads=1','--nocapture')
}
@{head=$head;diff=$diff;arguments=$arguments;rustc=(& $env:RUSTC --version);time=[datetime]::Now.ToString('o')}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath "$Evidence/source.json" -Encoding utf8
$files=@(git ls-files --cached --others --exclude-standard -- 'apps/desktop/tauri/src/infrastructure/windows*' 'apps/desktop/tauri/src/infrastructure/mod.rs' 'test/windows024-native' 'apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs' 'apps/desktop/tauri/src/bin/ocxd.rs' 'apps/desktop/tauri/src/lib.rs')
$files+=@(git ls-files -- 'apps/desktop/tauri/src/infrastructure/status_source.rs' 'apps/desktop/tauri/src/modules/status/**' 'apps/desktop/tauri/src/modules/runtime/client_route_declaration*.rs' 'apps/desktop/tauri/src/modules/runtime/client_artifact_policy*.rs' 'apps/desktop/tauri/src/modules/runtime/mod.rs')
$manifest=@()
foreach($relative in $files){
 $item=Get-Item -LiteralPath $relative
 if($item.PSIsContainer){throw 'source manifest directory entry refused'}
 $target=Join-Path (Join-Path $Evidence 'executed-source') $relative
 [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target))
 [IO.File]::Copy($item.FullName,$target,$false)
 $manifest+=@{path=$relative;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()}
}
ConvertTo-Json -InputObject @($manifest) -Depth 5|Set-Content -LiteralPath "$Evidence/source-manifest.json" -Encoding utf8
$watch=[Diagnostics.Stopwatch]::StartNew()
& $cargo @arguments 1> "$Evidence/stdout.log" 2> "$Evidence/stderr.log"
$rc=$LASTEXITCODE
$executedProfile=if($arguments -contains '--release'){'release'}else{'debug'}
@{exitCode=$rc;elapsedMs=$watch.ElapsedMilliseconds;arguments=$arguments;profile=$executedProfile;accepted=$false}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath "$Evidence/result.json" -Encoding utf8
exit $rc
