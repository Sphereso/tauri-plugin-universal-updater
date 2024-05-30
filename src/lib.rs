use reqwest::ClientBuilder;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};
use url::Url;

use std::{collections::HashMap, ffi::OsString, sync::Mutex};

pub use models::*;

mod commands;
mod config;
mod error;
mod models;

pub use config::Config;
pub use error::{Error, Result};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

#[cfg(desktop)]
use desktop::*;
#[cfg(mobile)]
use mobile::*;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the universal-updater APIs.
pub trait UniversalUpdaterExt<R: Runtime> {
    fn updater(&self) -> Result<UniversalUpdater<R>>;
}

impl<R: Runtime, T: Manager<R>> UniversalUpdaterExt<R> for T {
    fn updater(&self) -> Result<UniversalUpdater<R>> {
        Err(Error::EmptyEndpoints)
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R, Config> {
    println!("Initializing universal-updater plugin");
    PluginBuilder::new("universal-updater")
        .invoke_handler(tauri::generate_handler![commands::check])
        .setup(|app, api| {
            #[cfg(mobile)]
            let universal_updater = mobile::init(app, api)?;
            #[cfg(desktop)]
            let universal_updater = desktop::init(app, api)?;
            app.manage(universal_updater);

            // manage state so it is accessible by the commands
            Ok(())
        })
        .build()
}

impl<R: Runtime> UniversalUpdater<R> {
    pub async fn check(&self) -> Result<Option<Update>> {
        let url = self.endpoint.clone();

        let mut request = ClientBuilder::new();

        let response = request.build()?.get(url).send().await?;

        println!("success");

        Ok(Some(Update {}))
    }
}

pub struct Update {}
