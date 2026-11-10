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
        "env" => {
            let text = format!(
                "{}\n{}\n{}",
                std::env::var("HOME").unwrap(),
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
