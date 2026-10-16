package com.example.p2p

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ChatScreen(vm: ChatViewModel) {
    val messages by vm.messages.collectAsState()
    val connected by vm.connected.collectAsState()
    val listState = rememberLazyListState()
    val input by vm.draft.collectAsState()
    val sending by vm.sending.collectAsState()
    var address by rememberSaveable { mutableStateOf(vm.initialAddress) }

    LaunchedEffect(messages.size) {
        if (messages.isNotEmpty()) listState.animateScrollToItem(messages.lastIndex)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .imePadding()
    ) {
        Text("P2P Chat · Android", modifier = Modifier.padding(12.dp))
        OutlinedTextField(value = address, onValueChange = { address = it }, label = { Text("Peer address") }, modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp), singleLine = true)
        Row(modifier = Modifier.padding(12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Button(onClick = { vm.connect(address) }, enabled = address.isNotBlank()) { Text("Connect") }
            Text(if (connected) "Connected" else "Disconnected")
        }
        LazyColumn(
            state = listState,
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .padding(horizontal = 12.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            items(messages) { msg ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = msg,
                        modifier = Modifier.padding(10.dp)
                    )
                }
            }
        }

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            OutlinedTextField(
                value = input,
                onValueChange = vm::updateDraft,
                modifier = Modifier.weight(1f),
                placeholder = { Text("Message") },
                enabled = connected,
                singleLine = true
            )
            Button(
                enabled = connected && !sending && input.isNotBlank(),
                onClick = {
                    if (input.isNotBlank()) {
                        vm.send()
                    }
                }
            ) {
                Text("Send")
            }
        }
    }
}