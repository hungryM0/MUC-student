package cn.muc.student

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import java.io.File
import kotlin.concurrent.thread

class AndroidKeepaliveService : Service() {
  private var worker: Thread? = null
  private lateinit var stateFile: File

  override fun onCreate() {
    super.onCreate()
    stateFile = File(applicationContext.dataDir, "android_keepalive_state.json")
    startForeground(NOTIFICATION_ID, buildNotification(loadSummary()))
    worker =
      thread(isDaemon = true, name = "muc-keepalive") {
        while (!Thread.currentThread().isInterrupted) {
          try {
            Thread.sleep(5 * 60 * 1000L)
            updateNotification()
          } catch (_: InterruptedException) {
            Thread.currentThread().interrupt()
          }
        }
      }
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    updateNotification()
    return START_STICKY
  }

  override fun onDestroy() {
    worker?.interrupt()
    worker = null
    super.onDestroy()
  }

  override fun onBind(intent: Intent?): IBinder? = null

  private fun updateNotification() {
    val manager = NotificationManagerCompat.from(this)
    if (!manager.areNotificationsEnabled()) {
      return
    }
    manager.notify(NOTIFICATION_ID, buildNotification(loadSummary()))
  }

  private fun buildNotification(summary: String): Notification {
    ensureChannel()
    val contentIntent = PendingIntent.getActivity(
      this,
      0,
      Intent(this, MainActivity::class.java).apply {
        flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP
      },
      pendingIntentFlags()
    )
    return NotificationCompat.Builder(this, CHANNEL_ID)
      .setSmallIcon(R.mipmap.ic_launcher)
      .setContentTitle(getString(R.string.main_activity_title))
      .setContentText(summary)
      .setStyle(NotificationCompat.BigTextStyle().bigText(summary))
      .setOngoing(true)
      .setOnlyAlertOnce(true)
      .setContentIntent(contentIntent)
      .build()
  }

  private fun loadSummary(): String {
    return try {
      if (!stateFile.exists()) {
        "正在同步数据"
      } else {
        val text = stateFile.readText()
        val used =
          Regex(""""usedTrafficText"\s*:\s*"([^"]*)"""").find(text)?.groupValues?.get(1).orEmpty()
        val total =
          Regex(""""productBalanceText"\s*:\s*"([^"]*)"""").find(text)?.groupValues?.get(1).orEmpty()
        val account =
          Regex(""""currentAccountName"\s*:\s*"([^"]*)"""").find(text)?.groupValues?.get(1).orEmpty()
        val accountText = if (account.isBlank()) "当前账号：未登录" else "当前账号：$account"
        val trafficText = listOf(used, total).filter { it.isNotBlank() }.joinToString("/")
          .ifBlank { "已用流量/总量：未知" }
        listOf(accountText, trafficText).joinToString(" · ")
      }
    } catch (_: Exception) {
      "正在同步数据"
    }
  }

  private fun ensureChannel() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
      return
    }
    val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    val channel = NotificationChannel(
      CHANNEL_ID,
      "MUC-student 常驻同步",
      NotificationManager.IMPORTANCE_LOW,
    ).apply {
      setShowBadge(false)
      description = "显示当前账号流量同步状态"
      lockscreenVisibility = Notification.VISIBILITY_SECRET
    }
    manager.createNotificationChannel(channel)
  }

  private fun pendingIntentFlags(): Int {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
    } else {
      PendingIntent.FLAG_UPDATE_CURRENT
    }
  }

  companion object {
    private const val CHANNEL_ID = "muc_student_keepalive"
    private const val NOTIFICATION_ID = 8800
  }
}
