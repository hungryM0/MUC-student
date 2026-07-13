package cn.muc.student

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import android.util.Log
import java.net.Inet4Address

object WifiNetworkBinder {
  private const val TAG = "MucWifiBinder"

  @Volatile
  private var started = false

  @Volatile
  private var boundNetwork: Network? = null

  @Volatile
  private var connectivityManager: ConnectivityManager? = null

  @Volatile
  private var currentContext = WifiNetworkContext.unavailable("尚未绑定校园网 Wi-Fi")

  private val contextMonitor = Object()

  private val callback =
    object : ConnectivityManager.NetworkCallback() {
      override fun onAvailable(network: Network) {
        Log.i(TAG, "WiFi network available: ${describeNetwork(connectivityManager, network)}")
        connectivityManager?.let { bind(it, network) }
      }

      override fun onLinkPropertiesChanged(network: Network, linkProperties: android.net.LinkProperties) {
        if (boundNetwork == network) {
          connectivityManager?.let { updateBoundContext(it, network) }
        }
      }

      override fun onLost(network: Network) {
        Log.i(TAG, "WiFi network lost: $network")
        if (boundNetwork == network) {
          boundNetwork = null
          publishContext(WifiNetworkContext.unavailable("校园网 Wi-Fi 已断开"))
          connectivityManager?.let { clearBinding(it) }
        }
      }

      override fun onUnavailable() {
        Log.w(TAG, "WiFi network unavailable")
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
          .removeCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
          .build()
      try {
        manager.requestNetwork(request, callback)
        Log.i(TAG, "requested WiFi network binding")
        started = true
      } catch (error: RuntimeException) {
        Log.w(TAG, "request WiFi network failed", error)
      }
    }
  }

  fun awaitContext(context: Context, timeoutMs: Long): WifiNetworkContext {
    start(context)
    currentContext.takeIf { it.isReady() }?.let { return it }

    val deadline = System.currentTimeMillis() + timeoutMs.coerceIn(0L, 10_000L)
    synchronized(contextMonitor) {
      while (!currentContext.isReady()) {
        val remaining = deadline - System.currentTimeMillis()
        if (remaining <= 0L) {
          break
        }
        contextMonitor.wait(remaining)
      }
    }
    return currentContext
  }

  @Suppress("DEPRECATION")
  private fun bindBestWifi(manager: ConnectivityManager) {
    val wifiNetwork = manager.allNetworks
      .filter { network ->
        manager
          .getNetworkCapabilities(network)
          ?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true
      }
      .firstOrNull { network -> findIpv4(manager, network).isNotEmpty() }
    if (wifiNetwork != null) {
      Log.i(TAG, "binding existing WiFi network: ${describeNetwork(manager, wifiNetwork)}")
      bind(manager, wifiNetwork)
    } else {
      Log.w(TAG, "no existing WiFi network found")
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
      updateBoundContext(manager, network)
      Log.i(TAG, "bound process to WiFi network: ${describeNetwork(manager, network)}")
    } else {
      publishContext(WifiNetworkContext.unavailable("系统拒绝绑定校园网 Wi-Fi"))
      Log.w(TAG, "bind process to WiFi network failed: ${describeNetwork(manager, network)}")
    }
  }

  private fun updateBoundContext(manager: ConnectivityManager, network: Network) {
    val ipv4 = findIpv4(manager, network)
    val context =
      if (ipv4.isEmpty()) {
        WifiNetworkContext.unavailable("校园网 Wi-Fi 没有可用 IPv4")
      } else {
        WifiNetworkContext(
          bound = true,
          ipv4 = ipv4,
          networkHandle = network.networkHandle.toString(),
          detail = "校园网 Wi-Fi 已绑定",
        )
      }
    publishContext(context)
  }

  private fun findIpv4(manager: ConnectivityManager, network: Network): String =
    manager
      .getLinkProperties(network)
      ?.linkAddresses
      ?.asSequence()
      ?.map { it.address }
      ?.filterIsInstance<Inet4Address>()
      ?.firstOrNull { !it.isLoopbackAddress && !it.isLinkLocalAddress }
      ?.hostAddress
      .orEmpty()

  private fun publishContext(context: WifiNetworkContext) {
    currentContext = context
    synchronized(contextMonitor) {
      contextMonitor.notifyAll()
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

  private fun describeNetwork(manager: ConnectivityManager?, network: Network): String {
    val capabilities = manager?.getNetworkCapabilities(network)
    return "network=$network capabilities=${capabilities ?: "unknown"}"
  }
}

data class WifiNetworkContext(
  val bound: Boolean,
  val ipv4: String,
  val networkHandle: String,
  val detail: String,
) {
  fun isReady(): Boolean = bound && ipv4.isNotBlank() && networkHandle.isNotBlank()

  companion object {
    fun unavailable(detail: String) = WifiNetworkContext(false, "", "", detail)
  }
}
