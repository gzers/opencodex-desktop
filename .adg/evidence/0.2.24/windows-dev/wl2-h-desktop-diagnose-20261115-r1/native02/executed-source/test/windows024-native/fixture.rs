use std::{
    fs,
    net::{Ipv6Addr, TcpListener},
    path::Path,
    thread,
    time::{Duration, Instant},
};
fn wait(path: &Path) {
    let until = Instant::now() + Duration::from_secs(15);
    while !path.exists() {
        assert!(Instant::now() < until, "fixture release deadline");
        thread::sleep(Duration::from_millis(5));
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args[1].as_str() {
        "exit" => std::process::exit(args[2].parse().unwrap()),
        "wl2-queries" => {
            let system =
                std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32");
            let mut outcomes = Vec::new();
            for (exe, argv) in [
                ("sc.exe", vec!["query", "opencodex-proxy-native"]),
                (
                    "schtasks.exe",
                    vec!["/query", "/tn", "opencodex-proxy", "/xml"],
                ),
            ] {
                let mut child = std::process::Command::new(system.join(exe))
                    .args(argv)
                    .spawn()
                    .unwrap();
                let pid = child.id();
                let raw = child.wait().unwrap().code().unwrap();
                outcomes.push(format!("{pid} {exe} {raw}"));
            }
            fs::write(&args[2], outcomes.join("\n")).unwrap();
            wait(Path::new(&args[3]));
        }
        "burst" => {
            for _ in 0..24 {
                let status = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["exit", "0"])
                    .status()
                    .unwrap();
                assert!(status.success());
            }
            fs::write(&args[2], b"burst-complete").unwrap();
            wait(Path::new(&args[3]));
        }
        "system-child" => {
            let command = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
                .join("System32/cmd.exe");
            let mut child = std::process::Command::new(command)
                .args(["/d", "/c", "exit", "0"])
                .spawn()
                .unwrap();
            fs::write(&args[2], b"system-child-complete").unwrap();
            assert!(child.wait().unwrap().success());
            wait(Path::new(&args[3]));
        }
        "env" => {
            let text = format!(
                "{}\n{}\n{}\n{}",
                std::env::var("HOME").unwrap(),
                std::env::var("USERPROFILE").unwrap(),
                std::env::var("OPENCODEX_HOME").unwrap(),
                args[3]
            );
            fs::write(&args[2], text).unwrap();
        }
        "listen" => {
            let a = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let b = TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
            fs::write(
                &args[2],
                format!(
                    "{}\n{}",
                    a.local_addr().unwrap().port(),
                    b.local_addr().unwrap().port()
                ),
            )
            .unwrap();
            wait(Path::new(&args[3]));
            drop((a, b));
        }
        "child" => {
            fs::write(&args[2], std::process::id().to_string()).unwrap();
            wait(Path::new(&args[3]));
        }
        "family" => {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["child", &args[2], &args[3]])
                .spawn()
                .unwrap();
            wait(Path::new(&args[4]));
            let _ = &mut child;
            std::process::exit(0);
        }
        _ => panic!("explicit test mode required"),
    }
}
