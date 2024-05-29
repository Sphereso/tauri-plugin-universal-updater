import { invoke } from '@tauri-apps/api/core';

async function execute() {
    await invoke("plugin:universal-updater|execute");
}
async function ping() {
    await invoke("plugin:universal-updater|ping");
}

export { execute, ping };
