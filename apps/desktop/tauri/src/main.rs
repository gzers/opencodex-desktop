//! OpenCodeX Desktop 应用二进制入口。
//!
//! 桌面应用组装逻辑保持在库入口中，这里只负责把控制权交给 Tauri。

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() {
    opencodex_desktop_lib::run();
}
