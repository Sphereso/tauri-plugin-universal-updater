use futures_util::StreamExt;
use reqwest::ClientBuilder;
use semver::Version;
use tauri::{plugin::PluginApi, AppHandle, Runtime};
use url::Url;

use crate::{Config, Error, Result, Update};

pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Config>,
) -> crate::Result<UniversalUpdater<R>> {
    let config = api.config().clone();
    let endpoint = config.endpoint.ok_or(Error::EmptyEndpoints)?;

    let arch = get_updater_arch().ok_or(Error::UnsupportedArch)?;
    let (target, json_target) = {
        let target = get_updater_target().ok_or(Error::UnsupportedOs)?;
        (target.to_string(), format!("{target}-{arch}"))
    };

    Ok(UniversalUpdater {
        handle: app.clone(),
        endpoint: endpoint.0.clone(),
        current_version: app.package_info().version.clone(),
        arch,
        target,
        json_target,
    })
}
pub struct UniversalUpdater<R: Runtime> {
    pub handle: AppHandle<R>,
    pub endpoint: Url,
    pub current_version: Version,
    arch: &'static str,
    target: String,
    pub json_target: String,
}

impl Update {
    pub async fn download<R: Runtime, C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        handle: &AppHandle<R>,
        mut on_chunk: C,
        on_download_finish: D,
    ) -> Result<Vec<u8>> {
        let mut request = ClientBuilder::new();

        let response = request
            .build()?
            .get(self.download_url.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Download request failed with status: {}",
                response.status()
            )));
        }

        let content_length: Option<u64> = response
            .headers()
            .get("Content-Length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());

        let mut buffer = Vec::new();

        let mut stream = response.bytes_stream();
        let mut size = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let len = chunk.len();
            let prev_size = size;
            size = size + len;
            on_chunk(chunk.len(), content_length);
            println!("{prev_size} + {len} = {size}");
            buffer.extend(chunk);
        }
        on_download_finish();

        Ok(buffer)
    }

    pub fn install(&self, bytes: impl AsRef<[u8]>) -> Result<()> {
        Ok(())
    }

    pub async fn download_and_install<R: Runtime, C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        handle: &AppHandle<R>,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<()> {
        let bytes = self.download(handle, on_chunk, on_download_finish).await?;

        Ok(())
    }
}

pub(crate) fn get_updater_target() -> Option<&'static str> {
    if cfg!(target_os = "linux") {
        Some("linux")
    } else if cfg!(target_os = "macos") {
        // TODO shouldn't this be macos instead?
        Some("darwin")
    } else if cfg!(target_os = "windows") {
        Some("windows")
    } else {
        None
    }
}

pub(crate) fn get_updater_arch() -> Option<&'static str> {
    if cfg!(target_arch = "x86") {
        Some("i686")
    } else if cfg!(target_arch = "x86_64") {
        Some("x86_64")
    } else if cfg!(target_arch = "arm") {
        Some("armv7")
    } else if cfg!(target_arch = "aarch64") {
        Some("aarch64")
    } else {
        None
    }
}
