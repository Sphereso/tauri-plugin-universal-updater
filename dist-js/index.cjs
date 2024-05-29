'use strict';

var core = require('@tauri-apps/api/core');

async function execute() {
    await core.invoke('plugin:universal-updater|execute');
}

exports.execute = execute;
