use super::*;
fn value(command: &Command, key: &str) -> Option<std::ffi::OsString> {
    command
        .get_envs()
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| v)
        .map(std::ffi::OsStr::to_owned)
}
#[test]
fn isolated_npm_commands_receive_saved_bypass_and_keep_real_home_and_explicit_cache() {
    let t = tempfile::tempdir().unwrap();
    let home = t.path().join("profile");
    let cache = t.path().join("data/cache/npm");
    let runner = SystemNpmRunner::new(Some(home.clone()), cache.clone())
        .with_no_proxy("registry.example.test,127.0.0.1".into());
    let command = runner.base_command(Path::new("/fixture/npm"), None);
    assert_eq!(
        value(&command, "npm_config_noproxy"),
        Some("registry.example.test,127.0.0.1".into())
    );
    assert_eq!(
        value(&command, "NO_PROXY"),
        value(&command, "npm_config_noproxy")
    );
    assert_eq!(
        value(&command, "npm_config_cache"),
        Some(cache.into_os_string())
    );
    #[cfg(unix)]
    assert_eq!(value(&command, "HOME"), Some(home.into_os_string()));
    assert!(value(&command, "HTTP_PROXY").is_none());
    assert!(value(&command, "HTTPS_PROXY").is_none());
    let direct = SystemNpmRunner::new(None, t.path().join("cache"))
        .base_command(Path::new("/fixture/npm"), None);
    assert_eq!(value(&direct, "npm_config_noproxy"), Some("".into()));
}
#[test]
#[ignore = "requires explicit unchanged Node/npm inputs; uses loopback registry only, no install"]
fn actual_npm_registry_metadata_uses_manual_proxy_and_saved_bypass_without_touching_user_config() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        time::{Duration, Instant},
    };
    let node = PathBuf::from(std::env::var_os("OCXD_TEST_NODE").expect("explicit Node"));
    let npm = PathBuf::from(std::env::var_os("OCXD_TEST_NPM").expect("explicit npm"));
    assert!(node.is_absolute() && npm.is_absolute());
    for bypass in [false, true] {
        let t = tempfile::tempdir().unwrap();
        let home = t.path().join("profile");
        let cache = t.path().join("cache/npm");
        std::fs::create_dir(&home).unwrap();
        let userconfig = home.join(".npmrc");
        std::fs::write(&userconfig, b"# isolated unchanged fixture\n").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let (tx, rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(8);
            let (mut stream, _) = loop {
                match listener.accept() {
                    Ok(v) => break v,
                    Err(e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            && Instant::now() < deadline =>
                    {
                        std::thread::sleep(Duration::from_millis(10))
                    }
                    Err(e) => panic!("fixture accept {e}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                stream.read_exact(&mut byte).unwrap();
                request.extend_from_slice(&byte);
                assert!(request.len() < 16384)
            }
            let body=serde_json::json!({"name":OFFICIAL_PACKAGE,"dist-tags":{"latest":"2.99.0"},"versions":{"2.99.0":{"name":OFFICIAL_PACKAGE,"version":"2.99.0"}}}).to_string();
            let response=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            stream.write_all(response.as_bytes()).unwrap();
            tx.send(String::from_utf8(request).unwrap()).unwrap();
        });
        let registry = if bypass {
            format!("http://{addr}/")
        } else {
            "http://registry.invalid/".into()
        };
        let proxy = if bypass {
            "http://127.0.0.1:9".into()
        } else {
            format!("http://{addr}")
        };
        let runner = SystemNpmRunner::new(Some(home.clone()), cache).with_no_proxy(if bypass {
            "127.0.0.1".into()
        } else {
            String::new()
        });
        let mut command = runner.base_command(&npm, node.parent());
        command
            .current_dir(t.path())
            .args([
                "view",
                OFFICIAL_PACKAGE,
                "version",
                "--json",
                "--fetch-retries=0",
                "--fetch-timeout=2000",
                "--registry",
                &registry,
                "--proxy",
                &proxy,
            ])
            .env("npm_config_userconfig", &userconfig);
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "npm metadata failure: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "\"2.99.0\"");
        let request = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        server.join().unwrap();
        assert!(
            request.starts_with(if bypass {
                "GET /@bitkyc08"
            } else {
                "GET http://registry.invalid/@bitkyc08"
            }),
            "actual npm route differs"
        );
        assert_eq!(
            std::fs::read(&userconfig).unwrap(),
            b"# isolated unchanged fixture\n"
        );
        println!(
            "REAL_NPM_PROXY {}",
            serde_json::json!({"bypass":bypass,"actualRegistryRequest":true,"savedConfigurationUnchanged":true,"installed":false})
        );
    }
}
