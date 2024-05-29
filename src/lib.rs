use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

use std::{collections::HashMap, sync::Mutex};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::UniversalUpdater;
#[cfg(mobile)]
use mobile::UniversalUpdater;

#[derive(Default)]
struct MyState(Mutex<HashMap<String, String>>);

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the universal-updater APIs.
pub trait UniversalUpdaterExt<R: Runtime> {
    fn universal_updater(&self) -> &Result<UniversalUpdater<R>>;

    fn universal_updater_builder
}

impl<R: Runtime, T: Manager<R>> crate::UniversalUpdaterExt<R> for T {
    fn universal_updater(&self) -> &UniversalUpdater<R> {
        self.state::<UniversalUpdater<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    println!("Initializing universal-updater plugin");
    Builder::new("universal-updater")
        .invoke_handler(tauri::generate_handler![
            commands::check,
            commands::download,
            commands::install,
            commands::download_and_install,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let universal_updater = mobile::init(app, api)?;
            #[cfg(desktop)]
            let universal_updater = desktop::init(app, api)?;
            app.manage(universal_updater);

            // manage state so it is accessible by the commands
            app.manage(MyState::default());
            Ok(())
        })
        .build()
}
