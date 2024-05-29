import { invoke } from '@tauri-apps/api/core';

async function execute() {
    await invoke('plugin:universal-updater|execute');
}

export { execute };
