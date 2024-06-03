use http::StatusCode;
use reqwest::ClientBuilder;
use semver::Version;
use serde::{de::Error as DeError, Deserialize, Serialize};
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Resource, Runtime,
};
use time::OffsetDateTime;
use url::Url;

use std::{collections::HashMap, str::FromStr};

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
        .invoke_handler(tauri::generate_handler![
            commands::check,
            commands::download_and_install
        ])
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaseManifestPlatform {
    /// Download URL for the platform
    pub url: Url,
    // Signature for the platform
    //pub signature: String,
}

#[derive(Debug, Clone)]
pub struct RemoteRelease {
    /// Version to install.
    pub version: Version,
    /// Release notes.
    pub notes: Option<String>,
    /// Release date.
    pub pub_date: Option<OffsetDateTime>,
    /// Release data.
    pub platforms: HashMap<String, ReleaseManifestPlatform>,
}

impl RemoteRelease {
    pub fn download_url(&self, target: &str) -> Result<&Url> {
        self.platforms
            .get(target)
            .map_or(Err(Error::TargetNotFound(target.to_string())), |p| {
                Ok(&p.url)
            })
    }
}

impl<'de> Deserialize<'de> for RemoteRelease {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InnerRemoteRelease {
            #[serde(alias = "name", deserialize_with = "parse_version")]
            version: Version,
            notes: Option<String>,
            pub_date: Option<String>,
            platforms: HashMap<String, ReleaseManifestPlatform>,
        }

        let release = InnerRemoteRelease::deserialize(deserializer)?;

        let pub_date = if let Some(date) = release.pub_date {
            Some(
                OffsetDateTime::parse(&date, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| DeError::custom(format!("invalid value for `pub_date`: {e}")))?,
            )
        } else {
            None
        };

        Ok(RemoteRelease {
            version: release.version,
            notes: release.notes,
            pub_date,
            platforms: release.platforms,
        })
    }
}

fn parse_version<'de, D>(deserializer: D) -> std::result::Result<Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;

    Version::from_str(str.trim_start_matches('v')).map_err(serde::de::Error::custom)
}

impl<R: Runtime> UniversalUpdater<R> {
    pub async fn check(&self) -> Result<Option<Update>> {
        let url = self.endpoint.clone();

        let mut request = ClientBuilder::new();

        let response = request.build()?.get(url).send().await;

        let remote_release = if let Ok(res) = response {
            if res.status().is_success() {
                if StatusCode::NO_CONTENT == res.status() {
                    return Ok(None);
                };

                serde_json::from_value::<RemoteRelease>(res.json().await?)
                    .map_err(Into::<Error>::into)
            } else {
                Err(Error::ReleaseNotFound)
            }
        } else {
            Err(Error::ReleaseNotFound)
        }?;

        let should_update = remote_release.version > self.current_version;

        let update = if should_update {
            Some(Update {
                version: remote_release.version.to_string(),
                current_version: self.current_version.to_string(),
                date: remote_release.pub_date,
                body: remote_release.notes.clone(),
                download_url: remote_release.download_url(&self.json_target)?.to_owned(),
            })
        } else {
            None
        };

        Ok(update)
    }
}

pub struct Update {
    current_version: String,
    version: String,
    date: Option<OffsetDateTime>,
    body: Option<String>,
    download_url: Url,
}

impl Resource for Update {}
