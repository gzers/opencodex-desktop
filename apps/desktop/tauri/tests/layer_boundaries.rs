use opencodex_desktop_lib::{errors, modules, types};

#[test]
fn public_layers_are_available() {
    let _ = std::mem::size_of::<errors::AppErrorPayload>();
    let _ = std::mem::size_of::<types::AppStatus>();
    modules::__marker();
}
