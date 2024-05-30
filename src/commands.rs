use serde::Serialize;
use tauri::{
    ipc::Channel, AppHandle, Manager, Resource, ResourceId, Runtime, State, Webview, Window,
};

use crate::Update;

use crate::{Result, UniversalUpdater, UniversalUpdaterExt};
use std::time::Duration;

use url::Url;

#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        chunk_length: usize,
    },
    Finished,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    rid: Option<ResourceId>,
    available: bool,
    current_version: String,
    version: String,
    date: Option<String>,
    body: Option<String>,
}

struct DownloadedBytes(pub Vec<u8>);
impl Resource for DownloadedBytes {}

#[tauri::command]
pub(crate) async fn check<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, UniversalUpdater<R>>,
    headers: Option<Vec<(String, String)>>,
    timeout: Option<u64>,
    proxy: Option<String>,
    target: Option<String>,
) -> Result<Metadata> {
    let updater = state.inner();
    println!("check command hello from rust");

    let result = updater.check().await?;
    Ok(Metadata::default())
}
/*
#[tauri::command]
pub(crate) async fn download<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    on_event: Channel,
) -> Result<ResourceId> {
    let update = webview.resources_table().get::<Update>(rid)?;
    let mut first_chunk = true;
    let bytes = update
        .download(
            |chunk_length, content_length| {
                if first_chunk {
                    first_chunk = !first_chunk;
                    let _ = on_event.send(DownloadEvent::Started { content_length });
                }
                let _ = on_event.send(DownloadEvent::Progress { chunk_length });
            },
            || {
                let _ = on_event.send(&DownloadEvent::Finished);
            },
        )
        .await?;
    Ok(webview.resources_table().add(DownloadedBytes(bytes)))
}

#[tauri::command]
pub(crate) async fn install<R: Runtime>(
    webview: Webview<R>,
    update_rid: ResourceId,
    bytes_rid: ResourceId,
) -> Result<()> {
    let update = webview.resources_table().get::<Update>(update_rid)?;
    let bytes = webview
        .resources_table()
        .get::<DownloadedBytes>(bytes_rid)?;
    update.install(&bytes.0)?;
    let _ = webview.resources_table().close(bytes_rid);
    Ok(())
}

#[tauri::command]
pub(crate) async fn download_and_install<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    on_event: Channel,
) -> Result<()> {
    let update = webview.resources_table().get::<Update>(rid)?;

    let mut first_chunk = true;

    update
        .download_and_install(
            |chunk_length, content_length| {
                if first_chunk {
                    first_chunk = !first_chunk;
                    let _ = on_event.send(DownloadEvent::Started { content_length });
                }
                let _ = on_event.send(DownloadEvent::Progress { chunk_length });
            },
            || {
                let _ = on_event.send(&DownloadEvent::Finished);
            },
        )
        .await?;

    Ok(())
}
*/
