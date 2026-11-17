#![cfg(windows)]
#[path = "../../../../apps/desktop/tauri/src/infrastructure/windows_official_runtime024/mod.rs"]
#[cfg(feature="query")]
pub mod backend;
pub mod modules {
    pub mod backup {
        pub mod safety {
            include!(concat!(env!("OUT_DIR"), "/safety_section.rs"));
        }
    }
    pub mod runtime {
        pub mod program_image {
            include!(concat!(env!("OUT_DIR"), "/program_image_section.rs"));
        }
        pub mod program_layout {
            include!(concat!(env!("OUT_DIR"), "/layout_section.rs"));
        }
        pub mod program_selection {
            pub const IMAGE_MANIFEST: &str = ".runtime-image.json";
        }
        pub mod windows_lifecycle024_contract {
            pub const BOUNDED_SECURITY_SCOPE: &str = "windows-bounded-access-v1";
            pub const BOUNDED_SECURITY_QUERY_ACCESS: u32 = 0x0002_0000;
            pub const BOUNDED_SECURITY_INFORMATION: u32 = 0x0000_01f7;
            pub const BOUNDED_SECURITY_EXCLUDED_ASSURANCE: &str =
                "audit-sacl-and-unrequested-security-unknown";
        }
    }
}
#[cfg(feature="query")]
pub mod infrastructure;
#[cfg(not(feature="query"))]
pub mod infrastructure {
    #[path="../../../../../apps/desktop/tauri/src/infrastructure/hash.rs"]
    pub mod hash;
}

// Only an error envelope to compile the actual scanner section. No native
// call, success, selection, ownership, lifetime or recovery is simulated here.
pub mod errors {
    #[derive(Debug)]
    pub enum AppError {
        RuntimeManaged { code: String, detail: String },
        FileSystem { operation: String, detail: String },
    }
    impl std::fmt::Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
    impl std::error::Error for AppError {}
    pub type AppResult<T> = Result<T, AppError>;
}
