use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<UniversalUpdater<R>> {
  Ok(UniversalUpdater(app.clone()))
}

/// Access to the universal-updater APIs.
pub struct UniversalUpdater<R: Runtime>(AppHandle<R>);

impl<R: Runtime> UniversalUpdater<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
