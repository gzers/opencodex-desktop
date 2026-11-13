//! Test-only synthetic discovery endpoint. No real provider/model API.
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
pub struct Server {
    port: u16,
    stop: Arc<AtomicBool>,
    models: Arc<AtomicUsize>,
    unexpected: Arc<AtomicUsize>,
    observed: Arc<Mutex<Vec<String>>>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let models = Arc::new(AtomicUsize::new(0));
        let unexpected = Arc::new(AtomicUsize::new(0));
        let observed = Arc::new(Mutex::new(Vec::new()));
        let observed_requests = observed.clone();
        let (done, requests, rejected) = (stop.clone(), models.clone(), unexpected.clone());
        let worker = thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                let (mut stream, _) = match listener.accept() {
                    Ok(v) => v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(e) => panic!("owned discovery accept: {e}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut chunk = [0; 2048];
                while bytes.len() < 16384 && !bytes.windows(4).any(|v| v == b"\r\n\r\n") {
                    match stream.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                        Err(_) => break,
                    }
                }
                let first = bytes.split(|b| *b == b'\n').next().unwrap_or_default();
                let line = std::str::from_utf8(first).unwrap_or("");
                let mut parts = line.split_ascii_whitespace();
                let method = parts.next().unwrap_or("unknown");
                let target = parts.next().unwrap_or("");
                let path = if target.starts_with("http://") || target.starts_with("https://") {
                    reqwest::Url::parse(target)
                        .ok()
                        .map(|u| u.path().to_string())
                        .unwrap_or_default()
                } else {
                    target.split('?').next().unwrap_or("").to_string()
                };
                if observed_requests.lock().unwrap().len() < 32 {
                    let safe_method = if matches!(method, "GET" | "POST" | "HEAD" | "CONNECT") {
                        method
                    } else {
                        "unknown"
                    };
                    let safe_path = if path.len() <= 128
                        && path
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || "/-_.".contains(c))
                    {
                        path.as_str()
                    } else {
                        "[unclassified]"
                    };
                    observed_requests
                        .lock()
                        .unwrap()
                        .push(format!("{safe_method} {safe_path}"));
                }
                let is_models =
                    method == "GET" && matches!(path.as_str(), "/v1/models" | "/models");
                let (status, body) = if is_models {
                    requests.fetch_add(1, Ordering::SeqCst);
                    let data=["claude-opus-5-5","gpt-6-sol","vendor/gemini-3.5-pro","unselected-live-fixture"]
                        .map(|id|serde_json::json!({"id":id,"object":"model","owned_by":"synthetic-only"}));
                    (
                        "200 OK",
                        serde_json::json!({"object":"list","data":data}).to_string(),
                    )
                } else {
                    rejected.fetch_add(1, Ordering::SeqCst);
                    (
                        "405 Method Not Allowed",
                        "{\"error\":\"fixture accepts discovery only\"}".into(),
                    )
                };
                let response=format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                let _ = stream.write_all(response.as_bytes());
            }
        });
        Self {
            port,
            stop,
            models,
            unexpected,
            observed,
            worker: Some(worker),
        }
    }
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}/v1", self.port)
    }
    pub fn finish(&mut self) -> serde_json::Value {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
        serde_json::json!({"port":self.port,"getModelsRequests":self.models.load(Ordering::SeqCst),"unexpectedOrBusinessRequests":self.unexpected.load(Ordering::SeqCst),"observedMethodAndPathOnly":*self.observed.lock().unwrap(),"listenerClosed":true,"syntheticLoopbackOnly":true})
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
