package cn.muc.student

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import kotlin.concurrent.thread

@InvokeArg
class AwaitWifiContextArgs {
  var timeoutMs: Long = 4_000
}

@TauriPlugin
class CampusNetworkPlugin(private val activity: Activity) : Plugin(activity) {
  @Command
  fun awaitWifiContext(invoke: Invoke) {
    val args = invoke.parseArgs(AwaitWifiContextArgs::class.java)
    thread(isDaemon = true, name = "muc-wifi-context") {
      try {
        val context = WifiNetworkBinder.awaitContext(activity.applicationContext, args.timeoutMs)
        invoke.resolveObject(context)
      } catch (error: Exception) {
        invoke.reject(error.message ?: "读取校园网 Wi-Fi 状态失败")
      }
    }
  }
}
