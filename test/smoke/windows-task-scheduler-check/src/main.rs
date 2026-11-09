// Exercise the exact product source before a full Windows desktop build.
#[path = "../../../../apps/desktop/tauri/src/infrastructure/windows_task_scheduler.rs"]
mod windows_task_scheduler;
fn main() {
    let started = std::time::Instant::now();
    assert_eq!(windows_task_scheduler::opencodex_task_absent(), Ok(true));
    assert!(started.elapsed() <= std::time::Duration::from_secs(8));
    println!("actual-product-task-scheduler-observation: absent; within-original-8s");
}
