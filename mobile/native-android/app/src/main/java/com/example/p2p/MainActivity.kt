package com.example.p2p

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import com.example.p2p.ui.theme.P2PTheme

class MainActivity : ComponentActivity() {

    private val vm: ChatViewModel by viewModels {
        ChatViewModelFactory(application)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            P2PTheme {
                LaunchedEffect(Unit) {
                    vm.events.collect { event ->
                        when (event) {
                            is NodeEvent.MessageReceived ->
                                Log.d("P2P", "msg: ${event.topic}")
                            is NodeEvent.PeerDiscovered ->
                                Log.d("P2P", "peer: ${event.peer}")
                            is NodeEvent.Error ->
                                Log.e("P2P", "error: ${event.message}")
                            else -> {}
                        }
                    }
                }
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Box(modifier = Modifier.padding(innerPadding)) {
                        ChatScreen(vm = vm)
                    }
                }
            }
        }
    }
}