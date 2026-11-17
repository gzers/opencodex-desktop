fn main() {
    if let Some(path) = std::env::args_os().nth(1) {
        std::fs::write(path, b"fixture executed").unwrap();
    }
}
