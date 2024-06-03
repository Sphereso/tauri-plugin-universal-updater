use semver::Version;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};
use url::Url;

use crate::{Config, Error, Result, Update};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.sphereso.updater";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(universal_updater_plugin);

pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Config>,
) -> crate::Result<UniversalUpdater<R>> {
    let config = api.config().clone();
    let endpoint = config.endpoint.ok_or(Error::EmptyEndpoints)?;

    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "UniversalUpdaterPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(universal_updater_plugin)?;

    let arch = get_updater_arch().ok_or(Error::UnsupportedArch)?;
    let (target, json_target) = {
        let target = get_updater_target().ok_or(Error::UnsupportedOs)?;
        (target.to_string(), format!("{target}-{arch}"))
    };

    Ok(UniversalUpdater {
        handle: handle,
        endpoint: endpoint.0.clone(),
        current_version: app.package_info().version.clone(),
        arch,
        target,
        json_target,
    })
}
pub struct UniversalUpdater<R: Runtime> {
    pub handle: PluginHandle<R>,
    pub endpoint: Url,
    pub current_version: Version,
    arch: &'static str,
    target: String,
    pub json_target: String,
}

#[derive(Serialize, Deserialize)]
struct PingModel {
    value: String,
}

impl Update {
    pub async fn download<R: Runtime, C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        handle: &PluginHandle<R>,
        mut on_chunk: C,
        on_download_finish: D,
    ) -> Result<()> {
        let r = handle
            .run_mobile_plugin::<PingModel>(
                "ping",
                PingModel {
                    value: String::from("test"),
                },
            )
            .map_err(Into::<Error>::into)?
            .value;
        println!("Received: {r} from android");
        /*let mut request = ClientBuilder::new();

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
        on_download_finish();*/

        Ok(())
    }

    pub fn install(&self, bytes: impl AsRef<[u8]>) -> Result<()> {
        Ok(())
    }

    pub async fn download_and_install<R: Runtime, C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        handle: &PluginHandle<R>,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<()> {
        let _ = self.download(handle, on_chunk, on_download_finish).await?;

        Ok(())
    }
}

pub(crate) fn get_updater_target() -> Option<&'static str> {
    if cfg!(target_os = "android") {
        Some("android")
    } else {
        None
    }
}

pub(crate) fn get_updater_arch() -> Option<&'static str> {
    if cfg!(target_os = "android") {
        Some("universal")
    } else {
        None
    }
}
