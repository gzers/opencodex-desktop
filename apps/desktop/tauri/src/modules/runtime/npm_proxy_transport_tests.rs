//! Real fixed npm, local synthetic proxy/registry only. The TLS CA is an
//! explicit test input; strict-ssl stays true. No OS proxy settings are changed.
use super::*;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::Arc,
    time::Instant,
};

fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        match listener.accept() {
            Ok((s, _)) => {
                s.set_nonblocking(false).unwrap();
                s.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
                s.set_write_timeout(Some(Duration::from_secs(4))).unwrap();
                return s;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(e) => panic!("fixture accept: {e}"),
        }
    }
}
fn headers(stream: &mut impl Read) -> Vec<u8> {
    let mut out = Vec::new();
    let mut one = [0];
    while !out.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut one).unwrap();
        out.push(one[0]);
        assert!(out.len() < 16384);
    }
    out
}
fn metadata(stream: &mut (impl Read + Write)) -> String {
    let request = String::from_utf8(headers(stream)).unwrap();
    let body=serde_json::json!({"name":OFFICIAL_PACKAGE,"dist-tags":{"latest":"2.99.0"},"versions":{"2.99.0":{"name":OFFICIAL_PACKAGE,"version":"2.99.0"}}}).to_string();
    write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
    stream.flush().unwrap();
    request
}
fn socks(stream: &mut TcpStream) -> (String, u16) {
    let mut hello = [0; 2];
    stream.read_exact(&mut hello).unwrap();
    assert_eq!(hello[0], 5);
    assert!((1..=8).contains(&hello[1]));
    let mut methods = vec![0; hello[1] as usize];
    stream.read_exact(&mut methods).unwrap();
    assert!(methods.contains(&0));
    stream.write_all(&[5, 0]).unwrap();
    let mut command = [0; 4];
    stream.read_exact(&mut command).unwrap();
    assert_eq!(&command[..3], &[5, 1, 0]);
    // This is the remote-DNS guarantee: ATYP must be DOMAIN, not an IP resolved
    // locally. registry.invalid cannot be resolved by this fixture's client.
    assert_eq!(command[3], 3, "socks5h must send the hostname to the proxy");
    let mut size = [0];
    stream.read_exact(&mut size).unwrap();
    assert!((1..=253).contains(&size[0]));
    let mut host = vec![0; size[0] as usize];
    stream.read_exact(&mut host).unwrap();
    let mut port = [0; 2];
    stream.read_exact(&mut port).unwrap();
    stream.write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
    (String::from_utf8(host).unwrap(), u16::from_be_bytes(port))
}
fn tunnel(mut incoming: TcpStream, mut outgoing: TcpStream) {
    let mut left = incoming.try_clone().unwrap();
    let mut right = outgoing.try_clone().unwrap();
    let forward = std::thread::spawn(move || {
        let result = std::io::copy(&mut left, &mut right);
        let _ = right.shutdown(Shutdown::Write);
        result
    });
    let backward = std::io::copy(&mut outgoing, &mut incoming);
    let _ = incoming.shutdown(Shutdown::Write);
    let forward = forward.join().unwrap();
    // Both EOF and a peer closing immediately after a complete TLS reply are
    // ordinary fixture teardown. Application/TLS success is asserted by npm.
    for result in [forward, backward] {
        if let Err(e) = result {
            assert!(
                matches!(
                    e.kind(),
                    std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe
                ),
                "tunnel I/O: {e}"
            );
        }
    }
}

