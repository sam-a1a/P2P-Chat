package com.example.p2p

object P2pLib {

    val isAvailable: Boolean

    init {
        isAvailable = try {
            System.loadLibrary("p2p")
            true
        } catch (e: UnsatisfiedLinkError) {
            android.util.Log.e("P2pLib", "Failed to load native library: ${e.message}")
            false
        }
    }

    external fun start(keyPath: String, dbPath: String): Long
    external fun pollEvent(handle: Long): String?
    external fun subscribe(handle: Long, topic: String)
    external fun publish(handle: Long, topic: String, data: ByteArray)
    external fun shutdown(handle: Long)
    external fun destroy(handle: Long)
}