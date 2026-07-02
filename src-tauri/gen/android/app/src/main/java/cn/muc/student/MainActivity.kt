package cn.muc.student

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import io.crates.keyring.Keyring

class MainActivity : TauriActivity() {
  private val requestNotificationPermission =
    registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
      if (granted) {
        startKeepaliveService()
      }
    }

  override fun onCreate(savedInstanceState: Bundle?) {
    Keyring.initializeNdkContext(applicationContext)
    WifiNetworkBinder.start(applicationContext)
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    ensureKeepalivePermission()
  }

  private fun startKeepaliveService() {
    val intent = Intent(this, AndroidKeepaliveService::class.java)
    startForegroundService(intent)
  }

  private fun ensureKeepalivePermission() {
    if (android.os.Build.VERSION.SDK_INT < android.os.Build.VERSION_CODES.TIRAMISU) {
      startKeepaliveService()
      return
    }
    if (
      ContextCompat.checkSelfPermission(
        this,
        Manifest.permission.POST_NOTIFICATIONS,
      ) == PackageManager.PERMISSION_GRANTED
    ) {
      startKeepaliveService()
      return
    }
    requestNotificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
  }
}