#[test]
#[ignore = "explicit unchanged Node/npm and synthetic local TLS CA required; no real registry"]
fn actual_npm_https_connect_and_socks_remote_dns_keep_strict_certificate_validation() {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    let node = PathBuf::from(std::env::var_os("OCXD_TEST_NODE").expect("explicit Node"));
    let npm = PathBuf::from(std::env::var_os("OCXD_TEST_NPM").expect("explicit npm"));
    let tls =
        PathBuf::from(std::env::var_os("OCXD_TEST_TLS").expect("synthetic certificate directory"));
    assert!(node.is_absolute() && npm.is_absolute() && tls.is_absolute());
    let _ = rustls::crypto::ring::default_provider().install_default();
    let cert = CertificateDer::from(std::fs::read(tls.join("server.der")).unwrap());
    let key = PrivateKeyDer::try_from(std::fs::read(tls.join("server-key.der")).unwrap()).unwrap();
    let server = Arc::new(
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap(),
    );
    for (scheme, https, trust_ca) in [
        ("http", true, true),
        ("socks5h", false, true),
        ("socks5h", true, true),
        ("http", true, false),
    ] {
        let root = tempfile::tempdir().unwrap();
        let profile = root.path().join("profile");
        std::fs::create_dir(&profile).unwrap();
        let userconfig = profile.join(".npmrc");
        let user_bytes =
            b"proxy=http://127.0.0.1:9\nhttps-proxy=http://127.0.0.1:9\nstrict-ssl=true\n";
        std::fs::write(&userconfig, user_bytes).unwrap();
        let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = proxy.local_addr().unwrap();
        let tls_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let tls_address = tls_listener.local_addr().unwrap();
        let server = server.clone();
        let tls_job = if https {
            Some(std::thread::spawn(move || {
                let stream = accept(&tls_listener);
                let mut stream = rustls::StreamOwned::new(
                    rustls::ServerConnection::new(server).unwrap(),
                    stream,
                );
                if trust_ca {
                    Ok(metadata(&mut stream))
                } else {
                    let mut byte = [0];
                    match stream.read_exact(&mut byte) {
                        Err(_) => Ok(String::from("TLS_REJECTED_UNTRUSTED_CA")),
                        Ok(()) => Err(String::from("untrusted TLS unexpectedly accepted")),
                    }
                }
            }))
        } else {
            None
        };
        let proxy_job = std::thread::spawn(move || {
            let mut stream = accept(&proxy);
            let port = if https { 443 } else { 80 };
            let route = if scheme == "socks5h" {
                let (host, actual_port) = socks(&mut stream);
                assert_eq!(host, "registry.invalid");
                assert_eq!(actual_port, port);
                format!("SOCKS5_DOMAIN {host}:{actual_port}")
            } else {
                let request = String::from_utf8(headers(&mut stream)).unwrap();
                assert!(request.starts_with("CONNECT registry.invalid:443 HTTP/1.1\r\n"));
                stream
                    .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                    .unwrap();
                String::from("HTTP_CONNECT registry.invalid:443")
            };
            let request = if https {
                let outgoing = TcpStream::connect(tls_address).unwrap();
                outgoing
                    .set_read_timeout(Some(Duration::from_secs(4)))
                    .unwrap();
                outgoing
                    .set_write_timeout(Some(Duration::from_secs(4)))
                    .unwrap();
                tunnel(stream, outgoing);
                String::new()
            } else {
                metadata(&mut stream)
            };
            (route, request)
        });
        let proxy = format!("{scheme}://{address}");
        let snapshot = crate::modules::network_defaults::npm_proxy::Snapshot::system_fixture(
            Some(&proxy),
            Some(&proxy),
            "",
        )
        .unwrap();
        let runner = SystemNpmRunner::new(Some(profile), root.path().join("cache"))
            .with_network_snapshot(snapshot);
        let mut command = runner.base_command(&npm, node.parent());
        command
            .current_dir(root.path())
            .args([
                "view",
                OFFICIAL_PACKAGE,
                "version",
                "--json",
                "--fetch-retries=0",
                "--fetch-timeout=2500",
                "--strict-ssl=true",
                "--registry",
                if https {
                    "https://registry.invalid/"
                } else {
                    "http://registry.invalid/"
                },
            ])
            .env("npm_config_userconfig", &userconfig);
        if trust_ca {
            command.arg("--cafile").arg(tls.join("ca.pem"));
        }
        let output = command.output().unwrap();
        let (route, plain) = proxy_job.join().unwrap();
        let request = match tls_job {
            Some(job) => job.join().unwrap().unwrap(),
            None => plain,
        };
        if trust_ca {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "\"2.99.0\"");
            assert!(
                request.starts_with("GET /@bitkyc08"),
                "registry request must stay inside the tunnel"
            );
        } else {
            assert!(!output.status.success());
            let text = String::from_utf8_lossy(&output.stderr);
            assert!(
                text.contains("SELF_SIGNED_CERT")
                    || text.contains("UNABLE_TO_VERIFY")
                    || text.contains("UNABLE_TO_GET_ISSUER"),
                "npm must reject the certificate itself: {text}"
            );
            assert_eq!(request, "TLS_REJECTED_UNTRUSTED_CA");
            let code = text
                .lines()
                .find_map(|line| line.strip_prefix("npm error code "))
                .expect("native npm certificate error code");
            assert!(
                code.len() < 96
                    && code
                        .bytes()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
            );
            println!("NPM_PROXY_TLS_REJECTION {code}");
        }
        assert_eq!(std::fs::read(&userconfig).unwrap(), user_bytes);
        println!(
            "NPM_PROXY_TRANSPORT {}",
            serde_json::json!({"scheme":scheme,"https":https,"testCaTrusted":trust_ca,"strictSsl":true,"exitCode":output.status.code(),"route":route,"userConfigUnchanged":true,"nativeOsSettingsModified":false,"fixtureSystemSnapshot":true})
        );
    }
}
