fn main() -> std::process::ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2
        || args[0] != "--r7-r2-readonly024-v1"
        || !matches!(args[1].as_str(), "before" | "after")
    {
        eprintln!("Exact standalone readonly version1 before/after phase required.");
        return std::process::ExitCode::from(2);
    }
    match windows024_r7_query_proof::infrastructure::windows_client_originals024::collector::run(
        &args[1],
    ) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Read-only scene refused raw={:?}: {e}", e.raw_os_error());
            std::process::ExitCode::from(1)
        }
    }
}
