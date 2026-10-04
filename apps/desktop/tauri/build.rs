use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    track_frontend_dist();
    track_build_commit();
    track_frozen_defaults();
    tauri_build::build();
}

/// 固化默认配置（H-01/H-06~H-13）：构建期校验并嵌入，消费者从同一来源读取，
/// 不再各写 fallback。字段缺失或非法在编译期失败，避免默认值悄悄漂移。
fn track_frozen_defaults() {
    for name in [
        "preferences.defaults.json",
        "runtime.defaults.json",
        "network.defaults.json",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("config")
            .join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        let Ok(bytes) = std::fs::read(&path) else {
            panic!("固化默认配置缺失: {name}");
        };
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("固化默认配置不是合法 JSON: {name}: {error}"));
        if !value.is_object() {
            panic!("固化默认配置必须是对象: {name}");
        }
    }
}

/// 注入构建来源（F-09）：启动事件要能写出运行制品对应的源码提交，
/// 否则维护事故无法把本机安装映射回提交。
///
/// 工作树的 `.git` 可能是文件（linked worktree），所以用
/// `git rev-parse --git-path HEAD` 解析真实 HEAD 路径再声明依赖；
/// 取不到提交时留空，运行时如实说成「未知」，不伪造提交号。
fn track_build_commit() {
    if let Some(head) = git_output(&["rev-parse", "--git-path", "HEAD"]) {
        let head = PathBuf::from(head);
        if head.exists() {
            println!("cargo:rerun-if-changed={}", head.display());
        }
    }
    let commit = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    println!("cargo:rustc-env=OPENCODEX_BUILD_COMMIT={commit}");
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 前端产物变化必须触发 Rust 侧重编。
///
/// `frontendDist`（`../ui/dist`）的内容在编译期内嵌进二进制，但 cargo 默认不跟踪它：
/// 只改前端时不会重编，`tauri build` 也就把上一版界面打进制品，表现为「更新了还是旧界面」。
/// 这里逐个文件声明 `rerun-if-changed`，目录本身也声明一次（覆盖新增/删除文件）。
fn track_frontend_dist() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/dist");
    println!("cargo:rerun-if-changed={}", dist.display());

    let mut files = Vec::new();
    collect(&dist, &mut files);
    files.sort();
    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}
