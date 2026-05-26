package com.example.p2p

import android.content.Context
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class P2pNode(context: Context) {

    private val filesDir = context.filesDir.absolutePath
    private val keyPath = "$filesDir/identity.key"
    private val dbPath = "$filesDir/messages.db"

    private var handle: Long = 0L
    private var pollJob: Job? = null

    private val _events = MutableSharedFlow<NodeEvent>(extraBufferCapacity = 64)
    val events: SharedFlow<NodeEvent> = _events

    fun start(scope: CoroutineScope) {
        if (handle != 0L) return
        handle = P2pLib.start(keyPath, dbPath)
        if (handle == 0L) {
            scope.launch {
                _events.emit(NodeEvent.Error("Failed to start P2P node"))
            }
            return
        }
        startPolling(scope)
    }

    fun subscribe(topic: String) {
        if (handle == 0L) return
        P2pLib.subscribe(handle, topic)
    }

    fun publish(topic: String, data: ByteArray) {
        if (handle == 0L) return
        P2pLib.publish(handle, topic, data)
    }

    fun publishText(topic: String, text: String) {
        publish(topic, text.toByteArray(Charsets.UTF_8))
    }

    fun stop() {
        pollJob?.cancel()
        if (handle != 0L) {
            P2pLib.shutdown(handle)
            P2pLib.destroy(handle)
            handle = 0L
        }
    }

    private fun startPolling(scope: CoroutineScope) {
        pollJob = scope.launch(Dispatchers.IO) {
            while (isActive && handle != 0L) {
                val json = P2pLib.pollEvent(handle)
                if (json != null) {
                    val event = NodeEvent.fromJson(json)
                    _events.emit(event)
                } else {
                    delay(100)
                }
            }
        }
    }
}