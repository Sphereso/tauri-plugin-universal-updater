'use strict';

var core = require('@tauri-apps/api/core');

async function execute() {
    await core.invoke("plugin:universal-updater|execute");
}
async function ping() {
    await core.invoke("plugin:universal-updater|ping");
}

exports.execute = execute;
exports.ping = ping;
