package cn.muc.student

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import android.util.Log

object WifiNetworkBinder {
  private const val TAG = "MucWifiBinder"

  @Volatile
  private var started = false

  @Volatile
  private var boundNetwork: Network? = null

  @Volatile
  private var connectivityManager: ConnectivityManager? = null

  private val callback =
    object : ConnectivityManager.NetworkCallback() {
      override fun onAvailable(network: Network) {
        connectivityManager?.let { bind(it, network) }
      }

      override fun onLost(network: Network) {
        if (boundNetwork == network) {
          boundNetwork = null
          connectivityManager?.let { clearBinding(it) }
        }
      }
    }

  fun start(context: Context) {
    val appContext = context.applicationContext
    val manager = appContext.getSystemService(ConnectivityManager::class.java) ?: return
    connectivityManager = manager

    bindBestWifi(manager)

    if (started) {
      return
    }

    synchronized(this) {
      if (started) {
        return
      }
      val request =
        NetworkRequest.Builder()
          .addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
          .build()
      try {
        manager.registerNetworkCallback(request, callback)
        started = true
      } catch (error: RuntimeException) {
        Log.w(TAG, "register WiFi network callback failed", error)
      }
    }
  }

  @Suppress("DEPRECATION")
  private fun bindBestWifi(manager: ConnectivityManager) {
    val wifiNetwork =
      manager.allNetworks.firstOrNull { network ->
        manager
          .getNetworkCapabilities(network)
          ?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true
      }
    if (wifiNetwork != null) {
      bind(manager, wifiNetwork)
    }
  }

  private fun bind(manager: ConnectivityManager, network: Network) {
    val success =
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
        manager.bindProcessToNetwork(network)
      } else {
        @Suppress("DEPRECATION")
        ConnectivityManager.setProcessDefaultNetwork(network)
      }
    if (success) {
      boundNetwork = network
    } else {
      Log.w(TAG, "bind process to WiFi network failed")
    }
  }

  private fun clearBinding(manager: ConnectivityManager) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      manager.bindProcessToNetwork(null)
    } else {
      @Suppress("DEPRECATION")
      ConnectivityManager.setProcessDefaultNetwork(null)
    }
  }
}
