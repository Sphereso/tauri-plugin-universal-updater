package com.sphereso.updater

import android.app.Activity
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageInstaller
import androidx.core.content.ContextCompat
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import okhttp3.Call
import okhttp3.Callback
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.internal.headersContentLength
import okio.buffer
import okio.sink
import okio.use
import java.io.IOException


@InvokeArg
class DownloadArgs {
  lateinit var channel: Channel
  lateinit var url: String
}

@TauriPlugin
class UniversalUpdaterPlugin(private val activity: Activity): Plugin(activity) {
    private val client = OkHttpClient()

    private val packageActionReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val msg = intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE)
            Logger.debug("Message $msg")
            when (intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)) {
                PackageInstaller.STATUS_PENDING_USER_ACTION -> {
                    Logger.debug("PENDING_USER_ACTION")
                    val userAction = intent.getParcelableExtra<Intent>(Intent.EXTRA_INTENT)
                    if (userAction == null) {
                        Logger.debug("Fatal error for $intent")
                        return
                    }
                    userAction.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    activity.startActivity(userAction)
                }
                PackageInstaller.STATUS_FAILURE_ABORTED -> {
                    Logger.debug("STATUS_FAILURE_ABORTED")
                }
                PackageInstaller.STATUS_SUCCESS -> Logger.debug("PACKAGE INSTALL SUCCESS")
                else -> Logger.error("PACKAGE INSTALL ERROR")
            }
        }
    }

    @Command
    fun download(invoke: Invoke) {
        val args = invoke.parseArgs(DownloadArgs::class.java)

        val request = Request.Builder().url(args.url).build()
        Logger.verbose("download")

        //val response = client.newCall(request).execute()
        client.newCall(request).enqueue(object : Callback {
            override fun onFailure(call: Call, e: IOException) {
                e.printStackTrace()
                invoke.reject("Failure")
            }

            override fun onResponse(call: Call, response: Response) {
                Logger.verbose("onresponse")
                response.use {
                    if (!response.isSuccessful) throw IOException("Unexpected code $response")

                    val source = response.body!!.byteStream()
                    val sessionParams = PackageInstaller.SessionParams(
                        PackageInstaller.SessionParams.MODE_FULL_INSTALL
                    )
                    val packageInstaller = activity.packageManager.packageInstaller

                    val sessionId = packageInstaller.createSession(sessionParams)
                    val session = packageInstaller.openSession(sessionId)

                    val length = response.headersContentLength()


                    var bytes: Long = 0
                    source.use { bufferedSource ->
                        val sessionStream = session.openWrite("update.apk", 0, -1)
                        val startedEvent = JSObject().put("event", "Started")
                            .put("data", JSObject().put("contentLength", length))
                        args.channel.send(startedEvent)
                        sessionStream.buffered().use { bufferedSessionStream ->
                            bufferedSource.copyTo(bufferedSessionStream)
                            bufferedSessionStream.flush()
                            session.fsync(sessionStream)
                        }
                    }
                    val flags = PendingIntent.FLAG_MUTABLE

                    val intentSender = PendingIntent.getBroadcast(
                        activity,
                        sessionId,
                        Intent(INSTALL_ACTION).setPackage(activity.packageName),
                        flags
                    ).intentSender
                    session.commit(intentSender)
                    session.close()
                    invoke.resolve()
                }
            }
        })

        /*val source = response.body?.source()

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
            val startedEvent = JSObject().put("event", "Started").put("data", JSObject().put("contentLength", length))
            //args.channel.send(startedEvent)
            while (bufferedSource.read(buffer, 1024 * 8).also { bytes = it } != -1L) {
                val progressEvent = JSObject().put("event", "Progress").put("data", JSObject().put("chunkLength", bytes))
                //args.channel.send(progressEvent)
            }
        }
        //args.channel.send(JSObject().put("event", "Finished"))*/

    }
    init {
        ContextCompat.registerReceiver(activity, packageActionReceiver, IntentFilter(INSTALL_ACTION), ContextCompat.RECEIVER_EXPORTED)
    }
}
private const val INSTALL_ACTION = "PackageInstaller.INSTALL_ACTION"