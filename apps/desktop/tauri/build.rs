use std::path::{Path, PathBuf};

fn main() {
    track_frontend_dist();
    tauri_build::build();
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
