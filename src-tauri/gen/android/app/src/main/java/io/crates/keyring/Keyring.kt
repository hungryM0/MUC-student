package io.crates.keyring

import android.content.Context

class Keyring {
  companion object {
    init {
      System.loadLibrary("muc_student_tauri_lib")
    }

    external fun initializeNdkContext(context: Context)
  }
}
