//! FZ-38 可选 PATH symlink 注册。
//!
//! 默认关闭；只在用户显式启用时创建。已存在但指向其他目标时不覆盖，
//! 卸载只删除确认指向本应用二进制的 symlink。

use std::path::Path;

use crate::errors::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathRegistrationResult {
    Created,
    AlreadyRegistered,
    Conflict,
    Removed,
    NotFound,
}

fn link_target_matches(target: &Path, link: &Path) -> bool {
    target == link
}

pub fn register(binary_path: &Path, target: &Path) -> Result<PathRegistrationResult, AppError> {
    if !binary_path.is_file() {
        return Err(AppError::NotFound {
            entity: binary_path.display().to_string(),
        });
    }
    match std::fs::symlink_metadata(target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let link = std::fs::read_link(target).map_err(|error| AppError::FileSystem {
                operation: "inspect CLI PATH symlink".to_string(),
                detail: error.to_string(),
            })?;
            if link_target_matches(binary_path, &link) {
                Ok(PathRegistrationResult::AlreadyRegistered)
            } else {
                Ok(PathRegistrationResult::Conflict)
            }
        }
        Ok(_) => Ok(PathRegistrationResult::Conflict),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::os::unix::fs::symlink(binary_path, target).map_err(|error| AppError::FileSystem {
                operation: "register CLI PATH symlink".to_string(),
                detail: if error.kind() == std::io::ErrorKind::PermissionDenied {
                    "run: sudo mkdir -p /usr/local/bin && sudo ln -sf <binary> /usr/local/bin/ocxd".to_string()
                } else {
                    error.to_string()
                },
            })?;
            Ok(PathRegistrationResult::Created)
        }
        Err(error) => Err(AppError::FileSystem {
            operation: "inspect CLI PATH symlink".to_string(),
            detail: error.to_string(),
        }),
    }
}

pub fn unregister(binary_path: &Path, target: &Path) -> Result<PathRegistrationResult, AppError> {
    match std::fs::symlink_metadata(target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let link = std::fs::read_link(target).map_err(|error| AppError::FileSystem {
                operation: "inspect CLI PATH symlink".to_string(),
                detail: error.to_string(),
            })?;
            if !link_target_matches(binary_path, &link) {
                return Ok(PathRegistrationResult::Conflict);
            }
            std::fs::remove_file(target).map_err(|error| AppError::FileSystem {
                operation: "unregister CLI PATH symlink".to_string(),
                detail: if error.kind() == std::io::ErrorKind::PermissionDenied {
                    "run: sudo rm /usr/local/bin/ocxd".to_string()
                } else {
                    error.to_string()
                },
            })?;
            Ok(PathRegistrationResult::Removed)
        }
        Ok(_) => Ok(PathRegistrationResult::Conflict),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(PathRegistrationResult::NotFound)
        }
        Err(error) => Err(AppError::FileSystem {
            operation: "inspect CLI PATH symlink".to_string(),
            detail: error.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn registers_and_removes_only_owned_symlink() {
        let root = tempfile::tempdir().expect("temporary root");
        let bin = root.path().join("app/bin/ocxd");
        std::fs::create_dir_all(bin.parent().unwrap()).expect("create bin");
        std::fs::write(&bin, b"binary").expect("write binary");
        let mut mode = std::fs::metadata(&bin).unwrap().permissions();
        mode.set_mode(0o755);
        std::fs::set_permissions(&bin, mode).unwrap();
        let link = root.path().join("usr/local/bin/ocxd");
        std::fs::create_dir_all(link.parent().unwrap()).expect("create local bin");
        assert!(matches!(
            register(&bin, &link),
            Ok(PathRegistrationResult::Created)
        ));
        assert!(matches!(
            register(&bin, &link),
            Ok(PathRegistrationResult::AlreadyRegistered)
        ));
        assert!(matches!(
            unregister(&bin, &link),
            Ok(PathRegistrationResult::Removed)
        ));
        assert!(!link.exists());

        let foreign = root.path().join("foreign");
        std::fs::write(&foreign, b"other").expect("write foreign");
        std::os::unix::fs::symlink(&foreign, &link).expect("link foreign");
        assert!(matches!(
            register(&bin, &link),
            Ok(PathRegistrationResult::Conflict)
        ));
        assert!(matches!(
            unregister(&bin, &link),
            Ok(PathRegistrationResult::Conflict)
        ));
        assert!(link.symlink_metadata().is_ok());
    }
}
