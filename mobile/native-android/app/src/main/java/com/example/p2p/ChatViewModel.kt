package com.example.p2p

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class ChatViewModel(app: Application) : AndroidViewModel(app) {
    private val node = P2pNode(app)
    private val _messages = MutableStateFlow<List<String>>(emptyList())
    val messages = _messages.asStateFlow()
    private val _connected = MutableStateFlow(false)
    val connected = _connected.asStateFlow()
    private val peers = mutableSetOf<String>()
    private val preferences = app.getSharedPreferences("chat", 0)
    val initialAddress: String = preferences.getString("address", "/ip4/10.0.2.2/tcp/53493")!!
    private fun append(text: String) { _messages.value = (_messages.value + text).takeLast(500) }
    init {
        viewModelScope.launch(start = CoroutineStart.UNDISPATCHED) {
            node.events.collect { event ->
                when (event) {
                    is NodeEvent.MessageReceived -> append("${event.fromPeer.take(12)}: ${event.ciphertext.toString(Charsets.UTF_8)}")
                    is NodeEvent.MessagePublished -> append("You: ${event.text}")
                    is NodeEvent.ConnectionEstablished -> { peers.add(event.peer); _connected.value = true; append("Connected: ${event.peer.take(12)}") }
                    is NodeEvent.ConnectionClosed -> { peers.remove(event.peer); _connected.value = peers.isNotEmpty(); append("Disconnected: ${event.peer.take(12)}") }
                    is NodeEvent.Error -> append("Error: ${event.message}")
                    is NodeEvent.ListeningOn -> append("Listening: ${event.address}")
                    else -> Unit
                }
            }
        }
        node.start(viewModelScope)
        node.subscribe("chat")
    }
    fun connect(address: String) {
        preferences.edit().putString("address", address.trim()).apply()
        node.dial(address.trim())
    }
    fun send(text: String) { if (text.isNotBlank()) node.publishText("chat", text.trim()) }
    override fun onCleared() { node.stop() }
}
