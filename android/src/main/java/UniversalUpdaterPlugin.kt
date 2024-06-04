package com.sphereso.updater

import android.app.Activity
import android.content.pm.PackageInstaller
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.internal.headersContentLength
import okio.buffer
import okio.sink
import okio.use

@InvokeArg
class DownloadArgs {
  lateinit var channel: Channel
  lateinit var url: String
}

@TauriPlugin
class UniversalUpdaterPlugin(private val activity: Activity): Plugin(activity) {
    private val client = OkHttpClient()

    @Command
    fun download(invoke: Invoke) {
        val args = invoke.parseArgs(DownloadArgs::class.java)

        val request = Request.Builder().url(args.url).build()

        val response = client.newCall(request).execute()

        val source = response.body?.source()

        val sessionParams = PackageInstaller.SessionParams(
            PackageInstaller.SessionParams.MODE_FULL_INSTALL
        )
        val packageInstaller = activity.packageManager.packageInstaller

        val sessionId = packageInstaller.createSession(sessionParams)
        val session = packageInstaller.openSession(sessionId)

        val length = response.headersContentLength()

        val buffer = session.openWrite("update.apk", 0, -1).sink().buffer().buffer
        var bytes: Long = 0
        source?.use { bufferedSource ->
            while (bufferedSource.read(buffer, 1024 * 8).also { bytes = it } != -1L) {
                val progress = JSObject().put("chunk", bytes).put("length", length)
                args.channel.send(progress)
            }
        }
        invoke.resolve()
    }
}
