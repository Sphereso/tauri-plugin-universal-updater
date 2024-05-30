use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};
use url::Url;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.sphereso.updater";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(universal_updater_plugin);

use crate::{Config, Error};

pub fn init<R: Runtime>(
    _app: &AppHandle<R>,
    api: PluginApi<R, Config>,
) -> crate::Result<UniversalUpdater<R>> {
    let config = api.config().clone();
    let endpoint = config.endpoint.ok_or(Error::EmptyEndpoints)?;

    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "UniversalUpdaterPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(universal_updater_plugin)?;

    Ok(UniversalUpdater {
        handle: handle,
        endpoint: endpoint.0.clone(),
    })
}
pub struct UniversalUpdater<R: Runtime> {
    handle: PluginHandle<R>,
    pub endpoint: Url,
}
