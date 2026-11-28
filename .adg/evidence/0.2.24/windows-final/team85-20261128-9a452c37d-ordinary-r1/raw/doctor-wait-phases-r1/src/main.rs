use std::future::Future;
use std::io::{self, Write};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

const WAIT: Duration = Duration::from_secs(5);
const SLOW: Duration = Duration::from_millis(6250);

fn main() -> io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("normal") => println!("normal child"),
        Some("slow") => {
            std::thread::sleep(SLOW);
            println!("slow child");
        }
        Some("inherited-stdout") => {
            let descendant = std::process::Command::new(std::env::current_exe()?)
                .arg("descendant")
                .stdout(Stdio::inherit())
                .spawn()?;
            println!("descendant_pid={}", descendant.id());
        }
        Some("descendant") => {
            std::thread::sleep(SLOW);
            println!("descendant finished");
        }
        _ => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(async {
                for (case, expected) in [
                    ("normal", "complete"),
                    ("slow", "child_exit"),
                    ("inherited-stdout", "stdout_eof"),
                ] {
                    observe(case, expected).await?;
                }
                Ok::<_, io::Error>(())
            })?,
    }
    io::stdout().flush()
}

async fn observe(case: &str, expected: &str) -> io::Result<()> {
    let begin = Instant::now();
    let mut child = Command::new(std::env::current_exe()?)
        .arg(case)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(false)
        .spawn()?;
    let pid = child.id().expect("new child has a PID");
    let spawn_us = begin.elapsed().as_micros();
    let observed = Instant::now();
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut report = Vec::new();
    let mut child_exit = None;
    let mut stdout_eof = None;
    let phase;
    let first_window_ms;
    {
        let mut exited = Box::pin(child.wait());
        let mut eof = Box::pin(stdout.read_to_end(&mut report));
        match tokio::time::timeout(
            WAIT,
            std::future::poll_fn(|cx| {
                if child_exit.is_none() {
                    if let std::task::Poll::Ready(result) = exited.as_mut().poll(cx) {
                        child_exit = Some((result?, observed.elapsed()));
                    }
                }
                if stdout_eof.is_none() {
                    if let std::task::Poll::Ready(result) = eof.as_mut().poll(cx) {
                        result?;
                        stdout_eof = Some(observed.elapsed());
                    }
                }
                if child_exit.is_some() && stdout_eof.is_some() {
                    std::task::Poll::Ready(Ok::<_, io::Error>(()))
                } else {
                    std::task::Poll::Pending
                }
            }),
        )
        .await
        {
            Ok(result) => {
                result?;
                phase = "complete";
            }
            Err(_) => {
                phase = match (child_exit.is_some(), stdout_eof.is_some()) {
                    (false, false) => "child_exit",
                    (true, false) => "stdout_eof",
                    _ => "unexpected",
                };
            }
        }
        first_window_ms = observed.elapsed().as_millis();
        // Drain and reap the fixture independently of the reported observation window.
        if child_exit.is_none() {
            child_exit = Some((exited.await?, observed.elapsed()));
        }
        if stdout_eof.is_none() {
            eof.await?;
            stdout_eof = Some(observed.elapsed());
        }
    }
    let (exit, exited_at) = child_exit.expect("child exited");
    println!("case={case} child_pid={pid} spawn_us={spawn_us} window_ms={first_window_ms} phase={phase} child_exit_ms={} stdout_eof_ms={} exit={exit} output={:?}", exited_at.as_millis(), stdout_eof.unwrap().as_millis(), String::from_utf8_lossy(&report));
    assert!(exit.success());
    assert_eq!(phase, expected);
    Ok(())
}
