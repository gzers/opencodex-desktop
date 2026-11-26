//! First supply from the official Desktop release, with the embedded descriptor
//! as the only authority for what may be downloaded and installed.
use super::super::bundled::StartupAsset;
use crate::{
    errors::{AppError, AppResult},
    modules::{network_defaults, preferences},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

const MAX_REDIRECTS: usize = 3;
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);
const STAGING_PREFIX: &str = ".runtime-bundle-download-";

pub(crate) type Transfer =
    Box<dyn FnOnce(&StartupAsset, &mut DownloadSink) -> AppResult<()> + Send>;

pub(super) fn github(app: tauri::AppHandle) -> Transfer {
    Box::new(move |asset, sink| {
        let (policy, no_proxy) = preferences::http_options_for_app(&app);
        let client = network_defaults::http_client(policy, &no_proxy, DOWNLOAD_TIMEOUT)
            .map_err(|_| unavailable("无法准备运行资源下载连接"))?;
        tauri::async_runtime::block_on(async {
            tokio::time::timeout(DOWNLOAD_TIMEOUT, stream(&client, asset, sink))
                .await
                .map_err(|_| unavailable("运行资源下载超时"))?
        })
    })
}

fn release_url(asset: &StartupAsset) -> String {
    format!(
        "https://github.com/gzers/opencodex-desktop/releases/download/v{}/{}",
        env!("CARGO_PKG_VERSION"),
        asset.name
    )
}

async fn stream(
    client: &reqwest::Client,
    asset: &StartupAsset,
    sink: &mut DownloadSink,
) -> AppResult<()> {
    let mut url = reqwest::Url::parse(&release_url(asset))
        .map_err(|_| unavailable("当前应用的运行资源地址无效"))?;
    for count in 0..=MAX_REDIRECTS {
        let mut response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|_| unavailable("无法连接官方运行资源下载地址"))?;
        match response.status() {
            reqwest::StatusCode::OK => {
                if response
                    .content_length()
                    .is_some_and(|size| size != asset.bytes)
                {
                    return Err(integrity("下载响应体积与固定清单不符"));
                }
                while let Some(chunk) = response
                    .chunk()
                    .await
                    .map_err(|_| unavailable("读取运行资源下载响应失败"))?
                {
                    sink.write(&chunk)?;
                }
                return Ok(());
            }
            status if status.is_redirection() && count < MAX_REDIRECTS => {
                url = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| url.join(value).ok())
                    .filter(allowed_redirect)
                    .ok_or_else(|| unavailable("运行资源下载重定向不在允许范围内"))?;
            }
            _ => return Err(unavailable("官方地址未提供完整运行资源")),
        }
    }
    Err(unavailable("运行资源下载重定向次数超限"))
}

fn allowed_redirect(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && matches!(
            url.host_str(),
            Some("release-assets.githubusercontent.com" | "objects.githubusercontent.com")
        )
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
}

fn failure(code: &str, detail: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: detail.into(),
    }
}

fn unavailable(detail: &'static str) -> AppError {
    failure("bundle_download_unavailable", detail)
}

fn integrity(detail: &'static str) -> AppError {
    failure("bundle_download_invalid", detail)
}

fn staging_error(_: std::io::Error) -> AppError {
    unavailable("无法写入或清理本次运行资源暂存")
}

pub(crate) struct DownloadSink {
    file: File,
    remaining: u64,
    digest: Sha256,
}

impl DownloadSink {
    pub(super) fn write(&mut self, bytes: &[u8]) -> AppResult<()> {
        self.remaining = self
            .remaining
            .checked_sub(bytes.len() as u64)
            .ok_or_else(|| integrity("下载超过固定归档体积"))?;
        self.file.write_all(bytes).map_err(staging_error)?;
        self.digest.update(bytes);
        Ok(())
    }

    fn finish(self, asset: &StartupAsset) -> AppResult<()> {
        if self.remaining != 0 || format!("{:x}", self.digest.finalize()) != asset.sha256 {
            return Err(integrity("下载归档未通过固定体积和摘要校验"));
        }
        self.file.sync_all().map_err(staging_error)
    }
}

struct Staging {
    path: PathBuf,
    archive: PathBuf,
    cleaned: bool,
    #[cfg(windows)]
    pin: Option<crate::infrastructure::windows_official_runtime024::PinnedDirectory>,
}

impl Staging {
    fn new(desktop: &Path, asset: &StartupAsset) -> AppResult<Self> {
        let path = desktop.join(format!("{STAGING_PREFIX}{}", uuid::Uuid::new_v4()));
        #[cfg(windows)]
        let pin =
            crate::infrastructure::windows_official_runtime024::create_private_directory(&path)
                .map_err(staging_error)?;
        #[cfg(not(windows))]
        {
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&path).map_err(staging_error)?;
        }
        Ok(Self {
            archive: path.join(&asset.name),
            path,
            cleaned: false,
            #[cfg(windows)]
            pin: Some(pin),
        })
    }

    fn cleanup(&mut self) -> std::io::Result<()> {
        #[cfg(windows)]
        if let Some(pin) = &self.pin {
            pin.recheck()?;
        }
        match fs::remove_file(&self.archive) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error),
        }
        #[cfg(windows)]
        drop(self.pin.take());
        fs::remove_dir(&self.path)?;
        self.cleaned = true;
        Ok(())
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if !self.cleaned {
            let _ = self.cleanup();
        }
    }
}

pub(super) fn with_verified_archive(
    desktop: &Path,
    asset: &StartupAsset,
    transfer: Transfer,
    provision: impl FnOnce(&Path) -> AppResult<()>,
) -> AppResult<()> {
    let mut stage = Staging::new(desktop, asset)?;
    let result = (|| {
        let mut sink = DownloadSink {
            file: crate::infrastructure::platform::create_new_private_file(&stage.archive, 0o600)
                .map_err(staging_error)?,
            remaining: asset.bytes,
            digest: Sha256::new(),
        };
        transfer(asset, &mut sink)?;
        sink.finish(asset)?;
        #[cfg(windows)]
        if let Some(pin) = &stage.pin {
            pin.recheck().map_err(staging_error)?;
        }
        provision(&stage.archive)
    })();
    if stage.cleanup().is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(desktop)
            .append_event("could not clean the integrated runtime download staging");
    }
    result
}
