package com.example.p2p

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope

class ChatViewModel(app: Application) : AndroidViewModel(app) {

    private val node = P2pNode(app)

    init {
        node.start(viewModelScope)
        node.subscribe("chat/global")
    }

    val events = node.events

    fun send(text: String) = node.publishText("chat/global", text)

    override fun onCleared() {
        node.stop()
    }
}