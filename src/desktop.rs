use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};
use url::Url;

use crate::{Config, Error};

pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Config>,
) -> crate::Result<UniversalUpdater<R>> {
    let config = api.config().clone();
    let endpoint = config.endpoint.ok_or(Error::EmptyEndpoints)?;

    Ok(UniversalUpdater {
        app: app.clone(),
        endpoint: endpoint.0.clone(),
    })
}
pub struct UniversalUpdater<R: Runtime> {
    app: AppHandle<R>,
    pub endpoint: Url,
}
