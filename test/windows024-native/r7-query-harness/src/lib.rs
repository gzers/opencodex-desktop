#![cfg(windows)]
#[path = "../../../../apps/desktop/tauri/src/infrastructure/windows_official_runtime024/mod.rs"]
pub mod backend;
pub mod modules {
    pub mod runtime {
        pub mod windows_lifecycle024_contract {
            pub const BOUNDED_SECURITY_SCOPE: &str = "windows-bounded-access-v1";
            pub const BOUNDED_SECURITY_QUERY_ACCESS: u32 = 0x0002_0000;
            pub const BOUNDED_SECURITY_INFORMATION: u32 = 0x0000_01f7;
            pub const BOUNDED_SECURITY_EXCLUDED_ASSURANCE: &str =
                "audit-sacl-and-unrequested-security-unknown";
        }
    }
}
pub mod infrastructure;
